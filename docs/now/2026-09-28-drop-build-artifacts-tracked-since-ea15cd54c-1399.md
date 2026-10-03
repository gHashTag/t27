# NOW -- Drop build artifacts tracked since ea15cd54c (#1399) (2026-09-28)

## Drop build artifacts tracked since ea15cd54c (#1399) (Closes #5077)

- remove 4 tracked build artifacts born in the Jul-31 artifact sweep: bootstrap/__pycache__/t27c.cpython-314.pyc, bootstrap/libchimera_engine.rlib, bootstrap/main.zig, bootstrap/test (442KB Mach-O arm64)
- .gitignore: ignore *.rlib so stray pre-workspace library builds stay untracked
