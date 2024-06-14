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
    use super::{CharaterTest, Position};

    #[derive(Debug, PartialEq)]
    pub enum TokenKind {
        TodoKeyword,
        UnknownIdentifier,
        // LCurly,
        // RCurly,
        String,
        MultilineString,
        Eof,
    }

    #[derive(Debug, PartialEq)]
    pub struct Token<'src> {
        kind: TokenKind,
        text: &'src str,
        start: Position,
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

        pub fn input(&self) -> &'src str {
            std::str::from_utf8(self.src).expect("input should only contain utf-8 characters")
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

        pub fn next_token(&mut self) -> Token {
            use TokenKind::*;
            self.skip_whitespace();

            let ch = match self.ch() {
                Some(ch) => ch,
                None => {
                    return Token {
                        kind: Eof,
                        start: self.position,
                        text: "",
                    }
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
                "todo" => Token {
                    kind: TodoKeyword,
                    start: location,
                    text: string,
                },
                _ => self.string(Some(location)),
            }
        }

        fn string(&mut self, start: Option<Position>) -> Token<'src> {
            let start_pos = start.unwrap_or(self.position);

            let (_, e) = self.read_while(|&c| c != b'\n');

            self.step();
            let string = self.input_slice((start_pos.value, e));

            Token {
                kind: TokenKind::String,
                start: start_pos,
                text: string,
            }
        }

        fn multiline_string(&mut self) -> Token<'src> {
            let start_pos = self.position;

            self.step(); // eat the '{'

            let (s, e) = self.read_while(|&c| c != b'}');

            self.step();

            let string = self.input_slice((s, e));

            Token {
                kind: TokenKind::MultilineString,
                start: start_pos,
                text: string,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lexer::{Token, TokenKind};

    use super::*;

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
}
