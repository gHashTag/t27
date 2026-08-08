# Vivado closure kit — kills the seed-lottery with a real multicycle constraint

The open flow (yosys → nextpnr-xilinx → prjxray) **cannot express a multicycle timing
constraint** — nextpnr-xilinx's XDC parser supports only `create_clock`. So the deep
shared-core path (`rf → GftSmul/GftSadd → rf`) is left timing-relaxed and correctness
becomes placement-dependent: some `--seed` values train, most glitch (the "seed-lottery",
characterised across 12 measured dead ends in `SILICON_TRAINING_METHODOLOGY.md`).

Commercial P&R closes it directly. This kit is the minimal delta:

- **`bpseq_vivado.xdc`** — `create_clock` at 200 MHz **plus** `set_multicycle_path 16`
  on the `rf → rf` (through-core) paths. That path is genuinely multicycle: the
  microsequencer captures its result only once per `cen × settle` (~2560) cycles, so it
  never needs 200 MHz single-cycle setup. Vivado then closes it deterministically.
- **`vivado_build.tcl`** — batch synth → opt → place → phys_opt → route → bitstream,
  printing the worst setup slack (expect ≥ 0) and writing `bpseq_vivado.bit`.

## Run

```
# copy the RTL next to this kit:
cp ../bpseq_capstone.v bpseq.v
cp ../uart_bpseq.v ../gft_smul.v ../gft_sadd.v .
vivado -mode batch -source vivado_build.tcl
```

Then flash the result exactly like the open-flow bitstream — but **no seed-search**:

```
openFPGALoader -c digilent_hs2 --busdev-num 002:002 bpseq_vivado.bit
python3 ../drive_bpseq.py /dev/cu.usbserial-2120
```

## Why this is the fix (and the open flow is not)

Every *local* fix failed on silicon (endpoint / mid-cloud / write-control registration —
the fault is a global clock/placement effect of the relaxed placement). The cure is not a
different register; it is **telling the tool the truth about the path** so a timing-driven
placer closes it. Vivado can hear that constraint; nextpnr-xilinx cannot. This converts the
documented open-toolchain limitation into a solved problem with the right instrument, and it
is the prerequisite for training nets larger than XOR (where open-flow seed-search runs out).
