use crate::parser::ast::{AstNode, AstNodeType};
use crate::semantics::error::{SemanticError, SemanticErrorType};
use crate::semantics::symbol_table::{Symbol, SymbolTable, SymbolKind, ScopeType};
use crate::semantics::types::Type;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub description: String,
    pub start_value: Option<i32>,
    pub end_value: Option<i32>,
    pub step_value: Option<i32>,
}

impl LoopInfo {
    pub fn is_small_constant_range(&self) -> bool {
        if let (Some(start), Some(end), Some(step)) = (self.start_value, self.end_value, self.step_value) {
            let range_size = (end - start) / step;
            range_size >= 0 && range_size <= 4
        } else {
            false
        }
    }
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    current_function_return_type: Option<Type>,
    errors: Vec<SemanticError>,
    in_loop: bool,
    pub constant_expressions: Vec<String>,
    pub loop_infos: Vec<LoopInfo>,
    pub small_functions: Vec<String>,
    pub warnings: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            current_function_return_type: None,
            errors: Vec::new(),
            in_loop: false,
            constant_expressions: Vec::new(),
            loop_infos: Vec::new(),
            small_functions: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn analyze(&mut self, ast: &AstNode) -> Vec<SemanticError> {
        self.errors.clear();
        self.visit_node(ast);
        
        self.check_unused_variables();
        
        self.check_uninitialized_variables();
        
        self.errors.clone()
    }
    
    fn check_unused_variables(&mut self) {
        let unused_symbols = self.symbol_table.get_unused_symbols();
        
        for symbol in unused_symbols {
            if symbol.kind == SymbolKind::Function && symbol.name == "main" {
                continue;
            }
            
            self.add_warning(SemanticError::with_position(
                SemanticErrorType::Other,
                format!("'{}' {} tanımlandı fakat hiç kullanılmadı", 
                       symbol.name, symbol.kind),
                symbol.line,
                symbol.column,
            ));
        }
    }
    
    fn check_uninitialized_variables(&mut self) {
        let uninitialized_symbols = self.symbol_table.get_uninitialized_symbols();
        
        for symbol in uninitialized_symbols {
            self.add_error(SemanticError::with_position(
                SemanticErrorType::Other,
                format!("'{}' değişkeni kullanılmadan önce başlatılmamış", symbol.name),
                symbol.line,
                symbol.column,
            ));
        }
    }
    
    fn add_error(&mut self, error: SemanticError) {
        self.errors.push(error);
    }
    
    fn add_warning(&mut self, warning: SemanticError) {
        self.errors.push(warning);
    }
    
    fn visit_node(&mut self, node: &AstNode) -> Type {
        match &node.node_type {
            AstNodeType::Program => self.visit_program(node),
            AstNodeType::VarDecl => self.visit_var_declaration(node),
            AstNodeType::FuncDecl => self.visit_function_declaration(node),
            AstNodeType::ParamDecl => self.visit_param_declaration(node),
            AstNodeType::TypeAnnotation => self.visit_type_annotation(node),
            AstNodeType::BlockStmt => self.visit_block(node),
            AstNodeType::IfStmt => self.visit_if_stmt(node),
            AstNodeType::WhileStmt => self.visit_while_stmt(node),
            AstNodeType::ForStmt => self.visit_for_stmt(node),
            AstNodeType::ReturnStmt => self.visit_return_stmt(node),
            AstNodeType::ExprStmt => self.visit_expr_stmt(node),
            AstNodeType::BinaryExpr => self.visit_binary_expr(node),
            AstNodeType::UnaryExpr => self.visit_unary_expr(node),
            AstNodeType::LiteralExpr => self.visit_literal(node),
            AstNodeType::IdentifierExpr => self.visit_identifier(node),
            AstNodeType::CallExpr => self.visit_call_expr(node),
            AstNodeType::GroupExpr => self.visit_group_expr(node),
            AstNodeType::StructDecl => self.visit_struct_declaration(node),
            AstNodeType::ImplDecl => self.visit_impl_declaration(node),
            AstNodeType::MemberExpr => self.visit_member_expr(node),
            AstNodeType::IndexExpr => self.visit_index_expr(node),
            AstNodeType::BreakStmt => self.visit_break_stmt(node),
            AstNodeType::ContinueStmt => self.visit_continue_stmt(node),
            AstNodeType::ModDecl => self.visit_module_declaration(node),
            _ => {
                self.add_error(SemanticError::new(
                    SemanticErrorType::Other,
                    format!("Desteklenmeyen AST düğüm tipi: {:?}", node.node_type),
                    node.token.clone(),
                ));
                Type::Error
            }
        }
    }
    
    fn visit_program(&mut self, node: &AstNode) -> Type {
        for child in &node.children {
            self.visit_node(child);
        }
        
        match self.symbol_table.resolve("main") {
            Ok(symbol) => {
                if symbol.kind != SymbolKind::Function {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::Other,
                        "'main' bir fonksiyon olmalıdır".to_string(),
                        None,
                    ));
                }
                
                if let Err(err) = self.symbol_table.mark_used("main") {
                    self.add_error(err);
                }
            },
            Err(_) => {
                self.add_error(SemanticError::new(
                    SemanticErrorType::Other,
                    "'main' fonksiyonu tanımlanmamış".to_string(),
                    None,
                ));
            }
        }
        
        Type::Void
    }
    
    fn visit_var_declaration(&mut self, node: &AstNode) -> Type {
        let var_name = node.value.as_ref().expect("Değişken adı bulunamadı");
        let is_mutable = node.metadata.as_ref().map_or(false, |m| m == "mutable");
        
        let mut var_type = Type::Unknown;
        let mut init_value_type = Type::Unknown;
        let mut is_initialized = false;
        
        for child in &node.children {
            match child.node_type {
                AstNodeType::TypeAnnotation => {
                    var_type = self.visit_node(child);
                },
                _ => {
                    init_value_type = self.visit_node(child);
                    is_initialized = true;
                }
            }
        }
        
        if var_type == Type::Unknown && init_value_type != Type::Unknown && init_value_type != Type::Error {
            var_type = init_value_type.clone();
        }
        
        if var_type == Type::Unknown {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                format!("'{}' değişkeninin tipi belirtilmemiş ve çıkarsanamadı", var_name),
                node.token.clone(),
            ));
            var_type = Type::Error;
        }
        
        if is_initialized && init_value_type != Type::Error && var_type != Type::Error {
            if let Err(err) = var_type.can_assign_from(&init_value_type) {
                self.add_error(SemanticError::with_position(
                    SemanticErrorType::TypeMismatch,
                    format!("'{}' değişkeni için tip uyuşmazlığı: {}", var_name, err.message),
                    node.token.as_ref().map_or(0, |t| t.line),
                    node.token.as_ref().map_or(0, |t| t.column),
                ));
            }
        }
        
        let line = node.token.as_ref().map_or(0, |t| t.line);
        let column = node.token.as_ref().map_or(0, |t| t.column);
        
        if let Err(err) = self.symbol_table.define_variable(
            var_name.clone(), 
            var_type.clone(), 
            is_mutable, 
            is_initialized,
            line, 
            column
        ) {
            self.add_error(err);
        }
        
        var_type
    }
    
    fn visit_function_declaration(&mut self, node: &AstNode) -> Type {
        let func_name = node.value.as_ref().expect("Fonksiyon adı bulunamadı");
        
        let mut param_symbols = Vec::new();
        let mut return_type = Type::Void;
        let mut body_node = None;
        
        for child in &node.children {
            match child.node_type {
                AstNodeType::ParamDecl => {
                    let param_name = child.value.as_ref().expect("Parametre adı bulunamadı");
                    let mut param_type = Type::Unknown;
                    
                    for param_child in &child.children {
                        if let AstNodeType::TypeAnnotation = param_child.node_type {
                            param_type = self.visit_type_annotation(param_child);
                            break;
                        }
                    }
                    
                    if param_type == Type::Unknown {
                        self.add_error(SemanticError::new(
                            SemanticErrorType::Other,
                            format!("'{}' parametresinin tipi belirtilmemiş", param_name),
                            child.token.clone(),
                        ));
                        param_type = Type::Error;
                    }
                    
                    let line = child.token.as_ref().map_or(0, |t| t.line);
                    let column = child.token.as_ref().map_or(0, |t| t.column);
                    
                    let param_symbol = Symbol::new(
                        param_name.clone(),
                        param_type,
                        SymbolKind::Parameter,
                        false,
                        0,
                        line,
                        column,
                    );
                    
                    param_symbols.push(param_symbol);
                },
                AstNodeType::TypeAnnotation => {
                    return_type = self.visit_node(child);
                },
                AstNodeType::BlockStmt => {
                    body_node = Some(child);
                },
                _ => {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::Other,
                        format!("Beklenmeyen düğüm tipi: {:?}", child.node_type),
                        child.token.clone(),
                    ));
                }
            }
        }
        
        let line = node.token.as_ref().map_or(0, |t| t.line);
        let column = node.token.as_ref().map_or(0, |t| t.column);
        
        if let Err(err) = self.symbol_table.define_function(
            func_name.clone(),
            return_type.clone(),
            param_symbols.clone(),
            line,
            column
        ) {
            self.add_error(err);
        }
        
        if let Some(body) = body_node {
            self.symbol_table.enter_scope(ScopeType::Function);
            
            for param in &param_symbols {
                let scope_level = self.symbol_table.current_level();
                let mut param_copy = param.clone();
                param_copy.scope_level = scope_level;
                param_copy.is_initialized = true;
                
                if let Err(err) = self.symbol_table.define_symbol(param_copy) {
                    self.add_error(err);
                }
            }
            
            let prev_return_type = self.current_function_return_type.clone();
            self.current_function_return_type = Some(return_type.clone());
            
            self.visit_node(body);
            
            self.current_function_return_type = prev_return_type;
            
            self.symbol_table.exit_scope();
        }
        
        Type::Function(
            param_symbols.iter().map(|p| p.symbol_type.clone()).collect(),
            Box::new(return_type)
        )
    }
    
    fn visit_param_declaration(&mut self, node: &AstNode) -> Type {
        let param_name = node.value.as_ref().expect("Parametre adı bulunamadı");
        
        let mut param_type = Type::Unknown;
        
        for child in &node.children {
            if let AstNodeType::TypeAnnotation = child.node_type {
                param_type = self.visit_node(child);
                break;
            }
        }
        
        if param_type == Type::Unknown {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                format!("'{}' parametresinin tipi belirtilmemiş", param_name),
                node.token.clone(),
            ));
            param_type = Type::Error;
        }
        
        param_type
    }
    
    fn visit_type_annotation(&mut self, node: &AstNode) -> Type {
        let type_name = node.value.as_ref().expect("Tip adı bulunamadı");
        let result_type = Type::from_name(type_name);
        
        if let Type::Struct(struct_name) = &result_type {
            match self.symbol_table.resolve_type(struct_name) {
                Ok(_) => {},
                Err(_) => {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::Other,
                        format!("'{}' tipi tanımlanmamış", struct_name),
                        node.token.clone(),
                    ));
                    return Type::Error;
                }
            }
        }
        
        result_type
    }
    
    fn visit_block(&mut self, node: &AstNode) -> Type {
        self.symbol_table.enter_scope(ScopeType::Block);
        
        let mut last_type = Type::Void;
        
        for child in &node.children {
            last_type = self.visit_node(child);
        }
        
        self.symbol_table.exit_scope();
        
        last_type
    }
    
    fn visit_if_stmt(&mut self, node: &AstNode) -> Type {
        if node.children.len() < 2 {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "If ifadesi eksik".to_string(),
                node.token.clone(),
            ));
            return Type::Error;
        }
        
        let condition_type = self.visit_node(&node.children[0]);
        
        if condition_type != Type::Bool && condition_type != Type::Error {
            self.add_error(SemanticError::new(
                SemanticErrorType::TypeMismatch,
                format!("If koşulu boolean tipinde olmalı, bulunan: {}", condition_type),
                node.token.clone(),
            ));
        }
        
        self.symbol_table.enter_scope(ScopeType::If);
        self.visit_node(&node.children[1]);
        self.symbol_table.exit_scope();
        
        if node.children.len() > 2 {
            self.symbol_table.enter_scope(ScopeType::If);
            self.visit_node(&node.children[2]);
            self.symbol_table.exit_scope();
        }
        
        Type::Void
    }
    
    fn visit_while_stmt(&mut self, node: &AstNode) -> Type {
        if node.children.len() < 2 {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "While ifadesi eksik".to_string(),
                node.token.clone(),
            ));
            return Type::Error;
        }
        
        let condition_type = self.visit_node(&node.children[0]);
        
        if condition_type != Type::Bool && condition_type != Type::Error {
            self.add_error(SemanticError::new(
                SemanticErrorType::TypeMismatch,
                format!("While koşulu boolean tipinde olmalı, bulunan: {}", condition_type),
                node.token.clone(),
            ));
        }
        
        let prev_in_loop = self.in_loop;
        self.in_loop = true;
        
        self.symbol_table.enter_scope(ScopeType::Loop);
        self.visit_node(&node.children[1]);
        self.symbol_table.exit_scope();
        
        self.in_loop = prev_in_loop;
        
        Type::Void
    }
    
    fn visit_for_stmt(&mut self, node: &AstNode) -> Type {
        if node.children.len() < 3 {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "For ifadesi eksik".to_string(),
                node.token.clone(),
            ));
            return Type::Error;
        }
        
        self.symbol_table.enter_scope(ScopeType::Loop);
        
        let iterator_var = &node.children[0];
        let var_name = iterator_var.value.as_ref().expect("Döngü değişkeni adı bulunamadı");
        
        let range_type = self.visit_node(&node.children[1]);
        
        let element_type = match &range_type {
            Type::Array(elem_type, _) => *elem_type.clone(),
            Type::String => Type::String,
            _ => {
                if range_type != Type::Error {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::TypeMismatch,
                        format!("For döngüsünün '{}'  tipi üzerinde döngülenemez", range_type),
                        node.children[1].token.clone(),
                    ));
                }
                Type::Error
            }
        };
        
        let line = iterator_var.token.as_ref().map_or(0, |t| t.line);
        let column = iterator_var.token.as_ref().map_or(0, |t| t.column);
        
        if let Err(err) = self.symbol_table.define_variable(
            var_name.clone(),
            element_type,
            false,
            true,
            line,
            column
        ) {
            self.add_error(err);
        }
        
        let prev_in_loop = self.in_loop;
        self.in_loop = true;
        
        self.visit_node(&node.children[2]);
        
        self.in_loop = prev_in_loop;
        
        self.symbol_table.exit_scope();
        
        Type::Void
    }
    
    fn visit_return_stmt(&mut self, node: &AstNode) -> Type {
        let mut return_value_type = Type::Void;
        
        if !node.children.is_empty() {
            return_value_type = self.visit_node(&node.children[0]);
        }
        
        if let Some(expected_type) = &self.current_function_return_type {
            if expected_type != &Type::Error && return_value_type != Type::Error {
                if let Err(err) = expected_type.can_assign_from(&return_value_type) {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::InvalidReturn,
                        format!("Dönüş tipi uyuşmazlığı: {}", err.message),
                        node.token.clone(),
                    ));
                }
            }
        } else {
            self.add_error(SemanticError::new(
                SemanticErrorType::InvalidReturn,
                "Return ifadesi yalnızca fonksiyon içinde kullanılabilir".to_string(),
                node.token.clone(),
            ));
        }
        
        Type::Void
    }
    
    fn visit_expr_stmt(&mut self, node: &AstNode) -> Type {
        if node.children.is_empty() {
            return Type::Void;
        }
        
        self.visit_node(&node.children[0])
    }
    
    fn visit_binary_expr(&mut self, node: &AstNode) -> Type {
        if node.children.len() < 2 {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "Eksik ikili ifade".to_string(),
                node.token.clone(),
            ));
            return Type::Error;
        }
        
        let left_type = self.visit_node(&node.children[0]);
        let right_type = self.visit_node(&node.children[1]);
        
        if left_type == Type::Error || right_type == Type::Error {
            return Type::Error;
        }
        
        let operator = node.value.as_ref().expect("Operatör bulunamadı");
        
        match operator.as_str() {
            "+" | "-" | "*" | "/" | "%" => {
                match left_type.check_arithmetic_compatible(&right_type, operator) {
                    Ok(result_type) => result_type,
                    Err(err) => {
                        self.add_error(err);
                        Type::Error
                    }
                }
            },
            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                match left_type.check_comparison_compatible(&right_type, operator) {
                    Ok(_) => Type::Bool,
                    Err(err) => {
                        self.add_error(err);
                        Type::Error
                    }
                }
            },
            "&&" | "||" => {
                if left_type != Type::Bool || right_type != Type::Bool {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::TypeMismatch,
                        format!("Mantıksal operatör '{}' için boolean değerler gerekli, bulunan: {} ve {}", 
                                operator, left_type, right_type),
                        node.token.clone(),
                    ));
                    Type::Error
                } else {
                    Type::Bool
                }
            },
            "=" | "+=" | "-=" | "*=" | "/=" => {
                if let AstNodeType::IdentifierExpr = node.children[0].node_type {
                    let var_name = node.children[0].value.as_ref().expect("Değişken adı bulunamadı");
                    
                    match self.symbol_table.resolve(var_name) {
                        Ok(symbol) => {
                            if !symbol.is_mutable {
                                self.add_error(SemanticError::new(
                                    SemanticErrorType::Other,
                                    format!("'{}' değiştirilemez (mut değil)", var_name),
                                    node.token.clone(),
                                ));
                            }
                            
                            if operator == "=" {
                                if let Err(err) = symbol.symbol_type.can_assign_from(&right_type) {
                                    self.add_error(SemanticError::new(
                                        SemanticErrorType::TypeMismatch,
                                        format!("Tip uyuşmazlığı: {}", err.message),
                                        node.token.clone(),
                                    ));
                                }
                            } else {
                                let op = match operator.as_str() {
                                    "+=" => "+",
                                    "-=" => "-",
                                    "*=" => "*",
                                    "/=" => "/",
                                    _ => "?",
                                };
                                
                                if let Err(err) = symbol.symbol_type.check_arithmetic_compatible(&right_type, op) {
                                    self.add_error(SemanticError::new(
                                        SemanticErrorType::TypeMismatch,
                                        format!("Bileşik atama için tip uyuşmazlığı: {}", err.message),
                                        node.token.clone(),
                                    ));
                                }
                            }
                            
                            if let Err(err) = self.symbol_table.mark_initialized(var_name) {
                                self.add_error(err);
                            }
                            
                            symbol.symbol_type.clone()
                        },
                        Err(err) => {
                            self.add_error(err);
                            Type::Error
                        }
                    }
                } else if let AstNodeType::MemberExpr = node.children[0].node_type {
                    Type::Error
                } else if let AstNodeType::IndexExpr = node.children[0].node_type {
                    Type::Error
                } else {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::Other,
                        "Sol taraf atama için geçerli bir hedef değil".to_string(),
                        node.token.clone(),
                    ));
                    Type::Error
                }
            },
            _ => {
                self.add_error(SemanticError::new(
                    SemanticErrorType::Other,
                    format!("Bilinmeyen operatör: {}", operator),
                    node.token.clone(),
                ));
                Type::Error
            }
        }
    }
    
    fn visit_unary_expr(&mut self, node: &AstNode) -> Type {
        if node.children.is_empty() {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "Eksik tekli ifade".to_string(),
                node.token.clone(),
            ));
            return Type::Error;
        }
        
        let expr_type = self.visit_node(&node.children[0]);
        if expr_type == Type::Error {
            return Type::Error;
        }
        
        let operator = node.value.as_ref().expect("Operatör bulunamadı");
        
        match operator.as_str() {
            "-" => {
                if expr_type == Type::Int || expr_type == Type::Float {
                    expr_type
                } else {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::TypeMismatch,
                        format!("Operatör '{}' tip {} için geçerli değil", operator, expr_type),
                        node.token.clone(),
                    ));
                    Type::Error
                }
            },
            "!" => {
                if expr_type == Type::Bool {
                    Type::Bool
                } else {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::TypeMismatch,
                        format!("Operatör '{}' tip {} için geçerli değil, bool bekleniyor", operator, expr_type),
                        node.token.clone(),
                    ));
                    Type::Error
                }
            },
            _ => {
                self.add_error(SemanticError::new(
                    SemanticErrorType::Other,
                    format!("Bilinmeyen tekli operatör: {}", operator),
                    node.token.clone(),
                ));
                Type::Error
            }
        }
    }
    
    fn visit_literal(&mut self, node: &AstNode) -> Type {
        if let Some(ref token) = node.token {
            match token.token_type {
                crate::lexer::token::TokenType::IntLiteral => Type::Int,
                crate::lexer::token::TokenType::FloatLiteral => Type::Float,
                crate::lexer::token::TokenType::StringLiteral => Type::String,
                crate::lexer::token::TokenType::BoolLiteral => Type::Bool,
                _ => {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::Other,
                        format!("Desteklenmeyen literal tipi: {:?}", token.token_type),
                        Some(token.clone()),
                    ));
                    Type::Error
                }
            }
        } else {
            let value = node.value.as_ref().expect("Literal değeri bulunamadı");
            
            if value.parse::<i32>().is_ok() {
                Type::Int
            } else if value.parse::<f64>().is_ok() {
                Type::Float
            } else if value == "true" || value == "false" {
                Type::Bool
            } else if value.starts_with('\"') && value.ends_with('\"') {
                Type::String
            } else {
                self.add_error(SemanticError::new(
                    SemanticErrorType::Other,
                    format!("Bilinmeyen literal: {}", value),
                    node.token.clone(),
                ));
                Type::Error
            }
        }
    }
    
    fn visit_identifier(&mut self, node: &AstNode) -> Type {
        let name = node.value.as_ref().expect("Tanımlayıcı adı bulunamadı");
        
        match self.symbol_table.resolve(name) {
            Ok(symbol) => {
                if let Err(err) = self.symbol_table.mark_used(name) {
                    self.add_error(err);
                }
                
                if symbol.kind == SymbolKind::Variable && !symbol.is_initialized {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::Other,
                        format!("'{}' değişkeni kullanımdan önce başlatılmamış", name),
                        node.token.clone(),
                    ));
                }
                
                symbol.symbol_type.clone()
            },
            Err(err) => {
                self.add_error(err);
                Type::Error
            }
        }
    }
    
    fn visit_call_expr(&mut self, node: &AstNode) -> Type {
        let func_name = node.value.as_ref().expect("Fonksiyon adı bulunamadı");
        
        match self.symbol_table.resolve(func_name) {
            Ok(symbol) => {
                if let Err(err) = self.symbol_table.mark_used(func_name) {
                    self.add_error(err);
                }
                
                if let Type::Function(param_types, return_type) = &symbol.symbol_type {
                    let mut arg_types = Vec::new();
                    for child in &node.children {
                        let arg_type = self.visit_node(child);
                        arg_types.push(arg_type);
                    }
                    
                    if arg_types.len() != param_types.len() {
                        self.add_error(SemanticError::new(
                            SemanticErrorType::Other,
                            format!("Fonksiyon '{}' {} argüman alır, {} verilmiş", 
                                   func_name, param_types.len(), arg_types.len()),
                            node.token.clone(),
                        ));
                        return (**return_type).clone();
                    }
                    
                    for (i, (arg_type, param_type)) in arg_types.iter().zip(param_types.iter()).enumerate() {
                        if arg_type != &Type::Error && param_type != &Type::Error {
                            if let Err(err) = param_type.can_assign_from(arg_type) {
                                self.add_error(SemanticError::new(
                                    SemanticErrorType::TypeMismatch,
                                    format!("Argüman {} için tip uyuşmazlığı: {}", i+1, err.message),
                                    node.children[i].token.clone(),
                                ));
                            }
                        }
                    }
                    
                    (**return_type).clone()
                } else {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::Other,
                        format!("'{}' bir fonksiyon değil", func_name),
                        node.token.clone(),
                    ));
                    Type::Error
                }
            },
            Err(err) => {
                self.add_error(err);
                Type::Error
            }
        }
    }
    
    fn visit_group_expr(&mut self, node: &AstNode) -> Type {
        if node.children.is_empty() {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "Boş grup ifadesi".to_string(),
                node.token.clone(),
            ));
            return Type::Error;
        }
        
        self.visit_node(&node.children[0])
    }
    
    fn visit_struct_declaration(&mut self, node: &AstNode) -> Type {
        let struct_name = node.value.as_ref().expect("Struct adı bulunamadı");
        
        let struct_type = Type::Struct(struct_name.clone());
        
        let line = node.token.as_ref().map_or(0, |t| t.line);
        let column = node.token.as_ref().map_or(0, |t| t.column);
        
        let struct_symbol = Symbol::new(
            struct_name.clone(),
            struct_type.clone(),
            SymbolKind::Type,
            false,
            self.symbol_table.current_level(),
            line,
            column,
        );
        
        if let Err(err) = self.symbol_table.define_symbol(struct_symbol) {
            self.add_error(err);
            return Type::Error;
        }
        
        self.symbol_table.enter_scope(ScopeType::Struct);
        
        for field in &node.children {
            if field.node_type == AstNodeType::VarDecl {
                self.visit_node(field);
            } else {
                self.add_error(SemanticError::new(
                    SemanticErrorType::Other,
                    format!("Struct içinde beklenmeyen düğüm tipi: {:?}", field.node_type),
                    field.token.clone(),
                ));
            }
        }
        
        self.symbol_table.exit_scope();
        
        struct_type
    }
    
    fn visit_impl_declaration(&mut self, node: &AstNode) -> Type {
        let struct_name = node.value.as_ref().expect("Struct adı bulunamadı");
        
        match self.symbol_table.resolve_type(struct_name) {
            Ok(_) => {
                self.symbol_table.enter_scope(ScopeType::Impl);
                
                for method in &node.children {
                    if method.node_type == AstNodeType::FuncDecl {
                        self.visit_node(method);
                    } else {
                        self.add_error(SemanticError::new(
                            SemanticErrorType::Other,
                            format!("Impl içinde beklenmeyen düğüm tipi: {:?}", method.node_type),
                            method.token.clone(),
                        ));
                    }
                }
                
                self.symbol_table.exit_scope();
                
                Type::Void
            },
            Err(err) => {
                self.add_error(SemanticError::new(
                    SemanticErrorType::Other,
                    format!("'{}' struct'ı tanımlı değil, impl yapılamaz", struct_name),
                    node.token.clone(),
                ));
                Type::Error
            }
        }
    }
    
    fn visit_member_expr(&mut self, node: &AstNode) -> Type {
        if node.children.len() < 1 {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "Üye erişimi ifadesi eksik".to_string(),
                node.token.clone(),
            ));
            return Type::Error;
        }
        
        let struct_expr_type = self.visit_node(&node.children[0]);
        
        let member_name = node.value.as_ref().expect("Üye adı bulunamadı");
        
        if let Type::Struct(struct_name) = struct_expr_type {
            Type::Int
        } else {
            self.add_error(SemanticError::new(
                SemanticErrorType::TypeMismatch,
                format!("'.' operatörü struct tipi beklerken '{}' tipi bulundu", struct_expr_type),
                node.token.clone(),
            ));
            Type::Error
        }
    }
    
    fn visit_index_expr(&mut self, node: &AstNode) -> Type {
        if node.children.len() < 2 {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "Dizin erişimi ifadesi eksik".to_string(),
                node.token.clone(),
            ));
            return Type::Error;
        }
        
        let array_expr_type = self.visit_node(&node.children[0]);
        
        let index_expr_type = self.visit_node(&node.children[1]);
        
        if index_expr_type != Type::Int && index_expr_type != Type::Error {
            self.add_error(SemanticError::new(
                SemanticErrorType::TypeMismatch,
                format!("Dizin ifadesi integer tipinde olmalı, bulunan: {}", index_expr_type),
                node.children[1].token.clone(),
            ));
        }
        
        if let Type::Array(elem_type, _) = array_expr_type {
            *elem_type
        } else if array_expr_type == Type::String {
            Type::String
        } else if array_expr_type != Type::Error {
            self.add_error(SemanticError::new(
                SemanticErrorType::TypeMismatch,
                format!("'[]' operatörü dizi veya string tipi beklerken '{}' tipi bulundu", array_expr_type),
                node.token.clone(),
            ));
            Type::Error
        } else {
            Type::Error
        }
    }
    
    fn visit_break_stmt(&mut self, node: &AstNode) -> Type {
        if !self.in_loop {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "Break ifadesi sadece döngü içinde kullanılabilir".to_string(),
                node.token.clone(),
            ));
        }
        
        Type::Void
    }
    
    fn visit_continue_stmt(&mut self, node: &AstNode) -> Type {
        if !self.in_loop {
            self.add_error(SemanticError::new(
                SemanticErrorType::Other,
                "Continue ifadesi sadece döngü içinde kullanılabilir".to_string(),
                node.token.clone(),
            ));
        }
        
        Type::Void
    }
    
    fn visit_module_declaration(&mut self, node: &AstNode) -> Type {
        let module_name = node.value.as_ref().expect("Modül adı bulunamadı");
        
        let module_type = Type::Module(module_name.clone());
        
        let line = node.token.as_ref().map_or(0, |t| t.line);
        let column = node.token.as_ref().map_or(0, |t| t.column);
        
        let module_symbol = Symbol::new(
            module_name.clone(),
            module_type.clone(),
            SymbolKind::Module,
            false,
            self.symbol_table.current_level(),
            line,
            column,
        );
        
        if let Err(err) = self.symbol_table.define_symbol(module_symbol) {
            self.add_error(err);
            return Type::Error;
        }
        
        self.symbol_table.enter_scope(ScopeType::Module);
        
        for item in &node.children {
            self.visit_node(item);
        }
        
        self.symbol_table.exit_scope();
        
        module_type
    }
    
    pub fn get_errors(&self) -> &Vec<SemanticError> {
        &self.errors
    }
    
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn reset(&mut self) {
        self.symbol_table = SymbolTable::new();
        self.current_function_return_type = None;
        self.errors.clear();
        self.in_loop = false;
    }
    
    pub fn load_std_library(&mut self) -> Result<(), SemanticError> {
        let println_params = vec![Symbol::new(
            "message".to_string(),
            Type::String,
            SymbolKind::Parameter,
            false,
            0,
            0,
            0,
        )];
        
        self.symbol_table.define_function(
            "println".to_string(),
            Type::Void,
            println_params,
            0,
            0,
        )?;
        
        let print_params = vec![Symbol::new(
            "message".to_string(),
            Type::String,
            SymbolKind::Parameter,
            false,
            0,
            0,
            0,
        )];
        
        self.symbol_table.define_function(
            "print".to_string(),
            Type::Void,
            print_params,
            0,
            0,
        )?;
        
        Ok(())
    }
    
    pub fn import_module(&mut self, module_name: &str, module_path: &Path) -> Result<(), SemanticError> {
        self.symbol_table.enter_scope(ScopeType::Module);
        self.symbol_table.exit_scope();
        
        Ok(())
    }
    
    pub fn analyze_with_reports(&mut self, ast: &AstNode) -> (bool, Vec<SemanticError>) {
        let errors = self.analyze(ast);
        let success = errors.iter().all(|e| e.is_warning());
        
        (success, errors)
    }

    pub fn evaluate_constant_expressions(&mut self, ast: &AstNode) -> Result<(), Vec<SemanticError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    
    pub fn infer_types(&mut self, _ast: &mut AstNode) -> Result<(), Vec<SemanticError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    
    pub fn load_std_modules(&mut self) -> Result<(), SemanticError> {
        let std_modules = ["io", "collections", "core", "parallel"];
        
        for module_name in std_modules.iter() {
            self.symbol_table.enter_scope(ScopeType::Module);
            
            match *module_name {
                "io" => self.load_io_module()?,
                "collections" => self.load_collections_module()?,
                "core" => self.load_core_module()?,
                "parallel" => self.load_parallel_module()?,
                _ => {}
            }
            
            self.symbol_table.exit_scope();
        }
        
        Ok(())
    }
    
    fn load_io_module(&mut self) -> Result<(), SemanticError> {
        let read_line_params = vec![];
        self.symbol_table.define_function(
            "read_line".to_string(),
            Type::String,
            read_line_params,
            0,
            0,
        )?;
        
        let read_int_params = vec![];
        self.symbol_table.define_function(
            "read_int".to_string(),
            Type::Int,
            read_int_params,
            0,
            0,
        )?;
        
        let file_type = Type::Struct("File".to_string());
        let file_symbol = Symbol::new(
            "File".to_string(),
            file_type.clone(),
            SymbolKind::Type,
            false,
            self.symbol_table.current_level(),
            0,
            0,
        );
        self.symbol_table.define_symbol(file_symbol)?;
        
        Ok(())
    }
    
    fn load_collections_module(&mut self) -> Result<(), SemanticError> {
        let vector_type = Type::Struct("Vector".to_string());
        let vector_symbol = Symbol::new(
            "Vector".to_string(),
            vector_type.clone(),
            SymbolKind::Type,
            false,
            self.symbol_table.current_level(),
            0,
            0,
        );
        self.symbol_table.define_symbol(vector_symbol)?;
        
        let map_type = Type::Struct("Map".to_string());
        let map_symbol = Symbol::new(
            "Map".to_string(),
            map_type.clone(),
            SymbolKind::Type,
            false,
            self.symbol_table.current_level(),
            0,
            0,
        );
        self.symbol_table.define_symbol(map_symbol)?;
        
        Ok(())
    }
    
    fn load_core_module(&mut self) -> Result<(), SemanticError> {
        let to_string_params = vec![Symbol::new(
            "value".to_string(),
            Type::Any,
            SymbolKind::Parameter,
            false,
            0,
            0,
            0,
        )];
        self.symbol_table.define_function(
            "to_string".to_string(),
            Type::String,
            to_string_params,
            0,
            0,
        )?;
        
        let parse_params = vec![Symbol::new(
            "str".to_string(),
            Type::String,
            SymbolKind::Parameter,
            false,
            0,
            0,
            0,
        )];
        self.symbol_table.define_function(
            "parse".to_string(),
            Type::Any,
            parse_params,
            0,
            0,
        )?;
        
        Ok(())
    }
    
    fn load_parallel_module(&mut self) -> Result<(), SemanticError> {
        let thread_type = Type::Struct("Thread".to_string());
        let thread_symbol = Symbol::new(
            "Thread".to_string(),
            thread_type.clone(),
            SymbolKind::Type,
            false,
            self.symbol_table.current_level(),
            0,
            0,
        );
        self.symbol_table.define_symbol(thread_symbol)?;
        
        let spawn_params = vec![Symbol::new(
            "func".to_string(),
            Type::Function(vec![], Box::new(Type::Void)),
            SymbolKind::Parameter,
            false,
            0,
            0,
            0,
        )];
        self.symbol_table.define_function(
            "spawn".to_string(),
            thread_type,
            spawn_params,
            0,
            0,
        )?;
        
        Ok(())
    }
    
    pub fn post_analysis_optimization(&mut self, ast: &mut AstNode) {
    }
    
    pub fn format_diagnostics(&self) -> String {
        let mut result = String::new();
        
        for (index, error) in self.errors.iter().enumerate() {
            let error_type = if error.is_warning() { "UYARI" } else { "HATA" };
            
            result.push_str(&format!(
                "[{}] {} (satır {}, sütun {}): {}\n",
                index + 1,
                error_type,
                error.line,
                error.column,
                error.message
            ));
        }
        
        result
    }
    
    pub fn dump_symbol_table(&self) -> String {
        format!("{:?}", self.symbol_table)
    }
    
    pub fn process_imports(&mut self, ast: &AstNode, module_paths: &[&Path]) -> Result<(), Vec<SemanticError>> {
        let mut import_errors = Vec::new();
        
        if import_errors.is_empty() {
            Ok(())
        } else {
            Err(import_errors)
        }
    }

    pub fn check_type_compatibility(&mut self, expr: &AstNode, expected_type: &Type, context: &str) -> Type {
        let expr_type = self.visit_node(expr);
        
        if !self.is_compatible(&expr_type, expected_type) {
            self.errors.push(SemanticError::new(
                SemanticErrorType::TypeMismatch,
                format!("{} için beklenmeyen tip: {} (beklenen: {})", context, expr_type, expected_type),
                expr.token.clone(),
            ));
        }
        
        expr_type
    }
    
    pub fn is_compatible(&self, actual: &Type, expected: &Type) -> bool {
        match (actual, expected) {
            (_, Type::Any) => true,
            (Type::Any, _) => true,
            
            (a, b) if a == b => true,
            
            (Type::Int, Type::Float) => true,
            (Type::Int, Type::Bool) => true,
            (Type::Float, Type::String) => true,
            (Type::Int, Type::String) => true,
            (Type::Bool, Type::String) => true,
            
            (Type::Null, Type::Optional(_)) => true,
            
            (t, Type::Optional(inner)) => self.is_compatible(t, inner),
            
            (Type::Array(a_elem, _), Type::Array(b_elem, _)) => self.is_compatible(a_elem, b_elem),
            
            (Type::Function(a_params, a_ret), Type::Function(b_params, b_ret)) => {
                if a_params.len() != b_params.len() {
                    return false;
                }
                
                for (a_param, b_param) in a_params.iter().zip(b_params.iter()) {
                    if !self.is_compatible(a_param, b_param) {
                        return false;
                    }
                }
                
                self.is_compatible(a_ret, b_ret)
            }
            
            _ => false,
        }
    }
    
    pub fn check_accessibility(&mut self, _symbol: &Symbol, _usage_scope: &str, _node: &AstNode) {
    }

    pub fn resolve_generic_type(&mut self, generic_type: &Type, concrete_types: &[Type]) -> Type {
        match generic_type {
            Type::TypeParameter(name, _) => {
                for (i, param_name) in self.symbol_table.generic_type_params().iter().enumerate() {
                    if param_name == name && i < concrete_types.len() {
                        return concrete_types[i].clone();
                    }
                }
                Type::Unknown
            },
            Type::Array(elem_type, size) => {
                let resolved_elem = self.resolve_generic_type(elem_type, concrete_types);
                Type::Array(Box::new(resolved_elem), size.clone())
            },
            Type::Optional(inner_type) => {
                let resolved_inner = self.resolve_generic_type(inner_type, concrete_types);
                Type::Optional(Box::new(resolved_inner))
            },
            Type::Function(params, ret) => {
                let resolved_params = params.iter()
                    .map(|p| self.resolve_generic_type(p, concrete_types))
                    .collect();
                let resolved_ret = self.resolve_generic_type(ret, concrete_types);
                Type::Function(resolved_params, Box::new(resolved_ret))
            },
            _ => generic_type.clone(),
        }
    }
    
    pub fn check_recursive_types(&mut self, type_name: &str, visited: &mut Vec<String>) -> bool {
        if visited.contains(&type_name.to_string()) {
            return true;
        }
        
        visited.push(type_name.to_string());
        
        if let Some(symbol) = self.symbol_table.lookup(type_name) {
            if let Type::Struct(struct_name) = &symbol.type_info() {
                if let Some(struct_def) = self.symbol_table.get_struct_def(struct_name) {
                    for field in &struct_def.fields {
                        if let Type::Struct(field_type_name) = &field.type_info {
                            if self.check_recursive_types(field_type_name, visited) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        visited.pop();
        false
    }
    
    pub fn check_memory_safety(&mut self, _ast: &AstNode) {
    }
    
    pub fn detect_performance_issues(&mut self, _ast: &AstNode) {
    }
    
    pub fn detect_unused_code(&mut self) {
        let mut unused_symbols = Vec::new();
        
        for symbol in self.symbol_table.all_symbols() {
            if symbol.is_local() && !symbol.is_referenced() && !symbol.is_parameter() {
                unused_symbols.push((symbol.name().clone(), symbol.line(), symbol.column()));
            }
        }
        
        for (name, line, column) in unused_symbols {
            self.errors.push(SemanticError::with_position(
                SemanticErrorType::Other,
                format!("Kullanılmayan değişken: {}", name),
                line,
                column,
            ));
        }
    }

    pub fn generate_optimization_hints(&self) -> Vec<String> {
        let mut hints = Vec::new();
        
        for expr in &self.constant_expressions {
            hints.push(format!("Sabit ifade: {} - Derleme zamanında hesaplanabilir", expr));
        }
        
        for loop_info in &self.loop_infos {
            if loop_info.is_small_constant_range() {
                hints.push(format!("Döngü açılabilir (loop unrolling): {}", loop_info.description));
            }
        }
        
        for func in &self.small_functions {
            hints.push(format!("İnline edilebilir fonksiyon: {}", func));
        }
        
        hints
    }

    pub fn process_modules(&mut self, node: &AstNode) {
        match &node.node_type {
            AstNodeType::ModDecl => {
                let module_name = node.value.as_ref().expect("Module name missing");
                
                if !self.module_exists(module_name) {
                    self.add_error(SemanticError::new(
                        SemanticErrorType::Other,
                        format!("Module not found: {}", module_name),
                        node.token.clone(),
                    ));
                }
            },
            _ => {
                for child in &node.children {
                    self.process_modules(child);
                }
            }
        }
    }
    
    pub fn process_operator_overloading(&mut self, node: &AstNode) {
    }
    
    pub fn process_attributes(&mut self, node: &AstNode) {
    }
    
    pub fn analyze_concurrency(&mut self, node: &AstNode) {
    }

    pub fn analyze_generics(&mut self, node: &AstNode) {
    }
    
    pub fn analyze_pattern_matching(&mut self, node: &AstNode) -> Type {
        Type::Void
    }

    pub fn infer_type(&mut self, node: &AstNode, context_type: Option<&Type>) -> Type {
        self.visit_node(node)
    }
    
    pub fn visit_expression_with_context(&mut self, node: &AstNode, context_type: Option<&Type>) -> Type {
        self.visit_node(node)
    }
    
    fn validate_generic_constraint(&mut self, type_arg: &str, type_params: &[String], node: &AstNode) {
    }

    fn get_literal_type(&self, value: &str) -> Type {
        if value.parse::<i32>().is_ok() {
            Type::Int
        } else if value.parse::<f64>().is_ok() {
            Type::Float
        } else if value == "true" || value == "false" {
            Type::Bool
        } else if value.starts_with('"') && value.ends_with('"') {
            Type::String
        } else {
            Type::Error
        }
    }
    
    pub fn visit_expression(&mut self, node: &AstNode) -> Type {
        match node.node_type {
            AstNodeType::BinaryExpr => self.visit_binary_expr(node),
            AstNodeType::UnaryExpr => self.visit_unary_expr(node),
            AstNodeType::LiteralExpr => self.visit_literal(node),
            AstNodeType::IdentifierExpr => self.visit_identifier(node),
            AstNodeType::CallExpr => self.visit_call_expr(node),
            AstNodeType::GroupExpr => self.visit_group_expr(node),
            AstNodeType::MemberExpr => self.visit_member_expr(node),
            AstNodeType::IndexExpr => self.visit_index_expr(node),
            _ => {
                self.add_error(SemanticError::new(
                    SemanticErrorType::Other,
                    format!("Beklenmeyen ifade tipi: {:?}", node.node_type),
                    node.token.clone(),
                ));
                Type::Error
            }
        }
    }

    fn visit(&mut self, node: &AstNode) -> Type {
        self.visit_node(node)
    }
    
    fn module_exists(&self, module_path: &str) -> bool {
        true
    }
    
    fn symbol_exists(&self, module_path: &str, symbol_name: &str) -> bool {
        true
    }
    
    fn import_symbol(&mut self, module_path: &str, symbol_name: &str) {
    }

    fn node_kind(&self, node: &AstNode) -> AstNodeType {
        node.node_type.clone()
    }
    
    fn node_line(&self, node: &AstNode) -> usize {
        node.token.as_ref().map_or(0, |t| t.line)
    }
    
    fn node_column(&self, node: &AstNode) -> usize {
        node.token.as_ref().map_or(0, |t| t.column)
    }
}

impl std::fmt::Debug for SemanticAnalyzer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticAnalyzer")
            .field("symbol_table", &self.symbol_table)
            .field("current_function_return_type", &self.current_function_return_type)
            .field("errors_count", &self.errors.len())
            .field("in_loop", &self.in_loop)
            .finish()
    }
}