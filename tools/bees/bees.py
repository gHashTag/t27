#!/usr/bin/env python3
"""The reviewer bees' own GitHub identity: the `t27-bees` GitHub App.

WHY THIS EXISTS

Owner's rule, 2026-10-02: a merge happens only after a reviewer bee has
reviewed and verified the pull request (#5525). The scheduled merger,
`.github/workflows/auto-merge-ready-prs.yml`, wants an APPROVED review and the
`bee-reviewed` label, both after the head arrived.

Bees ran on the owner's token, and the owner's account `gHashTag` is also the
author of every bee pull request. GitHub refuses self-approval, so the reviewer
bee on #5526 could only leave `COMMENTED` -- measured: the one review on that
pull request is `gHashTag COMMENTED`. The merger could therefore never fire for
a bee pull request, and anyone reading "approved" would have been reading the
author's own voice anyway.

A GitHub App acts as `<slug>[bot]`, a distinct account. Its reviews are real
approvals, and its label events carry its own login, so the merger can ask
"did the BEE say so" instead of "did someone say so" (#5547).

THREE COMMANDS, ONE MODULE

  bee-app create      the owner runs this once. It serves a local page that
                      posts `manifest.json` to GitHub's manifest flow, catches
                      the redirect on 127.0.0.1, converts the one-hour code
                      (POST /app-manifests/{code}/conversions), writes the
                      private key to ~/.config/t27-bees/ with mode 0600, and
                      records the app id and key PATH in the macOS Keychain.
  bee-app convert C   the same conversion by hand, if the browser redirect was
                      lost: C is the `code` from the address bar.
  bee-token           a reviewer bee runs this. It signs a ten-minute JWT with
                      the private key and trades it for a one-hour installation
                      token scoped to ONE repository, printed on stdout and
                      nowhere else:

      GH_TOKEN=$(tools/bees/bee-token) gh pr review N --approve --body "..."
      GH_TOKEN=$(tools/bees/bee-token) gh pr edit N --add-label bee-reviewed

WHAT NEVER HAPPENS

  - No secret is written into the repository, printed, or logged. The
    conversion also returns `client_secret` and `webhook_secret`; both are
    dropped on the floor, because nothing here uses them.
  - The key file is refused if anyone but its owner can read it.
  - The JWT never leaves this process except in the Authorization header.

The RS256 signature is made by the `openssl` binary, not a Python package:
this stays standard-library only, so a bee needs nothing installed to run it.

    python3 tools/bees/bees.py self-test     # no network, no real secrets
"""

import argparse
import base64
import html
import http.server
import json
import os
import pathlib
import re
import secrets
import stat
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent
MANIFEST = HERE / "manifest.json"

KEYCHAIN_SERVICE = "t27-bees"
ACCOUNT_APP_ID = "app-id"
ACCOUNT_KEY_PATH = "private-key-path"
ENV_APP_ID = "BEE_APP_ID"
ENV_KEY_PATH = "BEE_PRIVATE_KEY_PATH"
ENV_REPO = "BEE_REPO"
DEFAULT_REPO = "gHashTag/t27"
DEFAULT_PORT = 8727
KEY_DIR = pathlib.Path.home() / ".config" / "t27-bees"
API = "https://api.github.com"
NEW_APP_URL = "https://github.com/settings/apps/new"

# GitHub accepts at most ten minutes between iat and exp. iat is backdated a
# minute so a clock that runs slightly fast is not refused as "issued in the
# future"; exp is nine minutes ahead, so the window is exactly ten.
JWT_BACKDATE = 60
JWT_LIFETIME = 540


class BeeError(Exception):
    """A failure whose message is safe to print: it never carries a secret."""


# ---------------------------------------------------------------------------
# configuration: env first, then the Keychain

def _keychain_read(account, runner=subprocess.run):
    try:
        r = runner(["security", "find-generic-password", "-s", KEYCHAIN_SERVICE,
                    "-a", account, "-w"], capture_output=True, text=True)
    except FileNotFoundError:
        return ""
    return r.stdout.strip() if r.returncode == 0 else ""


def _keychain_write(account, value, runner=subprocess.run):
    r = runner(["security", "add-generic-password", "-U", "-s", KEYCHAIN_SERVICE,
                "-a", account, "-w", value], capture_output=True, text=True)
    if r.returncode != 0:
        raise BeeError(f"could not write Keychain item {KEYCHAIN_SERVICE}/{account}")


def load_config(env=None, keychain=_keychain_read):
    """(app_id, key_path). Env wins over Keychain; nothing found is an error."""
    env = os.environ if env is None else env
    app_id = env.get(ENV_APP_ID) or keychain(ACCOUNT_APP_ID)
    key_path = env.get(ENV_KEY_PATH) or keychain(ACCOUNT_KEY_PATH)
    if not app_id or not key_path:
        raise BeeError(
            f"no app id / key path: set {ENV_APP_ID} and {ENV_KEY_PATH}, or run "
            f"`tools/bees/bee-app create` (Keychain service `{KEYCHAIN_SERVICE}`)")
    if not re.fullmatch(r"[0-9]+", app_id):
        raise BeeError(f"app id must be numeric, got {len(app_id)} characters of something else")
    return app_id, os.path.expanduser(key_path)


def check_key_file(path):
    """The key must exist and be readable by its owner only."""
    try:
        st = os.stat(path)
    except OSError:
        raise BeeError(f"private key not found at {path}")
    if st.st_mode & (stat.S_IRWXG | stat.S_IRWXO):
        raise BeeError(f"private key {path} is readable by others "
                       f"(mode {oct(st.st_mode & 0o777)}); run: chmod 600 {path}")


# ---------------------------------------------------------------------------
# the JWT

def b64url(data):
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def b64url_decode(s):
    return base64.urlsafe_b64decode(s + "=" * (-len(s) % 4))


def make_jwt(app_id, key_path, now=None, signer=None):
    now = int(time.time()) if now is None else now
    header = {"alg": "RS256", "typ": "JWT"}
    claims = {"iat": now - JWT_BACKDATE, "exp": now + JWT_LIFETIME, "iss": app_id}
    signing_input = (b64url(json.dumps(header, separators=(",", ":")).encode()) + "." +
                     b64url(json.dumps(claims, separators=(",", ":")).encode()))
    signer = signer or openssl_sign
    return signing_input + "." + b64url(signer(signing_input.encode(), key_path))


def openssl_sign(data, key_path):
    r = subprocess.run(["openssl", "dgst", "-sha256", "-sign", key_path],
                       input=data, capture_output=True)
    if r.returncode != 0:
        raise BeeError("openssl could not sign with the private key (is it a PEM RSA key?)")
    return r.stdout


# ---------------------------------------------------------------------------
# HTTP

def _request(method, url, token=None, body=None, opener=urllib.request.urlopen):
    headers = {"Accept": "application/vnd.github+json",
               "X-GitHub-Api-Version": "2022-11-28",
               "User-Agent": "t27-bees"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    data = None
    if body is not None:
        data = json.dumps(body).encode()
        headers["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with opener(req) as resp:
            return json.loads(resp.read().decode() or "null")
    except urllib.error.HTTPError as e:
        # The status and GitHub's message only: never the request headers.
        try:
            msg = json.loads(e.read().decode()).get("message", "")
        except Exception:
            msg = ""
        raise BeeError(f"{method} {urllib.parse.urlsplit(url).path} -> HTTP {e.code} {msg}".rstrip())


def default_repo(runner=subprocess.run):
    """owner/name from `git remote get-url origin`, else gHashTag/t27."""
    try:
        r = runner(["git", "remote", "get-url", "origin"], capture_output=True, text=True)
        url = r.stdout.strip() if r.returncode == 0 else ""
    except FileNotFoundError:
        url = ""
    m = re.search(r"github\.com[:/]([^/]+/[^/]+?)(?:\.git)?/?$", url)
    return m.group(1) if m else DEFAULT_REPO


def mint_token(repo, env=None, keychain=_keychain_read, signer=None, opener=urllib.request.urlopen,
               now=None):
    """A one-hour installation token, scoped to `repo` alone."""
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
        raise BeeError(f"repository must be owner/name, got {repo!r}")
    app_id, key_path = load_config(env, keychain)
    check_key_file(key_path)
    jwt = make_jwt(app_id, key_path, now=now, signer=signer)
    inst = _request("GET", f"{API}/repos/{repo}/installation", token=jwt, opener=opener)
    inst_id = inst.get("id") if isinstance(inst, dict) else None
    if not isinstance(inst_id, int):
        raise BeeError(f"the app is not installed on {repo}")
    tok = _request("POST", f"{API}/app/installations/{inst_id}/access_tokens", token=jwt,
                   body={"repositories": [repo.split("/", 1)[1]]}, opener=opener)
    token = tok.get("token") if isinstance(tok, dict) else None
    if not token:
        raise BeeError("GitHub returned no token")
    return token


# ---------------------------------------------------------------------------
# creating the app (owner, once)

def load_manifest(port=DEFAULT_PORT):
    m = json.loads(MANIFEST.read_text())
    m["redirect_url"] = f"http://127.0.0.1:{port}/callback"
    return m


def render_create_page(manifest, state):
    action = f"{NEW_APP_URL}?state={urllib.parse.quote(state)}"
    value = html.escape(json.dumps(manifest), quote=True)
    return f"""<!doctype html>
<meta charset="utf-8"><title>Create the t27-bees GitHub App</title>
<body style="font-family: system-ui; max-width: 40em; margin: 3em auto">
<h1>Create the <code>{html.escape(manifest["name"])}</code> GitHub App</h1>
<p>This posts <code>tools/bees/manifest.json</code> to GitHub. On the next page,
check the name and permissions, then press <b>Create GitHub App</b>. GitHub
sends you back here and the key is saved on this machine.</p>
<form id="f" action="{html.escape(action, quote=True)}" method="post">
<input type="hidden" name="manifest" value="{value}">
<button type="submit">Continue to GitHub</button>
</form>
</body>
"""


def save_conversion(conv, key_dir=None, keychain_write=_keychain_write):
    """Keep the id and the key; drop every other secret. Returns a printable summary."""
    key_dir = pathlib.Path(key_dir or KEY_DIR)
    app_id, slug, pem = conv.get("id"), conv.get("slug"), conv.get("pem")
    if not isinstance(app_id, int) or not slug or not pem or not re.fullmatch(r"[a-z0-9-]+", slug):
        raise BeeError("the conversion answer lacks id, slug or pem")
    key_dir.mkdir(parents=True, exist_ok=True)
    os.chmod(key_dir, 0o700)
    key_path = key_dir / f"{slug}.private-key.pem"
    if key_path.exists():
        raise BeeError(f"{key_path} already exists; move it aside first, nothing was overwritten")
    fd = os.open(key_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, "w") as fh:
        fh.write(pem)
    keychain_write(ACCOUNT_APP_ID, str(app_id))
    keychain_write(ACCOUNT_KEY_PATH, str(key_path))
    return (f"created GitHub App id {app_id}, slug {slug}, acting as {slug}[bot]\n"
            f"private key: {key_path} (mode 600; NOT in any repository)\n"
            f"Keychain: service {KEYCHAIN_SERVICE}, accounts {ACCOUNT_APP_ID} and {ACCOUNT_KEY_PATH}\n"
            f"next: install it -> https://github.com/apps/{slug}/installations/new")


def convert(code, opener=urllib.request.urlopen, key_dir=None, keychain_write=_keychain_write):
    if not re.fullmatch(r"[A-Za-z0-9_-]+", code or ""):
        raise BeeError("the code from the redirect is malformed")
    conv = _request("POST", f"{API}/app-manifests/{code}/conversions", opener=opener)
    return save_conversion(conv, key_dir=key_dir, keychain_write=keychain_write)


def make_handler(manifest, state, done, convert_fn=convert):
    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *a):  # the query string holds the code: do not log it
            pass

        def _send(self, code, body):
            data = body.encode()
            self.send_response(code)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self):
            u = urllib.parse.urlsplit(self.path)
            if u.path == "/":
                return self._send(200, render_create_page(manifest, state))
            if u.path != "/callback":
                return self._send(404, "not here")
            q = urllib.parse.parse_qs(u.query)
            if q.get("state", [""])[0] != state:
                return self._send(400, "state mismatch: this redirect was not started here")
            try:
                summary = convert_fn(q.get("code", [""])[0])
            except BeeError as e:
                done.append(("error", str(e)))
                return self._send(500, f"<pre>{html.escape(str(e))}</pre>")
            done.append(("ok", summary))
            return self._send(200, f"<pre>{html.escape(summary)}</pre><p>You can close this tab.</p>")
    return Handler


def cmd_create(args):
    manifest = load_manifest(args.port)
    state = secrets.token_urlsafe(24)
    done = []
    srv = http.server.HTTPServer(("127.0.0.1", args.port), make_handler(manifest, state, done))
    url = f"http://127.0.0.1:{args.port}/"
    print(f"open {url} if no browser window appears (Ctrl-C to abort)", file=sys.stderr)
    if not args.no_browser:
        subprocess.run(["open", url], capture_output=True)
    while not done:
        srv.handle_request()
    srv.server_close()
    kind, text = done[0]
    print(text, file=sys.stderr if kind == "error" else sys.stdout)
    return 0 if kind == "ok" else 1


# ---------------------------------------------------------------------------
# self-test: no network, no real secret

def self_test():
    failures = []

    def check(name, cond):
        print(f"  {'ok  ' if cond else 'FAIL'} {name}")
        if not cond:
            failures.append(name)

    tmp = pathlib.Path(tempfile.mkdtemp(prefix="bees-selftest-"))
    key = tmp / "throwaway.pem"
    pub = tmp / "throwaway.pub"
    subprocess.run(["openssl", "genrsa", "-out", str(key), "2048"], check=True, capture_output=True)
    subprocess.run(["openssl", "rsa", "-in", str(key), "-pubout", "-out", str(pub)],
                   check=True, capture_output=True)
    os.chmod(key, 0o600)

    def verify(jwt):
        signing_input, sig = jwt.rsplit(".", 1)
        sig_file = tmp / "sig"
        sig_file.write_bytes(b64url_decode(sig))
        r = subprocess.run(["openssl", "dgst", "-sha256", "-verify", str(pub), "-signature",
                            str(sig_file)], input=signing_input.encode(), capture_output=True)
        return r.returncode == 0

    # 1. JWT claims and signature
    now = 1_800_000_000
    jwt = make_jwt("123456", str(key), now=now)
    h, p, _ = jwt.split(".")
    header, claims = json.loads(b64url_decode(h)), json.loads(b64url_decode(p))
    check("header is RS256", header == {"alg": "RS256", "typ": "JWT"})
    check("iss is the app id", claims.get("iss") == "123456")
    check("iat is backdated", claims["iat"] == now - JWT_BACKDATE)
    check("window is at most ten minutes", 0 < claims["exp"] - claims["iat"] <= 600)
    check("exp is in the future", claims["exp"] > now)
    check("signature verifies with the throwaway public key", verify(jwt))
    forged = b64url(json.dumps({**claims, "iss": "999"}, separators=(",", ":")).encode())
    check("a tampered payload does NOT verify (negative control)",
          not verify(".".join([h, forged, jwt.rsplit(".", 1)[1]])))

    # 2. configuration: env beats Keychain, absence is an error
    kc = {ACCOUNT_APP_ID: "777", ACCOUNT_KEY_PATH: "/kc/key.pem"}.get
    check("Keychain is read when env is empty",
          load_config({}, lambda a: kc(a, "")) == ("777", "/kc/key.pem"))
    check("env wins over Keychain",
          load_config({ENV_APP_ID: "1", ENV_KEY_PATH: "/e.pem"}, lambda a: kc(a, ""))
          == ("1", "/e.pem"))
    try:
        load_config({}, lambda a: "")
        check("missing config refuses", False)
    except BeeError:
        check("missing config refuses", True)
    try:
        load_config({ENV_APP_ID: "abc", ENV_KEY_PATH: "/x"}, lambda a: "")
        check("non-numeric app id refuses", False)
    except BeeError:
        check("non-numeric app id refuses", True)

    # 3. key file permissions
    loose = tmp / "loose.pem"
    loose.write_text("x")
    os.chmod(loose, 0o644)
    try:
        check_key_file(str(loose))
        check("a group/world-readable key refuses", False)
    except BeeError:
        check("a group/world-readable key refuses", True)

    # 4. minting against a fake GitHub: token on success, scoped to one repo
    calls = []

    class FakeResp:
        def __init__(self, body):
            self.body = body

        def __enter__(self):
            return self

        def __exit__(self, *a):
            return False

        def read(self):
            return json.dumps(self.body).encode()

    def fake_opener(req):
        calls.append((req.get_method(), req.full_url, req.get_header("Authorization"),
                      req.data))
        if req.full_url.endswith("/repos/gHashTag/t27/installation"):
            return FakeResp({"id": 42})
        if req.full_url.endswith("/app/installations/42/access_tokens"):
            return FakeResp({"token": "ghs_FAKE_TOKEN_FOR_SELFTEST"})
        raise urllib.error.HTTPError(req.full_url, 404, "nf", {}, None)

    env = {ENV_APP_ID: "123456", ENV_KEY_PATH: str(key)}
    tok = mint_token("gHashTag/t27", env=env, keychain=lambda a: "", opener=fake_opener, now=now)
    check("mint returns the installation token", tok == "ghs_FAKE_TOKEN_FOR_SELFTEST")
    check("both calls carry a JWT signed by the key",
          len(calls) == 2 and all(c[2] and verify(c[2].split(" ", 1)[1]) for c in calls))
    check("the token is scoped to the one repository",
          bool(calls[1][3]) and json.loads(calls[1][3].decode()) == {"repositories": ["t27"]})
    try:
        mint_token("gHashTag/elsewhere", env=env, keychain=lambda a: "", opener=fake_opener, now=now)
        check("an app not installed on the repo refuses", False)
    except BeeError as e:
        check("an app not installed on the repo refuses", "404" in str(e))
    try:
        mint_token("not a repo", env=env, keychain=lambda a: "", opener=fake_opener)
        check("a malformed repo refuses", False)
    except BeeError:
        check("a malformed repo refuses", True)

    # 5. the remote decides the default repository
    def fake_git(url):
        return lambda argv, **kw: subprocess.CompletedProcess(argv, 0, url + "\n", "")
    check("ssh remote -> owner/name",
          default_repo(fake_git("git@github.com:gHashTag/trinity.git")) == "gHashTag/trinity")
    check("https remote -> owner/name",
          default_repo(fake_git("https://github.com/gHashTag/skills")) == "gHashTag/skills")
    check("no GitHub remote -> gHashTag/t27", default_repo(fake_git("")) == DEFAULT_REPO)

    # 6. the manifest and the page that posts it
    m = load_manifest(9999)
    check("manifest permissions are exactly the five asked for",
          m["default_permissions"] == {"pull_requests": "write", "contents": "read",
                                       "issues": "write", "checks": "read", "metadata": "read"})
    check("manifest has no live webhook", m["hook_attributes"]["active"] is False
          and m["default_events"] == [])
    check("manifest is private to its owner", m["public"] is False)
    check("redirect follows the port", m["redirect_url"] == "http://127.0.0.1:9999/callback")
    page = render_create_page(m, "S7ATE")
    check("page posts to the personal-account manifest URL",
          f'action="{NEW_APP_URL}?state=S7ATE"' in page)
    embedded = re.search(r'name="manifest" value="([^"]*)"', page)
    check("page embeds the manifest intact",
          embedded is not None and json.loads(html.unescape(embedded.group(1))) == m)

    # 7. conversion: key saved 0600, other secrets dropped, nothing overwritten
    written = {}
    conv = {"id": 31337, "slug": "t27-bees", "pem": "-----BEGIN RSA PRIVATE KEY-----\nSELFTEST\n",
            "client_secret": "CLIENT_SECRET_SELFTEST", "webhook_secret": "WEBHOOK_SECRET_SELFTEST"}
    summary = save_conversion(conv, key_dir=tmp / "keys",
                              keychain_write=lambda a, v: written.__setitem__(a, v))
    kp = tmp / "keys" / "t27-bees.private-key.pem"
    check("key file written with mode 600", kp.exists() and (kp.stat().st_mode & 0o777) == 0o600)
    check("Keychain gets the id and the PATH, not the key",
          written == {ACCOUNT_APP_ID: "31337", ACCOUNT_KEY_PATH: str(kp)})
    check("summary names the bot login", "t27-bees[bot]" in summary)
    check("summary carries no secret",
          not any(s in summary for s in ("SELFTEST\n", "CLIENT_SECRET", "WEBHOOK_SECRET", "BEGIN")))
    try:
        save_conversion(conv, key_dir=tmp / "keys", keychain_write=lambda a, v: None)
        check("an existing key is never overwritten", False)
    except BeeError:
        check("an existing key is never overwritten", True)

    # 8. the local callback rejects a redirect it did not start
    done = []
    Handler = make_handler(m, "GOOD", done, convert_fn=lambda c: "converted " + c)
    srv = http.server.HTTPServer(("127.0.0.1", 0), Handler)
    port = srv.server_address[1]

    def hit(path):
        import threading
        t = threading.Thread(target=srv.handle_request)
        t.start()
        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{port}{path}") as r:
                code = r.status
        except urllib.error.HTTPError as e:
            code = e.code
        t.join()
        return code

    check("wrong state is refused (400)", hit("/callback?code=abc&state=EVIL") == 400 and not done)
    check("right state converts", hit("/callback?code=abc&state=GOOD") == 200
          and done == [("ok", "converted abc")])
    srv.server_close()

    import shutil
    shutil.rmtree(tmp, ignore_errors=True)
    print(f"self-test: {len(failures)} failure(s)")
    return 1 if failures else 0


# ---------------------------------------------------------------------------

def main(argv=None):
    ap = argparse.ArgumentParser(prog="bees", description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("create", help="owner, once: create the app from manifest.json")
    c.add_argument("--port", type=int, default=DEFAULT_PORT)
    c.add_argument("--no-browser", action="store_true")
    v = sub.add_parser("convert", help="owner: convert a manifest code by hand")
    v.add_argument("code")
    t = sub.add_parser("token", help="bee: print a one-hour installation token")
    t.add_argument("--repo", default=None,
                   help=f"owner/name (default: ${ENV_REPO}, else the origin remote, else {DEFAULT_REPO})")
    sub.add_parser("self-test", help="no network, no real secrets")
    args = ap.parse_args(argv)
    try:
        if args.cmd == "create":
            return cmd_create(args)
        if args.cmd == "convert":
            print(convert(args.code))
            return 0
        if args.cmd == "token":
            repo = args.repo or os.environ.get(ENV_REPO) or default_repo()
            sys.stdout.write(mint_token(repo))
            return 0
        return self_test()
    except BeeError as e:
        print(f"bees: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
