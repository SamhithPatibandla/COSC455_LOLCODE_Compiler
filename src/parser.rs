
use crate::lexer::{Lexer, Token, TokenKind};
use std::collections::HashMap;

// Defines the required recursive-descent parser methods.
pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self) -> Result<(), String>;
    fn parse_head(&mut self) -> Result<(), String>;
    fn parse_title(&mut self) -> Result<(), String>;
    fn parse_comment(&mut self) -> Result<(), String>;
    fn parse_body(&mut self) -> Result<(), String>;
    fn parse_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_text(&mut self) -> Result<(), String>;
    fn parse_variable_define(&mut self) -> Result<(), String>;
    fn parse_variable_use(&mut self) -> Result<(), String>;
    fn parse_bold(&mut self) -> Result<(), String>;
    fn parse_list(&mut self) -> Result<(), String>;
    fn parse_list_items(&mut self) -> Result<(), String>;
    fn parse_inner_list(&mut self) -> Result<(), String>;
    fn parse_link(&mut self) -> Result<(), String>;
    fn parse_newline(&mut self) -> Result<(), String>;
    fn parse_text(&mut self) -> Result<(), String>;
}

// Stores the tokens and the current parsing position.
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    // Each HashMap represents one lexical scope. The last scope is innermost.
    scopes: Vec<HashMap<String, String>>,
    // Keep the resolved values for later HTML generation and for tests.
    resolved_variables: Vec<(String, String)>,
    // HTML is built only in memory until all checks pass.
    html: String,
}

impl Parser {
    // Creates a parser by tokenizing the source file.
    pub fn new(source: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        // Collect tokens before syntax analysis begins.
        while let Some(token) = lexer.next_typed_token()? {
            tokens.push(token);
        }

        Ok(Self {
            tokens,
            position: 0,
            // Global document scope is active for the entire program.
            scopes: vec![HashMap::new()],
            resolved_variables: Vec::new(),
            html: String::new(),
        })
    }

    // Exposes the variable values found during parsing for future HTML output.
    pub fn resolved_variables(&self) -> &[(String, String)] {
        &self.resolved_variables
    }

    // The caller reads HTML only after parse_lolcode() has succeeded.
    pub fn html(&self) -> &str {
        &self.html
    }

    // Escape text before placing it into an HTML text node or attribute.
    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    // A new block gets its own scope, allowing it to shadow outer variables.
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    // Exiting a block discards variables defined only inside that block.
    fn exit_scope(&mut self) {
        // Never remove the global document scope.
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    // Resolve from innermost scope outward, implementing lexical scoping.
    fn resolve_variable(&self, name: &str) -> Option<&str> {
        self.scopes.iter().rev()
            .find_map(|scope| scope.get(name).map(String::as_str))
    }

    // Check identifiers are single words made of ASCII letters and digits.
    // A name must begin with a letter; whitespace is not allowed.
    fn valid_variable_name(name: &str) -> bool {
        let mut chars = name.chars();
        matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric())
    }

    // Read a required, nonempty ordinary-text token and return trimmed content.
    fn take_required_text(&mut self, context: &str) -> Result<String, String> {
        if !self.has_meaningful_text() {
            return Err(format!(
                "Syntax error: expected {}, found {}",
                context,
                self.current_token().map(|t| t.value.as_str()).unwrap_or("EOF")
            ));
        }
        let value = self.current_token().unwrap().value.trim().to_string();
        self.advance();
        Ok(value)
    }

    // Returns the current token without consuming it.
    // This provides one-token lookahead.
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    // Advances to the next available token.
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    // Checks whether the current token matches an annotation.
    fn check(&self, expected: &str) -> bool {
        match self.current_token() {
            Some(token) => {
                token.kind == TokenKind::Annotation
                    && token.value.eq_ignore_ascii_case(expected)
            }
            None => false,
        }
    }

    // Consumes an expected annotation.
    // An unexpected annotation produces a syntax error.
    fn expect(&mut self, expected: &str) -> Result<(), String> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            let found = self.current_token()
                .map(|token| token.value.as_str())
                .unwrap_or("EOF");

            Err(format!(
                "Syntax error: expected '{}', found '{}'",
                expected, found
            ))
        }
    }

    // Returns true when the current token is ordinary text.
    fn is_text(&self) -> bool {
        matches!(
            self.current_token(),
            Some(token) if token.kind == TokenKind::Text
        )
    }

    // Checks whether text contains non-whitespace characters.
    fn has_meaningful_text(&self) -> bool {
        match self.current_token() {
            Some(token) if token.kind == TokenKind::Text => {
                !token.value.trim().is_empty()
            }
            _ => false,
        }
    }

    // Consumes one ordinary text token.
    fn consume_text(&mut self) -> Result<(), String> {
        if self.is_text() {
            // Preserve ordinary content and escape HTML-sensitive characters.
            let content = self.current_token().unwrap().value.clone();
            self.html.push_str(&Self::escape_html(&content));
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Syntax error: expected text; {}",
                self.unexpected_token()
            ))
        }
    }

    // Produces an error describing the unexpected token.
    fn unexpected_token(&self) -> String {
        let found = self.current_token()
            .map(|token| token.value.as_str())
            .unwrap_or("EOF");

        format!("unexpected token '{}'", found)
    }

    // Verifies that a text section contains actual content.
    // Used for titles and variable-like values.
    fn require_text(&mut self) -> Result<(), String> {
        if !self.has_meaningful_text() {
            return Err(format!(
                "Syntax error: expected nonempty text; {}",
                self.unexpected_token()
            ));
        }

        self.consume_text()
    }
}

// Implements the recursive-descent syntax analyzer.
impl SyntaxAnalyzer for Parser {
    // Parses the entire LOLCODE document.
    fn parse_lolcode(&mut self) -> Result<(), String> {
        // All valid programs must begin with #HAI.
        self.expect("#HAI")?;
        self.html.push_str("<!DOCTYPE html>\n<html>\n");

        // Process the optional head and body content.
        self.parse_body()?;

        // All valid programs must end with #KBYE.
        self.expect("#KBYE")?;

        // Reject content following the closing annotation.
        if self.current_token().is_some() {
            return Err(format!(
                "Syntax error: {} after #KBYE",
                self.unexpected_token()
            ));
        }

        self.html.push_str("</html>\n");
        Ok(())
    }

    // Parses #MAEK HEAD ... #MKAY.
    fn parse_head(&mut self) -> Result<(), String> {
        self.expect("#MAEK HEAD")?;
        self.html.push_str("<head>\n");

        // Comments may occur before the optional title.
        while self.check("#OBTW") {
            self.parse_comment()?;
        }

        // Only one title is allowed inside a head block.
        if self.check("#GIMMEH TITLE") {
            self.parse_title()?;
        }

        // Comments may also occur after the title.
        while self.check("#OBTW") {
            self.parse_comment()?;
        }

        self.expect("#MKAY")?;
        self.html.push_str("</head>\n");
        Ok(())
    }

    // Parses #GIMMEH TITLE ... #OIC.
    fn parse_title(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH TITLE")?;
        self.html.push_str("<title>");

        // A title contains ordinary text, not nested annotations.
        self.require_text()?;

        self.expect("#OIC")?;
        self.html.push_str("</title>\n");
        Ok(())
    }

    // Parses #OBTW ... #TLDR.
    fn parse_comment(&mut self) -> Result<(), String> {
        self.expect("#OBTW")?;
        self.html.push_str("<!-- ");

        // Comments contain plain text and cannot contain annotations.
        // Escape is harmless in a comment, and prevents HTML injection.
        self.require_text()?;

        self.expect("#TLDR")?;
        self.html.push_str(" -->\n");
        Ok(())
    }

    // Parses the optional head and the document body.
    fn parse_body(&mut self) -> Result<(), String> {
        // Comments may appear before the head.
        while self.check("#OBTW") {
            self.parse_comment()?;
        }

        // HEAD must appear before ordinary body elements.
        if self.check("#MAEK HEAD") {
            self.parse_head()?;
        }

        self.html.push_str("<body>\n");

        // A global variable definition may appear at the beginning of the body.
        if self.check("#IHAZ") {
            self.parse_variable_define()?;
        }

        // Parse body elements until the closing #KBYE.
        while self.current_token().is_some() && !self.check("#KBYE") {
            if self.is_text() {
                self.parse_text()?;
            } else if self.check("#OBTW") {
                self.parse_comment()?;
            } else if self.check("#MAEK PARAGRAF") {
                self.parse_paragraph()?;
            } else if self.check("#GIMMEH BOLD") {
                self.parse_bold()?;
            } else if self.check("#MAEK LIST") {
                self.parse_list()?;
            } else if self.check("#NEWLINE") {
                self.parse_newline()?;
            } else if self.check("#GIMMEH LINX") {
                self.parse_link()?;
            } else if self.check("#GIMMEH VAR") {
                self.parse_variable_use()?;
            } else {
                return Err(format!(
                    "Syntax error: {} in document body",
                    self.unexpected_token()
                ));
            }
        }

        self.html.push_str("</body>\n");
        Ok(())
    }

    // Parses a paragraph block.
    fn parse_paragraph(&mut self) -> Result<(), String> {
        self.expect("#MAEK PARAGRAF")?;
        self.html.push_str("<p>");
        // Variables declared in a paragraph disappear when it closes.
        self.enter_scope();

        // Allow a variable definition as the first paragraph annotation.
        if self.check("#IHAZ") {
            self.parse_variable_define()?;
        }

        // Parse the contents until the paragraph closes.
        self.parse_inner_paragraph()?;

        self.expect("#MKAY")?;
        self.html.push_str("</p>\n");
        self.exit_scope();
        Ok(())
    }

    // Parses the content inside a paragraph.
    fn parse_inner_paragraph(&mut self) -> Result<(), String> {
        // Stop when the paragraph's closing annotation is reached.
        while self.current_token().is_some() && !self.check("#MKAY") {
            if self.is_text() {
                self.parse_text()?;
            } else if self.check("#OBTW") {
                self.parse_comment()?;
            } else if self.check("#GIMMEH BOLD") {
                self.parse_bold()?;
            } else if self.check("#MAEK LIST") {
                self.parse_list()?;
            } else if self.check("#NEWLINE") {
                self.parse_newline()?;
            } else if self.check("#GIMMEH LINX") {
                self.parse_link()?;
            } else if self.check("#GIMMEH VAR") {
                self.parse_variable_use()?;
            } else {
                // This also rejects nested paragraphs.
                return Err(format!(
                    "Syntax error: {} inside paragraph",
                    self.unexpected_token()
                ));
            }
        }

        Ok(())
    }

    // Parses plain text where text content is required.
    fn parse_inner_text(&mut self) -> Result<(), String> {
        self.require_text()
    }

    // Parses one ordinary text token.
    fn parse_text(&mut self) -> Result<(), String> {
        self.consume_text()
    }

    // Parses the standalone #NEWLINE annotation.
    fn parse_newline(&mut self) -> Result<(), String> {
        self.expect("#NEWLINE")?;
        self.html.push_str("<br>\n");
        Ok(())
    }

    // Validate and store a variable definition in the current scope.
    fn parse_variable_define(&mut self) -> Result<(), String> {
        // Grammar: #IHAZ name #ITIZ value #MKAY
        self.expect("#IHAZ")?;
        let name = self.take_required_text("a variable name")?;
        if !Self::valid_variable_name(&name) {
            return Err(format!("Syntax error: invalid variable name '{}'", name));
        }
        self.expect("#ITIZ")?;
        let value = self.take_required_text("a variable value")?;
        self.expect("#MKAY")?;

        // Insert into the current scope, not into the parent scope.
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.contains_key(&name) {
            return Err(format!("Semantic error: variable '{}' is already defined in this scope", name));
        }
        current_scope.insert(name, value);
        Ok(())
    }

    fn parse_variable_use(&mut self) -> Result<(), String> {
        // Grammar: #GIMMEH VAR name #OIC
        self.expect("#GIMMEH VAR")?;
        let name = self.take_required_text("a variable name")?;
        if !Self::valid_variable_name(&name) {
            return Err(format!("Syntax error: invalid variable name '{}'", name));
        }
        self.expect("#OIC")?;

        // An undefined variable is a static-semantic error.
        let value = self.resolve_variable(&name)
            .ok_or_else(|| format!("Semantic error: undefined variable '{}'", name))?
            .to_string();
        self.html.push_str(&Self::escape_html(&value));
        self.resolved_variables.push((name, value));
        Ok(())
    }

    
    // Parses bold text between #GIMMEH BOLD and #OIC.
    fn parse_bold(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH BOLD")?;
        self.html.push_str("<b>");

        // Bold may contain ordinary text and references to variables.
        while self.current_token().is_some() && !self.check("#OIC") {
            if self.is_text() {
                self.parse_text()?;
            } else if self.check("#GIMMEH VAR") {
                self.parse_variable_use()?;
            } else {
                return Err(format!(
                    "Syntax error: {} inside bold block",
                    self.unexpected_token()
                ));
            }
        }

        // Every bold block must end with #OIC.
        self.expect("#OIC")?;
        self.html.push_str("</b>");
        Ok(())
    }

    // Parses a complete list block.
    fn parse_list(&mut self) -> Result<(), String> {
        self.expect("#MAEK LIST")?;
        self.html.push_str("<ul>\n");

        // A list must contain at least one item.
        self.parse_list_items()?;

        // Close the list after all items have been parsed.
        self.expect("#MKAY")?;
        self.html.push_str("</ul>\n");
        Ok(())
    }

    // Parses one or more list items.
    fn parse_list_items(&mut self) -> Result<(), String> {
        // Require the first item.
        self.expect("#GIMMEH ITEM")?;
        self.html.push_str("<li>");
        self.parse_inner_list()?;
        self.expect("#OIC")?;
        self.html.push_str("</li>\n");

        // Continue parsing additional list items.
        while self.check("#GIMMEH ITEM") {
            self.expect("#GIMMEH ITEM")?;
            self.html.push_str("<li>");
            self.parse_inner_list()?;
            self.expect("#OIC")?;
            self.html.push_str("</li>\n");
        }

        Ok(())
    }

    // Parses the content inside an individual list item.
    fn parse_inner_list(&mut self) -> Result<(), String> {
        // Each list item has an independent local scope.
        self.enter_scope();
        // A variable definition is only allowed as the
        // first annotation inside the item.
        if self.check("#IHAZ") {
            self.parse_variable_define()?;
        }

        // List items may contain text, bold, and variable use.
        while self.current_token().is_some() && !self.check("#OIC") {
            if self.is_text() {
                self.parse_text()?;
            } else if self.check("#GIMMEH BOLD") {
                self.parse_bold()?;
            } else if self.check("#GIMMEH VAR") {
                self.parse_variable_use()?;
            } else {
                return Err(format!(
                    "Syntax error: {} inside list item",
                    self.unexpected_token()
                ));
            }
        }

        self.exit_scope();
        Ok(())
    }


    // LINX accepts one nonempty address token and closes with #OIC.
    fn parse_link(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH LINX")?;
        let address = self.take_required_text("a link address")?;
        self.expect("#OIC")?;
        let safe = Self::escape_html(&address);
        self.html.push_str(&format!("<a href=\"{}\">{}</a>", safe, safe));
        Ok(())
    }
}

// Automated tests for the implemented parser methods.
#[cfg(test)]
mod tests {
    use super::*;

    // A complete document with ordinary text should parse.
    #[test]
    fn test_basic_document() {
        let mut parser =
            Parser::new("#HAI Hello world #KBYE").unwrap();

        assert!(parser.parse_lolcode().is_ok());
    }

    // The head may contain a title.
    #[test]
    fn test_head_and_title() {
        let source = r#"
#HAI
#MAEK HEAD
#GIMMEH TITLE My Page #OIC
#MKAY
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_ok());
    }

    // A comment should be accepted before ordinary body content.
    #[test]
    fn test_comment() {
        let source = r#"
#HAI
#OBTW This is a comment #TLDR
Hello world
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_ok());
    }

    // Tests a paragraph containing ordinary text and #NEWLINE.
    #[test]
    fn test_paragraph() {
        let source = r#"
#HAI
#MAEK PARAGRAF
Hello world
#NEWLINE
Goodbye
#MKAY
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_ok());
    }

    // A title outside the HEAD block must be rejected.
    #[test]
    fn test_title_outside_head() {
        let source = r#"
#HAI
#GIMMEH TITLE Invalid Title #OIC
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_err());
    }

    // Paragraphs cannot contain other paragraphs.
    #[test]
    fn test_nested_paragraph() {
        let source = r#"
#HAI
#MAEK PARAGRAF
Outer paragraph
#MAEK PARAGRAF
Inner paragraph
#MKAY
#MKAY
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_err());
    }

    // Missing closing annotations should produce errors.
    #[test]
    fn test_missing_paragraph_end() {
        let source = r#"
#HAI
#MAEK PARAGRAF
Hello world
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_err());
    }
    
    // Tests bold text inside a valid LOLCODE document.
    #[test]
    fn test_bold() {
        let source = r#"
#HAI
#GIMMEH BOLD Hello world #OIC
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();

        assert!(parser.parse_lolcode().is_ok());
    }

    // Verifies that a list can contain multiple items.
    #[test]
    fn test_list_multiple_items() {
        let source = r#"
#HAI
#MAEK LIST
#GIMMEH ITEM Apple #OIC
#GIMMEH ITEM Banana #OIC
#GIMMEH ITEM Orange #OIC
#MKAY
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();

        assert!(parser.parse_lolcode().is_ok());
    }

    // Verifies that bold formatting is allowed inside a list item.
    #[test]
    fn test_bold_inside_list() {
        let source = r#"
#HAI
#MAEK LIST
#GIMMEH ITEM Apple #OIC
#GIMMEH ITEM #GIMMEH BOLD Banana #OIC #OIC
#MKAY
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();

        assert!(parser.parse_lolcode().is_ok());
    }

    // An empty list is invalid because at least one item is required.
    #[test]
    fn test_empty_list() {
        let source = r#"
#HAI
#MAEK LIST
#MKAY
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();

        assert!(parser.parse_lolcode().is_err());
    }

    // List items cannot appear outside a list block.
    #[test]
    fn test_item_outside_list() {
        let source = r#"
#HAI
#GIMMEH ITEM Apple #OIC
#KBYE
"#;

        let mut parser = Parser::new(source).unwrap();

        assert!(parser.parse_lolcode().is_err());
    }


    // A global definition is visible to later code.
    #[test]
    fn test_global_variable() {
        let source = "#HAI #IHAZ answer #ITIZ 42 #MKAY #GIMMEH VAR answer #OIC #KBYE";
        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_ok());
        assert_eq!(parser.resolved_variables(), &[("answer".into(), "42".into())]);
    }

    // Using a missing name must produce an error, not a successful parse.
    #[test]
    fn test_undefined_variable() {
        let source = "#HAI #GIMMEH VAR answer #OIC #KBYE";
        let mut parser = Parser::new(source).unwrap();
        let error = parser.parse_lolcode().unwrap_err();
        assert!(error.contains("undefined variable"), "{}", error);
    }

    // Inner definitions override outer ones, but only inside that block.
    #[test]
    fn test_variable_shadowing() {
        let source = r#"
#HAI
#IHAZ answer #ITIZ happiness #MKAY
#MAEK PARAGRAF
#IHAZ answer #ITIZ 42 #MKAY
#GIMMEH VAR answer #OIC
#MKAY
#GIMMEH VAR answer #OIC
#KBYE
"#;
        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_ok());
        assert_eq!(parser.resolved_variables(), &[
            ("answer".into(), "42".into()),
            ("answer".into(), "happiness".into()),
        ]);
    }

    // A variable declared inside a paragraph cannot leak outside it.
    #[test]
    fn test_paragraph_variable_not_visible_outside() {
        let source = r#"
#HAI
#MAEK PARAGRAF
#IHAZ answer #ITIZ 42 #MKAY
#MKAY
#GIMMEH VAR answer #OIC
#KBYE
"#;
        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().unwrap_err().contains("undefined variable"));
    }

    // Variable uses work inside bold formatting.
    #[test]
    fn test_variable_inside_bold() {
        let source = "#HAI #IHAZ who #ITIZ Sam #MKAY #GIMMEH BOLD Hi #GIMMEH VAR who #OIC #OIC #KBYE";
        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_ok());
        assert_eq!(parser.resolved_variables()[0].1, "Sam");
    }

    // A variable defined within a list item may be used in the same item.
    #[test]
    fn test_list_item_variable() {
        let source = "#HAI #MAEK LIST #GIMMEH ITEM #IHAZ fruit #ITIZ apple #MKAY #GIMMEH VAR fruit #OIC #OIC #MKAY #KBYE";
        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().is_ok());
        assert_eq!(parser.resolved_variables()[0].1, "apple");
    }

    // Variable names must not contain multiple words.
    #[test]
    fn test_invalid_variable_name() {
        let source = "#HAI #IHAZ two words #ITIZ 42 #MKAY #KBYE";
        let mut parser = Parser::new(source).unwrap();
        assert!(parser.parse_lolcode().unwrap_err().contains("invalid variable name"));
    }

    // A link must have a URL and matching closing annotation.
    #[test]
    fn test_link_html() {
        let mut p = Parser::new("#HAI #GIMMEH LINX https://example.com #OIC #KBYE").unwrap();
        p.parse_lolcode().unwrap();
        assert!(p.html().contains("<a href=\"https://example.com\">https://example.com</a>"));
    }

    // Variables are substituted when they are referenced.
    #[test]
    fn test_variable_html() {
        let mut p = Parser::new("#HAI #IHAZ answer #ITIZ 42 #MKAY #GIMMEH VAR answer #OIC #KBYE").unwrap();
        p.parse_lolcode().unwrap();
        assert!(p.html().contains("42"));
        assert!(!p.html().contains("#GIMMEH VAR"));
    }

    // Confirm formatting tags appear in generated HTML.
    #[test]
    fn test_formatting_html() {
        let mut p = Parser::new("#HAI #MAEK PARAGRAF Hi #GIMMEH BOLD there #OIC #NEWLINE #MAEK LIST #GIMMEH ITEM apple #OIC #MKAY #MKAY #KBYE").unwrap();
        p.parse_lolcode().unwrap();
        for tag in ["<p>", "<b>", "</b>", "<br>", "<ul>", "<li>", "</li>", "</ul>", "</p>"] {
            assert!(p.html().contains(tag), "missing {}", tag);
        }
    }

    // A link without an address is a syntax error.
    #[test]
    fn test_link_missing_address() {
        let mut p = Parser::new("#HAI #GIMMEH LINX #OIC #KBYE").unwrap();
        assert!(p.parse_lolcode().is_err());
    }

}
