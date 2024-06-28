use std::fmt::Display;

use super::{CharaterTest, Position, Span};

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
    pub span: Span,
}

impl<'src> Token<'src> {
    pub fn new(kind: TokenKind, text: &'src str, span: Span) -> Self {
        Token { kind, text, span }
    }
}

pub struct Lexer<'src> {
    position: Position,
    eof_pos: Position,
    src: &'src [u8],
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Lexer {
            position: Position::default(),
            eof_pos: Position::default(),
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
        if self.peek_char().is_none() {
            self.eof_pos = self.position;
        }
        self.check_and_bump_new_line();
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
            None => {
                return Token::new(Eof, "", self.eof_pos.spanning_to(self.eof_pos));
            }
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
            "todo" => Token::new(TodoKeyword, string, location.spanning_to(self.position)),
            _ => self.string(Some(location)),
        }
    }

    fn string(&mut self, start: Option<Position>) -> Token<'src> {
        let start_pos = start.unwrap_or(self.position);

        let (_, e) = self.read_while(|&c| c != b'\n');

        let string = self.input_slice((start_pos.value, e));

        Token::new(
            TokenKind::String,
            string,
            start_pos.spanning_to(self.position),
        )
    }

    fn multiline_string(&mut self) -> Token<'src> {
        let start_pos = self.position;

        self.step(); // eat the '{'

        let (s, e) = self.read_while(|&c| c != b'}');

        self.step();

        let string = self.input_slice((s, e));

        Token::new(
            TokenKind::MultilineString,
            string,
            start_pos.spanning_to(self.position),
        )
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
            Token::new(
                TokenKind::TodoKeyword,
                "todo",
                Span {
                    start: Position::default(),
                    end: Position {
                        value: 3,
                        line: 0,
                        col: 3
                    }
                }
            )
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(
                TokenKind::String,
                "run this test",
                Span {
                    start: Position {
                        value: 5,
                        line: 0,
                        col: 5
                    },
                    end: Position {
                        value: 17,
                        line: 0,
                        col: 17
                    }
                }
            )
        );

        assert_eq!(
            lexer.next_token(),
            Token::new(
                TokenKind::Eof,
                "",
                Span {
                    start: Position {
                        value: 17,
                        line: 0,
                        col: 17
                    },
                    end: Position {
                        value: 17,
                        line: 0,
                        col: 17
                    }
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
                Span {
                    start: Position {
                        line: 1,
                        value: 1,
                        col: 0
                    },
                    end: Position {
                        line: 1,
                        value: 4,
                        col: 3
                    }
                }
            )
        );

        assert_eq!(
            lexer.next_token(),
            Token::new(
                TokenKind::String,
                "run this test",
                Span {
                    start: Position {
                        value: 6,
                        line: 1,
                        col: 5
                    },
                    end: Position {
                        value: 18,
                        line: 1,
                        col: 17
                    }
                }
            )
        );

        assert_eq!(
            lexer.next_token(),
            Token::new(
                TokenKind::TodoKeyword,
                "todo",
                Span {
                    start: Position {
                        line: 2,
                        value: 20,
                        col: 0
                    },
                    end: Position {
                        line: 2,
                        value: 23,
                        col: 3
                    }
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
                Span {
                    start: Position {
                        line: 2,
                        value: 25,
                        col: 5
                    },
                    end: Position {
                        line: 6,
                        value: 119,
                        col: 0
                    }
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
                span: Span {
                    start: Position {
                        value: 1,
                        line: 1,
                        col: 0,
                    },
                    end: Position {
                        value: 4,
                        line: 1,
                        col: 3,
                    },
                },
            },
            Token {
                kind: String,
                text: "testing on this",
                span: Span {
                    start: Position {
                        value: 6,
                        line: 1,
                        col: 5,
                    },
                    end: Position {
                        value: 20,
                        line: 1,
                        col: 19,
                    },
                },
            },
            Token {
                kind: TodoKeyword,
                text: "todo",
                span: Span {
                    start: Position {
                        value: 23,
                        line: 3,
                        col: 0,
                    },
                    end: Position {
                        value: 26,
                        line: 3,
                        col: 3,
                    },
                },
            },
            Token {
                kind: MultilineString,
                text: "\n    testing multiple\n    lines of text\n",
                span: Span {
                    start: Position {
                        value: 28,
                        line: 3,
                        col: 5,
                    },
                    end: Position {
                        value: 69,
                        line: 6,
                        col: 0,
                    },
                },
            },
            Token {
                kind: TodoKeyword,
                text: "todo",
                span: Span {
                    start: Position {
                        value: 72,
                        line: 8,
                        col: 0,
                    },
                    end: Position {
                        value: 75,
                        line: 8,
                        col: 3,
                    },
                },
            },
            Token {
                kind: String,
                text: "run this test",
                span: Span {
                    start: Position {
                        value: 77,
                        line: 8,
                        col: 5,
                    },
                    end: Position {
                        value: 89,
                        line: 8,
                        col: 17,
                    },
                },
            },
            Token {
                kind: TodoKeyword,
                text: "todo",
                span: Span {
                    start: Position {
                        value: 92,
                        line: 10,
                        col: 0,
                    },
                    end: Position {
                        value: 95,
                        line: 10,
                        col: 3,
                    },
                },
            },
            Token {
                kind: MultilineString,
                text: "\n    run this test with a single line toodo\n    as well as this multiline todo\n    blah blah\n",
                span: Span {
                    start: Position {
                        value: 97,
                        line: 10,
                        col: 5,
                    },
                    end: Position {
                        value: 191,
                        line: 14,
                        col: 0,
                    },
                },
            },
            Token {
                kind: TodoKeyword,
                text: "todo",
                span: Span {
                    start: Position {
                        value: 194,
                        line: 16,
                        col: 0,
                    },
                    end: Position {
                        value: 197,
                        line: 16,
                        col: 3,
                    },
                },
            },
            Token {
                kind: TodoKeyword,
                text: "todo",
                span: Span {
                    start: Position {
                        value: 199,
                        line: 16,
                        col: 5,
                    },
                    end: Position {
                        value: 202,
                        line: 16,
                        col: 8,
                    },
                },
            },
            Token {
                kind: TodoKeyword,
                text: "todo",
                span: Span {
                    start: Position {
                        value: 204,
                        line: 17,
                        col: 0,
                    },
                    end: Position {
                        value: 207,
                        line: 17,
                        col: 3,
                    },
                },
            },
        ]
        "###)
    }

    #[test]
    fn lex_eof() {
        let src = "todo";

        let mut lexer = Lexer::new(src);

        assert_eq!(
            lexer.next_token(),
            Token::new(
                TokenKind::TodoKeyword,
                "todo",
                Span {
                    start: Position::default(),
                    end: Position {
                        value: 3,
                        line: 0,
                        col: 3
                    }
                }
            )
        );

        assert_eq!(
            lexer.next_token(),
            Token::new(
                TokenKind::Eof,
                "",
                Span {
                    start: Position {
                        value: 3,
                        line: 0,
                        col: 3
                    },

                    end: Position {
                        value: 3,
                        line: 0,
                        col: 3
                    }
                }
            )
        );

        // assert_eq!(
        //     lexer.next_token(),
        //     Token::new(
        //         TokenKind::Eof,
        //         "",
        //         Span {
        //             start: Position {
        //                 value: 3,
        //                 line: 0,
        //                 col: 3
        //             },
        //
        //             end: Position {
        //                 value: 3,
        //                 line: 0,
        //                 col: 3
        //             }
        //         }
        //     )
        // )
    }
}
