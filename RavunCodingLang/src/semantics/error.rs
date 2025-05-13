use std::fmt;
use crate::lexer::token::Token;

#[derive(Debug, Clone)]
pub enum SemanticErrorType {
    UndefinedVariable,
    UndefinedFunction,
    TypeMismatch,
    Redefinition,
    InvalidReturn,
    Other,
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub error_type: SemanticErrorType,
    pub message: String,
    pub token: Option<Token>,
    pub line: usize,
    pub column: usize,
    pub is_warning: bool,
}

impl SemanticError {
    pub fn new(error_type: SemanticErrorType, message: String, token: Option<Token>) -> Self {
        let (line, column) = if let Some(ref tok) = token {
            (tok.line, tok.column)
        } else {
            (0, 0)
        };
        
        SemanticError {
            error_type,
            message,
            token,
            line,
            column,
            is_warning: false,
        }
    }
    
    pub fn with_position(error_type: SemanticErrorType, message: String, line: usize, column: usize) -> Self {
        SemanticError {
            error_type,
            message,
            token: None,
            line,
            column,
            is_warning: false,
        }
    }
    
    pub fn with_warning(message: String, line: usize, column: usize, is_warning: bool) -> Self {
        SemanticError {
            error_type: SemanticErrorType::Other,
            message,
            token: None,
            line,
            column,
            is_warning,
        }
    }

    pub fn new_simple(message: String, line: usize, column: usize, is_warning: bool) -> Self {
        SemanticError {
            error_type: SemanticErrorType::Other,
            message,
            token: None,
            line,
            column,
            is_warning,
        }
    }
    
    pub fn position_info(&self) -> String {
        if let Some(ref token) = self.token {
            format!("satır {}, sütun {}", token.line, token.column)
        } else {
            format!("satır {}, sütun {}", self.line, self.column)
        }
    }
    
    pub fn is_warning(&self) -> bool {
        self.is_warning
    }

    pub fn new_with_location(error_type: SemanticErrorType, message: String, line: usize, column: usize, is_warning: bool) -> Self {
        SemanticError {
            error_type,
            message,
            line,
            column,
            token: None,
            is_warning
        }
    }

    pub fn new_basic(message: String, line: usize, column: usize, is_warning: bool) -> Self {
        SemanticError {
            error_type: SemanticErrorType::Other,
            message,
            token: None,
            line,
            column,
            is_warning,
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos_info = self.position_info();
        write!(f, "{} - {}", pos_info, self.message)
    }
}

impl std::error::Error for SemanticError {}