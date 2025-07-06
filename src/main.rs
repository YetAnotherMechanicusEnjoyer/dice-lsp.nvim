use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[derive(Debug)]
enum Block {
    If { has_else: bool },
}

impl Backend {
    async fn check_syntax(&self, uri: Url, text: String) {
        let diagnostics = analyze_file(&text);

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "dice-lsp".into(),
                version: Some("0.1.0".into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Dice LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.check_syntax(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let content = &params.content_changes[0].text;
        self.check_syntax(params.text_document.uri, content.clone())
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text.unwrap_or_default();
        self.check_syntax(uri, content).await;
    }
}

pub fn analyze_file(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut variables: HashMap<String, String> = HashMap::new();
    let mut if_stack: Vec<(usize, Block)> = Vec::new();
    let mut paren_stack: VecDeque<(usize, usize)> = VecDeque::new();

    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        for (j, ch) in line.char_indices() {
            match ch {
                '(' => paren_stack.push_back((i, j)),
                ')' => {
                    if paren_stack.pop_back().is_none() {
                        diagnostics.push(Diagnostic {
                            range: Range::new(
                                Position::new(i as u32, j as u32),
                                Position::new(i as u32, (j + 1) as u32),
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: "Unmatched ')'".to_string(),
                            ..Default::default()
                        });
                    }
                }
                _ => {}
            }
        }
        let trimmed = line.trim();

        if trimmed.contains(';') {
            diagnostics.push(Diagnostic {
                range: Range::new(
                    Position::new(i as u32, 0),
                    Position::new(i as u32, trimmed.len() as u32),
                ),
                severity: Some(DiagnosticSeverity::WARNING),
                message: "Semicolons are unnecessary in Dice".to_string(),
                ..Default::default()
            });
        }

        if let Some(rest) = trimmed.strip_prefix("let ") {
            let parts: Vec<&str> = rest.split('=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim();
                let value = parts[1].trim();
                if let Some(vtype) = infer_type(value, &variables) {
                    variables.insert(name.to_string(), vtype.to_string());
                }
            }
        }

        if trimmed.starts_with("if ") && trimmed.ends_with("then") {
            let condition = trimmed
                .strip_prefix("if")
                .unwrap()
                .strip_suffix("then")
                .unwrap()
                .trim();

            let invalid_cmp = [">=", "<=", ">", "<", "==", "!="]
                .iter()
                .find(|op| condition.contains(**op));

            if let Some(op) = invalid_cmp {
                let parts: Vec<&str> = condition.splitn(2, op).collect();
                if parts.len() == 2 {
                    let left = parts[0].trim();
                    let right = parts[1].trim();
                    let left_type = infer_type(left, &variables);
                    let right_type = infer_type(right, &variables);

                    if let (Some(lt), Some(rt)) = (left_type, right_type) {
                        if lt != rt {
                            diagnostics.push(Diagnostic {
                                range: Range::new(
                                    Position::new(i as u32, 0),
                                    Position::new(i as u32, line.len() as u32),
                                ),
                                severity: Some(DiagnosticSeverity::ERROR),
                                message: format!(
                                    "Type mismatch in condition: cannot compare {} and {}",
                                    lt, rt
                                ),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }

        if trimmed.starts_with("if ") && trimmed.contains(" then") {
            if_stack.push((i, Block::If { has_else: false }));
        } else if trimmed == "else" {
            match if_stack.last_mut() {
                Some((_, Block::If { has_else })) => {
                    if *has_else {
                        diagnostics.push(Diagnostic {
                            range: Range::new(
                                Position::new(i as u32, 0),
                                Position::new(i as u32, 4),
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: "Multiple 'else' blocks for one 'if'".to_string(),
                            ..Default::default()
                        });
                    } else {
                        *has_else = true;
                    }
                }
                None => {
                    diagnostics.push(Diagnostic {
                        range: Range::new(Position::new(i as u32, 0), Position::new(i as u32, 4)),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Unmatched 'else' with no open 'if'".to_string(),
                        ..Default::default()
                    });
                }
            }
        } else if trimmed == "end" {
            if let Some((_line, _block)) = if_stack.pop() {
                // OK
            } else {
                diagnostics.push(Diagnostic {
                    range: Range::new(Position::new(i as u32, 0), Position::new(i as u32, 3)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Unmatched 'end' with no corresponding 'if'".to_string(),
                    ..Default::default()
                });
            }
        }
    }

    for (line_num, _) in if_stack {
        diagnostics.push(Diagnostic {
            range: Range::new(
                Position::new(line_num as u32, 0),
                Position::new(line_num as u32, 2),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Unclosed 'if' block, missing 'end'".to_string(),
            ..Default::default()
        });
    }

    for (i, j) in paren_stack {
        diagnostics.push(Diagnostic {
            range: Range::new(
                Position::new(i as u32, j as u32),
                Position::new(i as u32, (j + 1) as u32),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            message: "Unmatched '('".to_string(),
            ..Default::default()
        });
    }

    diagnostics
}

fn infer_type<'a>(expr: &str, variables: &'a HashMap<String, String>) -> Option<&'a str> {
    let expr = expr.trim();

    if expr.starts_with('"') && expr.ends_with('"') {
        Some("string")
    } else if expr.parse::<i64>().is_ok() {
        Some("int")
    } else if let Some(var_type) = variables.get(expr) {
        Some(var_type.as_str())
    } else {
        None
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend { client }).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
