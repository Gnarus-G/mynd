#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Position {
    /// Byte position. Zero-based.
    pub value: u32,
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Position {
    fn spanning_to(&self, end: Position) -> Span {
        Span { start: *self, end }
    }
}

trait CharaterTest {
    fn passes<P: Fn(&u8) -> bool>(&self, predicate: P) -> bool;
}

impl CharaterTest for Option<&u8> {
    fn passes<P: Fn(&u8) -> bool>(&self, predicate: P) -> bool {
        match self {
            Some(c) => predicate(c),
            None => false,
        }
    }
}

pub mod lexer;

pub mod parser {

    use super::{
        lexer::{self, Token, TokenKind},
        Span,
    };

    pub mod ast {
        #[derive(Debug)]
        pub struct TodoItem {
            pub message: String,
            pub span: super::Span,
        }

        #[derive(Debug)]
        pub enum Item {
            OneLine(TodoItem),
            Multiline(TodoItem),
        }

        #[derive(Debug)]
        pub struct Text {
            pub items: Vec<super::Result<Item>>,
        }
    }

    impl<'src> From<&'src str> for ast::Text {
        fn from(value: &'src str) -> Self {
            Parser::new(lexer::Lexer::new(value)).parse()
        }
    }

    pub struct Parser<'src> {
        lexer: lexer::Lexer<'src>,
        peeked: Option<Token<'src>>,
    }

    impl<'src> Parser<'src> {
        pub fn new(lexer: lexer::Lexer<'src>) -> Self {
            Self {
                lexer,
                peeked: None,
            }
        }

        fn next_token(&mut self) -> Token<'src> {
            return match self.peeked.take() {
                Some(t) => t,
                None => self.lexer.next_token(),
            };
        }

        pub fn parse(&mut self) -> ast::Text {
            let mut items: Vec<Result<ast::Item>> = vec![];

            let mut token = self.next_token();

            if token.kind == TokenKind::Eof {
                items.push(Err(ParseError::UnexpectedEof(token.span)));
                return ast::Text { items };
            }

            while token.kind != TokenKind::Eof {
                let item = match token.kind {
                    TokenKind::TodoKeyword => self.parse_todo(),
                    TokenKind::String | TokenKind::MultilineString => {
                        Err(ParseError::ExtraText(token.span))
                    }
                    TokenKind::Eof => {
                        unreachable!("top level parse loop [should]only runs when token is not eof")
                    }
                };

                items.push(item);

                token = self.next_token();
            }

            return ast::Text { items };
        }

        fn parse_todo(&mut self) -> Result<ast::Item> {
            let token = self.next_token();
            match token.kind {
                TokenKind::TodoKeyword => Err(ParseError::UnexpectedToken {
                    expected: TokenKind::String,
                    found: TokenKind::TodoKeyword,
                    span: token.span,
                }),
                TokenKind::String => Ok(ast::Item::OneLine(ast::TodoItem {
                    message: token.text.to_string(),
                    span: token.span,
                })),
                TokenKind::MultilineString => {
                    let message = token
                        .text
                        .to_string()
                        .trim()
                        .lines()
                        .map(|line| line.trim_start())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");

                    Ok(ast::Item::Multiline(ast::TodoItem {
                        message,
                        span: token.span,
                    }))
                }
                TokenKind::Eof => Err(ParseError::UnexpectedEof(token.span)),
            }
        }
    }

    pub type Result<T> = std::result::Result<T, ParseError>;

    #[derive(thiserror::Error, Debug)]
    pub enum ParseError {
        #[error("dangling text; without todo")]
        ExtraText(Span),
        #[error("reached an unexpected end of file")]
        UnexpectedEof(Span),
        #[error("expected a {expected}, but found a {found}")]
        UnexpectedToken {
            expected: TokenKind,
            found: TokenKind,
            span: Span,
        },
    }

    #[cfg(test)]
    mod tests {
        use insta::assert_debug_snapshot;

        use crate::lang::lexer;

        use super::ast;

        #[test]
        fn parses_todos_singles() {
            let src = "todo run this test\ntodo run this as well\ntodo and this";

            let tokens: Vec<_> = lexer::Lexer::new(src).collect();

            assert_debug_snapshot!(tokens, @r###"
            [
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    span: Span {
                        start: Position {
                            value: 0,
                            line: 0,
                            col: 0,
                        },
                        end: Position {
                            value: 3,
                            line: 0,
                            col: 3,
                        },
                    },
                },
                Token {
                    kind: String,
                    text: "run this test",
                    span: Span {
                        start: Position {
                            value: 5,
                            line: 0,
                            col: 5,
                        },
                        end: Position {
                            value: 17,
                            line: 0,
                            col: 17,
                        },
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    span: Span {
                        start: Position {
                            value: 19,
                            line: 1,
                            col: 0,
                        },
                        end: Position {
                            value: 22,
                            line: 1,
                            col: 3,
                        },
                    },
                },
                Token {
                    kind: String,
                    text: "run this as well",
                    span: Span {
                        start: Position {
                            value: 24,
                            line: 1,
                            col: 5,
                        },
                        end: Position {
                            value: 39,
                            line: 1,
                            col: 20,
                        },
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    span: Span {
                        start: Position {
                            value: 41,
                            line: 2,
                            col: 0,
                        },
                        end: Position {
                            value: 44,
                            line: 2,
                            col: 3,
                        },
                    },
                },
                Token {
                    kind: String,
                    text: "and this",
                    span: Span {
                        start: Position {
                            value: 46,
                            line: 2,
                            col: 5,
                        },
                        end: Position {
                            value: 53,
                            line: 2,
                            col: 12,
                        },
                    },
                },
            ]
            "###);

            let text = ast::Text::from(src);

            assert_debug_snapshot!(text, @r###"
            Text {
                items: [
                    Ok(
                        OneLine(
                            TodoItem {
                                message: "run this test",
                                span: Span {
                                    start: Position {
                                        value: 5,
                                        line: 0,
                                        col: 5,
                                    },
                                    end: Position {
                                        value: 17,
                                        line: 0,
                                        col: 17,
                                    },
                                },
                            },
                        ),
                    ),
                    Ok(
                        OneLine(
                            TodoItem {
                                message: "run this as well",
                                span: Span {
                                    start: Position {
                                        value: 24,
                                        line: 1,
                                        col: 5,
                                    },
                                    end: Position {
                                        value: 39,
                                        line: 1,
                                        col: 20,
                                    },
                                },
                            },
                        ),
                    ),
                    Ok(
                        OneLine(
                            TodoItem {
                                message: "and this",
                                span: Span {
                                    start: Position {
                                        value: 46,
                                        line: 2,
                                        col: 5,
                                    },
                                    end: Position {
                                        value: 53,
                                        line: 2,
                                        col: 12,
                                    },
                                },
                            },
                        ),
                    ),
                ],
            }
            "###);
        }

        #[test]
        fn parses_todos_mix() {
            let src = r#"todo run this test

    todo {
        run this test with a single line toodo
        as well as this multiline todo
        blah blah
    }"#;

            let text = ast::Text::from(src);

            assert_debug_snapshot!(text, @r###"
            Text {
                items: [
                    Ok(
                        OneLine(
                            TodoItem {
                                message: "run this test",
                                span: Span {
                                    start: Position {
                                        value: 5,
                                        line: 0,
                                        col: 5,
                                    },
                                    end: Position {
                                        value: 17,
                                        line: 0,
                                        col: 17,
                                    },
                                },
                            },
                        ),
                    ),
                    Ok(
                        Multiline(
                            TodoItem {
                                message: "run this test with a single line toodo\nas well as this multiline todo\nblah blah",
                                span: Span {
                                    start: Position {
                                        value: 29,
                                        line: 2,
                                        col: 9,
                                    },
                                    end: Position {
                                        value: 139,
                                        line: 6,
                                        col: 4,
                                    },
                                },
                            },
                        ),
                    ),
                ],
            }
            "###);
        }

        #[test]
        fn parses_todos_errors() {
            let src = r#"run this test
    todo todo
    todo"#;

            let text = ast::Text::from(src);

            assert_debug_snapshot!(text, @r###"
            Text {
                items: [
                    Err(
                        ExtraText(
                            Span {
                                start: Position {
                                    value: 0,
                                    line: 0,
                                    col: 0,
                                },
                                end: Position {
                                    value: 12,
                                    line: 0,
                                    col: 12,
                                },
                            },
                        ),
                    ),
                    Err(
                        UnexpectedToken {
                            expected: String,
                            found: TodoKeyword,
                            span: Span {
                                start: Position {
                                    value: 23,
                                    line: 1,
                                    col: 9,
                                },
                                end: Position {
                                    value: 26,
                                    line: 1,
                                    col: 12,
                                },
                            },
                        },
                    ),
                    Err(
                        UnexpectedEof(
                            Span {
                                start: Position {
                                    value: 35,
                                    line: 2,
                                    col: 7,
                                },
                                end: Position {
                                    value: 35,
                                    line: 2,
                                    col: 7,
                                },
                            },
                        ),
                    ),
                ],
            }
            "###);
        }
    }
}
