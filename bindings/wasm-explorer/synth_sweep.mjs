// Synthesise every spec's generated Verilog with yosys and record what
// actually elaborates for an Artix-7.
//
// This is synthesis only -- RTL to netlist. It does NOT place, route, or
// produce a bitstream, and it does not touch a board. What it proves is
// narrower and still worth having: that the Verilog the compiler emits is
// accepted by a real synthesiser, plus a cell and LUT count per spec.
//
// Bitstream generation needs nextpnr-xilinx via the openXC7 docker image;
// neither is available on this host right now, so that stays unmeasured
// rather than guessed.

import { readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

const wasmPath = new URL('./target/wasm32-unknown-unknown/release/t27_wasm_explorer.wasm', import.meta.url)
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {})
const { memory, t27_alloc, t27_free, t27_analyze } = instance.exports

function analyze(src) {
  const b = Buffer.from(src, 'utf8')
  const p = t27_alloc(b.length)
  new Uint8Array(memory.buffer, p, b.length).set(b)
  const o = t27_analyze(p, b.length)
  const n = new DataView(memory.buffer).getUint32(o, true)
  const j = Buffer.from(new Uint8Array(memory.buffer, o + 4, n)).toString('utf8')
  t27_free(o, 4 + n)
  return JSON.parse(j)
}

const ROOT = '/Users/playom/t27'
const files = execFileSync('find', [
  ROOT, '-name', '*.t27', '-type', 'f',
  '-not', '-path', `${ROOT}/.git/*`,
  '-not', '-path', `${ROOT}/.claude/*`,
], { encoding: 'utf8' }).split('\n').filter(Boolean).sort()

const dir = mkdtempSync(join(tmpdir(), 't27-synth-'))

// Self-check: a design with known logic must come back with non-zero counts.
// The first version of this script reported 0 cells for everything because
// `-q` suppressed the stat report; a corpus number of zero is indistinguishable
// from a muzzled tool unless something is known to be non-zero.
{
  const probe = join(dir, 'probe.v')
  writeFileSync(probe, 'module adder8(input clk, input [7:0] a, input [7:0] b, output reg [8:0] s);\n  always @(posedge clk) s <= a + b;\nendmodule\n')
  const out = execFileSync('yosys', ['-p', `read_verilog ${probe}; synth_xilinx -nodsp; stat`], { encoding: 'utf8' })
  const luts = [...out.matchAll(/^\s+(\d+)\s+LUT[1-6]\s*$/gm)].reduce((a, m) => a + Number(m[1]), 0)
  const ffs = [...out.matchAll(/^\s+(\d+)\s+(?:FDRE|FDSE|FDCE|FDPE)\s*$/gm)].reduce((a, m) => a + Number(m[1]), 0)
  if (luts === 0 || ffs === 0) {
    console.error(`self-check FAILED: known-good adder reported ${luts} LUTs / ${ffs} FFs. Not measuring the corpus.`)
    process.exit(1)
  }
  console.error(`self-check ok: known-good adder -> ${luts} LUTs, ${ffs} FFs`)
}

const rows = []
let done = 0

for (const f of files) {
  const rel = f.replace(`${ROOT}/`, '')
  let v = null
  try {
    const r = analyze(readFileSync(f, 'utf8'))
    v = r.targets.verilog?.ok ? r.targets.verilog.code : null
  } catch { /* analysed elsewhere; nothing to synthesise here */ }
  if (!v) { rows.push({ rel, status: 'no-verilog' }); continue }

  const vf = join(dir, 'd.v')
  writeFileSync(vf, v)
  // -p read_verilog then synth_xilinx for the 7-series primitives. No -top:
  // let yosys pick, since module names vary and a wrong -top is a false fail.
  try {
    // NOT -q. Quiet mode suppresses the `stat` report itself, which made an
    // earlier run of this script report 0 cells for all 361 designs that
    // synthesised -- an instrument reading zero because it was gagged, not
    // because the designs were empty.
    //
    // The counts also read count-then-name ("8   LUT2"), not name-then-count.
    // Both were verified against a known-good adder that yosys reports as
    // 35 cells / 8 LUT2 / 9 FDRE / 3 CARRY4 before trusting any corpus number.
    const out = execFileSync('yosys', ['-p', `read_verilog ${vf}; synth_xilinx -nodsp; stat`], {
      encoding: 'utf8', timeout: 60000, stdio: ['ignore', 'pipe', 'pipe'],
    })
    const cells = [...out.matchAll(/^\s+(\d+)\s+cells\s*$/gm)].reduce((a, m) => a + Number(m[1]), 0)
    const luts = [...out.matchAll(/^\s+(\d+)\s+LUT[1-6]\s*$/gm)].reduce((a, m) => a + Number(m[1]), 0)
    const ffs = [...out.matchAll(/^\s+(\d+)\s+(?:FDRE|FDSE|FDCE|FDPE)\s*$/gm)].reduce((a, m) => a + Number(m[1]), 0)
    const carry = [...out.matchAll(/^\s+(\d+)\s+CARRY4\s*$/gm)].reduce((a, m) => a + Number(m[1]), 0)
    const lcs = [...out.matchAll(/Estimated number of LCs:\s+(\d+)/g)].reduce((a, m) => a + Number(m[1]), 0)
    rows.push({ rel, status: 'synth-ok', cells, luts, ffs, carry, lcs })
  } catch (e) {
    const msg = String(e.stderr || e.message || '').split('\n').filter(Boolean).pop() || 'unknown'
    rows.push({ rel, status: 'synth-fail', err: msg.slice(0, 160) })
  }
  if (++done % 50 === 0) console.error(`  ...${done} synthesised`)
}

rmSync(dir, { recursive: true, force: true })

const ok = rows.filter((r) => r.status === 'synth-ok')
const bad = rows.filter((r) => r.status === 'synth-fail')
const none = rows.filter((r) => r.status === 'no-verilog')

console.log(JSON.stringify({
  tool: execFileSync('yosys', ['-V'], { encoding: 'utf8' }).trim(),
  scanned: rows.length,
  synthesised: ok.length,
  failed: bad.length,
  noVerilog: none.length,
  totalLuts: ok.reduce((a, r) => a + r.luts, 0),
  totalFfs: ok.reduce((a, r) => a + r.ffs, 0),
  largest: ok.sort((a, b) => b.luts - a.luts).slice(0, 10).map((r) => ({ rel: r.rel, luts: r.luts, ffs: r.ffs, cells: r.cells })),
  failures: bad.slice(0, 25),
}, null, 1))
