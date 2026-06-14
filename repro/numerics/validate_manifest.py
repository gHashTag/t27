#!/usr/bin/env python3
"""
Validate an NMSE results manifest against schemas/nmse-protocol-v1.json.

This is the conformance gate for the *certifying* manifest produced by
nmse_gf16.py --emit-protocol-v1. The rich, human-oriented manifest
(nmse_manifest.json) deliberately carries extra fields (D_WIDE, ULP, overflow
rates, etc.) and is NOT schema-bound; only the protocol-v1 manifest is.

R5-HONEST: passing this validator only proves the manifest is well-formed and
that seal_hash is either the FROZEN_HASH digest or the literal "unsealed". It
does NOT certify silicon; an "unsealed" manifest is informational only.

Usage:
  python repro/numerics/validate_manifest.py [MANIFEST_JSON]
Exit 0 if the manifest conforms; non-zero (with a diagnostic) otherwise.

Anchor: phi^2 + phi^-2 = 3
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
SCHEMA_PATH = os.path.join(REPO, "schemas", "nmse-protocol-v1.json")
DEFAULT_MANIFEST = os.path.join(HERE, "nmse_manifest_protocol_v1.json")
FROZEN_HASH_PATH = os.path.join(REPO, "bootstrap", "stage0", "FROZEN_HASH")


def frozen_digest():
    """Return the 64-hex digest recorded in bootstrap/stage0/FROZEN_HASH, or None."""
    try:
        with open(FROZEN_HASH_PATH) as f:
            tok = f.read().split()[0].strip()
        if len(tok) == 64 and all(c in "0123456789abcdef" for c in tok):
            return tok
    except Exception:
        pass
    return None


def main():
    try:
        import jsonschema
    except Exception as exc:  # pragma: no cover
        print("ERROR: jsonschema is required:", exc)
        return 2

    manifest_path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_MANIFEST
    if not os.path.exists(manifest_path):
        print("ERROR: manifest not found:", manifest_path)
        return 2

    with open(SCHEMA_PATH) as f:
        schema = json.load(f)
    with open(manifest_path) as f:
        manifest = json.load(f)

    try:
        jsonschema.validate(instance=manifest, schema=schema)
    except jsonschema.ValidationError as err:
        print("SCHEMA VIOLATION at", list(err.absolute_path), "->", err.message)
        return 1

    # Honesty cross-check: if seal_hash is not "unsealed", it must equal the
    # recorded FROZEN_HASH digest. The schema accepts any 64-hex; we are stricter.
    seal = manifest["seal_hash"]
    if seal != "unsealed":
        fd = frozen_digest()
        if fd is None:
            print("ERROR: seal_hash claims a seal but FROZEN_HASH is unreadable")
            return 1
        if seal != fd:
            print("HONESTY VIOLATION: seal_hash", seal[:12], "!= FROZEN_HASH",
                  (fd or "?")[:12])
            return 1
        print("seal_hash matches FROZEN_HASH digest (sealed manifest).")
    else:
        print("seal_hash = unsealed (informational manifest; not a silicon claim).")

    print("OK:", os.path.relpath(manifest_path, REPO),
          "conforms to schemas/nmse-protocol-v1.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
