// Backend for t27 Language Server

pub mod parser;

use crate::config::ServerConfig;
use crate::services::{
    CompletionService, DiagnosticsService, DocumentManager, HoverService, NavigationService,
    SymbolService,
};
use crate::types::Document;
use std::sync::Arc;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::*,
    Client, LanguageServer,
};

/// Backend implementation
#[derive(Clone)]
pub struct Backend {
    client: Client,
    documents: Arc<DocumentManager>,
    config: Arc<ServerConfig>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DocumentManager::new()),
            config: Arc::new(ServerConfig::default()),
        }
    }

    /// Initialize the backend
    pub async fn initialize(&mut self, params: InitializeParams) -> Result<InitializeResult> {
        // Set workspace root if provided
        if let Some(workspace_folders) = params.workspace_folders {
            if let Some(folder) = workspace_folders.first() {
                if let Ok(path) = folder.uri.to_file_path() {
                    self.documents.set_workspace_root(path).await;
                }
            }
        }

        // Load configuration from workspace
        if let Some(root) = self.documents.workspace_root().await {
            let loaded_config = ServerConfig::from_workspace(&root);
            self.config = Arc::new(loaded_config);
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "t27 Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(
                        self.config
                            .completion
                            .trigger_characters
                            .iter()
                            .map(|c| c.to_string())
                            .collect(),
                    ),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                // references_provider: Some(OneOf::Left(true)), // TODO: Fix type compatibility
                document_symbol_provider: Some(OneOf::Left(true)),
                // workspace_symbol_provider: Some(OneOf::Left(true)), // TODO: Fix method name
                semantic_tokens_provider: None, // TODO: Implement
                code_action_provider: None,    // TODO: Implement
                ..Default::default()
            },
        })
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut backend = self.clone();
        backend.initialize(params).await
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "t27 Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;

        self.documents.open(uri, text).await;

        // Publish diagnostics
        if let Some(doc) = self.documents.get(&params.text_document.uri).await {
            let diagnostics = DiagnosticsService::generate_from_document(&doc);
            self.client
                .publish_diagnostics(
                    params.text_document.uri.clone(),
                    diagnostics,
                    None,
                )
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        self.documents
            .update(&uri, &params.content_changes, version)
            .await;

        // Publish updated diagnostics
        if let Some(doc) = self.documents.get(&uri).await {
            let diagnostics = DiagnosticsService::generate_from_document(&doc);
            self.client
                .publish_diagnostics(
                    uri,
                    diagnostics,
                    None,
                )
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.close(&params.text_document.uri).await;
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        // Handle save if needed
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc) = self.documents.get(&uri).await {
            let trigger_char = params
                .context
                .and_then(|c| c.trigger_character)
                .and_then(|s| s.chars().next());

            let items = CompletionService::complete(&doc, position, trigger_char);
            Ok(Some(CompletionResponse::List(items)))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri).await {
            Ok(HoverService::hover(&doc, position))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri).await {
            Ok(NavigationService::goto_definition(&doc, position)
                .map(GotoDefinitionResponse::Scalar))
        } else {
            Ok(None)
        }
    }

    // TODO: Fix type compatibility with tower-lsp 0.20.0
    // async fn references(&self, params: ReferencesParams) -> Result<Option<Vec<Location>>> {
    //     let uri = params.text_document_position.text_document.uri;
    //     let position = params.text_document_position.position;
    //     let include_declaration = params.context.include_declaration;

    //     if let Some(doc) = self.documents.get(&uri).await {
    //         let all_docs = self.documents.all_documents().await;
    //         Ok(Some(NavigationService::find_references(
    //             &all_docs,
    //             position,
    //             include_declaration,
    //         )))
    //     } else {
    //         Ok(None)
    //     }
    // }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        if let Some(doc) = self.documents.get(&uri).await {
            let symbols = SymbolService::document_symbols(&doc);
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        } else {
            Ok(None)
        }
    }

    // TODO: Fix method name for tower-lsp 0.20.0
    // async fn workspace_symbol(
    //     &self,
    //     params: WorkspaceSymbolParams,
    // ) -> Result<Option<Vec<SymbolInformation>>> {
    //     let all_docs = self.documents.all_documents().await;
    //     let symbols = SymbolService::workspace_symbols(&all_docs, params.query);
    //     Ok(Some(symbols))
    // }
}
