use anyhow::Context;
use todo::persist::ActualTodosDB;
use todo::Todos;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::lang::parser::ast;

#[derive(Debug)]
struct Backend {
    client: Client,
    todos: Todos<ActualTodosDB>,
}

struct ChangedDocumentItem {
    pub uri: Url,
    pub version: Option<i32>,
    pub text: String,
}

impl Backend {
    async fn on_change(&self, params: ChangedDocumentItem) {
        let text = ast::Text::from(params.text.as_str());

        let mut diags = vec![];

        for maybeitem in text.items {
            match maybeitem {
                Ok(item) => {
                    let todo = match item {
                        ast::Item::OneLine(t) => t,
                        ast::Item::Multiline(t) => t,
                    };

                    match self.todos.add(&todo.message) {
                        Ok(_) => {}
                        Err(err) => {
                            return self
                                .client
                                .log_message(MessageType::ERROR, format!("{err}"))
                                .await
                        }
                    };
                }
                Err(err) => diags.push(Diagnostic::new_simple(Range::default(), err.to_string())),
            }
        }

        return self
            .client
            .publish_diagnostics(params.uri, diags, params.version)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: None,
                }),
                hover_provider: None,
                document_formatting_provider: None,
                ..Default::default()
            },
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "todolang server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(ChangedDocumentItem {
            uri: params.text_document.uri,
            version: Some(params.text_document.version),
            text: params.text_document.text,
        })
        .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let text = match std::fs::read_to_string(params.text_document.uri.path())
            .context("failed to read file after save")
        {
            Ok(text) => text,
            Err(err) => {
                self.client
                    .log_message(MessageType::WARNING, format!("{err:#}"))
                    .await;
                return;
            }
        };

        self.on_change(ChangedDocumentItem {
            uri: params.text_document.uri,
            version: None,
            text,
        })
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(ChangedDocumentItem {
            uri: params.text_document.uri,
            version: Some(params.text_document.version),
            text: params.content_changes[0].text.clone(),
        })
        .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub fn start() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run());
}

async fn run() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        todos: Todos::load_up_with_persistor(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
