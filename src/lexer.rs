
pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self, c: char);
    fn lookup(&self, s: &str) -> bool;
}

// Identifies whether a token is an annotation or ordinary text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Annotation,
    Text,
}

// Stores the token category and its original value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
}

// Stores input characters and tracks the reading position.
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    lexeme: String,
}

impl Lexer {
    // Creates a lexer from the source code.
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0,
            lexeme: String::new(),
        }
    }

    // Looks at the next character without consuming it.
    fn peek_char(&self) -> char {
        if self.position >= self.input.len() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    // Checks whether a character is allowed in ordinary text.
    fn valid_text_char(c: char) -> bool {
        c.is_ascii_alphanumeric()
            || c.is_ascii_whitespace()
            || matches!(
                c,
                ',' | '.' | '"' | ':' | '?' |
                '!' | '%' | '=' | '/'
            )
    }

    // Reads the next complete token from the source.
    // Returns None when the source has been fully processed.
    pub fn next_token(&mut self) -> Result<Option<String>, String> {
        self.lexeme.clear();

        // Skip whitespace before the next token.
        while self.peek_char().is_whitespace() {
            self.get_char();
        }

        if self.peek_char() == '\0' {
            return Ok(None);
        }

        // Every annotation starts with '#'.
        if self.peek_char() == '#' {
            let c = self.get_char();
            self.add_char(c);

            // Read the first word of the annotation.
            while self.peek_char().is_ascii_alphabetic() {
                let c = self.get_char();
                self.add_char(c);
            }

            let first_word = self.lexeme.to_uppercase();

            // MAEK and GIMMEH require a second word.
            if first_word == "#MAEK" || first_word == "#GIMMEH" {
                if self.peek_char() != ' ' {
                    return Err(format!(
                        "Lexical error: incomplete annotation '{}'",
                        self.lexeme
                    ));
                }

                // Read the space between annotation words.
                let c = self.get_char();
                self.add_char(c);

                // Gather the second word.
                while self.peek_char().is_ascii_alphabetic() {
                    let c = self.get_char();
                    self.add_char(c);
                }
            }

            // Check whether the complete annotation is valid.
            if self.lookup(&self.lexeme) {
                return Ok(Some(self.lexeme.clone()));
            }

            return Err(format!(
                "Lexical error: invalid annotation '{}'",
                self.lexeme
            ));
        }

        // Read ordinary text until another annotation or EOF.
        while self.peek_char() != '#' && self.peek_char() != '\0' {
            let c = self.get_char();

            // Reject characters outside the permitted text set.
            if !Self::valid_text_char(c) {
                return Err(format!(
                    "Lexical error: invalid character '{}'",
                    c
                ));
            }

            self.add_char(c);
        }

        Ok(Some(self.lexeme.clone()))
    }

    // Returns the next token with its category.
    pub fn next_typed_token(&mut self) -> Result<Option<Token>, String> {
        match self.next_token()? {
            Some(value) => {
                let kind = if value.starts_with('#') {
                    TokenKind::Annotation
                } else {
                    TokenKind::Text
                };

                Ok(Some(Token { kind, value }))
            }

            None => Ok(None),
        }
    }
}

// Implements the lexical analyzer methods required by the project.
impl LexicalAnalyzer for Lexer {
    // Gets a character and advances the input position.
    fn get_char(&mut self) -> char {
        if self.position >= self.input.len() {
            return '\0';
        }

        let c = self.input[self.position];
        self.position += 1;
        c
    }

    // Adds a character to the lexeme being constructed.
    fn add_char(&mut self, c: char) {
        self.lexeme.push(c);
    }

    // Checks whether an annotation is part of LOLCODE.
    // Uppercase comparison makes annotations case-insensitive.
    fn lookup(&self, s: &str) -> bool {
        matches!(
            s.to_uppercase().as_str(),
            "#HAI"
                | "#KBYE"
                | "#OBTW"
                | "#TLDR"
                | "#MAEK HEAD"
                | "#MAEK PARAGRAF"
                | "#MAEK LIST"
                | "#GIMMEH TITLE"
                | "#GIMMEH BOLD"
                | "#GIMMEH ITEM"
                | "#GIMMEH LINX"
                | "#GIMMEH VAR"
                | "#NEWLINE"
                | "#IHAZ"
                | "#ITIZ"
                | "#MKAY"
                | "#OIC"
        )
    }
}

// Unit tests for lexical analysis.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_word_annotations() {
        let mut lexer = Lexer::new("#HAI #KBYE");

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#HAI".to_string())
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#KBYE".to_string())
        );

        assert_eq!(lexer.next_token().unwrap(), None);
    }

    #[test]
    fn test_two_word_annotations() {
        let mut lexer = Lexer::new(
            "#MAEK PARAGRAF #GIMMEH BOLD"
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#MAEK PARAGRAF".to_string())
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#GIMMEH BOLD".to_string())
        );
    }

    #[test]
    fn test_case_insensitive_annotations() {
        let mut lexer = Lexer::new(
            "#hai #MaEk LiSt #kbye"
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#hai".to_string())
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#MaEk LiSt".to_string())
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#kbye".to_string())
        );
    }

    #[test]
    fn test_plain_text() {
        let mut lexer = Lexer::new(
            "#HAI Hello world #KBYE"
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#HAI".to_string())
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("Hello world ".to_string())
        );

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#KBYE".to_string())
        );
    }

    #[test]
    fn test_invalid_annotation() {
        let mut lexer = Lexer::new("#HAI #HEY #KBYE");

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#HAI".to_string())
        );

        let result = lexer.next_token();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Lexical error"));
    }

    #[test]
    fn test_typed_tokens() {
        let mut lexer = Lexer::new("#HAI Hello #KBYE");

        let first = lexer.next_typed_token().unwrap().unwrap();
        assert_eq!(first.kind, TokenKind::Annotation);
        assert_eq!(first.value, "#HAI");

        let second = lexer.next_typed_token().unwrap().unwrap();
        assert_eq!(second.kind, TokenKind::Text);
        assert_eq!(second.value, "Hello ");

        let third = lexer.next_typed_token().unwrap().unwrap();
        assert_eq!(third.kind, TokenKind::Annotation);
        assert_eq!(third.value, "#KBYE");
    }

    #[test]
    fn test_invalid_text_character() {
        let mut lexer = Lexer::new("#HAI Hello @ world #KBYE");

        assert_eq!(
            lexer.next_token().unwrap(),
            Some("#HAI".to_string())
        );

        assert!(lexer.next_token().is_err());
    }
}
