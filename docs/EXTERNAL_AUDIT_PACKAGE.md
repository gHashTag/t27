# External audit package — ~1 hour review path

**For:** Senior reviewers who will **not** read the entire monorepo.

---

## Five claims to validate first

1. **SSOT:** Product math lives in `.t27` and is checked by `t27c` + CI — see `docs/T27-CONSTITUTION.md`, `docs/RESEARCH_CLAIMS.md` row 1.  
2. **Integrity:** Bootstrap core is sealed — `FROZEN.md`, `stage0/FROZEN_HASH`, `cargo build` in `bootstrap/`.  
3. **Conformance:** JSON vectors — `conformance/`, `tests/validate_conformance.sh`.  
4. **Generated code discipline:** `gen/` headers — `tests/validate_gen_headers.sh`.  
5. **Honesty about limits:** `docs/STATE_OF_THE_PROJECT.md`, `docs/WHAT_REMAINS_SPECULATIVE.md`.

---

## Ten files (priority reading order)

1. `docs/REPO_MAP.md`  
2. `docs/RESEARCH_CLAIMS.md`  
3. `docs/T27-CONSTITUTION.md`  
4. `docs/ARCHITECTURE.md`  
5. `CANON.md`  
6. `FROZEN.md`  
7. `docs/STATE_OF_THE_PROJECT.md`  
8. `docs/NUMERIC-STANDARD-001.md`  
9. `specs/base/types.t27` (sample SOOT)  
10. `architecture/ADR-005-de-zig-strict.md`  

---

## Three commands

```bash
cd bootstrap && cargo build --release
cd .. && ./bootstrap/target/release/t27c compile-all
bash tests/run_all.sh && bash tests/validate_conformance.sh && bash tests/validate_gen_headers.sh
```

Or: `make -C repro repro-smoke` (see `repro/README.md`).

---

## Five known limitations (ask us if these worry you)

1. Formal **full-language** semantics is a **skeleton** (`docs/LANGUAGE_SPEC.md`).  
2. Cross-backend **bit-exact** equivalence is **not** guaranteed yet.  
3. Parser **fuzzing** is not yet flagship-grade.  
4. Some **physics-flavored** specs mix reference and empirical models — labels in progress.  
5. Rings **32–35** hardening explicitly **in progress**.

---

*If this package is insufficient, tell us which discipline you represent — we will add a 30-minute add-on path.*
