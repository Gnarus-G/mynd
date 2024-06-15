#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Position {
    /// Byte position. Zero-based.
    pub value: u32,
    pub line: u32,
    pub col: u32,
}

trait CharaterTest {
    fn passes<P: Fn(&u8) -> bool>(&self, predicate: P) -> bool;
    fn is(&self, ch: u8) -> bool {
        self.passes(|&c| ch == c)
    }
}

impl CharaterTest for Option<&u8> {
    fn passes<P: Fn(&u8) -> bool>(&self, predicate: P) -> bool {
        match self {
            Some(c) => predicate(c),
            None => false,
        }
    }
}

pub mod lexer {
    use std::fmt::Display;

    use super::{CharaterTest, Position};

    #[derive(Debug, PartialEq)]
    pub enum TokenKind {
        TodoKeyword,
        String,
        MultilineString,
        Eof,
    }

    impl Display for TokenKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TokenKind::TodoKeyword => write!(f, "todo keyword"),
                TokenKind::String => write!(f, "text"),
                TokenKind::MultilineString => write!(f, "text block"),
                TokenKind::Eof => write!(f, "EOF"),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct Token<'src> {
        pub kind: TokenKind,
        pub text: &'src str,
        pub start: Position,
    }

    impl<'src> Token<'src> {
        pub fn new(kind: TokenKind, text: &'src str, start: Position) -> Self {
            Token { kind, text, start }
        }
    }

    pub struct Lexer<'src> {
        position: Position,
        src: &'src [u8],
    }

    impl<'src> Lexer<'src> {
        pub fn new(src: &'src str) -> Self {
            Lexer {
                position: Position::default(),
                src: src.as_bytes(),
            }
        }

        fn input_slice(&self, range: (u32, u32)) -> &'src str {
            let (start, end) = (range.0 as usize, range.1 as usize);

            std::str::from_utf8(&self.src[start..end])
                .expect("input should only contain utf-8 characters")
        }

        fn char_at(&self, position: usize) -> Option<&u8> {
            if position < self.src.len() {
                return Some(&self.src[position]);
            }
            None
        }

        fn ch(&self) -> Option<&u8> {
            self.char_at(self.position.value as usize)
        }

        fn step(&mut self) {
            if self.peek_char().is_some() {
                self.check_and_bump_new_line();
            }
            self.position.value += 1;
        }

        fn check_and_bump_new_line(&mut self) {
            if let Some(b'\n') = self.ch() {
                self.position.line += 1;
                self.position.col = 0;
            } else {
                self.position.col += 1;
            };
        }

        fn peek_n_char(&self, n: u32) -> Option<&u8> {
            self.char_at((self.position.value + 1 + n) as usize)
        }

        fn peek_char(&self) -> Option<&u8> {
            self.peek_n_char(0)
        }

        /// Assumes that the character at the current position, immediately before calling
        /// this function is also true for the predicate function given.
        fn read_while<P: Fn(&u8) -> bool>(&mut self, predicate: P) -> (u32, u32) {
            let start_pos = self.position.value;

            while self.peek_char().passes(&predicate) {
                self.step();
            }

            (start_pos, self.position.value + 1)
        }

        fn skip_whitespace(&mut self) {
            while self.ch().passes(|c| c.is_ascii_whitespace()) {
                self.step();
            }
        }

        pub fn next_token(&mut self) -> Token<'src> {
            use TokenKind::*;
            self.skip_whitespace();

            let ch = match self.ch() {
                Some(ch) => ch,
                None => return Token::new(Eof, "", self.position),
            };

            let token = match ch {
                b'{' => self.multiline_string(),
                c if c.is_ascii_alphabetic() => self.keyword_or_identifier(),
                _ => self.string(None),
            };

            self.step();

            return token;
        }

        fn keyword_or_identifier(&mut self) -> Token<'src> {
            let location = self.position;
            let (s, e) = self.read_while(|&c| c.is_ascii_alphabetic() || c == b'_');
            let string = self.input_slice((s, e));

            use TokenKind::*;

            match string {
                "todo" => Token::new(TodoKeyword, string, location),
                _ => self.string(Some(location)),
            }
        }

        fn string(&mut self, start: Option<Position>) -> Token<'src> {
            let start_pos = start.unwrap_or(self.position);

            let (_, e) = self.read_while(|&c| c != b'\n');

            self.step();
            let string = self.input_slice((start_pos.value, e));

            Token::new(TokenKind::String, string, start_pos)
        }

        fn multiline_string(&mut self) -> Token<'src> {
            let start_pos = self.position;

            self.step(); // eat the '{'

            let (s, e) = self.read_while(|&c| c != b'}');

            self.step();

            let string = self.input_slice((s, e));

            Token::new(TokenKind::MultilineString, string, start_pos)
        }
    }

    #[cfg(test)]
    mod tests {
        use insta::assert_debug_snapshot;

        use super::*;
        use crate::lang::lexer;

        #[test]
        fn lexes_single_line_todo() {
            let src = "todo run this test";

            let mut lexer = lexer::Lexer::new(src);

            assert_eq!(
                lexer.next_token(),
                Token::new(TokenKind::TodoKeyword, "todo", Position::default())
            );
            assert_eq!(
                lexer.next_token(),
                Token::new(
                    TokenKind::String,
                    "run this test",
                    Position {
                        value: 5,
                        line: 0,
                        col: 5
                    }
                )
            );

            assert_eq!(
                lexer.next_token(),
                Token::new(
                    TokenKind::Eof,
                    "",
                    Position {
                        value: 19,
                        line: 0,
                        col: 17
                    }
                )
            );
        }

        #[test]
        fn lexes_multi_line_todo() {
            let src = r#"
todo run this test
todo {
    run this test with a single line toodo
    as well as this multiline todo
    blah blah
}"#;

            let mut lexer = lexer::Lexer::new(src);

            assert_eq!(
                lexer.next_token(),
                Token::new(
                    TokenKind::TodoKeyword,
                    "todo",
                    Position {
                        line: 1,
                        value: 1,
                        col: 0
                    }
                )
            );

            assert_eq!(
                lexer.next_token(),
                Token::new(
                    TokenKind::String,
                    "run this test",
                    Position {
                        value: 6,
                        line: 1,
                        col: 5
                    }
                )
            );

            assert_eq!(
                lexer.next_token(),
                Token::new(
                    TokenKind::TodoKeyword,
                    "todo",
                    Position {
                        line: 2,
                        value: 20,
                        col: 0
                    }
                )
            );

            assert_eq!(
                lexer.next_token(),
                Token::new(
                    TokenKind::MultilineString,
                    r#"
    run this test with a single line toodo
    as well as this multiline todo
    blah blah
"#,
                    Position {
                        line: 2,
                        value: 25,
                        col: 5
                    }
                )
            );
        }

        impl<'src> Iterator for lexer::Lexer<'src> {
            type Item = Token<'src>;

            fn next(&mut self) -> Option<Self::Item> {
                let token = self.next_token();
                if token.kind == TokenKind::Eof {
                    return None;
                }

                return Some(token);
            }
        }

        #[test]
        fn lexes_mix() {
            let tokens: Vec<_> = lexer::Lexer::new(
                r#"
todo testing on this

todo {
    testing multiple
    lines of text
}

todo run this test

todo {
    run this test with a single line toodo
    as well as this multiline todo
    blah blah
}

todo todo
todo"#,
            )
            .collect();

            assert_debug_snapshot!(tokens, @r###"
            [
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 1,
                        line: 1,
                        col: 0,
                    },
                },
                Token {
                    kind: String,
                    text: "testing on this",
                    start: Position {
                        value: 6,
                        line: 1,
                        col: 5,
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 23,
                        line: 3,
                        col: 0,
                    },
                },
                Token {
                    kind: MultilineString,
                    text: "\n    testing multiple\n    lines of text\n",
                    start: Position {
                        value: 28,
                        line: 3,
                        col: 5,
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 72,
                        line: 8,
                        col: 0,
                    },
                },
                Token {
                    kind: String,
                    text: "run this test",
                    start: Position {
                        value: 77,
                        line: 8,
                        col: 5,
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 92,
                        line: 10,
                        col: 0,
                    },
                },
                Token {
                    kind: MultilineString,
                    text: "\n    run this test with a single line toodo\n    as well as this multiline todo\n    blah blah\n",
                    start: Position {
                        value: 97,
                        line: 10,
                        col: 5,
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 194,
                        line: 16,
                        col: 0,
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 199,
                        line: 16,
                        col: 5,
                    },
                },
                Token {
                    kind: TodoKeyword,
                    text: "todo",
                    start: Position {
                        value: 204,
                        line: 17,
                        col: 0,
                    },
                },
            ]
            "###)
        }
    }
}

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

        fn peek_token(&mut self) -> &Token<'src> {
            self.peeked.get_or_insert_with(|| self.lexer.next_token())
        }

        pub fn parse(&mut self) -> ast::Text {
            let mut items: Vec<Result<ast::Item>> = vec![];

            let mut token = self.next_token();

            if token.kind == TokenKind::Eof {
                items.push(Err(ParseError::UnexpectedEof));
                return ast::Text { items };
            }

            while token.kind != TokenKind::Eof {
                dbg!(&token, "top level");
                let item = match token.kind {
                    TokenKind::TodoKeyword => self.parse_todo(),
                    TokenKind::String | TokenKind::MultilineString => Err(ParseError::ExtraText),
                    TokenKind::Eof => {
                        unreachable!("top level parse loop [should]only runs when token is not eof")
                    }
                };

                items.push(item);

                dbg!(&token, "top level later");

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
                TokenKind::String => {
                    dbg!(&token, "single");
                    Ok(ast::Item::OneLine(ast::TodoItem {
                        message: token.text.to_string(),
                    }))
                }
                TokenKind::MultilineString => {
                    dbg!(&token, "multi");

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
