#!/usr/bin/env node
// No C-backend test may name a compiler-specific flag for itself.
//
// Fifteen of them used to, all spelling `-ferror-limit=0`, which only clang
// has. On CI's gcc that flag is refused, and the refusal reads as a compile
// error -- so half the files went red and the other half passed while
// compiling nothing at all. The flag is chosen in exactly one place now, by
// asking the compiler; this gate is what keeps it one place.
import { readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'

const DIR = 'bootstrap/tests'

// Where the choice legitimately lives, and the test that documents it.
const ALLOWED = new Set(['common/mod.rs', 'cc_flags.rs'])

// Flags that exist on one compiler and not the other. A test that names one
// has decided which compiler CI runs, which is not a decision a test may make.
const PARTISAN = ['-ferror-limit=', '-fmax-errors=']

const files = []
for (const e of readdirSync(DIR, { withFileTypes: true })) {
  if (e.isFile() && e.name.endsWith('.rs')) files.push(e.name)
  else if (e.isDirectory()) {
    for (const f of readdirSync(join(DIR, e.name))) {
      if (f.endsWith('.rs')) files.push(`${e.name}/${f}`)
    }
  }
}

const bad = []
for (const rel of files.sort()) {
  if (ALLOWED.has(rel)) continue
  const lines = readFileSync(join(DIR, rel), 'utf8').split('\n')
  lines.forEach((line, i) => {
    // Prose about the defect is allowed; a call site is not.
    if (line.trimStart().startsWith('//')) return
    for (const flag of PARTISAN) {
      if (line.includes(flag)) bad.push(`${DIR}/${rel}:${i + 1}: ${line.trim()}`)
    }
  })
}

if (bad.length) {
  console.error(`${bad.length} hard-coded compiler-specific flag(s):\n`)
  for (const b of bad) console.error(`  ${b}`)
  console.error(
    `\nUse common::cc_syntax_args() or common::cc_strict_args(), which ask ` +
      `cc which spelling it takes. A flag one compiler lacks turns its refusal ` +
      `into what looks like a broken header.`
  )
  process.exit(1)
}
console.log(`check-cc-args: ${files.length} test file(s), no hard-coded compiler-specific flags`)
