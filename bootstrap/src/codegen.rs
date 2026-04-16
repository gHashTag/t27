1	// Trinity Codegen - Ring-001
2	// Generates Rust code from .tri AST (foundation for codegen pipeline)
3
//!
4	//! ## Purpose
5	//! Converts .tri AST (ModuleDecl) into Rust source code
6	//! Target: `bootstrap/src/codegen.rs` will be compiled with t27c
7	//!
8	//! ## Supported Nodes (from foundation)
9	//! - Constants: `ConstDecl` → `pub const NAME: TYPE = VALUE`
10	//! - Types: `TypeDecl` → `pub struct NAME { field: TYPE }` or `pub enum NAME { VARIANT = VALUE }`
11	//! - Functions: `FnDecl` → `pub fn name(params) -> ReturnType { ... }`
12	//! - Opcodes: `Opcode` from vm_core (enum variants → match arms)
13	//!
14	//! ## Codegen Structure
15	//! - CodeWriter: accumulates output with indentation
16	//! - visit_fn_decl: generates function signature and body
17	//! - visit_struct_decl: generates struct with fields
18	//! - visit_enum_decl: generates enum with variants
19
//!
20	use std::io::{self, Write};
21	use std::collections::HashMap;
22
use std::fmt;
23
24	/// Writes Rust code with proper indentation
25	pub struct CodeWriter {
26	  buffer: String,
27	  indent_level: usize,
28	  }
29
30	impl CodeWriter {
31	  pub fn new() -> Self {
32	    Self {
33	      buffer: String::new(),
34	      indent_level: 0,
35	    }
36	  }
37
38	  pub fn indent(&mut self) {
39	    self.indent_level += 1;
40	  }
41
42	  pub fn dedent(&mut self) {
43	    if self.indent_level > 0 {
44	      self.indent_level -= 1;
45	    }
46	  }
47
48	  pub fn write_indent(&mut self) {
49	    for _ in 0..self.indent_level {
50	      self.buffer.push_str("  ");
51	    }
52	  }
53
54	  pub fn write_str(&mut self, s: &str) {
55	    self.buffer.push_str(s);
56	  }
57
58	  pub fn writeln(&mut self, s: &str) {
59	    self.write_str(s);
60	    self.buffer.push('\n');
61	  }
62
63	  pub fn finish(self) -> String {
64	    let mut result = self.buffer.clone();
65	    // Remove trailing whitespace
66	    while result.ends_with('\n') || result.ends_with(' ') {
67	      result.pop();
68	    }
69	    result
70	  }
71	}
72
73	/// Generates Rust code from AST
74	pub fn generate(ast: &ast::AST) -> Result<String, String> {
75	  let mut writer = CodeWriter::new();
76
77	  // Header
78	  writer.writeln("// Auto-generated from .tri spec");
79	  writer.writeln("// Ring-001 Codegen");
80	  writer.writeln("");
81
82	  // Modules
83	  for module in &ast.modules {
84	    writer.writeln(&format!("module {} {{", module.name));
85	    if let Some(version) = &module.version {
86	      writer.write_str(&format!("  version: \"{}\", version));
87	    }
88	    writer.write_str(", ");
89	    if let Some(ring) = &module.ring {
90	      writer.write_str(&format!("  ring: \"{}\", ring));
91	    }
92	    if let Some(seal) = &module.seal {
93	      writer.write_str(&format!("  seal: \"{}\"", seal));
94	    }
95	    writer.write_str(" }}");
96	    writer.writeln("");
97
98
99	  // Imports
100	  for imp in &module.imports {
101	    writer.writeln(&format!("mod {};", imp));
102	  }
103	  if !module.imports.is_empty() {
104	    writer.writeln("");
105	  }
106
107	  // Sections
108	  for section in &module.sections {
109	    match §section.content {
110	      §ast::SectionContent::ConstantsSection(constants) => {
111	        writer.writeln("// === Constants ===");
112	        for decl in &constants.constants {
113	          match decl {
114	            §ast::ConstDecl => {
115	              writer.write_str(&format!("pub const {}: {} = {};",
116	                decl.name, decl.type, decl.value));
117	              writer.writeln("");
118	            }
119	            _ => {}
120	          }
121	        }
122
123	      §ast::SectionContent::TypesSection(types) => {
124	        writer.writeln("// === Types ===");
125	        for decl in &types.types {
126	          match decl {
127	            §ast::TypeDecl::StructDecl(decl) => {
128	              writer.write_str(&format!("pub struct {} {{", decl.name));
129	              writer.indent();
130	              for field in &decl.fields {
131	                writer.write_str(&format!("  {}: {},", field.name, field.type));
132	              }
133	              writer.writeln(",");
134	              }
135	              writer.dedent();
136	              writer.write_str("};");
137	              writer.writeln("");
138	            }
139	            }
140	            §ast::TypeDecl::EnumDecl(decl) => {
141	              writer.write_str(&format!("pub enum {} {{", decl.name));
142	              writer.indent();
143	              for (i, variant) in decl.variants.iter().enumerate() {
144	                let is_last = i == decl.variants.len() - 1;
145	                writer.write_str(&format!("  {} = {}", variant.name, variant.value));
146	                if !is_last {
147	                  writer.write_str(",");
148	                }
149	                writer.writeln(",");
150	              } else {
151	                writer.writeln(",");
152	              }
153	              }
154	      writer.dedent();
155	      writer.write_str("};");
156	      writer.writeln("");
157	    }
158	    §ast::SectionContent::FunctionsSection(functions) => {
159	        writer.writeln("// === Functions ===");
160	        for func in &functions.functions {
161	          match func {
162	            §ast::FnDecl(func) => {
163	              writer.write_str(&format!("pub fn {}(", func.name));
164
165
166	              // Parameters
167	      match func.return_type {
168	        Some(rt) => writer.write_str(&format!(") -> {}", rt)),
169        None => writer.write_str(")"),
170	    }
171	      writer.writeln("");
172	      writer.indent();
173
174	      for param in &func.params {
175	      writer.write_str(&format!("  {}: {}", param.name, param.type));
176        if param != func.params.last() {
177          writer.write_str(",");
178        }
179      }
180	  writer.dedent();
181
182      writer.write_str(" {");
183      writer.writeln("");
184
185      writer.indent();
186      writer.writeln("// ... function body ...");
187      writer.write_str("}");
188      writer.writeln("");
189      }
190	    _ => {}
191	  }
192	}
193
194
195	/// Generate opcode definitions from vm_core spec
196	pub fn generate_opcodes(modules: &Vec<ast::ModuleDecl>) -> Result<String, String> {
197	  let mut writer = CodeWriter::new();
198
199	  let mut opcodes_found = std::collections::HashMap::new();
200
201	  // Search for vm_core module and extract Opcode enum
202	  for module in modules {
203	    for section in &module.sections {
204	      match §section.content {
205	        §ast::SectionContent::TypesSection(types) => {
206	          for decl in &types.types {
207	            match decl {
208	              §ast::TypeDecl::EnumDecl(decl) if decl.name == "Opcode" => {
209	                for variant in &decl.variants {
210	                  opcodes_found.insert(variant.value.clone(), ());
211	                }
212	              }
213	            _ => {}
214	            }
215	        }
216	      }
217	}
218
219	if opcodes_found.is_empty() {
220	  return Err("Opcode enum not found in modules".to_string());
221	}
222
223	let opcodes: Vec<String> = opcodes_found.keys().cloned().collect();
224	opcodes.sort();
225
226	// Generate opcode enum and match statement
227	writer.writeln("// === Opcodes (from vm_core) ===");
228	writer.write_str("pub enum Opcode {");
229	writer.indent();
230	for (i, opc) in opcodes.iter().enumerate() {
231	  let is_last = i == opcodes.len() - 1;
232	  writer.write_str(&format!("  {} = {}// {}", opc, opc, i + 1, i + 2));
233	  if !is_last {
234	    writer.write_str(",");
235	  }
236	}
237	writer.dedent();
238	writer.write_str("};");
239	writer.writeln("");
240	writer.writeln("// Opcode dispatch table placeholder");
241	writer.writeln("pub const DISPATCH_TABLE: [fn(&mut VMState) -> ()] = [");
242	for _ in 0..255 {
243	  writer.write_str("None // 0x{:02x}, ");
244	}
245	writer.write_str("];");
246	writer.writeln("");
247
248	Ok(writer.finish())
}
