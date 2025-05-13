use std::collections::HashMap;
use std::fmt;
use crate::semantics::error::{SemanticError, SemanticErrorType};
use crate::semantics::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Type,
    Module,
    TypeParameter,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Variable => write!(f, "değişken"),
            SymbolKind::Function => write!(f, "fonksiyon"),
            SymbolKind::Parameter => write!(f, "parametre"),
            SymbolKind::Type => write!(f, "tür"),
            SymbolKind::Module => write!(f, "modül"),
            SymbolKind::TypeParameter => write!(f, "tip parametresi"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: Type,
    pub kind: SymbolKind,
    pub is_mutable: bool,
    pub scope_level: usize,
    pub line: usize,
    pub column: usize,
    pub is_used: bool,
    pub is_initialized: bool,
    pub parameters: Option<Vec<Symbol>>,
}

impl Symbol {
    pub fn new(
        name: String, 
        symbol_type: Type, 
        kind: SymbolKind,
        is_mutable: bool, 
        scope_level: usize,
        line: usize, 
        column: usize
    ) -> Self {
        Symbol {
            name,
            symbol_type,
            kind,
            is_mutable,
            scope_level,
            line,
            column,
            is_used: false,
            is_initialized: false,
            parameters: None,
        }
    }
    
    pub fn new_variable(
        name: String, 
        symbol_type: Type, 
        is_mutable: bool, 
        scope_level: usize,
        line: usize, 
        column: usize,
        is_initialized: bool
    ) -> Self {
        let mut symbol = Symbol::new(name, symbol_type, SymbolKind::Variable, is_mutable, scope_level, line, column);
        symbol.is_initialized = is_initialized;
        symbol
    }
    
    pub fn new_function(
        name: String, 
        return_type: Type, 
        parameters: Vec<Symbol>,
        scope_level: usize,
        line: usize, 
        column: usize
    ) -> Self {
        let param_types = parameters.iter().map(|p| p.symbol_type.clone()).collect();
        let func_type = Type::Function(param_types, Box::new(return_type));
        
        let mut symbol = Symbol::new(name, func_type, SymbolKind::Function, false, scope_level, line, column);
        symbol.parameters = Some(parameters);
        symbol.is_initialized = true;
        symbol
    }

    pub fn type_info(&self) -> &Type {
        &self.symbol_type
    }
    
    pub fn is_local(&self) -> bool {
        self.kind == SymbolKind::Variable && self.scope_level > 0
    }
    
    pub fn is_referenced(&self) -> bool {
        self.is_used
    }
    
    pub fn is_parameter(&self) -> bool {
        self.kind == SymbolKind::Parameter
    }
    
    pub fn name(&self) -> &String {
        &self.name
    }
    
    pub fn line(&self) -> usize {
        self.line
    }
    
    pub fn column(&self) -> usize {
        self.column
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} (tip: {}, kapsam: {})", self.kind, self.name, self.symbol_type, self.scope_level)?;
        
        if self.is_mutable {
            write!(f, ", değiştirilebilir")?;
        }
        
        if !self.is_initialized {
            write!(f, ", başlatılmamış")?;
        }
        
        if !self.is_used {
            write!(f, ", kullanılmıyor")?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub level: usize,
    symbols: HashMap<String, Symbol>,
    pub scope_type: ScopeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    Global,
    Function,
    Block,
    Loop,
    If,
    Struct,
    Impl,
    Module,
}

impl fmt::Display for ScopeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScopeType::Global => write!(f, "global"),
            ScopeType::Function => write!(f, "fonksiyon"),
            ScopeType::Block => write!(f, "blok"),
            ScopeType::Loop => write!(f, "döngü"),
            ScopeType::If => write!(f, "if"),
            ScopeType::Struct => write!(f, "struct"),
            ScopeType::Impl => write!(f, "impl"),
            ScopeType::Module => write!(f, "modül"),
        }
    }
}

impl Scope {
    fn new(level: usize, scope_type: ScopeType) -> Self {
        Scope {
            level,
            symbols: HashMap::new(),
            scope_type,
        }
    }
    
    fn define(&mut self, symbol: Symbol) -> Result<(), SemanticError> {
        let name = symbol.name.clone();
        
        if self.symbols.contains_key(&name) {
            let existing = self.symbols.get(&name).unwrap();
            return Err(SemanticError::with_position(
                SemanticErrorType::Redefinition,
                format!("'{}' daha önce {} olarak tanımlanmış (satır: {}, sütun: {})", 
                       name, existing.kind, existing.line, existing.column),
                symbol.line,
                symbol.column,
            ));
        }
        
        self.symbols.insert(name, symbol);
        Ok(())
    }
    
    fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
    
    pub fn display_symbols(&self) -> String {
        let mut result = format!("Kapsam {} ({}):\n", self.level, self.scope_type);
        
        for (_, symbol) in &self.symbols {
            result.push_str(&format!("  {}\n", symbol));
        }
        
        result
    }
    
    pub fn get_all_symbols(&self) -> Vec<&Symbol> {
        self.symbols.values().collect()
    }
    
    pub fn get_symbols_by_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.symbols.values().filter(|s| s.kind == kind).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_function: Option<String>,
    current_struct: Option<String>,
    generic_params: Vec<String>,
    generic_definitions: HashMap<String, GenericDefinition>,
    generic_constraints: Vec<(String, Vec<GenericConstraint>)>,
    trait_implementations: HashMap<String, Vec<String>>,
    generic_instantiations: HashMap<String, (String, Vec<Type>)>,
    struct_definitions: HashMap<String, StructDefinition>,
    enum_definitions: HashMap<String, EnumDefinition>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = SymbolTable {
            scopes: Vec::new(),
            current_function: None,
            current_struct: None,
            generic_params: Vec::new(),
            generic_definitions: HashMap::new(),
            generic_constraints: Vec::new(),
            trait_implementations: HashMap::new(),
            generic_instantiations: HashMap::new(),
            struct_definitions: HashMap::new(),
            enum_definitions: HashMap::new(),
        };
        table.enter_scope(ScopeType::Global);
        table
    }
    
    pub fn enter_scope(&mut self, scope_type: ScopeType) {
        let level = self.scopes.len();
        self.scopes.push(Scope::new(level, scope_type));
        
        if scope_type == ScopeType::Function {
            if let Some(func_scope) = self.scopes.last() {
                for (name, symbol) in &func_scope.symbols {
                    if symbol.kind == SymbolKind::Function {
                        self.current_function = Some(name.clone());
                        break;
                    }
                }
            }
        } else if scope_type == ScopeType::Struct {
            if let Some(struct_scope) = self.scopes.last() {
                for (name, symbol) in &struct_scope.symbols {
                    if symbol.kind == SymbolKind::Type {
                        self.current_struct = Some(name.clone());
                        break;
                    }
                }
            }
        }
    }
    
    pub fn exit_scope(&mut self) -> Option<Scope> {
        if self.scopes.len() > 1 {
            if let Some(scope) = self.scopes.last() {
                if scope.scope_type == ScopeType::Function {
                    self.current_function = None;
                } else if scope.scope_type == ScopeType::Struct {
                    self.current_struct = None;
                }
            }
            
            Some(self.scopes.pop().unwrap())
        } else {
            println!("Uyarı: Evrensel kapsamdan çıkma girişimi");
            None
        }
    }
    
    pub fn current_scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }
    
    pub fn current_scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }
    
    pub fn current_level(&self) -> usize {
        self.scopes.len() - 1
    }
    
    pub fn current_function_name(&self) -> Option<&String> {
        self.current_function.as_ref()
    }
    
    pub fn current_struct_name(&self) -> Option<&String> {
        self.current_struct.as_ref()
    }
    
    pub fn define_symbol(&mut self, symbol: Symbol) -> Result<(), SemanticError> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.define(symbol)
        } else {
            Err(SemanticError::new(
                SemanticErrorType::Other,
                "Kapsam bulunamadı".to_string(),
                None,
            ))
        }
    }
    
    pub fn define_var(&mut self, name: &str, symbol_type: Type, line: usize, column: usize) -> Result<(), SemanticError> {
        let symbol = Symbol::new(
            name.to_string(),
            symbol_type,
            SymbolKind::Variable,
            false,
            self.current_level(),
            line,
            column,
        );
        self.define_symbol(symbol)
    }
    
    pub fn define_variable(
        &mut self, 
        name: String, 
        var_type: Type, 
        is_mutable: bool,
        is_initialized: bool,
        line: usize, 
        column: usize
    ) -> Result<(), SemanticError> {
        let level = self.current_level();
        let symbol = Symbol::new_variable(name, var_type, is_mutable, level, line, column, is_initialized);
        self.define_symbol(symbol)
    }
    
    pub fn define_function(
        &mut self, 
        name: String, 
        return_type: Type, 
        parameters: Vec<Symbol>,
        line: usize, 
        column: usize
    ) -> Result<(), SemanticError> {
        let level = self.current_level();
        let symbol = Symbol::new_function(name, return_type, parameters, level, line, column);
        self.define_symbol(symbol)
    }
    
    pub fn resolve(&self, name: &str) -> Result<&Symbol, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.resolve(name) {
                return Ok(symbol);
            }
        }
        
        Err(SemanticError::new(
            SemanticErrorType::UndefinedVariable,
            format!("'{}' tanımlı değil", name),
            None,
        ))
    }
    
    pub fn lookup(&self, name: &str, max_level: Option<usize>) -> Result<&Symbol, SemanticError> {
        let max = max_level.unwrap_or(self.scopes.len());
        
        for scope in self.scopes.iter().rev() {
            if scope.level > max {
                continue;
            }
            
            if let Some(symbol) = scope.resolve(name) {
                return Ok(symbol);
            }
        }
        
        Err(SemanticError::new(
            SemanticErrorType::UndefinedVariable,
            format!("'{}' tanımlı değil", name),
            None,
        ))
    }
    
    pub fn resolve_local(&self, name: &str) -> Result<&Symbol, SemanticError> {
        if let Some(scope) = self.scopes.last() {
            if let Some(symbol) = scope.resolve(name) {
                return Ok(symbol);
            }
        }
        
        Err(SemanticError::new(
            SemanticErrorType::UndefinedVariable,
            format!("'{}' mevcut kapsamda tanımlı değil", name),
            None,
        ))
    }
    
    pub fn mark_used(&mut self, name: &str) -> Result<(), SemanticError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.symbols.get_mut(name) {
                symbol.is_used = true;
                return Ok(());
            }
        }
        
        Err(SemanticError::new(
            SemanticErrorType::UndefinedVariable,
            format!("'{}' tanımlı değil, kullanılamaz", name),
            None,
        ))
    }
    
    pub fn mark_initialized(&mut self, name: &str) -> Result<(), SemanticError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.symbols.get_mut(name) {
                symbol.is_initialized = true;
                return Ok(());
            }
        }
        
        Err(SemanticError::new(
            SemanticErrorType::UndefinedVariable,
            format!("'{}' tanımlı değil, değer atanamaz", name),
            None,
        ))
    }
    
    pub fn check_assignable(&self, name: &str) -> Result<(), SemanticError> {
        let symbol = self.resolve(name)?;
        
        if !symbol.is_mutable {
            return Err(SemanticError::new(
                SemanticErrorType::Other,
                format!("'{}' değiştirilemez (mut değil)", name),
                None,
            ));
        }
        
        Ok(())
    }
    
    pub fn resolve_function(&self, name: &str) -> Result<&Symbol, SemanticError> {
        let symbol = self.resolve(name)?;
        
        match symbol.kind {
            SymbolKind::Function => Ok(symbol),
            _ => Err(SemanticError::new(
                SemanticErrorType::UndefinedFunction,
                format!("'{}' bir fonksiyon değil, {} türünde", name, symbol.kind),
                None,
            )),
        }
    }
    
    pub fn resolve_type(&self, name: &str) -> Result<&Symbol, SemanticError> {
        let symbol = self.resolve(name)?;
        
        match symbol.kind {
            SymbolKind::Type => Ok(symbol),
            _ => Err(SemanticError::new(
                SemanticErrorType::Other,
                format!("'{}' bir tür değil, {} türünde", name, symbol.kind),
                None,
            )),
        }
    }
    
    pub fn display_scopes(&self) -> String {
        let mut result = String::new();
        
        for (i, scope) in self.scopes.iter().enumerate() {
            result.push_str(&format!("\n=== Kapsam {} ({}) ===\n", i, scope.scope_type));
            
            for (name, symbol) in &scope.symbols {
                result.push_str(&format!("  {} : {}\n", name, symbol.symbol_type));
                if symbol.kind == SymbolKind::Function && symbol.parameters.is_some() {
                    result.push_str("    Parametreler:\n");
                    for param in symbol.parameters.as_ref().unwrap() {
                        result.push_str(&format!("      {} : {}\n", param.name, param.symbol_type));
                    }
                }
            }
        }
        
        result
    }
    
    pub fn get_unused_symbols(&self) -> Vec<&Symbol> {
        let mut unused = Vec::new();
        
        for scope in &self.scopes {
            for symbol in scope.symbols.values() {
                if !symbol.is_used && symbol.kind != SymbolKind::Function {
                    unused.push(symbol);
                }
            }
        }
        
        unused
    }
    
    pub fn get_uninitialized_symbols(&self) -> Vec<&Symbol> {
        let mut uninitialized = Vec::new();
        
        for scope in &self.scopes {
            for symbol in scope.symbols.values() {
                if !symbol.is_initialized && symbol.kind == SymbolKind::Variable {
                    uninitialized.push(symbol);
                }
            }
        }
        
        uninitialized
    }
    
    pub fn generic_type_params(&self) -> Vec<String> {
        self.generic_params.clone()
    }
    
    pub fn get_generic_type_params(&self, generic_name: &str) -> Vec<String> {
        self.generic_definitions.get(generic_name)
            .map(|gen_def| gen_def.type_params.clone())
            .unwrap_or_default()
    }
    
    pub fn get_generic_constraints(&self) -> Vec<(String, Vec<GenericConstraint>)> {
        self.generic_constraints.clone()
    }
    
    pub fn implements_trait(&self, type_name: &str, trait_name: &str) -> bool {
        if let Some(trait_impls) = self.trait_implementations.get(trait_name) {
            trait_impls.contains(&type_name.to_string())
        } else {
            false
        }
    }
    
    pub fn register_instantiated_generic(&mut self, 
                                        instantiated_name: &str, 
                                        base_type: &str, 
                                        concrete_types: Vec<Type>,
                                        line: usize,
                                        column: usize) -> Result<(), SemanticError> {
        let symbol = Symbol::new(
            instantiated_name.to_string(),
            Type::Struct(instantiated_name.to_string()),
            SymbolKind::Type,
            false,
            self.current_level(),
            line,
            column,
        );
        
        self.define_symbol(symbol)?;
        
        self.generic_instantiations.insert(
            instantiated_name.to_string(),
            (base_type.to_string(), concrete_types)
        );
        
        Ok(())
    }
    
    pub fn define_generic(&mut self, 
                         name: &str, 
                         type_params: Vec<String>,
                         line: usize,
                         column: usize) -> Result<(), SemanticError> {
        let symbol = Symbol::new(
            name.to_string(),
            Type::Struct(name.to_string()),
            SymbolKind::Type,
            false,
            self.current_level(),
            line,
            column,
        );
        
        self.define_symbol(symbol)?;
        
        self.generic_definitions.insert(
            name.to_string(),
            GenericDefinition {
                name: name.to_string(),
                type_params,
            }
        );
        
        Ok(())
    }
    
    pub fn define_type_parameter(&mut self, 
                                name: &str, 
                                param_type: Type,
                                line: usize,
                                column: usize) -> Result<(), SemanticError> {
        let symbol = Symbol::new(
            name.to_string(),
            param_type,
            SymbolKind::TypeParameter,
            false,
            self.current_level(),
            line, 
            column,
        );
        
        self.define_symbol(symbol)?;
        
        self.generic_params.push(name.to_string());
        
        Ok(())
    }
    
    pub fn get_field_type(&self, struct_name: &str, field_name: &str) -> Option<Type> {
        if let Some(struct_def) = self.struct_definitions.get(struct_name) {
            for field in &struct_def.fields {
                if field.name == field_name {
                    return Some(field.type_info.clone());
                }
            }
        }
        None
    }
    
    pub fn get_enum_variants(&self, enum_name: &str) -> Vec<String> {
        if let Some(enum_def) = self.enum_definitions.get(enum_name) {
            enum_def.variants.iter().map(|v| v.name.clone()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn type_exists(&self, type_name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            for (name, symbol) in &scope.symbols {
                if name == type_name && symbol.kind == SymbolKind::Type {
                    return true;
                }
            }
        }
        false
    }
    
    pub fn contains(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.symbols.contains_key(name) {
                return true;
            }
        }
        false
    }
    
    pub fn mark_as_exported(&mut self, name: &str) -> Result<(), SemanticError> {
        Ok(())
    }
    
    pub fn mark_as_deprecated(&mut self, name: &str, message: String) -> Result<(), SemanticError> {
        Ok(())
    }
    
    pub fn define_extern(&mut self, 
                        name: &str, 
                        func_type: Type, 
                        external_name: Option<String>,
                        line: usize, 
                        column: usize) -> Result<(), SemanticError> {
        let mut symbol = Symbol::new(
            name.to_string(),
            func_type,
            SymbolKind::Function,
            false,
            self.current_level(),
            line,
            column,
        );
        
        symbol.is_initialized = true;
        
        if let Some(ext_name) = external_name {
        }
        
        self.define_symbol(symbol)
    }

    pub fn get_struct_def(&self, struct_name: &str) -> Option<&StructDefinition> {
        self.struct_definitions.get(struct_name)
    }
    
    pub fn all_symbols(&self) -> Vec<&Symbol> {
        let mut all_symbols = Vec::new();
        
        for scope in &self.scopes {
            for symbol in scope.symbols.values() {
                all_symbols.push(symbol);
            }
        }
        
        all_symbols
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.resolve(name) {
                return Some(symbol);
            }
        }
        
        None
    }
}

#[derive(Debug, Clone)]
pub struct GenericDefinition {
    pub name: String,
    pub type_params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_info: Type,
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub types: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericConstraint {
    Implements(String),
    Lifetime(String),
    SuperTrait(String),
    Equals(String),
    Default,
    Clone,
    Copy,
    Send,
    Sync,
}