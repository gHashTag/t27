// TypeScript LSP Client for t27

import {
    createConnection,
    TextDocuments,
    ProposedFeatures,
    TextDocumentSyncKind,
    InitializeResult,
    CodeAction,
    Diagnostic,
    Position,
    Hover,
    CompletionItem,
    DocumentSymbol,
    SymbolInformation,
    SemanticTokens,
    SignatureHelp,
} from 'vscode-languageserver/node';

export class T27LanguageClient {
    private server: any = null;
    private documents: TextDocuments;

    constructor(serverProcess: any) {
        this.server = serverProcess;
        this.documents = new TextDocuments(TextDocuments.getUri());
    }

    async initialize(params: any): Promise<InitializeResult> {
        console.log('[t27] Initializing...');
        const result: InitializeResult = {
            capabilities: {
                textDocumentSync: {
                    openClose: true,
                    change: { synchronize: true }
                },
                hoverProvider: true,
                completionProvider: {
                    triggerCharacters: ['.', ':', '(', '[', '{', ' ']
                },
                definitionProvider: true,
                referencesProvider: true,
                documentSymbolProvider: true,
                workspaceSymbolProvider: true,
                semanticTokensProvider: {
                    legend: this.getSemanticTokensLegend(),
                    full: {
                        delta: true
                    }
                },
                codeActionProvider: {
                    codeActionKinds: ['quickfix', 'refactor']
                },
                signatureHelpProvider: {
                    triggerCharacters: ['(', ',']
                }
            }
        };
        console.log('[t27] Capabilities:', result.capabilities);
        return result;
    }

    async didOpen(params: any): Promise<void> {
        console.log('[t27] didOpen:', params.textDocument.uri);
        this.documents.get(params.textDocument.uri).then(doc => {
            if (doc) {
                const diagnostics = this.validateDocument(doc);
                if (diagnostics.length > 0) {
                    console.log('[t27] Diagnostics:', diagnostics);
                    this.connection.sendNotification('textDocument/publishDiagnostics', {
                        uri: params.textDocument.uri,
                        diagnostics: diagnostics
                    });
                }
            }
        });
    }

    async didChange(params: any): Promise<void> {
        console.log('[t27] didChange:', params.textDocument.uri);
        this.documents.get(params.textDocument.uri).then(doc => {
            if (doc) {
                const diagnostics = this.validateDocument(doc);
                if (diagnostics.length > 0) {
                    this.connection.sendNotification('textDocument/publishDiagnostics', {
                        uri: params.textDocument.uri,
                        diagnostics: diagnostics
                    });
                }
            }
        });
    }

    async didClose(params: any): Promise<void> {
        console.log('[t27] didClose:', params.textDocument.uri);
        this.documents.delete(params.textDocument.uri);
    }

    async definition(params: any): Promise<any> {
        console.log('[t27] definition:', params.textDocumentPosition);

        const result = {
            uri: params.textDocument.uri,
            range: params.textDocumentPosition.range
        };

        // In a real implementation, this would query the t27 Language Server
        // For now, return the location directly
        console.log('[t27] definition result:', result);
        return result;
    }

    async references(params: any): Promise<any> {
        console.log('[t27] references:', params.textDocumentPosition);

        const result = [{
            uri: params.textDocument.uri,
            range: params.textDocumentPosition.range
        }];

        console.log('[t27] references result:', result);
        return result;
    }

    async documentSymbol(params: any): Promise<any> {
        console.log('[t27] documentSymbol:', params.textDocument.uri);

        const result: this.getDocumentSymbols(params.textDocument);
        console.log('[t27] documentSymbol result:', result);
        return result;
    }

    async workspaceSymbols(params: any): Promise<any> {
        console.log('[t27] workspaceSymbols:', params.query);

        // For now, return empty array
        const result = [];
        console.log('[t27] workspaceSymbols result:', result);
        return result;
    }

    async semanticTokens(params: any): Promise<any> {
        console.log('[t27] semanticTokens:', params.textDocument.uri);

        const result = {
            data: this.encodeSemanticTokens(params.textDocument)
        };

        console.log('[t27] semanticTokens result:', result);
        return result;
    }

    async codeAction(params: any): Promise<any> {
        console.log('[t27] codeAction:', params.range);

        // For now, return empty array
        const result: [];
        console.log('[t27] codeAction result:', result);
        return result;
    }

    async signatureHelp(params: any): Promise<any> {
        console.log('[t27] signatureHelp:', params.textDocumentPosition);

        // For now, return null
        const result = null;
        console.log('[t27] signatureHelp result:', result);
        return result;
    }

    async hover(params: any): Promise<any> {
        console.log('[t27] hover:', params.textDocumentPosition);

        const result = {
            contents: 'Hover for t27'
        };

        console.log('[t27] hover result:', result);
        return result;
    }

    async completion(params: any): Promise<any> {
        console.log('[t27] completion:', params.textDocumentPosition);

        // For now, return empty array
        const result = [];
        console.log('[t27] completion result:', result);
        return result;
    }

    /**
     * Validate a t27 document and return diagnostics
     */
    private validateDocument(textDocument: any): Diagnostic[] {
        const diagnostics: Diagnostic[] = [];

        // Parse basic syntax errors (stub)
        if (textDocument.getText().includes('TODO')) {
            diagnostics.push({
                severity: 4, // Information
                range: {
                    start: { line: 0, character: 0 },
                    end: { line: 0, character: 4 }
                },
                source: 't27',
                message: 'TODO found'
            });
        }

        return diagnostics;
    }

    /**
     * Get document symbols for a document
     */
    private getDocumentSymbols(textDocument: any): DocumentSymbol[] {
        const symbols: DocumentSymbol[] = [];

        // Parse functions (stub)
        const text = textDocument.getText();
        const functionRegex = /\b(test|invariant|bench|const|fn|import|let|return)\b/g;
        const matches = text.matchAll(functionRegex);

        for (const match of matches) {
            symbols.push({
                name: match[0],
                kind: 12, // Function
                range: {
                    start: { line: match.index, character: 0 },
                    end: { line: match.index, character: match[0].length }
                },
                children: []
            });
        }

        return symbols;
    }

    /**
     * Get semantic tokens legend
     */
    private getSemanticTokensLegend() {
        return {
            tokenTypes: ['keyword', 'type', 'function', 'variable', 'comment'],
            tokenModifiers: ['readonly', 'definition'],
            tokenModifiersLegend: {
                readonly: { description: 'Read-only' },
                definition: { description: 'Definition' }
            },
            tokenTypesLegend: {
                keyword: { description: 't27 keyword', foreground: '0x42A5F5' },
                type: { description: 't27 type', foreground: '0x1E1E1' },
                function: { description: 't27 function', foreground: '0x5697D9' },
                variable: { description: 't27 variable', foreground: '0xCE9178' },
                comment: { description: 't27 comment', foreground: '0x6A997B' }
            }
        };
    }

    /**
     * Encode semantic tokens for a document
     */
    private encodeSemanticTokens(textDocument: any): any[] {
        // For now, return empty array
        return [];
    }
}
