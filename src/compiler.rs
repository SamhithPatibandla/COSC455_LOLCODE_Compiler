use crate::lexer::Lexer;
use crate::parser::{Parser, SyntaxAnalyzer};

// Required assignment interface: the methods intentionally use the specified signatures.
pub trait Compiler {
    fn compile(&mut self, source: &str);
    fn next_token(&mut self) -> String;
    fn parse(&mut self);
    fn current_token(&self) -> String;
    fn set_current_token(&mut self, tok: String);
}

// Coordinates lexical, syntax, and semantic analysis, then retains the HTML.
// Errors are stored because the required trait methods return no Result.
pub struct LolCompiler {
    source: String,
    lexer: Lexer,
    current: String,
    html: String,
    error: Option<String>,
}

impl LolCompiler {
    // Constructs a compiler initially containing an empty source document.
    pub fn new() -> Self {
        Self {
            source: String::new(),
            lexer: Lexer::new(""),
            current: String::new(),
            html: String::new(),
            error: None,
        }
    }

    // Returns generated HTML only when every compilation stage succeeded.
    pub fn html(&self) -> Result<&str, &str> {
        match &self.error {
            Some(message) => Err(message.as_str()),
            None => Ok(self.html.as_str()),
        }
    }

    // Exposes an error for the command-line front end and automated tests.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl Compiler for LolCompiler {
    // Resets the state for each source file and starts syntax/semantic analysis.
    fn compile(&mut self, source: &str) {
        self.source = source.to_string();
        self.lexer = Lexer::new(source);
        self.current.clear();
        self.html.clear();
        self.error = None;
        self.parse();
    }

    // Reads one lexical token. The empty string represents the end of input.
    // Lexical errors are saved because this trait must return a String.
    fn next_token(&mut self) -> String {
        if self.error.is_some() {
            return String::new();
        }
        match self.lexer.next_token() {
            Ok(Some(tok)) => {
                self.set_current_token(tok.clone());
                tok
            }
            Ok(None) => {
                self.set_current_token(String::new());
                String::new()
            }
            Err(message) => {
                self.error = Some(message);
                self.set_current_token(String::new());
                String::new()
            }
        }
    }

    // The Parser performs recursive descent and scope-aware variable checks.
    // Its HTML is retained only if parsing completes successfully.
    fn parse(&mut self) {
        self.html.clear();
        self.error = None;
        match Parser::new(&self.source) {
            Ok(mut parser) => match parser.parse_lolcode() {
                Ok(()) => self.html = parser.html().to_string(),
                Err(message) => self.error = Some(message),
            },
            Err(message) => self.error = Some(message),
        }
    }

    // Returns the most recently retrieved token.
    fn current_token(&self) -> String {
        self.current.clone()
    }

    // Updates the current token as required by the interface.
    fn set_current_token(&mut self, tok: String) {
        self.current = tok;
    }
}

#[cfg(test)]
mod tests {
    use super::{Compiler, LolCompiler};

    #[test]
    fn test_compile_produces_html() {
        let mut compiler = LolCompiler::new();
        compiler.compile("#HAI Hello #KBYE");
        assert!(compiler.html().unwrap().contains("Hello"));
    }

    #[test]
    fn test_compile_rejects_undefined_variable() {
        let mut compiler = LolCompiler::new();
        compiler.compile("#HAI #GIMMEH VAR absent #OIC #KBYE");
        assert!(compiler.error().unwrap().contains("undefined variable"));
        assert!(compiler.html().is_err());
    }

    #[test]
    fn test_next_and_current_token() {
        let mut compiler = LolCompiler::new();
        compiler.compile("#HAI text #KBYE");
        assert_eq!(compiler.next_token(), "#HAI");
        assert_eq!(compiler.current_token(), "#HAI");
        compiler.set_current_token("manual".into());
        assert_eq!(compiler.current_token(), "manual");
    }

    #[test]
    fn test_recompile_clears_old_output() {
        let mut compiler = LolCompiler::new();
        compiler.compile("#HAI valid #KBYE");
        assert!(compiler.html().is_ok());
        compiler.compile("#HAI #HEY #KBYE");
        assert!(compiler.html().is_err());
    }
}
