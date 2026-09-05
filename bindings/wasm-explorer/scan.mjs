// Scan the whole corpus through the wasm bridge: how many specs lose content
// to parser error recovery, and how many backends reject each spec.
import { readFileSync } from 'node:fs'
import { execFileSync } from 'node:child_process'

const wasmPath = new URL('./target/wasm32-unknown-unknown/release/t27_wasm_explorer.wasm', import.meta.url)
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {})
const { memory, t27_alloc, t27_free, t27_analyze } = instance.exports

function analyze(source) {
  const b = new TextEncoder().encode(source)
  const p = t27_alloc(b.length)
  new Uint8Array(memory.buffer, p, b.length).set(b)
  const o = t27_analyze(p, b.length)
  const n = new DataView(memory.buffer).getUint32(o, true)
  const j = new TextDecoder().decode(new Uint8Array(memory.buffer, o + 4, n))
  t27_free(o, 4 + n)
  return JSON.parse(j)
}

const files = execFileSync('find', ['/Users/playom/t27/specs', '-name', '*.t27'], { encoding: 'utf8' })
  .split('\n').filter(Boolean).sort()

let lossy = 0, astFail = 0, tcFail = 0
const targetFail = {}
const worst = []
let totalMs = 0

for (const f of files) {
  const src = readFileSync(f, 'utf8')
  const t0 = performance.now()
  let r
  try { r = analyze(src) } catch (e) { console.log(`CRASH ${f}: ${e}`); continue }
  totalMs += performance.now() - t0

  const loss = r.discarded.length + r.swallowed.length + r.lexerDiscarded.length
  if (loss > 0) { lossy++; worst.push([loss, f, r.discarded.length, r.swallowed.length, r.lexerDiscarded.length]) }
  if (r.astError) astFail++
  if (r.typecheck && r.typecheck.ok === false) tcFail++
  for (const [k, v] of Object.entries(r.targets)) if (!v.ok) targetFail[k] = (targetFail[k] || 0) + 1
}

console.log(`specs scanned:        ${files.length}`)
console.log(`total analyse time:   ${(totalMs / 1000).toFixed(1)}s  (avg ${(totalMs / files.length).toFixed(0)}ms)`)
console.log(`AST parse failures:   ${astFail}`)
console.log(`typecheck failures:   ${tcFail}`)
console.log(`specs losing content: ${lossy}  (${(100 * lossy / files.length).toFixed(1)}%)`)
console.log(`backend rejections:   ${JSON.stringify(targetFail)}`)
worst.sort((a, b) => b[0] - a[0])
console.log(`\nworst 8 by dropped content (total, discarded, swallowed, lexer):`)
for (const [n, f, d, s, l] of worst.slice(0, 8)) {
  console.log(`  ${String(n).padStart(4)}  ${f.replace('/Users/playom/t27/specs/', '')}  (${d}/${s}/${l})`)
}
