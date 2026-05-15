import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    console.log('t27 extension is activating');

    // Register LSP client
    const config = vscode.workspace.getConfiguration('t27');
    const serverPath = config.get('server.path') as string;

    const clientOptions: LanguageClientOptions = {
        run: {
            command: serverPath,
            args: ['--stdio'],
        },
        debug: {
            command: serverPath,
            args: ['--stdio'],
        },
        options: {
            env: {
                ...process.env,
                RUST_LOG: config.get('trace.server')
            }
        }
    };

    client = new LanguageClient(
        't27',
        't27 Language Server',
        clientOptions
    );

    client.start().catch((error) => {
        console.error(`Failed to start t27 LSP server: ${error}`);
    });

    // Register commands
    const runTestsCommand = vscode.commands.registerCommand('t27.runTests', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (activeEditor) {
            const document = activeEditor.document;
            if (document.uri.fsPath) {
                await runTests(document.uri.fsPath);
            }
        }
    });

    const generateCommand = vscode.commands.registerCommand('t27.generate', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (activeEditor) {
            const document = activeEditor.document;
            if (document.uri.fsPath) {
                await generateCode(document.uri.fsPath);
            }
        }
    });

    const parseCommand = vscode.commands.registerCommand('t27.parse', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (activeEditor) {
            const document = activeEditor.document;
            if (document.uri.fsPath) {
                await parseSpec(document.uri.fsPath);
            }
        }
    });

    context.subscriptions.push(
        runTestsCommand,
        generateCommand,
        parseCommand
    );

    // Show welcome message
    if (!context.globalState.get('welcomeShown')) {
        vscode.window.showInformationMessage(
            'Welcome to t27 — Spec-first ternary programming language!',
            'Get Started',
            'View Documentation'
        ).then(selection => {
            if (selection === 'Get Started') {
                vscode.env.openExternal(vscode.Uri.parse('https://trinity-s3ai.org/docs/tutorials/001-getting-started.md'));
            } else if (selection === 'View Documentation') {
                vscode.env.openExternal(vscode.Uri.parse('https://trinity-s3ai.org/docs/'));
            }
            context.globalState.update('welcomeShown', true);
        });
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

async function runTests(filePath: string): Promise<void> {
    const terminal = vscode.window.createTerminal('t27 Tests');
    terminal.sendText(`cd $(dirname ${filePath}) && tri test\n`);
    terminal.show();
}

async function generateCode(filePath: string): Promise<void> {
    const terminal = vscode.window.createTerminal('t27 Generate');
    terminal.sendText(`tri gen ${filePath}\n`);
    terminal.show();
}

async function parseSpec(filePath: string): Promise<void> {
    const terminal = vscode.window.createTerminal('t27 Parse');
    terminal.sendText(`tri parse ${filePath}\n`);
    terminal.show();
}