#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Position {
    /// Byte position. Zero-based.
    pub value: u32,
    pub line: u32,
    pub col: u32,
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

    use super::lexer::{self, Token, TokenKind};

    pub mod ast {
        #[derive(Debug)]
        pub struct TodoItem {
            pub message: String,
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
                items.push(Err(ParseError::UnexpectedEof));
                return ast::Text { items };
            }

            while token.kind != TokenKind::Eof {
                let item = match token.kind {
                    TokenKind::TodoKeyword => self.parse_todo(),
                    TokenKind::String | TokenKind::MultilineString => Err(ParseError::ExtraText),
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
                }),
                TokenKind::String => Ok(ast::Item::OneLine(ast::TodoItem {
                    message: token.text.to_string(),
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

                    Ok(ast::Item::Multiline(ast::TodoItem { message }))
                }
                TokenKind::Eof => Err(ParseError::UnexpectedEof),
            }
        }
    }

    pub type Result<T> = std::result::Result<T, ParseError>;

    #[derive(thiserror::Error, Debug)]
    pub enum ParseError {
        #[error("dangling text; without todo")]
        ExtraText,
        #[error("reached an unexpected end of file")]
        UnexpectedEof,
        #[error("expected a {expected}, but found a {found}")]
        UnexpectedToken {
            expected: TokenKind,
            found: TokenKind,
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
                    start: Position {
                        value: 0,
                        line: 0,
                        col: 0,
                    },
                },
                Token {
                    kind: String,
                    text: "run this test",
                    start: Position {
                        value: 5,
                        line: 0,
                        col: 5,
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 19,
                        line: 1,
                        col: 0,
                    },
                },
                Token {
                    kind: String,
                    text: "run this as well",
                    start: Position {
                        value: 24,
                        line: 1,
                        col: 5,
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 41,
                        line: 2,
                        col: 0,
                    },
                },
                Token {
                    kind: String,
                    text: "and this",
                    start: Position {
                        value: 46,
                        line: 2,
                        col: 5,
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
                            },
                        ),
                    ),
                    Ok(
                        OneLine(
                            TodoItem {
                                message: "run this as well",
                            },
                        ),
                    ),
                    Ok(
                        OneLine(
                            TodoItem {
                                message: "and this",
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
                            },
                        ),
                    ),
                    Ok(
                        Multiline(
                            TodoItem {
                                message: "run this test with a single line toodo\nas well as this multiline todo\nblah blah",
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
                        ExtraText,
                    ),
                    Err(
                        UnexpectedToken {
                            expected: String,
                            found: TodoKeyword,
                        },
                    ),
                    Err(
                        UnexpectedEof,
                    ),
                ],
            }
            "###);
        }
    }
}
