#!/usr/bin/env python3
"""t27b benchmarks and the clang differential (arm64 macOS).

  bench.py gen     N DIR          write DIR/bench.t27 and DIR/bench.c
  bench.py diff    N [N...]       t27b objects vs clang on every k_i, all inputs
  bench.py metrics RUNS N [N...]  compile pipelines (wall, cpu, rss) and phases
  bench.py runtime RUNS N [N...]  __text size and ns/call of every object
  bench.py phases  RUNS N [N...]  t27b --time phases: per-phase min and median
  bench.py build   RUNS [PKG]     clean release build of PKG (default t27b):
                                 wall, units; the last binary is kept as WORK/PKG

The synthetic program is N functions `k_i(x: u32) -> u32`, each an 8-trip
loop of u32 arithmetic, plus one `assert_eq` test per function -- the same
program in t27 and in C, so the C file is the reference implementation.

Environment: T27B (default <repo>/target/release/t27b), T27C (default
<repo>/target/release/t27c), WORK (default /tmp/t27b-bench),
ZIG_GLOBAL_CACHE_DIR (default WORK/zig-global, warm after the first run).
Needs clang; `metrics` also needs zig. Results go to WORK/*.json.
"""
import json
import os
import re
import resource
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
T27B = os.environ.get("T27B", f"{REPO}/target/release/t27b")
T27C = os.environ.get("T27C", f"{REPO}/target/release/t27c")
WORK = os.environ.get("WORK", "/tmp/t27b-bench")
ZIG_GLOBAL = os.environ.get("ZIG_GLOBAL_CACHE_DIR", f"{WORK}/zig-global")


# ------------------------------------------------------------------ inputs

def t27_src(n):
    fns = []
    for i in range(n):
        fns.append(f"""fn k{i}(x: u32) -> u32 {{
    var acc : u32 = x + {i};
    var i : u32 = 0;
    while (i < 8) {{
        acc = (acc * 3 + i) ^ (acc >> 2);
        i = i + 1;
    }}
    return acc;
}}
""")
    tests = "".join(f"test t{i} {{ assert_eq(k{i}(1), k{i}(1)); }}\n" for i in range(n))
    return "module Bench;\n\n" + "\n".join(fns) + "\n" + tests + "endmodule\n"


def c_src(n):
    out = ["#include <stdint.h>\n"]
    for i in range(n):
        out.append(f"""uint32_t k{i}(uint32_t x) {{
    uint32_t acc = x + {i};
    uint32_t i = 0;
    while (i < 8) {{
        acc = (acc * 3 + i) ^ (acc >> 2);
        i = i + 1;
    }}
    return acc;
}}
""")
    return "\n".join(out)


def gen(n, d):
    os.makedirs(d, exist_ok=True)
    open(f"{d}/bench.t27", "w").write(t27_src(n))
    open(f"{d}/bench.c", "w").write(c_src(n))
    return f"{d}/bench.t27", f"{d}/bench.c"


def need(*exes):
    for e in exes:
        if not os.access(e, os.X_OK):
            sys.exit(f"bench.py: {e} is not an executable file (set T27B / T27C)")


def sh(*cmd, **kw):
    r = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if r.returncode != 0:
        sys.exit(f"FAILED ({r.returncode}): {' '.join(cmd)}\n{r.stdout[-2000:]}\n{r.stderr[-2000:]}")
    return r


# ------------------------------------------------------- clang differential

DIFF_DRIVER = r"""
static sigjmp_buf env;
static void on_trap(int sig) { (void)sig; siglongjmp(env, 1); }
/* 1 and *v on a normal return, 0 on a trap (brk -> SIGTRAP, ubsan -> SIGILL) */
static int run(fn_t f, uint32_t x, uint32_t *v) {
    if (sigsetjmp(env, 1)) return 0;
    *v = f(x);
    return 1;
}
int main(void) {
    struct sigaction sa; memset(&sa, 0, sizeof sa);
    sa.sa_handler = on_trap; sigemptyset(&sa.sa_mask);
    sigaction(SIGTRAP, &sa, 0); sigaction(SIGILL, &sa, 0);
    uint32_t in[1005] = {0u, 1u, 7u, 0x7FFFFFFFu, 0xFFFFFFFFu};
    uint64_t s = 0x9E3779B97F4A7C15ull;
    for (int i = 5; i < 1005; i++) { s ^= s << 13; s ^= s >> 7; s ^= s << 17; in[i] = (uint32_t)s; }
    long calls = 0, mism = 0, traps = 0;
    for (unsigned k = 0; k < sizeof T / sizeof T[0]; k++) {
        for (int i = 0; i < 1005; i++) {
            uint32_t a = 0, b = 0;
            int oa = run(T[k], in[i], &a), ob = run(R[k], in[i], &b);
            calls++;
            if (!oa) traps++;
            if (oa != ob || (oa && a != b)) {
                if (mism < 10) printf("MISMATCH k%u(%u): t27b %s %u, clang %s %u\n", k, in[i],
                    oa ? "ret" : "trap", a, ob ? "ret" : "trap", b);
                mism++;
            }
        }
    }
    printf("functions=%lu inputs=1005 calls=%ld traps=%ld mismatches=%ld\n",
        (unsigned long)(sizeof T / sizeof T[0]), calls, traps, mism);
    return mism != 0;
}
"""


def diff(ns):
    """t27b --overflow wrap vs clang -O2, and t27b trap mode vs clang -O0 with
    -fsanitize=...-overflow,shift -fsanitize-trap: same value or both trap."""
    ok = True
    for n in ns:
        d = f"{WORK}/diff{n}"
        src, csrc = gen(n, d)
        with open(f"{d}/ren.h", "w") as f:
            f.writelines(f"#define k{i} ref_k{i}\n" for i in range(n))
        with open(f"{d}/drv.c", "w") as f:
            f.write("#include <stdint.h>\n#include <stdio.h>\n#include <setjmp.h>\n#include <signal.h>\n"
                    "#include <string.h>\ntypedef uint32_t (*fn_t)(uint32_t);\n")
            f.writelines(f"uint32_t k{i}(uint32_t); uint32_t ref_k{i}(uint32_t);\n" for i in range(n))
            f.write("static fn_t T[] = {" + ",".join(f"k{i}" for i in range(n)) + "};\n")
            f.write("static fn_t R[] = {" + ",".join(f"ref_k{i}" for i in range(n)) + "};\n")
            f.write(DIFF_DRIVER)
        sh(T27B, "build", src, "-o", f"{d}/wrap.o", "--overflow", "wrap")
        sh(T27B, "build", src, "-o", f"{d}/trap.o")
        sh("clang", "-O2", "-c", "-include", f"{d}/ren.h", csrc, "-o", f"{d}/ref_o2.o")
        sh("clang", "-O0", "-c", "-include", f"{d}/ren.h",
           "-fsanitize=unsigned-integer-overflow,signed-integer-overflow,shift",
           "-fsanitize-trap=all", csrc, "-o", f"{d}/ref_san.o")
        sh("clang", "-O1", f"{d}/drv.c", f"{d}/wrap.o", f"{d}/ref_o2.o", "-o", f"{d}/dw")
        sh("clang", "-O1", f"{d}/drv.c", f"{d}/trap.o", f"{d}/ref_san.o", "-o", f"{d}/dt")
        for tag, exe in [("wrap vs clang -O2", "dw"), ("trap vs clang -O0 ubsan-trap", "dt")]:
            r = subprocess.run([f"{d}/{exe}"], capture_output=True, text=True)
            print(f"n={n} {tag}: exit={r.returncode} {r.stdout.strip()}", flush=True)
            ok = ok and r.returncode == 0
    return 0 if ok else 1


# ----------------------------------------------------------------- metrics

def timed(cmd, cwd, stdout_path=None, env=None):
    """Wall ms, max RSS MB (/usr/bin/time -l), user+sys ms of the command
    and every descendant it waited for (getrusage, microsecond resolution)."""
    out = open(stdout_path, "w") if stdout_path else subprocess.PIPE
    r0 = resource.getrusage(resource.RUSAGE_CHILDREN)
    t0 = time.perf_counter()
    p = subprocess.run(["/usr/bin/time", "-l"] + cmd, cwd=cwd, env=env, stdout=out,
                       stderr=subprocess.PIPE, text=True)
    wall = (time.perf_counter() - t0) * 1e3
    r1 = resource.getrusage(resource.RUSAGE_CHILDREN)
    if stdout_path:
        out.close()
    cpu = ((r1.ru_utime - r0.ru_utime) + (r1.ru_stime - r0.ru_stime)) * 1e3
    m = re.search(r"(\d+)\s+maximum resident set size", p.stderr)
    rss = int(m.group(1)) / 1e6 if m else None
    return p.returncode, wall, rss, cpu, (p.stdout or ""), p.stderr


def phases(stderr):
    m = re.search(r"t27b phases \(ms\):(.*)", stderr)
    return {k: float(v) for k, v in re.findall(r"(\S+)=([0-9.]+)", m.group(1))} if m else {}


def compile_cases(n, src, csrc, t):
    zenv = dict(os.environ, ZIG_GLOBAL_CACHE_DIR=ZIG_GLOBAL, ZIG_LOCAL_CACHE_DIR=f"{t}/zc")
    passed = f"{n} passed, 0 failed"
    return [
        ("t27b test", [T27B, "test", src, "-q", "--time"], None, None, lambda o: passed in o),
        ("t27c gen", [T27C, "gen", src], f"{t}/g.zig", None, None),
        ("zig test g.zig", ["zig", "test", f"{t}/g.zig"], None, zenv, None),
        ("t27c gen-c", [T27C, "gen-c", src], f"{t}/g.c", None, None),
        ("clang -O0 -DT27_TEST_MAIN g.c (link)",
         ["clang", "-O0", "-w", "-DT27_TEST_MAIN", f"{t}/g.c", "-o", f"{t}/ctest"], None, None, None),
        ("./ctest", [f"{t}/ctest"], None, None, lambda o: f"All {n} tests passed" in o),
        ("t27b build trap", [T27B, "build", src, "-o", f"{t}/b_trap.o", "--time"], None, None, None),
        ("t27b build wrap", [T27B, "build", src, "-o", f"{t}/b_wrap.o", "--overflow", "wrap", "--time"],
         None, None, None),
        ("clang -O0 -c g.c", ["clang", "-O0", "-w", "-c", f"{t}/g.c", "-o", f"{t}/g_o0.o"], None, None, None),
        ("clang -O2 -c g.c", ["clang", "-O2", "-w", "-c", f"{t}/g.c", "-o", f"{t}/g_o2.o"], None, None, None),
        ("clang -O0 -c bench.c", ["clang", "-O0", "-c", csrc, "-o", f"{t}/c_o0.o"], None, None, None),
        ("clang -O2 -c bench.c", ["clang", "-O2", "-c", csrc, "-o", f"{t}/c_o2.o"], None, None, None),
    ]


OBJECTS = {"t27b trap": "b_trap.o", "t27b wrap": "b_wrap.o", "t27c gen-c + clang -O0": "g_o0.o",
           "t27c gen-c + clang -O2": "g_o2.o", "bench.c clang -O0": "c_o0.o", "bench.c clang -O2": "c_o2.o"}

TIMING_DRIVER = r"""
/* Thread CPU time, not wall time: on a loaded machine the time this thread
   is descheduled is not the callee's. QoS user-interactive asks for a
   performance core. Each process takes SAMPLES timed samples. */
static double now(void) { struct timespec t; clock_gettime(CLOCK_THREAD_CPUTIME_ID, &t); return t.tv_sec * 1e9 + t.tv_nsec; }
static int cmp(const void *a, const void *b) { double x = *(const double *)a, y = *(const double *)b; return (x > y) - (x < y); }
#define SAMPLES 15
int main(int argc, char **argv) {
    pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
    long reps = argc > 1 ? atol(argv[1]) : 1;
    const unsigned N = sizeof T / sizeof T[0];
    uint32_t s = 0;
    /* inputs 0..1023 never overflow u32 in k_i, so trap and wrap builds agree */
    for (unsigned k = 0; k < N; k++) for (uint32_t j = 0; j < 1024; j++) s += T[k](j);
    uint32_t check = s;
    double ns[SAMPLES];
    for (int q = 0; q < SAMPLES; q++) {
        double t0 = now();
        for (long r = 0; r < reps; r++)
            for (unsigned k = 0; k < N; k++)
                for (uint32_t j = 0; j < 1024; j++) s += T[k](j);
        ns[q] = (now() - t0) / ((double)reps * N * 1024);
    }
    qsort(ns, SAMPLES, sizeof ns[0], cmp);
    printf("ns_min=%.4f ns_median=%.4f checksum=%u calls=%ld sink=%u\n",
        ns[0], ns[SAMPLES / 2], check, reps * N * 1024L, s);
    return 0;
}
"""


def text_size(obj):
    out = subprocess.run(["size", "-m", obj], capture_output=True, text=True).stdout
    m = re.search(r"Section \(__TEXT, __text\): (\d+)", out)
    return int(m.group(1)) if m else None


def metrics(runs, ns):
    res = {"runs": runs, "t27b": T27B, "t27c": T27C, "sizes": {}}
    for n in ns:
        src, csrc = gen(n, f"{WORK}/syn{n}")
        rows, ph, loads, keep = {}, {"t27b test": [], "t27b build trap": []}, [os.getloadavg()[0]], None
        # 1 warmup + `runs` measured; the cases are interleaved inside each run
        for k in range(runs + 1):
            t = tempfile.mkdtemp(prefix=f"n{n}_", dir=WORK)
            for label, cmd, so, env, check in compile_cases(n, src, csrc, t):
                rc, wall, rss, cpu, out, err = timed(cmd, WORK, so, env)
                for _ in range(3):
                    # A full disk is the machine's problem, not the compiler's:
                    # say so, wait, and measure again.
                    if rc == 0 or "No space left on device" not in out + err:
                        break
                    print(f"  note: n={n} {label}: no space left on device, retrying in 30 s", flush=True)
                    time.sleep(30)
                    rc, wall, rss, cpu, out, err = timed(cmd, WORK, so, env)
                if rc != 0 or (check and not check(out + err)):
                    shutil.rmtree(t, ignore_errors=True)
                    sys.exit(f"FAIL n={n} {label}: rc={rc}\n{out[-800:]}\n{err[-800:]}")
                if k > 0:
                    v = rows.setdefault(label, {"wall": [], "rss": [], "cpu": []})
                    v["wall"].append(wall)
                    v["rss"].append(rss)
                    v["cpu"].append(cpu)
                    if label in ph:
                        ph[label].append(phases(err))
            shutil.rmtree(f"{t}/zc", ignore_errors=True)
            if keep:
                shutil.rmtree(keep, ignore_errors=True)
            keep = t
            loads.append(os.getloadavg()[0])
        compile_ = {label: {"median_ms": statistics.median(v["wall"]), "min_ms": min(v["wall"]),
                            "cpu_ms": statistics.median(v["cpu"]), "rss_mb": statistics.median(v["rss"])}
                    for label, v in rows.items()}
        phase_med = {label: {kk: statistics.median(d[kk] for d in lst) for kk in lst[0]}
                     for label, lst in ph.items()}
        shutil.rmtree(keep, ignore_errors=True)
        res["sizes"][n] = {"compile": compile_, "phases": phase_med, "load_avg_1m": loads}
        json.dump(res, open(f"{WORK}/metrics.json", "w"), indent=1)
        print(f"== n={n}  load average {min(loads):.0f}..{max(loads):.0f}")
        for label, s in compile_.items():
            print(f"  {label:38s} wall median {s['median_ms']:9.1f} ms  min {s['min_ms']:9.1f}"
                  f"  cpu {s['cpu_ms']:9.2f} ms  rss {s['rss_mb']:7.1f} MB")
        for label, p in phase_med.items():
            print(f"  phases {label}: " + " ".join(f"{a}={b:.2f}" for a, b in p.items()))
        sys.stdout.flush()
    return 0


def phase_runs(runs, ns):
    """`t27b test` and `t27b build` with --time, RUNS times each. The phases
    are wall-clock inside one process, so on a loaded machine the per-phase
    minimum is the better estimate of what the phase costs."""
    res = {"runs": runs, "sizes": {}}
    for n in ns:
        src, _ = gen(n, f"{WORK}/syn{n}")
        load0 = os.getloadavg()[0]
        out = {}
        for label, cmd in [("t27b test", [T27B, "test", src, "-q", "--time"]),
                           ("t27b build", [T27B, "build", src, "-o", f"{WORK}/phases.o", "--time"])]:
            got = [phases(sh(*cmd).stderr) for _ in range(runs)]
            out[label] = {k: {"min": min(g[k] for g in got), "median": statistics.median(g[k] for g in got)}
                          for k in got[0]}
        os.remove(f"{WORK}/phases.o")
        res["sizes"][n] = {"phases": out, "load_avg_1m": [load0, os.getloadavg()[0]]}
        json.dump(res, open(f"{WORK}/phases.json", "w"), indent=1)
        print(f"== n={n}  load average {load0:.0f}..{os.getloadavg()[0]:.0f}  ({runs} runs; ms, min / median)")
        for label, p in out.items():
            print(f"  {label:10s} " + " ".join(f"{k}={v['min']:.2f}/{v['median']:.2f}" for k, v in p.items()))
        sys.stdout.flush()
    return 0


def runtime(runs, ns):
    """Build each object once, then time it through one driver (a function
    pointer table, so nothing is inlined across objects). Every object must
    produce the same checksum."""
    res = {"runs": runs, "sizes": {}}
    for n in ns:
        src, csrc = gen(n, f"{WORK}/syn{n}")
        t = tempfile.mkdtemp(prefix=f"rt{n}_", dir=WORK)
        with open(f"{t}/g.c", "w") as f:
            f.write(sh(T27C, "gen-c", src).stdout)
        sh(T27B, "build", src, "-o", f"{t}/b_trap.o")
        sh(T27B, "build", src, "-o", f"{t}/b_wrap.o", "--overflow", "wrap")
        for opt in ("0", "2"):
            sh("clang", f"-O{opt}", "-w", "-c", f"{t}/g.c", "-o", f"{t}/g_o{opt}.o")
            sh("clang", f"-O{opt}", "-c", csrc, "-o", f"{t}/c_o{opt}.o")
        with open(f"{t}/tdrv.c", "w") as f:
            f.write("#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <time.h>\n"
                    "#include <pthread.h>\ntypedef uint32_t (*fn_t)(uint32_t);\n")
            f.writelines(f"uint32_t k{i}(uint32_t);\n" for i in range(n))
            f.write("static fn_t T[] = {" + ",".join(f"k{i}" for i in range(n)) + "};\n")
            f.write(TIMING_DRIVER)
        sh("clang", "-O2", "-c", f"{t}/tdrv.c", "-o", f"{t}/tdrv.o")
        reps = max(1, 2_000_000 // (n * 1024))
        out = {}
        load0 = os.getloadavg()[0]
        for k, o in OBJECTS.items():
            exe = f"{t}/x_{o[:-2]}"
            sh("clang", f"{t}/tdrv.o", f"{t}/{o}", "-o", exe)
            mins, meds, sums = [], [], set()
            for _ in range(runs):
                m = re.search(r"ns_min=([0-9.]+) ns_median=([0-9.]+) checksum=(\d+) calls=(\d+)",
                              sh(exe, str(reps)).stdout)
                mins.append(float(m.group(1)))
                meds.append(float(m.group(2)))
                sums.add(int(m.group(3)))
            out[k] = {"text": text_size(f"{t}/{o}"), "file": os.path.getsize(f"{t}/{o}"),
                      "ns_min": min(mins), "ns_median": statistics.median(meds),
                      "checksums": sorted(sums), "calls_per_sample": int(m.group(4))}
        shutil.rmtree(t, ignore_errors=True)
        res["sizes"][n] = {"objects": out, "load_avg_1m": [load0, os.getloadavg()[0]]}
        json.dump(res, open(f"{WORK}/runtime.json", "w"), indent=1)
        print(f"== n={n}  load average {load0:.0f}..{os.getloadavg()[0]:.0f}  "
              f"({runs} processes x 15 samples x {out['t27b trap']['calls_per_sample']} calls)")
        for k, r in out.items():
            print(f"  {k:24s} __text {r['text']:8d} B  file {r['file']:8d} B  ns/call min {r['ns_min']:7.3f}"
                  f"  median {r['ns_median']:7.3f}  checksum {r['checksums']}")
        sys.stdout.flush()
    return 0


# -------------------------------------------------------------- clean build

def build(runs, pkg):
    walls, units = [], []
    for k in range(runs):
        tgt = f"{WORK}/cargo{k}"
        shutil.rmtree(tgt, ignore_errors=True)
        env = dict(os.environ, CARGO_TARGET_DIR=tgt)
        t0 = time.perf_counter()
        p = sh("cargo", "build", "--release", "-p", pkg, "--timings", cwd=REPO, env=env)
        wall = time.perf_counter() - t0
        html = [f for f in os.listdir(f"{tgt}/cargo-timings") if f.endswith(".html")][0]
        u = re.search(r"Total units:</td><td>(\d+)", open(f"{tgt}/cargo-timings/{html}").read())
        crates = len(re.findall(r"^\s+Compiling ", p.stderr, re.M))
        size = os.path.getsize(f"{tgt}/release/{pkg}")
        print(f"run {k}: {wall:.2f} s, units {u.group(1) if u else '?'}, crates {crates}, "
              f"binary {size} B, load {os.getloadavg()[0]:.0f}", flush=True)
        walls.append(wall)
        if u:
            units.append(int(u.group(1)))
        shutil.copy2(f"{tgt}/release/{pkg}", f"{WORK}/{pkg}")
        shutil.rmtree(tgt, ignore_errors=True)
    print(f"median {statistics.median(walls):.2f} s, min {min(walls):.2f} s, units {sorted(set(units))}")
    return 0


def main(a):
    os.makedirs(WORK, exist_ok=True)
    if len(a) == 3 and a[0] == "gen":
        gen(int(a[1]), a[2])
        return 0
    if len(a) >= 2 and a[0] == "diff":
        need(T27B)
        return diff([int(x) for x in a[1:]])
    if len(a) >= 3 and a[0] == "metrics":
        need(T27B, T27C)
        return metrics(int(a[1]), [int(x) for x in a[2:]])
    if len(a) >= 3 and a[0] == "phases":
        need(T27B)
        return phase_runs(int(a[1]), [int(x) for x in a[2:]])
    if len(a) >= 3 and a[0] == "runtime":
        need(T27B, T27C)
        return runtime(int(a[1]), [int(x) for x in a[2:]])
    if len(a) in (2, 3) and a[0] == "build":
        return build(int(a[1]), a[2] if len(a) == 3 else "t27b")
    print(__doc__)
    return 64


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
