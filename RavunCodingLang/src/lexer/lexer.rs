use std::iter::Peekable;
use std::str::Chars;
use crate::lexer::token::{Token, TokenType};

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            line: 1,
            column: 1,
            position: 0,
        }
    }
    
    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        
        if let Some(ch) = c {
            self.position += 1;
            self.column += 1;
            
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            }
        }
        
        c
    }
    
    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }
    
    fn identifier(&mut self, first_char: char) -> Token {
        let start_pos = self.column - 1;
        let mut identifier = String::new();
        identifier.push(first_char);
        
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        let token_type = match identifier.as_str() {
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "const" => TokenType::Const,
            "fn" => TokenType::Fn,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "in" => TokenType::In,
            "struct" => TokenType::Struct,
            "impl" => TokenType::Impl,
            "mod" => TokenType::Mod,
            "pub" => TokenType::Pub,
            "async" => TokenType::Async,
            "parallel" => TokenType::Parallel,
            "match" => TokenType::Match,
            "true" | "false" => TokenType::BoolLiteral,
            _ => TokenType::Identifier,
        };
        
        Token::new(token_type, identifier, self.line, start_pos)
    }
    
    fn number(&mut self, first_digit: char) -> Token {
        let start_pos = self.column - 1;
        let mut number = String::new();
        let mut is_float = false;
        
        number.push(first_digit);
        
        while let Some(&c) = self.peek() {
            if c.is_digit(10) {
                number.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                is_float = true;
                number.push(c);
                self.advance();
                
                if let Some(&next) = self.peek() {
                    if !next.is_digit(10) {
                        return Token::new(TokenType::Invalid, number, self.line, start_pos);
                    }
                }
            } else {
                break;
            }
        }
        
        let token_type = if is_float {
            TokenType::FloatLiteral
        } else {
            TokenType::IntLiteral
        };
        
        Token::new(token_type, number, self.line, start_pos)
    }
    
    fn string(&mut self) -> Token {
        let start_pos = self.column - 1;
        let mut string = String::new();
        let mut escaped = false;
        
        while let Some(c) = self.advance() {
            if escaped {
                match c {
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    'r' => string.push('\r'),
                    '\\' => string.push('\\'),
                    '"' => string.push('"'),
                    _ => {
                        string.push('\\');
                        string.push(c);
                    }
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                return Token::new(TokenType::StringLiteral, string, self.line, start_pos);
            } else {
                string.push(c);
            }
        }
        
        Token::new(TokenType::Invalid, string, self.line, start_pos)
    }
    
    fn comment(&mut self) -> Token {
        let start_pos = self.column - 1;
        let mut comment = String::from("/");
        
        if let Some(&next) = self.peek() {
            if next == '/' {
                self.advance();
                comment.push('/');
                
                while let Some(c) = self.advance() {
                    comment.push(c);
                    if c == '\n' {
                        break;
                    }
                }
            } else if next == '*' {
                self.advance();
                comment.push('*');
                
                let mut prev_char = '\0';
                
                while let Some(c) = self.advance() {
                    comment.push(c);
                    
                    if prev_char == '*' && c == '/' {
                        break;
                    }
                    
                    prev_char = c;
                }
            } else {
                return Token::new(TokenType::Slash, "/".to_string(), self.line, start_pos);
            }
        } else {
            return Token::new(TokenType::Slash, "/".to_string(), self.line, start_pos);
        }
        
        Token::new(TokenType::Comment, comment, self.line, start_pos)
    }
    
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        if let Some(c) = self.advance() {
            match c {
                c if c.is_alphabetic() || c == '_' => self.identifier(c),
                
                c if c.is_digit(10) => self.number(c),
                
                '"' => self.string(),
                
                '/' => self.comment(),
                
                '+' => {
                    let start_pos = self.column - 1;
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            Token::new(TokenType::PlusAssign, "+=".to_string(), self.line, start_pos)
                        } else {
                            Token::new(TokenType::Plus, "+".to_string(), self.line, start_pos)
                        }
                    } else {
                        Token::new(TokenType::Plus, "+".to_string(), self.line, start_pos)
                    }
                },
                '-' => {
                    let start_pos = self.column - 1;
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            Token::new(TokenType::MinusAssign, "-=".to_string(), self.line, start_pos)
                        } else if next == '>' {
                            self.advance();
                            Token::new(TokenType::Arrow, "->".to_string(), self.line, start_pos)
                        } else {
                            Token::new(TokenType::Minus, "-".to_string(), self.line, start_pos)
                        }
                    } else {
                        Token::new(TokenType::Minus, "-".to_string(), self.line, start_pos)
                    }
                },
                '*' => {
                    let start_pos = self.column - 1;
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            Token::new(TokenType::MulAssign, "*=".to_string(), self.line, start_pos)
                        } else {
                            Token::new(TokenType::Asterisk, "*".to_string(), self.line, start_pos)
                        }
                    } else {
                        Token::new(TokenType::Asterisk, "*".to_string(), self.line, start_pos)
                    }
                },
                '%' => Token::new(TokenType::Percent, "%".to_string(), self.line, self.column - 1),
                '^' => Token::new(TokenType::Caret, "^".to_string(), self.line, self.column - 1),
                
                '=' => {
                    let start_pos = self.column - 1;
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            Token::new(TokenType::Equal, "==".to_string(), self.line, start_pos)
                        } else {
                            Token::new(TokenType::Assign, "=".to_string(), self.line, start_pos)
                        }
                    } else {
                        Token::new(TokenType::Assign, "=".to_string(), self.line, start_pos)
                    }
                },
                '!' => {
                    let start_pos = self.column - 1;
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            Token::new(TokenType::NotEqual, "!=".to_string(), self.line, start_pos)
                        } else {
                            Token::new(TokenType::Invalid, "!".to_string(), self.line, start_pos)
                        }
                    } else {
                        Token::new(TokenType::Invalid, "!".to_string(), self.line, start_pos)
                    }
                },
                '>' => {
                    let start_pos = self.column - 1;
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            Token::new(TokenType::GreaterEq, ">=".to_string(), self.line, start_pos)
                        } else {
                            Token::new(TokenType::Greater, ">".to_string(), self.line, start_pos)
                        }
                    } else {
                        Token::new(TokenType::Greater, ">".to_string(), self.line, start_pos)
                    }
                },
                '<' => {
                    let start_pos = self.column - 1;
                    if let Some(&next) = self.peek() {
                        if next == '=' {
                            self.advance();
                            Token::new(TokenType::LessEq, "<=".to_string(), self.line, start_pos)
                        } else {
                            Token::new(TokenType::Less, "<".to_string(), self.line, start_pos)
                        }
                    } else {
                        Token::new(TokenType::Less, "<".to_string(), self.line, start_pos)
                    }
                },
                
                '(' => Token::new(TokenType::LeftParen, "(".to_string(), self.line, self.column - 1),
                ')' => Token::new(TokenType::RightParen, ")".to_string(), self.line, self.column - 1),
                '{' => Token::new(TokenType::LeftBrace, "{".to_string(), self.line, self.column - 1),
                '}' => Token::new(TokenType::RightBrace, "}".to_string(), self.line, self.column - 1),
                '[' => Token::new(TokenType::LeftBracket, "[".to_string(), self.line, self.column - 1),
                ']' => Token::new(TokenType::RightBracket, "]".to_string(), self.line, self.column - 1),
                ';' => Token::new(TokenType::Semicolon, ";".to_string(), self.line, self.column - 1),
                ':' => Token::new(TokenType::Colon, ":".to_string(), self.line, self.column - 1),
                ',' => Token::new(TokenType::Comma, ",".to_string(), self.line, self.column - 1),
                '.' => {
                    let start_pos = self.column - 1;
                    if let Some(&next) = self.peek() {
                        if next == '.' {
                            self.advance();
                            Token::new(TokenType::DoubleDot, "..".to_string(), self.line, start_pos)
                        } else {
                            Token::new(TokenType::Dot, ".".to_string(), self.line, start_pos)
                        }
                    } else {
                        Token::new(TokenType::Dot, ".".to_string(), self.line, start_pos)
                    }
                },
                
                _ => Token::new(TokenType::Invalid, c.to_string(), self.line, self.column - 1),
            }
        } else {
            Token::new(TokenType::EOF, "".to_string(), self.line, self.column)
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token();
            
            if token.token_type == TokenType::EOF {
                tokens.push(token);
                break;
            }
            
            if token.token_type != TokenType::Comment && token.token_type != TokenType::Whitespace {
                tokens.push(token);
            }
        }
        
        tokens
    }
}
