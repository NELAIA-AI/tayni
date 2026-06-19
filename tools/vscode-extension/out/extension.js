"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    console.log('TAYNI extension is now active');
    const config = vscode.workspace.getConfiguration('tayni');
    const lspEnabled = config.get('lsp.enabled', true);
    if (!lspEnabled) {
        console.log('TAYNI LSP is disabled');
        return;
    }
    // Get LSP path from config or use default
    let lspPath = config.get('lsp.path', '');
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
    const serverOptions = {
        run: {
            command: lspPath,
            transport: node_1.TransportKind.stdio
        },
        debug: {
            command: lspPath,
            transport: node_1.TransportKind.stdio
        }
    };
    const clientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'tayni' }
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{tyn,tayni}')
        }
    };
    client = new node_1.LanguageClient('tayni-lsp', 'TAYNI Language Server', serverOptions, clientOptions);
    // Start the client (also starts the server)
    client.start().catch(err => {
        console.error('Failed to start TAYNI LSP:', err);
        vscode.window.showWarningMessage('TAYNI Language Server not found. Some features may be limited. ' +
            'Install tayni-lsp or set tayni.lsp.path in settings.');
    });
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('tayni.restartLsp', async () => {
        if (client) {
            await client.stop();
            await client.start();
            vscode.window.showInformationMessage('TAYNI LSP restarted');
        }
    }));
    context.subscriptions.push(vscode.commands.registerCommand('tayni.showVersion', () => {
        vscode.window.showInformationMessage('TAYNI Extension v0.1.0');
    }));
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map