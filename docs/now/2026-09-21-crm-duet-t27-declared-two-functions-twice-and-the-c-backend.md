# NOW -- crm-duet.t27 declared two functions twice and the C backend emitted the redefinition (2026-09-21)

## crm-duet.t27 declared two functions twice and the C backend emitted the redefinition

- specs/automation/crm-duet.t27: state_after and retry_allowed were each declared twice, byte-identical. t27c check had been warning 'every backend rejects a redeclaration' and was right: gen-c exits 0 and emits both, and cc answers 'error: redefinition of state_after'. Deleted the second copy of each; typecheck goes 2 warnings -> 0 and that C error goes away. Fixes #4557.
