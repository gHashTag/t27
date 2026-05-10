const vscode = require('vscode');
const path = require('path');
const { spawn } = require('child_process');

class T27LanguageServer {
    constructor() {
        this.server = null;
        this.outputChannel = vscode.window.createOutputChannel('t27 Language Server');
    }

    start() {
        const config = vscode.workspace.getConfiguration('t27');
        const serverPath = config.get('t27ServerPath') || 'cargo run --package t27-language-server';

        this.outputChannel.appendLine(`Starting t27 Language Server...`);
        this.outputChannel.appendLine(`Command: ${serverPath}`);

        const serverArgs = ['stdio'];  // Use stdio for LSP communication
        const serverOptions = {
            cwd: path.join(__dirname, '..', '..'),
            env: {
                PATH: process.env.PATH // Ensure cargo is in PATH
            }
        };

        this.server = spawn(serverPath, serverArgs, serverOptions);

        this.server.stdout.on('data', (data) => {
            this.outputChannel.append(data.toString());
        });

        this.server.stderr.on('data', (data) => {
            this.outputChannel.append(`Error: ${data.toString()}`);
        });

        this.server.on('close', (code) => {
            if (code !== 0) {
                this.outputChannel.appendLine(`t27 Language Server exited with code ${code}`);
            }
        });

        // Handle workspace changes
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('t27.t27ServerPath')) {
                this.restartServer();
            }
        });
    }

    restartServer() {
        if (this.server) {
            this.outputChannel.appendLine('Stopping t27 Language Server...');
            this.server.kill();
            this.server = null;
        }
        this.start();
    }

    dispose() {
        if (this.server) {
            this.server.kill();
            this.server = null;
        }
    }
}

function activate(context) {
    const t27 = new T27LanguageServer();

    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('t27')) {
                t27.restartServer();
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('vscode-t27.openLog', () => {
            t27.outputChannel.show(true);
        })
    );

    t27.start();

    return {
        disposable: vscode.Disposable.from(() => {
            t27.dispose();
        })
    };
}

exports.activate = activate;
