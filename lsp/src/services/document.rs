// Document manager for t27 Language Server

use crate::types::Document;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::{
    TextDocumentContentChangeEvent, TextDocumentIdentifier, Url,
};

/// Document manager for handling open documents
pub struct DocumentManager {
    documents: RwLock<HashMap<Url, Document>>,
    workspace_root: RwLock<Option<PathBuf>>,
}

impl DocumentManager {
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
            workspace_root: RwLock::new(None),
        }
    }

    /// Set the workspace root directory
    pub async fn set_workspace_root(&self, root: PathBuf) {
        *self.workspace_root.write().await = Some(root);
    }

    /// Get the workspace root directory
    pub async fn workspace_root(&self) -> Option<PathBuf> {
        self.workspace_root.read().await.clone()
    }

    /// Open a document
    pub async fn open(&self, uri: Url, text: String) {
        let mut docs = self.documents.write().await;
        let mut doc = Document::new(uri.clone(), text);
        doc.version = 1;

        // Parse the document (placeholder - will be connected to parser)
        // self.parse_document(&mut doc).await;

        docs.insert(uri, doc);
    }

    /// Close a document
    pub async fn close(&self, uri: &Url) {
        self.documents.write().await.remove(uri);
    }

    /// Update a document with changes
    pub async fn update(
        &self,
        uri: &Url,
        changes: &[TextDocumentContentChangeEvent],
        version: i32,
    ) {
        let mut docs = self.documents.write().await;
        if let Some(doc) = docs.get_mut(uri) {
            doc.version = version;
            doc.update(changes);

            // Re-parse after update
            // self.parse_document(doc).await;
        }
    }

    /// Get a document by URI
    pub async fn get(&self, uri: &Url) -> Option<Document> {
        self.documents.read().await.get(uri).cloned()
    }

    /// Get document version
    pub async fn get_version(&self, uri: &Url) -> Option<i32> {
        self.documents
            .read()
            .await
            .get(uri)
            .map(|d| d.version)
    }

    /// Get all documents
    pub async fn all_documents(&self) -> Vec<Document> {
        self.documents
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Check if a document is open
    pub async fn is_open(&self, uri: &Url) -> bool {
        self.documents.read().await.contains_key(uri)
    }

    /// Get all document URIs
    pub async fn document_uris(&self) -> Vec<Url> {
        self.documents
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    /// Find documents by file extension
    pub async fn find_documents_by_extension(&self, ext: &str) -> Vec<Url> {
        self.documents
            .read()
            .await
            .iter()
            .filter(|(uri, _)| {
                uri.path()
                    .to_lowercase()
                    .ends_with(&format!(".{}", ext.to_lowercase()))
            })
            .map(|(uri, _)| uri.clone())
            .collect()
    }

    /// Parse a document (placeholder for actual parser integration)
    async fn parse_document(&self, _doc: &mut Document) {
        // TODO: Integrate with t27c parser
        // let parser = T27Parser::new();
        // match parser.parse(&doc.text) {
        //     Ok(ast) => {
        //         doc.symbols = SymbolExtractor::extract(&ast);
        //         doc.diagnostics = DiagnosticsGenerator::generate(&ast);
        //         doc.parsed = true;
        //     }
        //     Err(e) => {
        //         doc.diagnostics = vec![error_diagnostic(/* ... */)];
        //         doc.parsed = false;
        //     }
        // }
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}
