//! TAYNI Language Server
//! Provides IDE support for TAYNI programming language

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug)]
struct TayniLanguageServer {
    client: Client,
    documents: RwLock<HashMap<Url, String>>,
}

impl TayniLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    async fn validate_document(&self, uri: &Url, text: &str) {
        let diagnostics = self.parse_and_diagnose(text);
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    fn parse_and_diagnose(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        for (line_num, line) in text.lines().enumerate() {
            // Check for common TAYNI syntax issues
            
            // Check for unclosed strings
            let quote_count = line.chars().filter(|c| *c == '"').count();
            if quote_count % 2 != 0 {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_num as u32, character: 0 },
                        end: Position { line: line_num as u32, character: line.len() as u32 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("E001".to_string())),
                    source: Some("tayni".to_string()),
                    message: "Unclosed string literal".to_string(),
                    ..Default::default()
                });
            }
            
            // Check for invalid operators
            if line.contains("===") {
                if let Some(pos) = line.find("===") {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_num as u32, character: pos as u32 },
                            end: Position { line: line_num as u32, character: (pos + 3) as u32 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("E002".to_string())),
                        source: Some("tayni".to_string()),
                        message: "Invalid operator '==='. Use '==' for equality comparison.".to_string(),
                        ..Default::default()
                    });
                }
            }
            
            // Check for missing semicolons in v1.0 style (lines ending with operations)
            let trimmed = line.trim();
            if !trimmed.is_empty() 
                && !trimmed.starts_with("//") 
                && !trimmed.starts_with('#')
                && !trimmed.ends_with('{')
                && !trimmed.ends_with('}')
                && !trimmed.ends_with(':')
                && !trimmed.contains("->")
                && (trimmed.starts_with("LET") || trimmed.starts_with("SET") || trimmed.starts_with("PRT"))
            {
                // v1.0 syntax check - these should be valid
            }
            
            // Warn about deprecated syntax
            if line.contains("PRINT") && !line.contains("//") {
                if let Some(pos) = line.find("PRINT") {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_num as u32, character: pos as u32 },
                            end: Position { line: line_num as u32, character: (pos + 5) as u32 },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        code: Some(NumberOrString::String("W001".to_string())),
                        source: Some("tayni".to_string()),
                        message: "Consider using 'PRT' for consistency with TAYNI v1.5 syntax.".to_string(),
                        ..Default::default()
                    });
                }
            }
        }
        
        diagnostics
    }

    fn get_hover_info(&self, text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }
        
        let line = lines[position.line as usize];
        let char_pos = position.character as usize;
        
        // Find word at position
        let word = self.get_word_at_position(line, char_pos)?;
        
        // Return documentation for known keywords/operations
        match word.to_uppercase().as_str() {
            "LET" => Some("**LET** - Variable declaration\n\nDeclares a new variable.\n\n```tayni\nLET x = 42\nLET name = \"hello\"\n```".to_string()),
            "SET" => Some("**SET** - Variable assignment\n\nAssigns a value to an existing variable.\n\n```tayni\nSET x = x + 1\n```".to_string()),
            "PRT" => Some("**PRT** - Print output\n\nPrints a value to stdout.\n\n```tayni\nPRT \"Hello, World!\"\nPRT x\n```".to_string()),
            "FN" | "fn" => Some("**FN** - Function definition\n\nDefines a new function.\n\n```tayni\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n```".to_string()),
            "IF" | "if" => Some("**IF** - Conditional\n\nConditional execution.\n\n```tayni\nif x > 0 {\n    PRT \"positive\"\n}\n```".to_string()),
            "LOOP" | "loop" => Some("**LOOP** - Loop construct\n\nRepeats a block.\n\n```tayni\nloop 10 {\n    PRT i\n}\n```".to_string()),
            "CAP" | "cap" => Some("**CAP** - Capability declaration\n\nDeclares required capabilities.\n\n```tayni\ncap net:tcp\ncap fs:read\n```".to_string()),
            "ADD" => Some("**ADD** - Addition\n\nAdds two values.\n\n```tayni\nADD a b  // a + b\n```".to_string()),
            "SUB" => Some("**SUB** - Subtraction\n\nSubtracts two values.\n\n```tayni\nSUB a b  // a - b\n```".to_string()),
            "MUL" => Some("**MUL** - Multiplication\n\nMultiplies two values.\n\n```tayni\nMUL a b  // a * b\n```".to_string()),
            "DIV" => Some("**DIV** - Division\n\nDivides two values.\n\n```tayni\nDIV a b  // a / b\n```".to_string()),
            "CMP" => Some("**CMP** - Comparison\n\nCompares two values.\n\n```tayni\nCMP a b  // returns -1, 0, or 1\n```".to_string()),
            "JMP" => Some("**JMP** - Jump\n\nUnconditional jump to label.\n\n```tayni\nJMP loop_start\n```".to_string()),
            "JEQ" => Some("**JEQ** - Jump if equal\n\nJumps if last comparison was equal.\n\n```tayni\nJEQ end_loop\n```".to_string()),
            "JNE" => Some("**JNE** - Jump if not equal\n\nJumps if last comparison was not equal.\n\n```tayni\nJNE continue\n```".to_string()),
            "RET" => Some("**RET** - Return\n\nReturns from function.\n\n```tayni\nRET result\n```".to_string()),
            "CALL" => Some("**CALL** - Function call\n\nCalls a function.\n\n```tayni\nCALL my_function arg1 arg2\n```".to_string()),
            _ => None,
        }
    }

    fn get_word_at_position(&self, line: &str, pos: usize) -> Option<String> {
        if pos >= line.len() {
            return None;
        }
        
        let chars: Vec<char> = line.chars().collect();
        
        // Find word boundaries
        let mut start = pos;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        let mut end = pos;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        
        if start == end {
            return None;
        }
        
        Some(chars[start..end].iter().collect())
    }

    fn get_completions(&self, text: &str, position: Position) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        
        // TAYNI v1.0 keywords
        let v1_keywords = vec![
            ("LET", "Variable declaration", "LET ${1:name} = ${2:value}"),
            ("SET", "Variable assignment", "SET ${1:name} = ${2:value}"),
            ("PRT", "Print output", "PRT ${1:value}"),
            ("ADD", "Addition", "ADD ${1:a} ${2:b}"),
            ("SUB", "Subtraction", "SUB ${1:a} ${2:b}"),
            ("MUL", "Multiplication", "MUL ${1:a} ${2:b}"),
            ("DIV", "Division", "DIV ${1:a} ${2:b}"),
            ("CMP", "Comparison", "CMP ${1:a} ${2:b}"),
            ("JMP", "Jump", "JMP ${1:label}"),
            ("JEQ", "Jump if equal", "JEQ ${1:label}"),
            ("JNE", "Jump if not equal", "JNE ${1:label}"),
            ("RET", "Return", "RET ${1:value}"),
            ("CALL", "Function call", "CALL ${1:function}"),
        ];
        
        // TAYNI v1.5 keywords
        let v15_keywords = vec![
            ("fn", "Function definition", "fn ${1:name}(${2:params}) -> ${3:type} {\n\t$0\n}"),
            ("if", "Conditional", "if ${1:condition} {\n\t$0\n}"),
            ("else", "Else branch", "else {\n\t$0\n}"),
            ("loop", "Loop", "loop ${1:count} {\n\t$0\n}"),
            ("while", "While loop", "while ${1:condition} {\n\t$0\n}"),
            ("cap", "Capability", "cap ${1:capability}"),
            ("use", "Import", "use ${1:module}"),
            ("struct", "Structure", "struct ${1:Name} {\n\t$0\n}"),
            ("return", "Return value", "return ${1:value}"),
        ];
        
        // Types
        let types = vec![
            ("i32", "32-bit integer"),
            ("i64", "64-bit integer"),
            ("f32", "32-bit float"),
            ("f64", "64-bit float"),
            ("str", "String type"),
            ("bool", "Boolean type"),
            ("void", "No return value"),
        ];
        
        // Capabilities
        let capabilities = vec![
            ("net:tcp", "TCP networking"),
            ("net:http", "HTTP client/server"),
            ("fs:read", "File system read"),
            ("fs:write", "File system write"),
            ("time", "Time operations"),
            ("env", "Environment variables"),
        ];
        
        for (label, detail, snippet) in v1_keywords {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("v1.0 - {}", detail)),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
        
        for (label, detail, snippet) in v15_keywords {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("v1.5 - {}", detail)),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
        
        for (label, detail) in types {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                detail: Some(detail.to_string()),
                ..Default::default()
            });
        }
        
        for (label, detail) in capabilities {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some(format!("Capability: {}", detail)),
                ..Default::default()
            });
        }
        
        items
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for TayniLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "tayni-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "TAYNI Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        
        self.documents.write().unwrap().insert(uri.clone(), text.clone());
        self.validate_document(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;
            self.documents.write().unwrap().insert(uri.clone(), text.clone());
            self.validate_document(&uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.write().unwrap().remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let documents = self.documents.read().unwrap();
        if let Some(text) = documents.get(uri) {
            if let Some(info) = self.get_hover_info(text, position) {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: info,
                    }),
                    range: None,
                }));
            }
        }
        
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        let documents = self.documents.read().unwrap();
        let text = documents.get(uri).map(|s| s.as_str()).unwrap_or("");
        
        let items = self.get_completions(text, position);
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // Basic implementation - find function/variable definitions
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let documents = self.documents.read().unwrap();
        if let Some(text) = documents.get(uri) {
            let lines: Vec<&str> = text.lines().collect();
            if let Some(line) = lines.get(position.line as usize) {
                if let Some(word) = self.get_word_at_position(line, position.character as usize) {
                    // Search for definition
                    for (line_num, search_line) in lines.iter().enumerate() {
                        // Look for LET declarations
                        if search_line.contains(&format!("LET {} ", word)) 
                            || search_line.contains(&format!("LET {}=", word))
                            || search_line.contains(&format!("fn {}(", word))
                            || search_line.contains(&format!("fn {} (", word))
                        {
                            let char_pos = search_line.find(&word).unwrap_or(0);
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: line_num as u32,
                                        character: char_pos as u32,
                                    },
                                    end: Position {
                                        line: line_num as u32,
                                        character: (char_pos + word.len()) as u32,
                                    },
                                },
                            })));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| TayniLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
