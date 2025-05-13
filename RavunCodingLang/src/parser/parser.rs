use std::iter::Peekable;
use std::vec::IntoIter;
use crate::lexer::token::{Token, TokenType};
use crate::parser::ast::{AstNode, AstNodeType, BinaryOperator, UnaryOperator};

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    current_token: Option<Token>,
    errors: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Parser {
            tokens: tokens.into_iter().peekable(),
            current_token: None,
            errors: Vec::new(),
        };
        
        parser.advance();
        
        parser
    }
    
    fn advance(&mut self) {
        self.current_token = self.tokens.next();
    }
    
    fn check(&self, expected_type: TokenType) -> bool {
        if let Some(ref token) = self.current_token {
            token.token_type == expected_type
        } else {
            false
        }
    }
    
    fn consume(&mut self, expected_type: TokenType) -> Result<Token, String> {
        if let Some(ref token) = self.current_token {
            if token.token_type == expected_type {
                let token_clone = token.clone();
                self.advance();
                return Ok(token_clone);
            }
            
            Err(format!(
                "Beklenen token tipi: {:?}, bulunan: {:?} (satır: {}, sütun: {})",
                expected_type, token.token_type, token.line, token.column
            ))
        } else {
            Err(format!("Beklenen token tipi: {:?}, dosya sonu bulundu", expected_type))
        }
    }
    
    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }
    
    fn error(&mut self, message: String) {
        self.errors.push(message);
    }
    
    pub fn parse(&mut self) -> Result<AstNode, Vec<String>> {
        let program = self.parse_program();
        
        if !self.errors.is_empty() {
            Err(self.errors.clone())
        } else {
            Ok(program)
        }
    }
    
    fn parse_program(&mut self) -> AstNode {
        let mut program = AstNode::new(AstNodeType::Program, None);
        
        while let Some(_) = self.current_token {
            match self.parse_declaration() {
                Ok(declaration) => program.add_child(declaration),
                Err(err) => {
                    self.error(err);
                    self.synchronize();
                }
            }
        }
        
        program
    }
    
    fn synchronize(&mut self) {
        while let Some(ref token) = self.current_token {
            if token.token_type == TokenType::Semicolon {
                self.advance();
                return;
            }
            
            match token.token_type {
                TokenType::Let | TokenType::Fn | TokenType::For | 
                TokenType::If | TokenType::While | TokenType::Return |
                TokenType::Struct | TokenType::Impl | TokenType::Mod => {
                    return;
                }
                _ => {}
            }
            
            self.advance();
        }
    }
    
    fn parse_declaration(&mut self) -> Result<AstNode, String> {
        match self.current_token {
            Some(ref token) => match token.token_type {
                TokenType::Let => self.parse_var_declaration(),
                TokenType::Fn => self.parse_function_declaration(),
                TokenType::Struct => self.parse_struct_declaration(),
                TokenType::Impl => self.parse_impl_declaration(),
                TokenType::Mod => self.parse_module_declaration(),
                _ => self.parse_statement(),
            },
            None => Err("Beklenmeyen dosya sonu".to_string()),
        }
    }
    
    fn parse_var_declaration(&mut self) -> Result<AstNode, String> {
        let let_token = self.consume(TokenType::Let)?;
        
        let is_mutable = if self.check(TokenType::Mut) {
            self.advance();
            true
        } else {
            false
        };
        
        let identifier = self.consume(TokenType::Identifier)?;
        
        let type_annotation = if self.check(TokenType::Colon) {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        
        self.consume(TokenType::Assign)?;
        let initializer = self.parse_expression()?;
        
        self.consume(TokenType::Semicolon)?;
        
        let mut var_decl = AstNode::new(AstNodeType::VarDecl, Some(let_token));
        var_decl.set_value(identifier.lexeme);
        
        if is_mutable {
            var_decl.set_metadata("mutable".to_string());
        }
        
        if let Some(type_node) = type_annotation {
            var_decl.add_child(type_node);
        }
        
        var_decl.add_child(initializer);
        
        Ok(var_decl)
    }
    
    fn parse_function_declaration(&mut self) -> Result<AstNode, String> {
        let fn_token = self.consume(TokenType::Fn)?;
        
        let identifier = self.consume(TokenType::Identifier)?;
        
        self.consume(TokenType::LeftParen)?;
        let parameters = self.parse_parameters()?;
        self.consume(TokenType::RightParen)?;
        
        let return_type = if self.check(TokenType::Arrow) {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        
        let body = self.parse_block_statement()?;
        
        let mut func_decl = AstNode::new(AstNodeType::FuncDecl, Some(fn_token));
        func_decl.set_value(identifier.lexeme);
        
        for param in parameters {
            func_decl.add_child(param);
        }
        
        if let Some(type_node) = return_type {
            func_decl.add_child(type_node);
        }
        
        func_decl.add_child(body);
        
        Ok(func_decl)
    }
    
    fn parse_parameters(&mut self) -> Result<Vec<AstNode>, String> {
        let mut parameters = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume(TokenType::Identifier)?;
                
                self.consume(TokenType::Colon)?;
                let param_type = self.parse_type_annotation()?;
                
                let mut param = AstNode::new(AstNodeType::ParamDecl, Some(param_name.clone()));
                param.set_value(param_name.lexeme);
                param.add_child(param_type);
                
                parameters.push(param);
                
                if !self.check(TokenType::Comma) {
                    break;
                }
                
                self.advance();
            }
        }
        
        Ok(parameters)
    }
    
    fn parse_struct_declaration(&mut self) -> Result<AstNode, String> {
        let struct_token = self.consume(TokenType::Struct)?;
        
        let identifier = self.consume(TokenType::Identifier)?;
        
        self.consume(TokenType::LeftBrace)?;
        let fields = self.parse_struct_fields()?;
        self.consume(TokenType::RightBrace)?;
        
        let mut struct_decl = AstNode::new(AstNodeType::StructDecl, Some(struct_token));
        struct_decl.set_value(identifier.lexeme);
        
        for field in fields {
            struct_decl.add_child(field);
        }
        
        Ok(struct_decl)
    }
    
    fn parse_struct_fields(&mut self) -> Result<Vec<AstNode>, String> {
        let mut fields = Vec::new();
        
        if !self.check(TokenType::RightBrace) {
            loop {
                let field_name = self.consume(TokenType::Identifier)?;
                
                self.consume(TokenType::Colon)?;
                let field_type = self.parse_type_annotation()?;
                
                self.consume(TokenType::Comma)?;
                
                let mut field = AstNode::new(AstNodeType::VarDecl, Some(field_name.clone()));
                field.set_value(field_name.lexeme);
                field.add_child(field_type);
                
                fields.push(field);
                
                if self.check(TokenType::RightBrace) {
                    break;
                }
            }
        }
        
        Ok(fields)
    }
    
    fn parse_impl_declaration(&mut self) -> Result<AstNode, String> {
        let impl_token = self.consume(TokenType::Impl)?;
        
        let identifier = self.consume(TokenType::Identifier)?;
        
        self.consume(TokenType::LeftBrace)?;
        let methods = self.parse_impl_methods()?;
        self.consume(TokenType::RightBrace)?;
        
        let mut impl_decl = AstNode::new(AstNodeType::ImplDecl, Some(impl_token));
        impl_decl.set_value(identifier.lexeme);
        
        for method in methods {
            impl_decl.add_child(method);
        }
        
        Ok(impl_decl)
    }
    
    fn parse_impl_methods(&mut self) -> Result<Vec<AstNode>, String> {
        let mut methods = Vec::new();
        
        while !self.check(TokenType::RightBrace) {
            let method = self.parse_function_declaration()?;
            methods.push(method);
        }
        
        Ok(methods)
    }
    
    fn parse_module_declaration(&mut self) -> Result<AstNode, String> {
        let mod_token = self.consume(TokenType::Mod)?;
        
        let identifier = self.consume(TokenType::Identifier)?;
        
        self.consume(TokenType::LeftBrace)?;
        let declarations = self.parse_module_declarations()?;
        self.consume(TokenType::RightBrace)?;
        
        let mut mod_decl = AstNode::new(AstNodeType::ModDecl, Some(mod_token));
        mod_decl.set_value(identifier.lexeme);
        
        for decl in declarations {
            mod_decl.add_child(decl);
        }
        
        Ok(mod_decl)
    }
    
    fn parse_module_declarations(&mut self) -> Result<Vec<AstNode>, String> {
        let mut declarations = Vec::new();
        
        while !self.check(TokenType::RightBrace) {
            let declaration = self.parse_declaration()?;
            declarations.push(declaration);
        }
        
        Ok(declarations)
    }
    
    fn parse_type_annotation(&mut self) -> Result<AstNode, String> {
        let type_name = self.consume(TokenType::Identifier)?;
        
        let mut type_node = AstNode::new(AstNodeType::TypeAnnotation, Some(type_name.clone()));
        type_node.set_value(type_name.lexeme);
        
        Ok(type_node)
    }
    
    fn parse_statement(&mut self) -> Result<AstNode, String> {
        match self.current_token {
            Some(ref token) => match token.token_type {
                TokenType::If => self.parse_if_statement(),
                TokenType::While => self.parse_while_statement(),
                TokenType::For => self.parse_for_statement(),
                TokenType::Return => self.parse_return_statement(),
                TokenType::LeftBrace => self.parse_block_statement(),
                _ => self.parse_expression_statement(),
            },
            None => Err("Beklenmeyen dosya sonu".to_string()),
        }
    }
    
    fn parse_if_statement(&mut self) -> Result<AstNode, String> {
        let if_token = self.consume(TokenType::If)?;
        
        let condition = self.parse_expression()?;
        
        let then_branch = self.parse_block_statement()?;
        
        let else_branch = if self.check(TokenType::Else) {
            self.advance();
            
            if self.check(TokenType::If) {
                Some(self.parse_if_statement()?)
            } else {
                Some(self.parse_block_statement()?)
            }
        } else {
            None
        };
        
        let mut if_stmt = AstNode::new(AstNodeType::IfStmt, Some(if_token));
        
        if_stmt.add_child(condition);
        if_stmt.add_child(then_branch);
        
        if let Some(else_node) = else_branch {
            if_stmt.add_child(else_node);
        }
        
        Ok(if_stmt)
    }
    
    fn parse_while_statement(&mut self) -> Result<AstNode, String> {
        let while_token = self.consume(TokenType::While)?;
        
        let condition = self.parse_expression()?;
        
        let body = self.parse_block_statement()?;
        
        let mut while_stmt = AstNode::new(AstNodeType::WhileStmt, Some(while_token));
        
        while_stmt.add_child(condition);
        while_stmt.add_child(body);
        
        Ok(while_stmt)
    }
    
    fn parse_for_statement(&mut self) -> Result<AstNode, String> {
        let for_token = self.consume(TokenType::For)?;
        
        let variable = self.consume(TokenType::Identifier)?;
        
        self.consume(TokenType::In)?;
        
        let range = self.parse_expression()?;
        
        let body = self.parse_block_statement()?;
        
        let mut for_stmt = AstNode::new(AstNodeType::ForStmt, Some(for_token));
        
        let mut var_node = AstNode::new(AstNodeType::IdentifierExpr, Some(variable.clone()));
        var_node.set_value(variable.lexeme);
        
        for_stmt.add_child(var_node);
        for_stmt.add_child(range);
        for_stmt.add_child(body);
        
        Ok(for_stmt)
    }
    
    fn parse_return_statement(&mut self) -> Result<AstNode, String> {
        let return_token = self.consume(TokenType::Return)?;
        
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume(TokenType::Semicolon)?;
        
        let mut return_stmt = AstNode::new(AstNodeType::ReturnStmt, Some(return_token));
        
        if let Some(expr) = value {
            return_stmt.add_child(expr);
        }
        
        Ok(return_stmt)
    }
    
    fn parse_block_statement(&mut self) -> Result<AstNode, String> {
        let brace_token = self.consume(TokenType::LeftBrace)?;
        
        let mut statements = Vec::new();
        
        while !self.check(TokenType::RightBrace) && self.current_token.is_some() {
            let statement = self.parse_declaration()?;
            statements.push(statement);
        }
        
        self.consume(TokenType::RightBrace)?;
        
        let mut block = AstNode::new(AstNodeType::BlockStmt, Some(brace_token));
        
        for stmt in statements {
            block.add_child(stmt);
        }
        
        Ok(block)
    }
    
    fn parse_expression_statement(&mut self) -> Result<AstNode, String> {
        let expression = self.parse_expression()?;
        
        self.consume(TokenType::Semicolon)?;
        
        let mut expr_stmt = AstNode::new(AstNodeType::ExprStmt, None);
        expr_stmt.add_child(expression);
        
        Ok(expr_stmt)
    }
    
    fn parse_expression(&mut self) -> Result<AstNode, String> {
        self.parse_assignment()
    }
    
    fn parse_assignment(&mut self) -> Result<AstNode, String> {
        let expr = self.parse_equality()?;
        
        if let Some(ref token) = self.current_token {
            match token.token_type {
                TokenType::Assign | TokenType::PlusAssign | 
                TokenType::MinusAssign | TokenType::MulAssign | 
                TokenType::DivAssign => {
                    let operator = token.clone();
                    self.advance();
                    
                    let value = self.parse_assignment()?;
                    
                    match expr.node_type {
                        AstNodeType::IdentifierExpr | AstNodeType::MemberExpr | AstNodeType::IndexExpr => {
                            let mut assign_expr = AstNode::new(AstNodeType::BinaryExpr, Some(operator.clone()));
                            assign_expr.set_value(match operator.token_type {
                                TokenType::Assign => "=",
                                TokenType::PlusAssign => "+=",
                                TokenType::MinusAssign => "-=",
                                TokenType::MulAssign => "*=",
                                TokenType::DivAssign => "/=",
                                _ => "?=",
                            }.to_string());
                            
                            assign_expr.add_child(expr);
                            assign_expr.add_child(value);
                            
                            return Ok(assign_expr);
                        },
                        _ => {
                            return Err(format!("Geçersiz atama hedefi, satır: {}, sütun: {}", 
                                              operator.line, operator.column));
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<AstNode, String> {
        let mut expr = self.parse_comparison()?;
        
        while let Some(ref token) = self.current_token {
            match token.token_type {
                TokenType::Equal | TokenType::NotEqual => {
                    let operator = token.clone();
                    self.advance();
                    
                    let right = self.parse_comparison()?;
                    
                    let mut binary_expr = AstNode::new(AstNodeType::BinaryExpr, Some(operator.clone()));
                    binary_expr.set_value(match operator.token_type {
                        TokenType::Equal => "==",
                        TokenType::NotEqual => "!=",
                        _ => "?",
                    }.to_string());
                    
                    binary_expr.add_child(expr);
                    binary_expr.add_child(right);
                    
                    expr = binary_expr;
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> Result<AstNode, String> {
        let mut expr = self.parse_term()?;
        
        while let Some(ref token) = self.current_token {
            match token.token_type {
                TokenType::Greater | TokenType::GreaterEq | 
                TokenType::Less | TokenType::LessEq => {
                    let operator = token.clone();
                    self.advance();
                    
                    let right = self.parse_term()?;
                    
                    let mut binary_expr = AstNode::new(AstNodeType::BinaryExpr, Some(operator.clone()));
                    binary_expr.set_value(match operator.token_type {
                        TokenType::Greater => ">",
                        TokenType::GreaterEq => ">=",
                        TokenType::Less => "<",
                        TokenType::LessEq => "<=",
                        _ => "?",
                    }.to_string());
                    
                    binary_expr.add_child(expr);
                    binary_expr.add_child(right);
                    
                    expr = binary_expr;
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_term(&mut self) -> Result<AstNode, String> {
        let mut expr = self.parse_factor()?;
        
        while let Some(ref token) = self.current_token {
            match token.token_type {
                TokenType::Plus | TokenType::Minus => {
                    let operator = token.clone();
                    self.advance();
                    
                    let right = self.parse_factor()?;
                    
                    let mut binary_expr = AstNode::new(AstNodeType::BinaryExpr, Some(operator.clone()));
                    binary_expr.set_value(match operator.token_type {
                        TokenType::Plus => "+",
                        TokenType::Minus => "-",
                        _ => "?",
                    }.to_string());
                    
                    binary_expr.add_child(expr);
                    binary_expr.add_child(right);
                    
                    expr = binary_expr;
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_factor(&mut self) -> Result<AstNode, String> {
        let mut expr = self.parse_unary()?;
        
        while let Some(ref token) = self.current_token {
            match token.token_type {
                TokenType::Asterisk | TokenType::Slash | TokenType::Percent => {
                    let operator = token.clone();
                    self.advance();
                    
                    let right = self.parse_unary()?;
                    
                    let mut binary_expr = AstNode::new(AstNodeType::BinaryExpr, Some(operator.clone()));
                    binary_expr.set_value(match operator.token_type {
                        TokenType::Asterisk => "*",
                        TokenType::Slash => "/",
                        TokenType::Percent => "%",
                        _ => "?",
                    }.to_string());
                    
                    binary_expr.add_child(expr);
                    binary_expr.add_child(right);
                    
                    expr = binary_expr;
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_power(&mut self) -> Result<AstNode, String> {
        let mut expr = self.parse_primary()?;
        
        while let Some(ref token) = self.current_token {
            if token.token_type == TokenType::Caret {
                let operator = token.clone();
                self.advance();
                
                let right = self.parse_primary()?;
                
                let mut binary_expr = AstNode::new(AstNodeType::BinaryExpr, Some(operator.clone()));
                binary_expr.set_value("^".to_string());
                
                binary_expr.add_child(expr);
                binary_expr.add_child(right);
                
                expr = binary_expr;
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> Result<AstNode, String> {
        if let Some(ref token) = self.current_token {
            match token.token_type {
                TokenType::Minus => {
                    let operator = token.clone();
                    self.advance();
                    
                    let right = self.parse_unary()?;
                    
                    let mut unary_expr = AstNode::new(AstNodeType::UnaryExpr, Some(operator.clone()));
                    unary_expr.set_value("-".to_string());
                    unary_expr.add_child(right);
                    
                    return Ok(unary_expr);
                }
                _ => {}
            }
        }
        
        self.parse_power()
    }
    
    fn parse_primary(&mut self) -> Result<AstNode, String> {
        if let Some(ref token) = self.current_token {
            match token.token_type {
                TokenType::IntLiteral | TokenType::FloatLiteral | 
                TokenType::StringLiteral | TokenType::BoolLiteral => {
                    let literal_token = token.clone();
                    self.advance();
                    
                    let mut literal = AstNode::new(AstNodeType::LiteralExpr, Some(literal_token.clone()));
                    literal.set_value(literal_token.lexeme);
                    
                    Ok(literal)
                }
                
                TokenType::Identifier => {
                    let identifier = token.clone();
                    self.advance();
                    
                    if self.check(TokenType::LeftParen) {
                        self.parse_call_expr(identifier)
                    } else {
                        let mut ident_expr = AstNode::new(AstNodeType::IdentifierExpr, Some(identifier.clone()));
                        ident_expr.set_value(identifier.lexeme);
                        
                        Ok(ident_expr)
                    }
                }
                
                TokenType::LeftParen => {
                    self.advance();
                    
                    let expr = self.parse_expression()?;
                    
                    self.consume(TokenType::RightParen)?;
                    
                    let mut group_expr = AstNode::new(AstNodeType::GroupExpr, None);
                    group_expr.add_child(expr);
                    
                    Ok(group_expr)
                }
                
                _ => Err(format!("Beklenmeyen token: {:?}, satır: {}, sütun: {}", 
                                token.token_type, token.line, token.column))
            }
        } else {
            Err("Beklenmeyen dosya sonu".to_string())
        }
    }
    
    fn parse_call_expr(&mut self, identifier: Token) -> Result<AstNode, String> {
        self.consume(TokenType::LeftParen)?;
        
        let mut arguments = Vec::new();
        
        if !self.check(TokenType::RightParen) {
            loop {
                let arg = self.parse_expression()?;
                arguments.push(arg);
                
                if !self.check(TokenType::Comma) {
                    break;
                }
                
                self.advance();
            }
        }
        
        self.consume(TokenType::RightParen)?;
        
        let mut call_expr = AstNode::new(AstNodeType::CallExpr, Some(identifier.clone()));
        call_expr.set_value(identifier.lexeme);
        
        for arg in arguments {
            call_expr.add_child(arg);
        }
        
        Ok(call_expr)
    }
}
