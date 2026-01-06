use logos::Logos;
use serde::{Serialize, Deserialize};

/// HolyC token types using logos for fast lexing
#[derive(Logos, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // Keywords
    #[token("U0")]
    U0,
    #[token("U8")]
    U8,
    #[token("U16")]
    U16,
    #[token("U32")]
    U32,
    #[token("U64")]
    U64,
    #[token("I8")]
    I8,
    #[token("I16")]
    I16,
    #[token("I32")]
    I32,
    #[token("I64")]
    I64,
    #[token("F64")]
    F64,
    #[token("Bool")]
    Bool,
    #[token("Void")]
    Void,

    #[token("class")]
    Class,
    #[token("union")]
    Union,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,

    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("do")]
    Do,
    #[token("for")]
    For,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,
    #[token("goto")]
    Goto,

    #[token("public")]
    Public,
    #[token("static")]
    Static,
    #[token("extern")]
    Extern,
    #[token("const")]
    Const,

    #[token("sizeof")]
    Sizeof,
    #[token("offset")]
    Offset,

    #[token("TRUE")]
    True,
    #[token("FALSE")]
    False,
    #[token("NULL")]
    Null,

    // Identifiers and literals
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    IntLiteral(u64),

    #[regex("0x[0-9a-fA-F]+", |lex| u64::from_str_radix(&lex.slice()[2..], 16).ok())]
    HexLiteral(u64),

    #[regex("0b[01]+", |lex| u64::from_str_radix(&lex.slice()[2..], 2).ok())]
    BinLiteral(u64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLiteral(String),

    #[regex(r"'([^'\\]|\\.)'", |lex| {
        let s = lex.slice();
        s.chars().nth(1).unwrap() as u8
    })]
    CharLiteral(u8),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse().ok())]
    FloatLiteral(f64),

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,

    #[token("&&")]
    LogicalAnd,
    #[token("||")]
    LogicalOr,
    #[token("!")]
    LogicalNot,

    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    #[token("=")]
    Assign,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("%=")]
    PercentAssign,
    #[token("&=")]
    AndAssign,
    #[token("|=")]
    OrAssign,
    #[token("^=")]
    XorAssign,
    #[token("<<=")]
    LeftShiftAssign,
    #[token(">>=")]
    RightShiftAssign,

    #[token("++")]
    Increment,
    #[token("--")]
    Decrement,

    #[token("->")]
    Arrow,
    #[token(".")]
    Dot,

    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,

    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("?")]
    Question,

    // Preprocessor directives
    #[regex(r"#define\s+[a-zA-Z_][a-zA-Z0-9_]*\s+.*", |lex| lex.slice().to_string())]
    Define(String),

    #[regex(r#"#include\s*["<][^>"]*[">]"#, |lex| lex.slice().to_string())]
    Include(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::IntLiteral(n) => write!(f, "Int({})", n),
            Token::HexLiteral(n) => write!(f, "Hex(0x{:x})", n),
            Token::BinLiteral(n) => write!(f, "Bin(0b{:b})", n),
            Token::StringLiteral(s) => write!(f, "String(\"{}\")", s),
            Token::CharLiteral(c) => write!(f, "Char('{}')", *c as char),
            Token::FloatLiteral(fl) => write!(f, "Float({})", fl),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Lexer wrapper for easier usage
pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }

    pub fn collect_tokens(source: &'source str) -> Result<Vec<(Token, std::ops::Range<usize>)>, String> {
        let mut lexer = Token::lexer(source);
        let mut tokens = Vec::new();

        while let Some(token_result) = lexer.next() {
            match token_result {
                Ok(token) => {
                    tokens.push((token, lexer.span()));
                }
                Err(_) => {
                    return Err(format!(
                        "Lexical error at position {}: unexpected character '{}'",
                        lexer.span().start,
                        &source[lexer.span()]
                    ));
                }
            }
        }

        Ok(tokens)
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result.map_err(|_| {
                format!(
                    "Lexical error at position {}",
                    self.inner.span().start
                )
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let source = "U64 main() { return 42; }";
        let tokens = Lexer::collect_tokens(source).unwrap();

        assert_eq!(tokens[0].0, Token::U64);
        assert_eq!(tokens[1].0, Token::Ident("main".to_string()));
        assert_eq!(tokens[2].0, Token::LeftParen);
        assert_eq!(tokens[3].0, Token::RightParen);
        assert_eq!(tokens[4].0, Token::LeftBrace);
        assert_eq!(tokens[5].0, Token::Return);
        assert_eq!(tokens[6].0, Token::IntLiteral(42));
        assert_eq!(tokens[7].0, Token::Semicolon);
        assert_eq!(tokens[8].0, Token::RightBrace);
    }

    #[test]
    fn test_hex_literals() {
        let source = "0xdeadbeef";
        let tokens = Lexer::collect_tokens(source).unwrap();
        assert_eq!(tokens[0].0, Token::HexLiteral(0xdeadbeef));
    }

    #[test]
    fn test_operators() {
        let source = "a += b << 3";
        let tokens = Lexer::collect_tokens(source).unwrap();
        assert_eq!(tokens[1].0, Token::PlusAssign);
        assert_eq!(tokens[3].0, Token::LeftShift);
    }

    #[test]
    fn test_class_definition() {
        let source = "class CAccountInfo { U64 key; };";
        let tokens = Lexer::collect_tokens(source).unwrap();
        assert_eq!(tokens[0].0, Token::Class);
        assert_eq!(tokens[1].0, Token::Ident("CAccountInfo".to_string()));
    }

    #[test]
    fn test_comments() {
        let source = r#"
            // Single line comment
            U64 x; /* Multi
                      line
                      comment */
            U64 y;
        "#;
        let tokens = Lexer::collect_tokens(source).unwrap();
        assert_eq!(tokens.len(), 6); // U64 x ; U64 y ;
    }

    #[test]
    fn test_xor_obfuscation() {
        let source = "vault_deobf = vault_slot ^ 0x6e9de2b30b19f9ea;";
        let tokens = Lexer::collect_tokens(source).unwrap();
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Caret)));
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::HexLiteral(0x6e9de2b30b19f9ea))));
    }
}
