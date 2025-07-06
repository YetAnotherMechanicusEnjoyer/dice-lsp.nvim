use std::sync::Arc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
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
        let uri = &params.text_document.uri;
        let text = &params.text_document.text;

        let mut diagnostics = Vec::new();
        let mut parens = 0;
        for (i, line) in text.lines().enumerate() {
            for c in line.chars() {
                if c == '(' {
                    parens += 1;
                } else if c == ')' {
                    parens -= 1;
                    if parens < 0 {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position::new(i as u32, 0),
                                end: Position::new(i as u32, line.len() as u32),
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: "Unmatched closing ')'".into(),
                            ..Default::default()
                        });
                    }
                }
            }

            if line.ends_with(';') {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position::new(i as u32, (line.len() - 1) as u32),
                        end: Position::new(i as u32, line.len() as u32),
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: "Semicolons are not required in Dice".into(),
                    ..Default::default()
                });
            }
        }

        if parens > 0 {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 1),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Unmatched opening '('".into(),
                ..Default::default()
            });
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend { client }).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
