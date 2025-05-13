#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Let,
    Mut,
    Const,
    Fn,
    Return,
    If,
    Else,
    For,
    While,
    In,
    Struct,
    Impl,
    Mod,
    Pub,
    Async,
    Parallel,
    Match,
    
    Identifier,
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    CharLiteral,
    BoolLiteral,
    
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Caret,
    
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    
    Assign,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    DoubleDot,
    Arrow,
    
    Comment,
    Whitespace,
    EOF,
    
    Invalid,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}
