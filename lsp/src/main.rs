use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Incremental,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".", ":", "{".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(DefinitionProviderCapability::Simple(true)),
                references_provider: Some(ReferencesProviderCapability::Simple(true)),
                document_highlight_provider: Some(DocumentHighlightProviderCapability::Simple(true)),
                document_symbol_provider: Some(DocumentSymbolProviderCapability::Simple(true)),
                workspace_symbol_provider: Some(WorkspaceSymbolProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokens(SemanticTokensOptions {
                        legend: SemanticTokensLegend {
                            token_types: vec![
                                SemanticTokenType::FUNCTION,
                                SemanticTokenType::VARIABLE,
                                SemanticTokenType::CONSTANT,
                                SemanticTokenType::TYPE,
                                SemanticTokenType::KEYWORD,
                                SemanticTokenType::COMMENT,
                            ],
                            token_modifiers: vec![],
                        },
                        ..Default::default()
                    }),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "t27 LSP Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let line = params.text_document_position.position.line;
        let line_content = self.get_line_content(&uri, line).unwrap_or_default();

        let items = self.get_completions(&line_content, &params.text_document_position.position);
        Ok(Some(CompletionResponse::List(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.clone();
        let word = self.get_word_at_position(&uri, &params.text_document_position_params.position);

        if let Some(w) = word {
            let docs = self.get_documentation(&w);
            Ok(Some(Hover {
                contents: HoverContents::Markdown(docs),
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.clone();
        let word = self.get_word_at_position(&uri, &params.text_document_position_params.position);

        if let Some(w) = word {
            if let Some(loc) = self.find_definition(&uri, &w) {
                Ok(Some(GotoDefinitionResponse::Scalar(loc)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let word = self.get_word_at_position(&uri, &params.text_document_position.position);

        if let Some(w) = word {
            Ok(Some(self.find_references(&uri, &w)))
        } else {
            Ok(None)
        }
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = params.text_document_params.text_document.uri.clone();
        let position = params.text_document_params.position;
        let word = self.get_word_at_position(&uri, &position);

        if let Some(w) = word {
            if let Some(range) = self.find_definition(&uri, &w) {
                Ok(Some(vec![DocumentHighlight {
                    range,
                    kind: Some(DocumentHighlightKind::WRITE),
                }]))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.clone();
        Ok(Some(DocumentSymbolResponse::Nested(self.get_document_symbols(&uri))))
    }

    async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<WorkspaceSymbol>>> {
        Ok(Some(self.search_symbols(&params.query)))
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<Vec<CodeAction>>> {
        let uri = params.text_document.uri.clone();
        Ok(Some(self.get_code_actions(&uri)))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.clone();
        Ok(Some(SemanticTokensResult::Tokens(self.get_semantic_tokens(&uri))))
    }
}

impl Backend {
    fn new(client: Client) -> Self {
        Self { client }
    }

    fn get_line_content(&self, uri: &str, line: u32) -> Option<String> {
        // TODO: Implement file reading
        None
    }

    fn get_completions(&self, line: &str, position: &Position) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        let line_prefix = &line[..position.character as usize];

        // Keywords
        let keywords = vec![
            "module", "fn", "const", "let", "if", "else", "return",
            "phi", "gf16", "gf32", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64",
            "f32", "f64", "bool", "str", "test", "invariant", "bench",
        ];

        for kw in keywords {
            if kw.starts_with(line_prefix) && kw.len() > line_prefix.len() {
                items.push(CompletionItem {
                    label: kw.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("keyword".to_string()),
                    ..Default::default()
                });
            }
        }

        items
    }

    fn get_word_at_position(&self, uri: &str, position: &Position) -> Option<String> {
        // TODO: Implement word extraction
        None
    }

    fn get_documentation(&self, word: &str) -> String {
        match word {
            "phi" => format!(
                "**phi** (φ)\n\nGolden ratio ≈ 1.618\n\nType: GoldenFloat (GF16)\n\nProperties:\n- φ² = φ + 1\n- φ² + φ⁻² = 3\n"
            ),
            "gf16" => "**GF16**\n\n16-bit GoldenFloat format\n\n- 6 bits exponent\n- 9 bits mantissa\n- 1 bit sign",
            _ => format!("**{}**\n\nNo documentation available.", word),
        }
    }

    fn find_definition(&self, uri: &str, word: &str) -> Option<Location> {
        // TODO: Implement cross-reference lookup
        None
    }

    fn find_references(&self, uri: &str, word: &str) -> Vec<Location> {
        // TODO: Implement reference search
        vec![]
    }

    fn get_document_symbols(&self, uri: &str) -> Vec<DocumentSymbol> {
        // TODO: Parse .t27 file and extract symbols
        vec![]
    }

    fn search_symbols(&self, query: &str) -> Vec<WorkspaceSymbol> {
        // TODO: Implement workspace-wide symbol search
        vec![]
    }

    fn get_code_actions(&self, uri: &str) -> Vec<CodeAction> {
        vec![
            CodeAction {
                title: "Format Spec".to_string(),
                kind: Some(CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
                diagnostics: None,
                edit: None,
                is_preferred: None,
                disabled: None,
                data: None,
                command: None,
            },
            CodeAction {
                title: "Run Tests".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: None,
                is_preferred: None,
                disabled: None,
                data: None,
                command: Some(Command {
                    title: "Run Tests".to_string(),
                    command: "t27.runTests".to_string(),
                    arguments: Some(vec![uri.clone()]),
                }),
            },
        ]
    }

    fn get_semantic_tokens(&self, uri: &str) -> SemanticTokens {
        // TODO: Implement tokenization
        SemanticTokens::default()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client)).await;

    Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}