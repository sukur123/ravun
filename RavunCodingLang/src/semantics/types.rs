use std::fmt;
use crate::parser::ast::{AstNode, AstNodeType};
use crate::semantics::error::{SemanticError, SemanticErrorType};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Void,
    Function(Vec<Type>, Box<Type>),
    Array(Box<Type>, Option<usize>),
    Struct(String),
    Module(String),
    Ref(Box<Type>),
    Optional(Box<Type>),
    Any,
    Null,
    TypeParameter(String, Option<Box<GenericConstraint>>),
    Unknown,
    Error,
}

#[derive(Debug, Clone)]
pub enum GenericConstraint {
    Trait(String),
    Type(Type),
}

impl Type {
    pub fn from_name(name: &str) -> Self {
        match name {
            "int" => Type::Int,
            "float" => Type::Float,
            "string" => Type::String,
            "bool" => Type::Bool,
            "void" => Type::Void,
            _ => {
                if name.ends_with(']') && name.contains('[') {
                    let base_end = name.find('[').unwrap();
                    let base_type_name = &name[0..base_end];
                    let base_type = Type::from_name(base_type_name);
                    
                    let size_str = &name[base_end + 1..name.len() - 1];
                    let size = if size_str.is_empty() {
                        None
                    } else {
                        size_str.parse::<usize>().ok()
                    };
                    
                    return Type::Array(Box::new(base_type), size);
                }
                
                if name.starts_with('&') {
                    let inner_type_name = &name[1..];
                    let inner_type = Type::from_name(inner_type_name);
                    return Type::Ref(Box::new(inner_type));
                }
                
                if name.ends_with('?') {
                    let inner_type_name = &name[0..name.len() - 1];
                    let inner_type = Type::from_name(inner_type_name);
                    return Type::Optional(Box::new(inner_type));
                }
                
                if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    Type::Struct(name.to_string())
                } else {
                    Type::Unknown
                }
            }
        }
    }
    
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            (t1, t2) if t1 == t2 => true,
            
            (Type::Any, _) | (_, Type::Any) => true,
            
            (Type::Null, Type::Optional(_)) => true,
            
            (Type::Int, Type::Float) => true,
            
            (Type::Optional(t1), t2) => t1.is_compatible_with(t2),
            (t1, Type::Optional(t2)) => t1.is_compatible_with(t2),
            
            (Type::Ref(t1), Type::Ref(t2)) => t1.is_compatible_with(t2),
            
            (Type::Array(t1, _), Type::Array(t2, _)) => t1.is_compatible_with(t2),
            
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return false;
                }
                
                ret1.is_compatible_with(ret2) && 
                params1.iter().zip(params2.iter()).all(|(p1, p2)| p1.is_compatible_with(p2))
            },
            
            (Type::TypeParameter(name1, _), Type::TypeParameter(name2, _)) => name1 == name2,
            
            _ => false,
        }
    }
    
    pub fn can_assign_from(&self, other: &Type) -> Result<(), SemanticError> {
        if self.is_compatible_with(other) {
            Ok(())
        } else {
            Err(SemanticError::new(
                SemanticErrorType::TypeMismatch,
                format!("Tip uyuşmazlığı: '{}' tipine '{}' tipi atanamaz", self, other),
                None,
            ))
        }
    }
    
    pub fn check_return_type(&self, expected: &Type) -> Result<(), SemanticError> {
        if self.is_compatible_with(expected) {
            Ok(())
        } else {
            Err(SemanticError::new(
                SemanticErrorType::InvalidReturn,
                format!("Geçersiz dönüş tipi: Beklenen '{}', bulunan '{}'", expected, self),
                None,
            ))
        }
    }
    
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Type::Int => 4,
            Type::Float => 8,
            Type::Bool => 1,
            Type::String => 0,
            Type::Void => 0,
            Type::Array(elem_type, Some(size)) => elem_type.size_in_bytes() * size,
            Type::Array(_, None) => 0,
            Type::Struct(_) => 0,
            Type::Function(_, _) => 8,
            Type::Module(_) => 0,
            Type::Ref(_) => 8,
            Type::Optional(inner) => inner.size_in_bytes() + 1,
            Type::Any => 0,
            Type::Null => 0,
            Type::TypeParameter(_, _) => 0,
            Type::Unknown => 0,
            Type::Error => 0,
        }
    }
    
    pub fn check_arithmetic_compatible(&self, other: &Type, operator: &str) -> Result<Type, SemanticError> {
        match (self, other) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            
            (Type::Float, Type::Float) => Ok(Type::Float),
            (Type::Int, Type::Float) => Ok(Type::Float),
            (Type::Float, Type::Int) => Ok(Type::Float),
            
            (Type::String, Type::String) if operator == "+" => Ok(Type::String),
            
            (Type::Any, _) => Ok(other.clone()),
            (_, Type::Any) => Ok(self.clone()),
            
            (Type::TypeParameter(_, _), other) if 
                [Type::Int, Type::Float, Type::String].contains(other) && 
                (operator == "+" || operator == "-" || operator == "*" || operator == "/") => 
                    Ok(other.clone()),
                    
            (self_type, Type::TypeParameter(_, _)) if 
                [Type::Int, Type::Float, Type::String].contains(self_type) && 
                (operator == "+" || operator == "-" || operator == "*" || operator == "/") => 
                    Ok(self_type.clone()),
            
            _ => Err(SemanticError::new(
                SemanticErrorType::TypeMismatch,
                format!("'{}' operatörü '{}' ve '{}' tipleri için geçerli değil", 
                        operator, self, other),
                None,
            )),
        }
    }
    
    pub fn check_comparison_compatible(&self, other: &Type, operator: &str) -> Result<Type, SemanticError> {
        if self == &Type::Any || other == &Type::Any {
            return Ok(Type::Bool);
        }
        
        if operator == "==" || operator == "!=" {
            if self.is_compatible_with(other) {
                return Ok(Type::Bool);
            }
        }
        else if operator == "<" || operator == ">" || operator == "<=" || operator == ">=" {
            match (self, other) {
                (Type::Int, Type::Int) | 
                (Type::Float, Type::Float) |
                (Type::Int, Type::Float) |
                (Type::Float, Type::Int) |
                (Type::String, Type::String) => return Ok(Type::Bool),
                
                (Type::TypeParameter(_, _), other) if 
                    [Type::Int, Type::Float, Type::String].contains(other) => 
                        return Ok(Type::Bool),
                    
                (self_type, Type::TypeParameter(_, _)) if 
                    [Type::Int, Type::Float, Type::String].contains(self_type) => 
                        return Ok(Type::Bool),
                
                _ => {}
            }
        }
        
        Err(SemanticError::new(
            SemanticErrorType::TypeMismatch,
            format!("'{}' karşılaştırma operatörü '{}' ve '{}' tipleri için geçerli değil", 
                    operator, self, other),
            None,
        ))
    }
    
    pub fn from_type_annotation(node: &AstNode) -> Result<Type, SemanticError> {
        if node.node_type != AstNodeType::TypeAnnotation {
            return Err(SemanticError::new(
                SemanticErrorType::Other,
                format!("Tip tanımlaması bekleniyordu, {:?} bulundu", node.node_type),
                node.token.clone(),
            ));
        }
        
        let type_name = node.value.as_ref().ok_or_else(|| 
            SemanticError::new(
                SemanticErrorType::Other,
                "Tip adı bulunamadı".to_string(),
                node.token.clone(),
            )
        )?;
        
        Ok(Type::from_name(type_name))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Void => write!(f, "void"),
            Type::Function(params, return_type) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            },
            Type::Array(elem_type, Some(size)) => write!(f, "{}[{}]", elem_type, size),
            Type::Array(elem_type, None) => write!(f, "{}[]", elem_type),
            Type::Struct(name) => write!(f, "{}", name),
            Type::Module(name) => write!(f, "module:{}", name),
            Type::Ref(inner) => write!(f, "&{}", inner),
            Type::Optional(inner) => write!(f, "{}?", inner),
            Type::Any => write!(f, "any"),
            Type::Null => write!(f, "null"),
            Type::TypeParameter(name, _) => write!(f, "{}", name),
            Type::Unknown => write!(f, "bilinmeyen"),
            Type::Error => write!(f, "hata"),
        }
    }
}

pub struct TypeConverter;

impl TypeConverter {
    pub fn string_literal_to_type(literal: &str) -> Type {
        Type::String
    }
    
    pub fn int_literal_to_type(literal: &str) -> Type {
        Type::Int
    }
    
    pub fn float_literal_to_type(literal: &str) -> Type {
        Type::Float
    }
    
    pub fn bool_literal_to_type(literal: &str) -> Type {
        Type::Bool
    }
}