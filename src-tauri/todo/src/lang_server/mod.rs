use std::collections::{HashMap, HashSet};

use anyhow::Context;
use dashmap::DashMap;
use serde_json::Value;
use todo::persist::ActualTodosDB;
use todo::{TodoID, Todos};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::lang;
use crate::lang::parser::ast;

#[derive(Debug)]
struct Backend {
    client: Client,
    todos: Todos<ActualTodosDB>,
    /// To remember todo in the edit buffer by their start byte position
    previous: DashMap<Url, HashSet<TodoID>>,
}

struct ChangedDocumentItem {
    pub uri: Url,
    pub version: Option<i32>,
    pub text: String,
}

impl lang::Position {
    fn into_lsp_pos(self) -> Position {
        Position {
            line: self.line,
            character: self.col,
        }
    }
}

impl lang::Span {
    fn into_lsp_range(self) -> Range {
        Range {
            start: self.start.into_lsp_pos(),
            end: self.end.into_lsp_pos(),
        }
    }
}

impl lang::parser::ParseError {
    fn span(&self) -> &lang::Span {
        match self {
            lang::parser::ParseError::ExtraText(s) => s,
            lang::parser::ParseError::UnexpectedEof(s) => s,
            lang::parser::ParseError::UnexpectedToken { span, .. } => span,
        }
    }
}

impl Backend {
    async fn on_change(&self, params: ChangedDocumentItem) {
        if let Err(_err) = self.todos.reload() {
            self.client
                .log_message(MessageType::ERROR, "failed to reload todos")
                .await
        };

        let text = ast::Text::from(params.text.as_str());

        let mut previous = self.previous.entry(params.uri.clone()).or_default();

        self.client
            .log_message(
                MessageType::INFO,
                format!("previous: {:?}", previous.clone()),
            )
            .await;

        let mut diagnostics = vec![];
        let mut new_previous = HashSet::new();

        for maybeitem in text.items {
            match maybeitem {
                Ok(item) => {
                    let todo = match item {
                        ast::Item::OneLine(t) => t,
                        ast::Item::Multiline(t) => t,
                    };

                    match self.todos.add(&todo.message) {
                        Ok(todo) => {
                            previous.remove(&todo.id);
                            new_previous.insert(todo.id);

                            self.client
                                .log_message(
                                    MessageType::INFO,
                                    format!("added todo message: '{}'", todo.message),
                                )
                                .await
                        }
                        Err(err) => {
                            self.client
                                .log_message(MessageType::ERROR, format!("{err}"))
                                .await
                        }
                    };
                }
                Err(err) => {
                    diagnostics.push(Diagnostic::new_simple(
                        err.span().into_lsp_range(),
                        err.to_string(),
                    ));
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("[Diagnostic] {err}: {:?}", err.span()),
                        )
                        .await;
                }
            }
        }

        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, params.version)
            .await;

        for todoid in previous.iter() {
            if let Err(err) = self.todos.remove(&todoid.0) {
                self.client
                    .log_message(MessageType::ERROR, format!("{err}"))
                    .await
            };
        }

        if let Err(err) = self.todos.flush() {
            self.client
                .log_message(MessageType::ERROR, format!("{err:#}"))
                .await
        }

        drop(previous);

        self.client
            .log_message(
                MessageType::INFO,
                format!("new previous: {:?}", new_previous),
            )
            .await;

        self.previous.insert(params.uri.clone(), new_previous);
    }

    async fn read_text_by_uri(&self, uri: Url) -> Option<String> {
        return match std::fs::read_to_string(uri.path()).context("failed to read file after save") {
            Ok(text) => Some(text),
            Err(err) => {
                self.client
                    .log_message(MessageType::WARNING, format!("{err:#}"))
                    .await;
                return None;
            }
        };
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
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["mark_done".to_string()],
                    ..Default::default()
                }),
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
                return self
                    .client
                    .log_message(MessageType::WARNING, format!("{err:#}"))
                    .await;
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

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.previous.remove(&params.text_document.uri);
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let Some(text) = self.read_text_by_uri(params.text_document.uri).await else {
            return Ok(None);
        };

        let text = ast::Text::from(text.as_ref());
        let todos = match self.todos.get_all() {
            Ok(list) => list,
            Err(err) => {
                self.client
                    .log_message(MessageType::ERROR, format!("{err:#}"))
                    .await;
                return Ok(None);
            }
        };

        let todos = todos
            .into_iter()
            .map(|todo| (todo.id.clone(), todo))
            .collect::<HashMap<_, _>>();

        let codelenses: Vec<_> = text
            .items
            .into_iter()
            .flatten()
            .map(|item| match item {
                ast::Item::OneLine(t) => t,
                ast::Item::Multiline(t) => t,
            })
            .filter_map(|item| {
                let todoid = todo::TodoID::hash_message(&item.message);
                if let Some(todo) = todos.get(&todoid) {
                    let is_done = if todo.done { "[x]" } else { "[ ]" };
                    let creation_time =
                        format!("created on: {}", todo.created_at.to_local_date_string());

                    return Some(CodeLens {
                        range: item.span.into_lsp_range(),
                        data: None,
                        command: Some(Command {
                            title: format!("{}, {}", is_done, creation_time),
                            command: "mark_done".to_string(), // TODO: implement this...
                            arguments: Some(vec![Value::String(todo.id.0.to_string())]),
                        }),
                    });
                }

                None
            })
            .collect();

        return Ok(Some(codelenses));
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        if params.command == "mark_done" {
            let todoid = params.arguments[0]
                .as_str()
                .expect("mark_done command should be set up in codelenses to the todo id string");

            if let Err(err) = self.todos.mark_done(todoid) {
                self.client
                    .log_message(MessageType::ERROR, format!("{err:#}"))
                    .await;

                return Err(tower_lsp::jsonrpc::Error {
                    code: tower_lsp::jsonrpc::ErrorCode::InternalError,
                    message: format!("{err:#}").into(),
                    data: None,
                });
            };

            return Ok(None);
        }
        Ok(None)
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
        previous: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
