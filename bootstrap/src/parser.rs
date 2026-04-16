1	// Trinity Bootstrap Parser - Ring 001
2	// Updated with support for numeric_tower, promotion, experience, imports sections
3
4	//! # Bootstrap Parser
5	//! Parses .t27 specification files into AST for code generation
6	//!
7	//! ## Supported Sections (Ring-001 additions):
8	//! - `numeric_tower {}` - Promotion rules between formats
9	//! - `promotion {}` - Type promotion functions
10	//! - `experience {}` - Experience hooks configuration
11	//! - `imports [...]` - Module import statements
12
//!
13	//! ## Section Kinds:
14	//! - SectionKind::NumericTower (NEW)
15	//! - SectionKind::Promotion (NEW)
16	//! - SectionKind::Experience (NEW)
17	//! - SectionKind::Imports (NEW)
18
19
20	use crate::anyhow::{anyhow, Result};
21	use std::collections::HashMap;
22
23	/// Module name registry (foundation modules that can be imported)
24	const FOUNDATION_MODULES: &[&str] = &["gf_family_foundation"];
25
26
27	/// All section kinds that can appear in foundation modules
28	const FOUNDATION_SECTION_KINDS: &[SectionKind] = &[
29	  SectionKind::Module,
30	  SectionKind::Constants,
31	  SectionKind::Types,
32	  SectionKind::Functions,
33	  SectionKind::Invariants,
34	  SectionKind::Tests,
35	  SectionKind::Benchmarks,
36	  SectionKind::NumericTower,  // NEW for Ring-001
37	  SectionKind::Promotion,     // NEW for Ring-001
38	  SectionKind::Experience,      // NEW for Ring-001
39	  SectionKind::Imports,       // NEW for Ring-001
40	];
41
42
43	// ============================================================================
44	// ERROR TYPES
45	// ============================================================================
46
47
48	#[derive(Debug, Clone)]
49	pub enum ParseError {
50	  UnknownSection { kind: String, name: String },
51	  DuplicateModule { name: String },
52	  DuplicateModuleImport { module: String, name: String },
53	  InvalidSectionSyntax { kind: String, msg: String },
54	  InvalidNumericTower { msg: String },
55	  InvalidPromotion { msg: String },
56	  InvalidExperience { msg: String },
57	  InvalidImports { msg: String },
58	  InvalidConstType { name: String, kind: String },
59	  UnknownType { name: String },
60	}
61
62	// ============================================================================
63	// AST NODES
64	// ============================================================================
65
66	#[derive(Debug, Clone)]
67	pub struct AST {
68	  pub modules: Vec<ModuleDecl>,
69	  pub errors: Vec<ParseError>,
70	}
71
72	#[derive(Debug, Clone)]
73	pub struct ModuleDecl {
74	  pub name: String,
75	  pub version: Option<String>,
76	  pub ring: Option<String>,
77	  pub seal: Option<String>,
78	  pub imports: Vec<String>,
79	  pub sections: Vec<Section>,
80	}
81
82	#[derive(Debug, Clone)]
83	pub struct Section {
84	  pub kind: SectionKind,
85	  pub content: SectionContent,
86	}
87
88	#[derive(Debug, Clone)]
89	pub enum SectionContent {
90	  ConstantsSection { constants: Vec<ConstDecl> },
91	  TypesSection { types: Vec<TypeDecl> },
92	  FunctionsSection { functions: Vec<FnDecl> },
93	  InvariantsSection { invariants: Vec<InvariantDecl> },
94	  BenchmarksSection { benchmarks: Vec<BenchDecl> },
95	  // NEW sections for Ring-001
96	  NumericTowerSection { rules: Vec<PromotionRule> },
97	  PromotionSection { rules: Vec<PromotionRule> },
98	  ExperienceSection { hooks: Vec<String> },
99	  ImportsSection { imports: Vec<String> },
100	}
101	#[derive(Debug, Clone)]
102	pub struct ConstDecl {
103	  pub name: String,
104	  pub type: String,
105	  pub value: String,
106	}
107	#[derive(Debug, Clone)]
108	pub struct TypeDecl {
109	  decl: TypeDeclBody,
110	}
111	#[derive(Debug, Clone)]
112	pub enum TypeDeclBody {
113	  StructDecl { fields: Vec<FieldDecl> },
114	  EnumDecl { variants: Vec<EnumVariant> },
115	}
116	#[derive(Debug, Clone)]
117	pub struct FieldDecl {
118	  pub name: String,
119	  pub type: String,
120	}
121	#[derive(Debug, Clone)]
122	pub struct EnumVariant {
123	  pub name: String,
124	  pub value: String,
125	}
126	#[derive(Debug, Clone)]
127	pub struct FnDecl {
128	  pub name: String,
129	  pub params: Vec<ParamDecl>,
130	  pub return_type: Option<String>,
131	  pub body: Expr,
132	}
133	#[derive(Debug, Clone)]
134	pub struct ParamDecl {
135	  pub name: String,
136	  pub type: String,
137	}
138	#[derive(Debug, Clone)]
139	pub enum Expr {
140	  Literal { value: String },
141	  Variable { name: String },
142	  Binary { op: String, left: Box<Expr>, right: Box<Expr> },
143	  Call { callee: String, args: Vec<Expr> },
144	  Block { statements: Vec<Expr> },
145	}
146	#[derive(Debug, Clone)]
147	// NEW: Numeric tower promotion rules (Ring-001)
148	pub struct PromotionRule {
149	  pub src_format: String,
150	  pub dst_format: String,
151	  pub condition: Option<String>,  -- e.g., "always", "range_check"
152	}
153	#[derive(Debug, Clone)]
154	// NEW: Invariant given clause
155	pub struct GivenClause {
156	  pub name: String,
157	  pub expr: Expr,
158	}
159	#[derive(Debug, Clone)]
160	pub struct InvariantDecl {
161	  pub name: String,
162	  pub given: Vec<GivenClause>,
163	  pub when: Expr,
164	}
165	#[derive(Debug, Clone)]
166	pub struct BenchDecl {
167	  pub name: String,
168	  pub measure: String,
169	  pub target: String,
170	  pub unit: String,
171	  pub warmup: u8,
172	  pub runs: u16,
173	}
174
175	/// Parse a .t27 specification file
176	/// Returns AST or errors
177	pub fn parse(file_path: &str) -> Result<AST, ParseError> {
178	  let content = std::fs::read_to_string(file_path)
179	      .map_err(|e| ParseError::IoError(e.to_string()))?;
180
181	  let mut ast = AST {
182	    modules: Vec::new(),
183	    errors: Vec::new(),
184	  };
185
186	  let mut lines: Vec::new();
187	  for line in content.lines() {
188	    lines.push(line);
189	  }
190	  parse_modules(&mut lines, &mut ast)?;
191
192	  if ast.errors.is_empty() {
193	    Ok(ast)
194	  } else {
195	    Err(ParseError::ParseFailed)
196	  }
197	}
198
199	/// Parse module declarations (module name { ... })
200	fn parse_module(line: &str) -> Option<ModuleDecl> {
201	  if !line.starts_with("module ") {
202	    return None;
203	  }
204	  let line = line.trim_start_matches("module ").unwrap();
205	  let parts: Vec::from_iter(line.split_whitespace());
206	  if parts.is_empty() || parts.len() < 2 {
207	    return None;
208	  }
209	  let name = parts[0].to_string();
210	  // Optional: version "1.0.0"
211	  // Optional: ring "001"
212	  // Optional: seal "<hash>"
213	  let mut module = ModuleDecl {
214	    name: name.clone(),
215	    version: None,
216	    ring: None,
217	    seal: None,
218	    imports: Vec::new(),
219	    sections: Vec::new(),
220	  };
221	  let mut i = 1;
222	  while i < parts.len() {
223	    if parts[i].contains('=') {
224	        let kv: Vec::from_iter(parts[i].split('='));
225	        if kv.len() != 2 {
226	          return None;
227	        }
228	        let key = kv[0].trim();
229	        let value = kv[1].trim();
230	        match key.as_str() {
231	          "imports" => {
232	            if !module.imports.is_empty() {
233	              return None;
234	            }
235	            for imp in value.split_whitespace() {
236	              if !module.imports.contains(&imp) {
237	                module.imports.push(imp.to_string());
238	              }
239	            }
240	          }
241	          _ => {}  // Ignore unknown keys for now
242	        }
243	      }
244	      i += 1;
245	    }
246	  Some(module)
247	}
248	/// Parse sections from module body
249	fn parse_sections(lines: &mut Vec<&str>, module: &mut ModuleDecl) -> Result<(), ParseError> {
250	  let mut in_section = false;
251	  let mut section_line_start = None;
252
253	  for (idx, line) in lines.iter().enumerate() {
254	    let trimmed = line.trim();
255
256
257	  // Skip empty lines and comments
258	  if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
259	      continue;
260	    }
261
262	  // Section start
263	  if trimmed.starts_with("types {") || trimmed.starts_with("invariants {") ||
264	         trimmed.starts_with("tests {") || trimmed.starts_with("benchmarks {") {
265	    let kind = match trimmed.as_bytes().get(..6) {
266	      b"types {" => Some(SectionKind::Types),
267	      b"invariants" => Some(SectionKind::Invariants),
268	      b"tests" => Some(SectionKind::Tests),
269	      b"benchmarks" => Some(SectionKind::Benchmarks),
270	      // NEW sections for Ring-001
271	      b"numeric_tower" => Some(SectionKind::NumericTower),
272	      b"promotion" => Some(SectionKind::Promotion),
273	      b"experience" => Some(SectionKind::Experience),
274	      b"imports" => Some(SectionKind::Imports),
275	      _ => None,
276	    };
277	    if let Some(k) = kind {
278	      in_section = true;
279	      section_line_start = Some(idx);
280	      continue;
281	    }
282
283	  // Section end
284	  if trimmed == "}" {
285	    if !in_section {
286	      return Err(ParseError::UnexpectedSectionEnd {
287	          kind: "None".to_string(),
288	          msg: format!("Found }} without {{ at line {}", idx + 1),
289	      });
290	    }
291	    in_section = false;
292	  section_line_start = None;
293	  }
294
295	  let mut current_section_content: SectionContent::Empty;
296	  let mut current_line_buffer = String::new();
297
298	  // Collect section content lines
299	  for (idx, line) in lines.iter().enumerate() {
300	    if !in_section {
301	      return Err(ParseError::UnexpectedContent {
302	          kind: "None".to_string(),
303	          msg: format!("Found content before {{ at line {}", idx + 1),
304	      });
305	    }
306
307	    // Skip empty lines and comments
308	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
309	      if in_section {
310	        current_line_buffer.push_str(line);
311	      }
312	      continue;
313	    }
314
315	    // Section end
316	    if trimmed == "}" {
317	      let content_lines: std::mem::take(current_line_buffer.as_str())
318	        .map_err(|e| ParseError::IoError(e.to_string()))?;
319
320
321	      // Parse section based on kind
322	      match section_line_start {
323	        Some(start_idx) => {
324	          let kind = match lines[start_idx].as_bytes().get(..6) {
325	            b"types" => parse_types_section(content_lines, module),
326	            b"invariants" => parse_invariants_section(content_lines, module),
327	            b"tests" => parse_tests_section(content_lines, module),
328	            b"benchmarks" => parse_benchmarks_section(content_lines, module),
329	            // NEW sections for Ring-001
330	            b"numeric_tower" => parse_numeric_tower_section(content_lines, module),
331	            b"promotion" => parse_promotion_section(content_lines, module),
332	            b"experience" => parse_experience_section(content_lines, module),
333	            b"imports" => parse_imports_section(content_lines, module),
334	            _ => return Err(ParseError::InvalidSectionSyntax {
335	              kind: "Unknown".to_string(),
336	              msg: format!("Unknown section kind at line {}", start_idx),
337	            }),
338	          };
339	        None => return Err(ParseError::UnexpectedSectionEnd {
340	          kind: "None".to_string(),
341	          msg: format!("Found }} without section start at line {}", idx),
342	        }),
343	      }
344	    in_section = false;
345	    section_line_start = None;
346	    current_line_buffer.clear();
347	  }
348
349	  Ok(())
350	}
351	/// Parse constants section: constants { NAME: TYPE VALUE }
352	fn parse_constants_section(lines: &[&str], module: &mut ModuleDecl) {
353	  let mut constants = Vec::new();
354	  for line in lines {
355	    let trimmed = line.trim();
356	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
357	      continue;
358	    }
359	    if trimmed.starts_with("pub const") || trimmed.starts_with("pub fn") {
360	      // End of section
361	      break;
362	    }
363	    if let Some(pos) = trimmed.find(": ") {
364	      let parts: Vec::from_iter(trimmed.splitn(':', 2).collect::<Vec<_>>());
365	      if parts.len() < 2 || parts.len() > 3 {
366	        continue;
367	      }
368	      let name = parts[0].trim();
369	      let type_and_value = parts[1].trim();
370	      let mut const_value = String::new();
371	      // Type and value may span multiple lines
372	      const_value.push_str(type_and_value);
373	      while idx + 1 < lines.len() {
374	        let next_line = lines[idx + 1].trim();
375	        if next_line.starts_with("pub const") || next_line.starts_with("pub fn") || next_line.starts_with("}") {
376	          break;
377	        }
378	        if !next_line.trim().is_empty() && !next_line.starts_with("--") && !next_line.starts_with("//") {
379	          const_value.push(' ');
380	          const_value.push_str(next_line.trim());
381	        }
382	      }
383	    }
384
385	  // Parse each constant
386	  for const_str in constants {
387	    let parts: Vec::from_iter(const_str.trim().split_whitespace()).collect::<Vec<_>>();
388	    if parts.len() >= 2 {
389	      let name = parts[0].to_string();
390	      let value = parts[1].to_string();
391	      constants.push(ConstDecl { name, type: "GF32".to_string(), value });
392	    }
393	}
394	  module.sections.push(Section {
395	    kind: SectionKind::Constants,
396	    content: SectionContent::ConstantsSection { constants },
397	  });
398	  Ok(())
399	}
400	/// Parse types section: types { STRUCT | ENUM } (simplified)
401	fn parse_types_section(lines: &[&str], _module: &mut ModuleDecl) {
402	  for line in lines {
403	    let trimmed = line.trim();
404	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
405	      continue;
406	    }
407	    if trimmed.starts_with("pub struct") || trimmed.starts_with("pub enum") {
408	      let rest = trimmed.strip_prefix("pub struct ").or_else("");
409	      let rest = rest.strip_prefix("pub enum ").or_else("");
410	      if rest.starts_with("}") {
411	        let type_name = rest[..rest.len() - 1].to_string();
412	        _module.sections.push(Section {
413	          kind: SectionKind::Types,
414	          content: SectionContent::TypesSection { types: vec![TypeDecl::StructDecl {
415	            decl: TypeDeclBody::StructDecl {
416	              fields: vec![],
417	            name: type_name,
418	          }}]
419	        });
420	        } else if rest.starts_with("{") {
421	        // Parse fields/variants
422	        let body = &rest[rest.len() - 1];
423	        let decl_type = if rest.starts_with("struct") { "StructDecl" } else { "EnumDecl" };
424	        let mut items = Vec::new();
425	        let mut current = String::new();
426	        for ch in body.chars() {
427	          if ch == '{' {
428	            if !current.is_empty() {
429	              items.push(current.clone());
430	              }
431	            current.clear();
432	          } else if ch == ',' {
433	            items.push(current.clone());
434	            current.clear();
435	          } else if ch != '}' {
436	            current.push(ch);
437	          }
438	        }
439	        let decl = match decl_type.as_str() {
440	          "StructDecl" => TypeDecl::StructDecl { fields: items, name: rest[..15].to_string() },
441	          "EnumDecl" => TypeDecl::EnumDecl { variants: items, name: rest[..9].to_string() },
442	          _ => return,
443	        };
444	        _module.sections.push(Section {
445	          kind: SectionKind::Types,
446	          content: SectionContent::TypesSection { types: vec![decl] },
447	        });
448	      }
449	  }
450	}
451	/// NEW: Parse numeric_tower section: numeric_tower { (SRC, DST) => DST }
452	fn parse_numeric_tower_section(lines: &[&str], module: &mut ModuleDecl) {
453	  let mut rules = Vec::new();
454	  for line in lines {
455	    let trimmed = line.trim();
456	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
457	      continue;
458	    }
459	    if trimmed.starts_with("}") {
460	      // End of section
461	      break;
462	    }
463	    if trimmed.contains("=>") {
464	      let parts: Vec::from_iter(trimmed.splitn("=>", 2).collect::<Vec<_>>());
465	      if parts.len() >= 2 && parts.len() <= 3 {
466	        let src = parts[0].trim();
467	        let dst = parts[1].trim();
468	        if is_valid_format(src) && is_valid_format(dst) {
469	          rules.push(PromotionRule {
470	            src_format: src.to_string(),
471	            dst_format: dst.to_string(),
472	            condition: None,
473	          });
474	      } else {
475	        return Err(ParseError::InvalidNumericTower {
476	          msg: format!("Invalid numeric_tower rule at line: '{}'", trimmed),
477	        });
478	      }
479	  }
480
481	  _module.sections.push(Section {
482	    kind: SectionKind::NumericTower,
483	    content: SectionContent::NumericTowerSection { rules },
484	  });
485	  Ok(())
486	}
487	/// NEW: Parse promotion section: promotion { (SRC, DST) => DST }
488	fn parse_promotion_section(lines: &[&str], _module: &mut ModuleDecl) {
489	  let mut rules = Vec::new();
490	  for line in lines {
491	    let trimmed = line.trim();
492	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
493	      continue;
494	    }
495	    if trimmed.starts_with("}") {
496	      // End of section
497	      break;
498	    }
499	    if trimmed.contains("=>") {
500	      let parts: Vec::from_iter(trimmed.splitn("=>", 2).collect::<Vec<_>>());
501	      if parts.len() >= 2 && parts.len() <= 3 {
502	        let src = parts[0].trim();
503	        let dst = parts[1].trim();
504	        let condition = if parts.len() == 3 { parts[2].to_string() } else { None };
505	        if is_valid_format(src) && is_valid_format(dst) {
506	          rules.push(PromotionRule {
507	            src_format: src.to_string(),
508	            dst_format: dst.to_string(),
509	            condition,
510	          });
511	      } else {
512	        return Err(ParseError::InvalidPromotion {
513	          msg: format!("Invalid promotion rule at line: '{}'", trimmed),
514	        });
515	      }
516	  }
517
518	  _module.sections.push(Section {
519	    kind: SectionKind::Promotion,
520	  content: SectionContent::PromotionSection { rules },
521	  });
522	  Ok(())
523	}
524	/// NEW: Parse experience section: experience { on_invariant_fail: save 'path.json' }
525	fn parse_experience_section(lines: &[&str], _module: &mut ModuleDecl) {
526	  let mut hooks = Vec::new();
527	  for line in lines {
528	    let trimmed = line.trim();
529	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
530	      continue;
531	    }
532	    if trimmed.starts_with("}") {
533	      break;
534	    }
535	    if trimmed.starts_with("on_") {
536	      let hook_name = trimmed[3..].to_string();
537	      let hook_value = trimmed[12..].to_string();  // Everything after "="
538	      if !hook_name.is_empty() && !hook_value.is_empty() {
539	        hooks.push(format!("{}: {}", hook_name, hook_value));
540	      }
541	  }
542	  }
543	  _module.sections.push(Section {
544	    kind: SectionKind::Experience,
545	    content: SectionContent::ExperienceSection { hooks },
546	  });
547	  Ok(())
548	}
549	/// NEW: Parse imports section: imports [module_name, ...]
550	fn parse_imports_section(lines: &[&str], _module: &mut ModuleDecl) {
551	  for line in lines {
552	    let trimmed = line.trim();
553	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
554	      continue;
555	    }
556	    if trimmed.starts_with("}") {
557	      break;
558	    }
559	    if trimmed.starts_with("[") {
560	      let imports = &trimmed[8..trimmed.len()-1];  // Everything after "["
561	      for imp in imports.split_whitespace() {
562	        if imp.is_empty() { continue; }
563	        if FOUNDATION_MODULES.contains(&imp) {
564	          module.imports.push(imp.to_string());
565	        } else {
566	          return Err(ParseError::InvalidImports {
567	            msg: format!("Unknown import '{}' at line: {}", imp, line),
568	          });
569	      }
570	  }
571
572	  _module.sections.push(Section {
573	    kind: SectionKind::Imports,
574	    content: SectionContent::ImportsSection { imports: module.imports },
575	  });
576	  Ok(())
577	}
578	/// Parse invariants section: invariants { NAME given: EXPR when: EXPR }
579	fn parse_invariants_section(lines: &[&str], module: &mut ModuleDecl) {
580	  let mut invariants = Vec::new();
581	  let mut current_invariant: None;
582	  let mut current_given = Vec::new();
583	  let mut collecting_given = false;
584
585
586	  for line in lines {
587	    let trimmed = line.trim();
588	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
589	      continue;
590	    }
591	    if trimmed.starts_with("invariant ") {
592	      collecting_given = true;
593	      let name = trimmed[9..].to_string();
594	      current_invariant = Some(InvariantDecl {
595	          name: name.to_string(),
596	          given: current_given.clone(),
597	          when: Expr::Literal { value: "true".to_string() },
598	      });
599	      current_given.clear();
600	    } else if trimmed.starts_with("given ") {
601	      collecting_given = true;
602	      let name = trimmed[5..].to_string();
603	      let expr = parse_expr(&trimmed[5..]);
604	      current_given.push(GivenClause { name: name.to_string(), expr });
605	    } else if trimmed.starts_with("when ") {
606	      collecting_given = false;
607	      if let Some(inv) = current_invariant.take() {
608	        let when_expr = parse_expr(&trimmed[5..]);
609	        inv.when = when_expr;
610	      }
611	    } else if trimmed.starts_with("}") {
612	      if let Some(mut inv) = current_invariant {
613	        invariants.push(inv);
614	      }
615	      current_invariant = None;
616	  }
617	  _module.sections.push(Section {
618	    kind: SectionKind::Invariants,
6	    content: SectionContent::InvariantsSection { invariants },
619	  });
620	  Ok(())
621	}
622	/// Parse tests section (simplified)
623	fn parse_tests_section(lines: &[&str], _module: &mut ModuleDecl) {
624	  let mut tests = Vec::new();
625	  for line in lines {
626	    let trimmed = line.trim();
627	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
628	      continue;
629	    }
630	    if trimmed.starts_with("}") {
631	      break;
632	    }
633	    if trimmed.starts_with("test ") {
634	      let test_name = trimmed[5..].to_string();
635	      tests.push(BenchDecl {
636            name: test_name.to_string(),
637            measure: "skip".to_string(),
638            target: "skip".to_string(),
639            unit: "skip".to_string(),
640            warmup: 0,
641            runs: 1,
642          });
643	    }
644	  }
645	  _module.sections.push(Section {
646	    kind: SectionKind::Tests,
647	    content: SectionContent::TestsSection { tests },
648	  });
649	  Ok(())
650	}
651	/// Parse benchmarks section (simplified)
652	fn parse_benchmarks_section(lines: &[&str], _module: &mut ModuleDecl) {
653	  let mut benchmarks = Vec::new();
654	  for line in lines {
655	    let trimmed = line.trim();
656	    if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
657	      continue;
658	    }
659	    if trimmed.starts_with("}") {
660	      break;
661	    }
662	    if trimmed.starts_with("bench ") {
663	      let parts: Vec::from_iter(trimmed.split_whitespace()).collect::<Vec<_>>();
664	      if parts.len() >= 4 && parts.len() <= 6 {
665	        let name = parts[0].to_string();
666	        let measure = parts[1].to_string();
667	        let target = parts[2].to_string();
668	        let unit = if parts.len() >= 5 { parts[4].to_string() } else { "ops_per_second".to_string() };
669	        let warmup = if parts.len() >= 6 { parts[5].parse::<u8>().unwrap_or(3) } else { 3u8 };
670	        benchmarks.push(BenchDecl {
671	          name, measure, target, unit, warmup, runs: 100u16,
672	        });
673	      }
674	  }
675	  _module.sections.push(Section {
676	    kind: SectionKind::Benchmarks,
677	    content: SectionContent::BenchmarksSection { benchmarks },
678	  });
679	  Ok(())
680	}
681	/// Parse all modules from input
682	fn parse_modules(lines: &mut Vec<&str>, ast: &mut AST) -> Result<(), ParseError> {
683	  let mut i = 0;
684	  while i < lines.len() {
685	    if let Some(module) = parse_module(lines[i]) {
686	      ast.modules.push(module);
687	      i += 1;
688	      }
689	      // Check for module name duplicates
690	      let mut names = Vec::new();
691	      for module in &ast.modules {
692	        names.push(module.name.clone());
693	      }
694	  }
695	  // Validate that all foundation sections are present
696	  if !contains_all_foundation_sections(&ast.modules) {
697	    ast.errors.push(ParseError::MissingFoundationSections {
698	        kind: "MissingFoundation".to_string(),
699	      msg: "Module must declare all foundation sections from imported module".to_string(),
700	      });
701	  }
702	  Ok(())
703	}
704	/// Check if a module contains all foundation section kinds
705	fn contains_all_foundation_sections(modules: &[ModuleDecl]) -> bool {
706	  for module in modules {
707	    let mut found_kinds = [false; 10];
708	    for section in &module.sections {
709	      match &section.content {
710	        SectionContent::ConstantsSection => found_kinds[0] = true,
711	        SectionContent::TypesSection => found_kinds[1] = true,
712	        SectionContent::FunctionsSection => found_kinds[2] = true,
713	        SectionContent::InvariantsSection => found_kinds[3] = true,
714	        SectionContent::BenchmarksSection => found_kinds[4] = true,
715	        SectionContent::NumericTowerSection => found_kinds[5] = true,  // NEW
716	        SectionContent::PromotionSection => found_kinds[6] = true,      // NEW
717	        SectionContent::ExperienceSection => found_kinds[7] = true,       // NEW
718	        SectionContent::ImportsSection => found_kinds[8] = true,       // NEW
719	      }
720	    }
721	  found_kinds.iter().all(|&x| *x)
722	}
723	/// Check if a format name is valid (foundation type or builtin)
724	fn is_valid_format(name: &str) -> bool {
725	  // Valid if it's in GF_FAMILY_TYPES or is "GF32", "GF16", etc.
726	  match name {
727	    "GF4" | "GF8" | "GF16" | "GF32" | "GF64" | "TF3" | "TF16" => true,
728	    "GF32" | "f64" | "f32" => true,
729	    _ => false,
730	  }
731	}
732	/// Helper: parse expression from string (simplified)
733	fn parse_expr(input: &str) -> Expr {
734	  let trimmed = input.trim();
735	  if trimmed.starts_with('"') && trimmed.ends_with('"') {
736	    Expr::Literal { value: trimmed[1..trimmed.len()-1].to_string() }
737	  } else if trimmed.contains('(') {
738	    // Function call
739	      let paren_idx = trimmed.find(')').unwrap();
740	      let callee = trimmed[..paren_idx].to_string();
741	      Expr::Call { callee, args: vec![] }
742	  } else {
743	    Expr::Variable { name: trimmed.to_string() }
744	  }
745	}
746	/// Helper: validate numeric_tower rules
747	fn validate_promotion_rules(rules: &[PromotionRule]) -> Result<(), ParseError> {
748	  for rule in rules {
749	    if !is_valid_format(&rule.src_format) || !is_valid_format(&rule.dst_format) {
750	      return Err(ParseError::InvalidNumericTower {
751	          msg: format!("Invalid format in numeric_tower rule: '{}' -> '{}'", rule.src_format, rule.dst_format),
752	      });
753	    }
754	  }
755	  Ok(())
756	}
