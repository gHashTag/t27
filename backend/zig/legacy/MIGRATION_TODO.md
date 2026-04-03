# Legacy Zig Migration TODO

**Status**: Pending Migration
**Created**: 2026-04-04
**Policy**: ADR-005 De-Zig Strict (SOUL Law #4)

## Files in Quarantine

| File | Original Location | Domain Logic | Migration Target |
|------|------------------|--------------|-----------------|
| `main_zig_handwritten.t27` | `src/tri/main.zig` | CLI runtime, commands, validation | `compiler/runtime/*.t27` → generated Zig |

## Migration Tasks

1. **main_zig_handwritten.t27** → generated from `compiler/runtime/runtime.t27`
   - Current: 1126 lines of handwritten Zig CLI runtime
   - Target: Generate from `compiler/codegen/zig/runtime.t27`
   - Status: Placeholder created in `src/tri/main.zig` with GENERATED-HEADER-POLICY header
   - Remaining: Full codegen implementation

## Migration Steps

1. Implement `tri gen compiler/runtime/runtime.t27` to produce Zig
2. Replace placeholder `src/tri/main.zig` with generated code
3. Verify generated code has DO NOT EDIT header
4. Test CLI functionality
5. Delete this quarantine file when complete

## Notes

- All domain logic MUST be specified in `.t27` files first
- Zig is ONLY for generated backends (with DO NOT EDIT header)
- Bootstrap I/O layer is permitted (file I/O, process startup only)
- See: `docs/GENERATED-HEADER-POLICY.md`
