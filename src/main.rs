use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl Backend {
    async fn check_syntax(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();
        let mut open_parens: Vec<(usize, usize)> = Vec::new();

        for (i, line) in text.lines().enumerate() {
            for (j, c) in line.chars().enumerate() {
                match c {
                    '(' => open_parens.push((i, j)),
                    ')' => {
                        if open_parens.pop().is_none() {
                            diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position::new(i as u32, j as u32),
                                    end: Position::new(i as u32, (j + 1) as u32),
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                message: "Unmatched ')'".to_string(),
                                ..Default::default()
                            });
                        }
                    }
                    _ => {}
                }
            }

            if line.trim_end().ends_with(';') {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position::new(i as u32, (line.len() - 1) as u32),
                        end: Position::new(i as u32, line.len() as u32),
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: "Semicolons are unnecessary in Dice".to_string(),
                    ..Default::default()
                });
            }
        }

        for (line, col) in open_parens {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position::new(line as u32, col as u32),
                    end: Position::new(line as u32, (col + 1) as u32),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Unmatched '('".to_string(),
                ..Default::default()
            });
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
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

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend { client }).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
