import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('TAYNI extension is now active');

    const config = vscode.workspace.getConfiguration('tayni');
    const lspEnabled = config.get<boolean>('lsp.enabled', true);
    
    if (!lspEnabled) {
        console.log('TAYNI LSP is disabled');
        return;
    }

    // Get LSP path from config or use default
    let lspPath = config.get<string>('lsp.path', '');
    
    if (!lspPath) {
        // Try to find tayni-lsp in common locations
        const possiblePaths = [
            path.join(context.extensionPath, 'bin', 'tayni-lsp'),
            path.join(context.extensionPath, 'bin', 'tayni-lsp.exe'),
            'tayni-lsp', // In PATH
        ];
        
        // For now, just use the name and assume it's in PATH
        lspPath = 'tayni-lsp';
    }

    const serverOptions: ServerOptions = {
        run: {
            command: lspPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: lspPath,
            transport: TransportKind.stdio
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'tayni' }
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{tyn,tayni}')
        }
    };

    client = new LanguageClient(
        'tayni-lsp',
        'TAYNI Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client (also starts the server)
    client.start().catch(err => {
        console.error('Failed to start TAYNI LSP:', err);
        vscode.window.showWarningMessage(
            'TAYNI Language Server not found. Some features may be limited. ' +
            'Install tayni-lsp or set tayni.lsp.path in settings.'
        );
    });

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('tayni.restartLsp', async () => {
            if (client) {
                await client.stop();
                await client.start();
                vscode.window.showInformationMessage('TAYNI LSP restarted');
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('tayni.showVersion', () => {
            vscode.window.showInformationMessage('TAYNI Extension v0.1.0');
        })
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
