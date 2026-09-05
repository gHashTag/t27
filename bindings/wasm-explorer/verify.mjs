// Verify the wasm bridge against real specs, in a real JS runtime.
// Usage: node verify.mjs <spec.t27> [...]
import { readFileSync } from 'node:fs'

const wasmPath = new URL('./target/wasm32-unknown-unknown/release/t27_wasm_explorer.wasm', import.meta.url)
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {})
const { memory, t27_alloc, t27_free, t27_analyze } = instance.exports

function analyze(source) {
  const bytes = new TextEncoder().encode(source)
  const inPtr = t27_alloc(bytes.length)
  new Uint8Array(memory.buffer, inPtr, bytes.length).set(bytes)
  const outPtr = t27_analyze(inPtr, bytes.length)
  const len = new DataView(memory.buffer).getUint32(outPtr, true)
  const json = new TextDecoder().decode(new Uint8Array(memory.buffer, outPtr + 4, len))
  t27_free(outPtr, 4 + len)
  return JSON.parse(json)
}

for (const path of process.argv.slice(2)) {
  const t0 = performance.now()
  const r = analyze(readFileSync(path, 'utf8'))
  const ms = (performance.now() - t0).toFixed(1)
  const tgt = Object.entries(r.targets).map(([k, v]) => `${k}:${v.ok ? v.bytes + 'B' : 'ERR'}`).join(' ')
  console.log(`\n${path}  (${ms}ms)`)
  console.log(`  src ${r.sourceLines}L/${r.sourceBytes}B  tokens ${r.tokenCount}  nodes ${r.nodeCount} depth ${r.astDepth} top ${r.topLevel}`)
  console.log(`  typecheck ok=${r.typecheck?.ok} errors=${r.typecheck?.errorCount}  hir=${r.hir.ok}`)
  console.log(`  discarded=${r.discarded.length} swallowed=${r.swallowed.length} lexBad=${r.lexerDiscarded.length}`)
  console.log(`  ${tgt}`)
  if (r.astError) console.log(`  AST ERROR: ${r.astError}`)
  if (r.discarded.length) console.log(`  e.g. discarded: ${JSON.stringify(r.discarded.slice(0, 2))}`)
}
