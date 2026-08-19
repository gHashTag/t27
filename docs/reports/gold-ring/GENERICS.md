# Design note: generics — the one real language question left

27 files fail on one construct, and 11 of them are a LIVE collections library:

    pub const Map(K, V) = struct { ... }
    pub const Stack(T)  = struct { ... }      (tri/collections/*, tri/pipeline)

Everything else in the dialect map is either documents (117, classify-manifest,
no ring), small forms (Rust-flavoured `let mut`/`impl`/`1..=`, 12 files), or
already shipped (0001/0002). Generics are the only feature where a DESIGN choice
precedes any patch.

## The three ways, priced

**1. Monomorphisation at parse time.** `Map(K,V)` becomes a template; each
instantiation `Map(u32, Str)` stamps a concrete struct. Matches Zig's comptime
model the corpus already leans on; every backend sees only concrete types.
Cost: instantiation table + name mangling in the compiler (~medium, one ring
proposal); no runtime cost; code size grows with instantiations.

**2. Type erasure / void-pointer core.** One compiled body; K/V handled by
size+copy fns. Smallest compiler change, worst fit for the Verilog backend
(no pointers on silicon) — the library would stop being synthesizable, which
tri/collections arguably must be.

**3. Reject: rewrite the library concretely.** Eleven files × the handful of
instantiations actually used (measure first!). No compiler change at all. If
the real instantiation count is small (e.g. Map appears only as Map(Str,Str)
and Map(u32,Node)), this is a day of mechanical work and zero ring risk.

## The measurement that decides it

Count actual instantiations across the corpus before choosing: if the set is
small and closed, option 3 wins outright; if it is open or user-extensible,
option 1 is the only one that keeps Verilog. Option 2 is priced for
completeness and recommended against.

*Next concrete step: grep the corpus for `Map(`, `Stack(`, `Queue(` … call
sites and publish the instantiation table beside this note.*
