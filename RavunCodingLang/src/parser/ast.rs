use std::fmt;
use crate::lexer::token::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeType {
    Program,
    
    BinaryExpr,
    UnaryExpr,
    LiteralExpr,
    IdentifierExpr,
    GroupExpr,
    CallExpr,
    IndexExpr,
    MemberExpr,
    
    ExprStmt,
    BlockStmt,
    IfStmt,
    WhileStmt,
    ForStmt,
    ReturnStmt,
    
    VarDecl,
    FuncDecl,
    StructDecl,
    ImplDecl,
    ModDecl,
    ParamDecl,
    
    TypeAnnotation,
}

#[derive(Clone)]
pub struct AstNode {
    pub node_type: AstNodeType,
    pub token: Option<Token>,
    pub children: Vec<AstNode>,
    pub value: Option<String>,
    pub metadata: Option<String>,
    pub line: usize,
    pub column: usize,
    pub kind: Option<AstNodeKind>,
}

impl PartialEq for AstNode {
    fn eq(&self, other: &Self) -> bool {
        self.node_type == other.node_type &&
        self.value == other.value &&
        self.metadata == other.metadata &&
        self.kind == other.kind &&
        self.children == other.children
    }
}

impl Eq for AstNode {}

impl AstNode {
    pub fn new(node_type: AstNodeType, token: Option<Token>) -> Self {
        let (line, column) = if let Some(ref tok) = token {
            (tok.line, tok.column)
        } else {
            (0, 0)
        };
        
        AstNode {
            node_type,
            token,
            children: Vec::new(),
            value: None,
            metadata: None,
            line,
            column,
            kind: None,
        }
    }
    
    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }
    
    pub fn set_value(&mut self, value: String) {
        self.value = Some(value);
    }
    
    pub fn set_metadata(&mut self, metadata: String) {
        self.metadata = Some(metadata);
    }
    
    pub fn line(&self) -> usize {
        self.line
    }
    
    pub fn column(&self) -> usize {
        self.column
    }
    
    pub fn kind(&self) -> &Option<AstNodeKind> {
        &self.kind
    }
    
    pub fn set_kind(&mut self, kind: AstNodeKind) {
        self.kind = Some(kind);
    }
}

impl fmt::Debug for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref val) = self.value {
            write!(f, "{:?}({})", self.node_type, val)?;
        } else {
            write!(f, "{:?}", self.node_type)?;
        }
        
        if let Some(ref meta) = self.metadata {
            write!(f, " [{}]", meta)?;
        }
        
        if !self.children.is_empty() {
            write!(f, " {{ ")?;
            for (i, child) in self.children.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{:?}", child)?;
            }
            write!(f, " }}")?;
        }
        
        Ok(())
    }
}

impl PartialEq for Box<AstNode> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for Box<AstNode> {}

impl PartialEq for (String, Box<AstNode>) {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for (String, Box<AstNode>) {}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
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
}

impl From<TokenType> for BinaryOperator {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Plus => BinaryOperator::Add,
            TokenType::Minus => BinaryOperator::Subtract,
            TokenType::Asterisk => BinaryOperator::Multiply,
            TokenType::Slash => BinaryOperator::Divide,
            TokenType::Percent => BinaryOperator::Modulo,
            TokenType::Caret => BinaryOperator::Power,
            TokenType::Equal => BinaryOperator::Equal,
            TokenType::NotEqual => BinaryOperator::NotEqual,
            TokenType::Greater => BinaryOperator::Greater,
            TokenType::Less => BinaryOperator::Less,
            TokenType::GreaterEq => BinaryOperator::GreaterEq,
            TokenType::LessEq => BinaryOperator::LessEq,
            TokenType::Assign => BinaryOperator::Assign,
            TokenType::PlusAssign => BinaryOperator::PlusAssign,
            TokenType::MinusAssign => BinaryOperator::MinusAssign,
            TokenType::MulAssign => BinaryOperator::MulAssign,
            TokenType::DivAssign => BinaryOperator::DivAssign,
            _ => panic!("Invalid binary operator token type: {:?}", token_type),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeKind {
    Identifier(String),
    FunctionDef(String, Vec<ParamInfo>, Type, Box<AstNode>),
    GenericDef(String, Vec<String>, Box<AstNode>),
    GenericInstantiation(String, Vec<String>),
    OperatorDef(String, Vec<ParamInfo>, Type, Box<AstNode>),
    Attribute(String, Vec<String>, Box<AstNode>),
    Import(Vec<String>, Vec<String>),
    Export(Vec<String>),
    Match(Box<AstNode>, Vec<AstNode>),
    MatchCase(Box<AstNode>, Option<Box<AstNode>>, Box<AstNode>),
    StructPattern(String, Vec<(String, Box<AstNode>)>),
    EnumPattern(String, String, Vec<Box<AstNode>>),
    TuplePattern(Vec<Box<AstNode>>),
    Let(String, Option<Type>, Option<Box<AstNode>>),
    Lambda(Vec<ParamInfo>, Box<AstNode>),
    Block(Vec<AstNode>),
    If(Box<AstNode>, Box<AstNode>, Option<Box<AstNode>>),
    Parallel(Box<AstNode>),
    AsyncCall(Box<AstNode>, Vec<AstNode>),
    Wildcard,
    Literal(String),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamInfo {
    pub name: String,
    pub type_info: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Unit,
    Unknown,
    Error,
    Any,
    Struct(String),
    Module(String),
    Array(Box<Type>),
    Optional(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    TypeParameter(String),
    Enum(String),
    Null,
}
