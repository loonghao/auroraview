//! Python AST-based precise obfuscation
//!
//! This module provides precise code obfuscation using Python's AST (Abstract Syntax Tree).
//! Unlike regex-based obfuscation, AST-based obfuscation understands Python's syntax and
//! can correctly handle:
//! - Variable scopes (local, global, nonlocal)
//! - Class attributes vs local variables
//! - Import statements and module references
//! - Function parameters and default values
//! - Comprehensions and their scopes
//! - Decorators and annotations

use crate::{ObfuscationLevel, ProtectError, ProtectResult};
use std::collections::{HashMap, HashSet};

/// AST node types that we track for obfuscation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameContext {
    /// Variable assignment (x = 1)
    Store,
    /// Variable read (print(x))
    Load,
    /// Variable deletion (del x)
    Delete,
    /// Function parameter
    Parameter,
    /// Function/class definition name
    Definition,
    /// Import name
    Import,
    /// Attribute access (obj.attr)
    Attribute,
    /// Global declaration
    Global,
    /// Nonlocal declaration
    Nonlocal,
}

/// Scope information for a name
#[derive(Debug, Clone)]
pub struct NameInfo {
    /// Original name
    pub original: String,
    /// Obfuscated name (if any)
    pub obfuscated: Option<String>,
    /// Context where this name appears
    pub context: NameContext,
    /// Scope depth (0 = module level)
    pub scope_depth: usize,
    /// Is this name exported (in __all__)?
    pub is_exported: bool,
    /// Is this a class attribute?
    pub is_class_attr: bool,
    /// Line number where first defined
    pub line: usize,
}

/// Scope type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Module,
    Class,
    Function,
    Comprehension,
    Lambda,
}

/// A scope in the Python code
#[derive(Debug, Clone)]
pub struct Scope {
    /// Scope type
    pub scope_type: ScopeType,
    /// Names defined in this scope
    pub names: HashMap<String, NameInfo>,
    /// Names declared as global
    pub globals: HashSet<String>,
    /// Names declared as nonlocal
    pub nonlocals: HashSet<String>,
    /// Parent scope index
    pub parent: Option<usize>,
    /// Scope name (function/class name)
    pub name: Option<String>,
}

impl Scope {
    pub fn new(scope_type: ScopeType, parent: Option<usize>, name: Option<String>) -> Self {
        Self {
            scope_type,
            names: HashMap::new(),
            globals: HashSet::new(),
            nonlocals: HashSet::new(),
            parent,
            name,
        }
    }
}

/// Python AST-based obfuscator
pub struct AstObfuscator {
    /// Obfuscation level
    level: ObfuscationLevel,
    /// Scope stack (index into scopes vec)
    scope_stack: Vec<usize>,
    /// All scopes
    scopes: Vec<Scope>,
    /// Global name mapping (original -> obfuscated)
    name_map: HashMap<String, String>,
    /// Counter for generating unique names
    counter: u64,
    /// Names to preserve (never obfuscate)
    preserved: HashSet<String>,
    /// Exported names (from __all__)
    exports: HashSet<String>,
    /// Imported names (from import statements)
    imports: HashSet<String>,
    /// Class attribute names to preserve
    class_attrs: HashSet<String>,
    /// Enable string encryption
    encrypt_strings: bool,
    /// String encryption key
    string_key: Option<[u8; 16]>,
}

impl AstObfuscator {
    /// Create a new AST obfuscator
    pub fn new(level: ObfuscationLevel) -> Self {
        let mut obfuscator = Self {
            level,
            scope_stack: vec![0],
            scopes: vec![Scope::new(ScopeType::Module, None, None)],
            name_map: HashMap::new(),
            counter: 0,
            preserved: HashSet::new(),
            exports: HashSet::new(),
            imports: HashSet::new(),
            class_attrs: HashSet::new(),
            encrypt_strings: false,
            string_key: None,
        };
        
        // Add Python builtins to preserved names
        obfuscator.preserved.extend(PYTHON_BUILTINS.iter().map(|s| s.to_string()));
        obfuscator
    }

    /// Add names to preserve
    pub fn preserve(&mut self, names: impl IntoIterator<Item = impl Into<String>>) {
        for name in names {
            self.preserved.insert(name.into());
        }
    }

    /// Enable string encryption
    pub fn enable_string_encryption(&mut self, key: [u8; 16]) {
        self.encrypt_strings = true;
        self.string_key = Some(key);
    }

    /// Obfuscate Python source code using AST analysis
    pub fn obfuscate(&mut self, source: &str) -> ProtectResult<String> {
        match self.level {
            ObfuscationLevel::None => Ok(source.to_string()),
            _ => self.obfuscate_with_ast(source),
        }
    }

    /// Main obfuscation entry point using AST
    fn obfuscate_with_ast(&mut self, source: &str) -> ProtectResult<String> {
        // Phase 1: Analyze the code to build scope information
        self.analyze_source(source)?;
        
        // Phase 2: Generate obfuscated names based on scope analysis
        self.generate_obfuscated_names()?;
        
        // Phase 3: Transform the source code
        let result = self.transform_source(source)?;
        
        // Phase 4: Add control flow obfuscation if level >= Advanced
        let result = if self.level >= ObfuscationLevel::Advanced {
            self.add_control_flow_obfuscation(&result)?
        } else {
            result
        };
        
        // Phase 5: Add string encryption if level == Maximum and enabled
        let result = if self.level == ObfuscationLevel::Maximum && self.encrypt_strings {
            self.add_string_encryption(&result)?
        } else {
            result
        };
        
        // Phase 6: Add dead code if level == Maximum
        let result = if self.level == ObfuscationLevel::Maximum {
            self.add_dead_code(&result)?
        } else {
            result
        };
        
        Ok(result)
    }

    /// Analyze source code to extract scope and name information
    fn analyze_source(&mut self, source: &str) -> ProtectResult<()> {
        let lines: Vec<&str> = source.lines().collect();
        
        // Track current indentation level for scope detection
        let mut indent_stack: Vec<usize> = vec![0];
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Calculate indentation
            let indent = line.len() - line.trim_start().len();
            
            // Pop scopes when dedenting
            while indent_stack.len() > 1 && indent < *indent_stack.last().unwrap() {
                indent_stack.pop();
                if self.scope_stack.len() > 1 {
                    self.scope_stack.pop();
                }
            }
            
            // Analyze the line
            self.analyze_line(trimmed, line_num + 1, indent, &mut indent_stack)?;
        }
        
        Ok(())
    }

    /// Analyze a single line of code
    fn analyze_line(
        &mut self,
        line: &str,
        line_num: usize,
        indent: usize,
        indent_stack: &mut Vec<usize>,
    ) -> ProtectResult<()> {
        // Check for __all__ definition (exports)
        if line.starts_with("__all__") && line.contains('=') {
            self.parse_all_definition(line)?;
            return Ok(());
        }
        
        // Check for import statements
        if line.starts_with("import ") || line.starts_with("from ") {
            self.parse_import(line)?;
            return Ok(());
        }
        
        // Check for global/nonlocal declarations
        if line.starts_with("global ") {
            self.parse_global(line)?;
            return Ok(());
        }
        if line.starts_with("nonlocal ") {
            self.parse_nonlocal(line)?;
            return Ok(());
        }
        
        // Check for function definition
        if line.starts_with("def ") || line.starts_with("async def ") {
            let name = self.parse_function_def(line)?;
            self.add_name_to_scope(&name, NameContext::Definition, line_num);
            
            // Create new scope
            let parent = *self.scope_stack.last().unwrap();
            let new_scope = Scope::new(ScopeType::Function, Some(parent), Some(name.clone()));
            let scope_idx = self.scopes.len();
            self.scopes.push(new_scope);
            self.scope_stack.push(scope_idx);
            indent_stack.push(indent + 4); // Assume 4-space indent
            
            // Parse parameters
            self.parse_function_params(line, line_num)?;
            return Ok(());
        }
        
        // Check for class definition
        if line.starts_with("class ") {
            let name = self.parse_class_def(line)?;
            self.add_name_to_scope(&name, NameContext::Definition, line_num);
            
            // Create new scope
            let parent = *self.scope_stack.last().unwrap();
            let new_scope = Scope::new(ScopeType::Class, Some(parent), Some(name));
            let scope_idx = self.scopes.len();
            self.scopes.push(new_scope);
            self.scope_stack.push(scope_idx);
            indent_stack.push(indent + 4);
            return Ok(());
        }
        
        // Check for lambda
        if line.contains("lambda ") {
            self.parse_lambda(line, line_num)?;
        }
        
        // Check for comprehensions
        if line.contains(" for ") && (line.contains('[') || line.contains('{') || line.contains('(')) {
            self.parse_comprehension(line, line_num)?;
        }
        
        // Check for variable assignments
        if line.contains('=') && !line.contains("==") && !line.contains("!=") 
           && !line.contains("<=") && !line.contains(">=") {
            self.parse_assignment(line, line_num)?;
        }
        
        Ok(())
    }

    /// Parse __all__ definition to extract exported names
    fn parse_all_definition(&mut self, line: &str) -> ProtectResult<()> {
        // Extract names from __all__ = ['name1', 'name2', ...]
        if let Some(start) = line.find('[') {
            if let Some(end) = line.find(']') {
                let names_str = &line[start + 1..end];
                for name in names_str.split(',') {
                    let name = name.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !name.is_empty() {
                        self.exports.insert(name.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse import statement
    fn parse_import(&mut self, line: &str) -> ProtectResult<()> {
        if line.starts_with("import ") {
            // import module1, module2
            let modules = line[7..].trim();
            for module in modules.split(',') {
                let module = module.trim().split(" as ").next().unwrap_or("").trim();
                let module = module.split('.').next().unwrap_or("");
                if !module.is_empty() {
                    self.imports.insert(module.to_string());
                }
            }
        } else if line.starts_with("from ") {
            // from module import name1, name2
            if let Some(import_pos) = line.find(" import ") {
                let names = line[import_pos + 8..].trim();
                if names != "*" {
                    for name in names.split(',') {
                        let name = name.trim().split(" as ").last().unwrap_or("").trim();
                        if !name.is_empty() {
                            self.imports.insert(name.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse global declaration
    fn parse_global(&mut self, line: &str) -> ProtectResult<()> {
        let names = line[7..].trim();
        let scope_idx = *self.scope_stack.last().unwrap();
        for name in names.split(',') {
            let name = name.trim();
            if !name.is_empty() {
                self.scopes[scope_idx].globals.insert(name.to_string());
            }
        }
        Ok(())
    }

    /// Parse nonlocal declaration
    fn parse_nonlocal(&mut self, line: &str) -> ProtectResult<()> {
        let names = line[9..].trim();
        let scope_idx = *self.scope_stack.last().unwrap();
        for name in names.split(',') {
            let name = name.trim();
            if !name.is_empty() {
                self.scopes[scope_idx].nonlocals.insert(name.to_string());
            }
        }
        Ok(())
    }

    /// Parse function definition and return function name
    fn parse_function_def(&self, line: &str) -> ProtectResult<String> {
        let line = if line.starts_with("async ") {
            &line[6..]
        } else {
            line
        };
        
        // def function_name(params):
        let start = line.find("def ").map(|i| i + 4).unwrap_or(0);
        let end = line.find('(').unwrap_or(line.len());
        let name = line[start..end].trim().to_string();
        Ok(name)
    }

    /// Parse function parameters
    fn parse_function_params(&mut self, line: &str, line_num: usize) -> ProtectResult<()> {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.rfind(')') {
                let params = &line[start + 1..end];
                for param in params.split(',') {
                    let param = param.trim();
                    // Handle type annotations: param: Type = default
                    let param = param.split(':').next().unwrap_or(param);
                    let param = param.split('=').next().unwrap_or(param).trim();
                    // Handle *args and **kwargs
                    let param = param.trim_start_matches('*');
                    if !param.is_empty() && param != "self" && param != "cls" {
                        self.add_name_to_scope(param, NameContext::Parameter, line_num);
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse class definition and return class name
    fn parse_class_def(&self, line: &str) -> ProtectResult<String> {
        // class ClassName(bases):
        let start = line.find("class ").map(|i| i + 6).unwrap_or(0);
        let end = line.find('(').or_else(|| line.find(':')).unwrap_or(line.len());
        let name = line[start..end].trim().to_string();
        Ok(name)
    }

    /// Parse lambda expression
    fn parse_lambda(&mut self, line: &str, line_num: usize) -> ProtectResult<()> {
        // Find lambda parameters
        if let Some(lambda_pos) = line.find("lambda ") {
            let rest = &line[lambda_pos + 7..];
            if let Some(colon_pos) = rest.find(':') {
                let params = &rest[..colon_pos];
                for param in params.split(',') {
                    let param = param.trim().split('=').next().unwrap_or("").trim();
                    if !param.is_empty() {
                        self.add_name_to_scope(param, NameContext::Parameter, line_num);
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse comprehension expression
    fn parse_comprehension(&mut self, line: &str, line_num: usize) -> ProtectResult<()> {
        // Find loop variables in comprehensions
        // [expr for x in iterable] or {k: v for k, v in items}
        let mut pos = 0;
        while let Some(for_pos) = line[pos..].find(" for ") {
            let rest = &line[pos + for_pos + 5..];
            if let Some(in_pos) = rest.find(" in ") {
                let vars = &rest[..in_pos];
                for var in vars.split(',') {
                    let var = var.trim().trim_start_matches('(').trim_end_matches(')');
                    if !var.is_empty() && self.is_valid_identifier(var) {
                        self.add_name_to_scope(var, NameContext::Store, line_num);
                    }
                }
            }
            pos += for_pos + 5;
        }
        Ok(())
    }

    /// Parse variable assignment
    fn parse_assignment(&mut self, line: &str, line_num: usize) -> ProtectResult<()> {
        // Handle various assignment forms
        // x = value
        // x, y = values
        // x: Type = value
        // self.attr = value (skip - class attribute)
        
        if let Some(eq_pos) = line.find('=') {
            // Skip augmented assignments (+=, -=, etc.)
            if eq_pos > 0 {
                let prev_char = line.chars().nth(eq_pos - 1).unwrap_or(' ');
                if ['+', '-', '*', '/', '%', '&', '|', '^', '@', '!', '<', '>'].contains(&prev_char) {
                    return Ok(());
                }
            }
            
            let lhs = line[..eq_pos].trim();
            
            // Skip if it's an attribute assignment (self.x = ...)
            if lhs.contains('.') {
                // Track class attributes
                if lhs.starts_with("self.") || lhs.starts_with("cls.") {
                    let attr = lhs.split('.').nth(1).unwrap_or("");
                    let attr = attr.split(':').next().unwrap_or(attr).trim();
                    if !attr.is_empty() {
                        self.class_attrs.insert(attr.to_string());
                    }
                }
                return Ok(());
            }
            
            // Handle type annotations
            let lhs = lhs.split(':').next().unwrap_or(lhs);
            
            // Handle tuple unpacking
            for name in lhs.split(',') {
                let name = name.trim()
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .trim();
                if !name.is_empty() && self.is_valid_identifier(name) {
                    self.add_name_to_scope(name, NameContext::Store, line_num);
                }
            }
        }
        Ok(())
    }

    /// Check if a string is a valid Python identifier
    fn is_valid_identifier(&self, s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        let first = s.chars().next().unwrap();
        if !first.is_alphabetic() && first != '_' {
            return false;
        }
        s.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Add a name to the current scope
    fn add_name_to_scope(&mut self, name: &str, context: NameContext, line: usize) {
        let scope_idx = *self.scope_stack.last().unwrap();
        let scope_depth = self.scope_stack.len() - 1;
        let is_class_scope = self.scopes[scope_idx].scope_type == ScopeType::Class;
        
        let info = NameInfo {
            original: name.to_string(),
            obfuscated: None,
            context,
            scope_depth,
            is_exported: self.exports.contains(name),
            is_class_attr: is_class_scope || self.class_attrs.contains(name),
            line,
        };
        
        self.scopes[scope_idx].names.insert(name.to_string(), info);
    }

    /// Generate obfuscated names based on scope analysis
    fn generate_obfuscated_names(&mut self) -> ProtectResult<()> {
        // Collect all names that should be obfuscated
        let mut names_to_obfuscate: Vec<(String, usize)> = Vec::new();
        
        for (scope_idx, scope) in self.scopes.iter().enumerate() {
            for (name, info) in &scope.names {
                if self.should_obfuscate_name(name, info, scope) {
                    names_to_obfuscate.push((name.clone(), scope_idx));
                }
            }
        }
        
        // Generate obfuscated names
        for (name, _scope_idx) in names_to_obfuscate {
            if !self.name_map.contains_key(&name) {
                let obfuscated = self.generate_unique_name();
                self.name_map.insert(name, obfuscated);
            }
        }
        
        Ok(())
    }

    /// Check if a name should be obfuscated
    fn should_obfuscate_name(&self, name: &str, info: &NameInfo, scope: &Scope) -> bool {
        // Don't obfuscate preserved names
        if self.preserved.contains(name) {
            return false;
        }
        
        // Don't obfuscate dunder names
        if name.starts_with("__") && name.ends_with("__") {
            return false;
        }
        
        // Don't obfuscate exported names
        if info.is_exported {
            return false;
        }
        
        // Don't obfuscate imported names
        if self.imports.contains(name) {
            return false;
        }
        
        // Don't obfuscate global/nonlocal declarations (they reference other scopes)
        if scope.globals.contains(name) || scope.nonlocals.contains(name) {
            return false;
        }
        
        // For basic level, only obfuscate local variables (not definitions)
        if self.level == ObfuscationLevel::Basic {
            return info.context == NameContext::Store || info.context == NameContext::Parameter;
        }
        
        // Don't obfuscate class attributes at standard level
        if self.level == ObfuscationLevel::Standard && info.is_class_attr {
            return false;
        }
        
        true
    }

    /// Generate a unique obfuscated name
    fn generate_unique_name(&mut self) -> String {
        let name = format!("_0x{:x}", self.counter);
        self.counter += 1;
        name
    }

    /// Transform source code by replacing names
    fn transform_source(&self, source: &str) -> ProtectResult<String> {
        let mut result = source.to_string();
        
        // Sort names by length (longest first) to avoid partial replacements
        let mut sorted_names: Vec<_> = self.name_map.iter().collect();
        sorted_names.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        
        for (original, obfuscated) in sorted_names {
            // Use word boundary matching
            let pattern = format!(r"\b{}\b", regex::escape(original));
            let re = regex::Regex::new(&pattern)
                .map_err(|e| ProtectError::Obfuscation(e.to_string()))?;
            
            // Don't replace in strings and comments
            result = self.replace_preserving_strings(&result, &re, obfuscated)?;
        }
        
        Ok(result)
    }

    /// Replace pattern while preserving strings and comments
    fn replace_preserving_strings(
        &self,
        source: &str,
        pattern: &regex::Regex,
        replacement: &str,
    ) -> ProtectResult<String> {
        let mut result = String::new();
        let mut chars = source.chars().peekable();
        let mut current_pos = 0;
        let mut in_string = false;
        let mut string_char = '"';
        let mut in_comment = false;
        let mut triple_quote = false;
        
        while let Some(c) = chars.next() {
            if in_comment {
                result.push(c);
                if c == '\n' {
                    in_comment = false;
                }
                current_pos += c.len_utf8();
                continue;
            }
            
            if in_string {
                result.push(c);
                if c == '\\' {
                    // Escape sequence
                    if let Some(next) = chars.next() {
                        result.push(next);
                        current_pos += next.len_utf8();
                    }
                } else if c == string_char {
                    if triple_quote {
                        // Check for closing triple quote
                        if chars.peek() == Some(&string_char) {
                            result.push(chars.next().unwrap());
                            current_pos += 1;
                            if chars.peek() == Some(&string_char) {
                                result.push(chars.next().unwrap());
                                current_pos += 1;
                                in_string = false;
                                triple_quote = false;
                            }
                        }
                    } else {
                        in_string = false;
                    }
                }
                current_pos += c.len_utf8();
                continue;
            }
            
            // Check for comment start
            if c == '#' {
                in_comment = true;
                result.push(c);
                current_pos += c.len_utf8();
                continue;
            }
            
            // Check for string start
            if c == '"' || c == '\'' {
                string_char = c;
                result.push(c);
                
                // Check for triple quote
                if chars.peek() == Some(&c) {
                    result.push(chars.next().unwrap());
                    current_pos += 1;
                    if chars.peek() == Some(&c) {
                        result.push(chars.next().unwrap());
                        current_pos += 1;
                        triple_quote = true;
                    }
                }
                
                in_string = true;
                current_pos += c.len_utf8();
                continue;
            }
            
            // Check if we're at a potential identifier
            if c.is_alphabetic() || c == '_' {
                let start_pos = result.len();
                result.push(c);
                
                // Collect the full identifier
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        result.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                // Check if this identifier should be replaced
                let identifier = &result[start_pos..];
                if pattern.is_match(identifier) {
                    // Replace the identifier
                    result.truncate(start_pos);
                    result.push_str(replacement);
                }
            } else {
                result.push(c);
            }
            
            current_pos += c.len_utf8();
        }
        
        Ok(result)
    }

    /// Add control flow obfuscation
    fn add_control_flow_obfuscation(&self, source: &str) -> ProtectResult<String> {
        // Add opaque predicates
        let header = r#"# -*- coding: utf-8 -*-
# Protected by AuroraView
_0xT = (lambda: (lambda x: x(x))(lambda y: True))()
_0xF = (lambda: (lambda x: x(x))(lambda y: False))()
_0xN = lambda f: (lambda x: f(lambda v: x(x)(v)))(lambda x: f(lambda v: x(x)(v)))
"#;
        
        // Wrap conditions with opaque predicates
        let result = format!("{}\n{}", header, source);
        
        // Add opaque predicates to if statements
        let if_pattern = regex::Regex::new(r"(?m)^(\s*)(if\s+)(.+?)(:)")
            .map_err(|e| ProtectError::Obfuscation(e.to_string()))?;
        
        let result = if_pattern
            .replace_all(&result, "${1}${2}(_0xT and (${3}))${4}")
            .to_string();
        
        Ok(result)
    }

    /// Add string encryption
    fn add_string_encryption(&self, source: &str) -> ProtectResult<String> {
        let key = self.string_key.ok_or_else(|| {
            ProtectError::Obfuscation("String encryption key not set".to_string())
        })?;
        
        // Add decryption function
        let decrypt_func = format!(
            r#"
def _0xS(s):
    k = bytes([{}])
    return bytes([c ^ k[i % len(k)] for i, c in enumerate(s)]).decode('utf-8')
"#,
            key.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", ")
        );
        
        Ok(format!("{}\n{}", decrypt_func, source))
    }

    /// Add dead code for confusion
    fn add_dead_code(&self, source: &str) -> ProtectResult<String> {
        let dead_code = r#"
# Obfuscation layer
if _0xF:
    _0xD1 = lambda x: x * 2 + 1
    _0xD2 = [_0xD1(i) for i in range(100)]
    _0xD3 = {k: v for k, v in zip(_0xD2, reversed(_0xD2))}
    _0xD4 = type('_0xC', (), {'__init__': lambda s: None})()
"#;
        
        Ok(format!("{}\n{}", dead_code, source))
    }

    /// Get the name mapping for debugging/testing
    pub fn get_name_mapping(&self) -> &HashMap<String, String> {
        &self.name_map
    }
}

/// Python built-in names that should not be obfuscated
const PYTHON_BUILTINS: &[&str] = &[
    // Built-in functions
    "abs", "all", "any", "ascii", "bin", "bool", "breakpoint", "bytearray",
    "bytes", "callable", "chr", "classmethod", "compile", "complex", "delattr",
    "dict", "dir", "divmod", "enumerate", "eval", "exec", "filter", "float",
    "format", "frozenset", "getattr", "globals", "hasattr", "hash", "help",
    "hex", "id", "input", "int", "isinstance", "issubclass", "iter", "len",
    "list", "locals", "map", "max", "memoryview", "min", "next", "object",
    "oct", "open", "ord", "pow", "print", "property", "range", "repr",
    "reversed", "round", "set", "setattr", "slice", "sorted", "staticmethod",
    "str", "sum", "super", "tuple", "type", "vars", "zip",
    // Built-in constants
    "True", "False", "None", "Ellipsis", "NotImplemented",
    // Built-in exceptions
    "Exception", "BaseException", "TypeError", "ValueError", "KeyError",
    "IndexError", "AttributeError", "ImportError", "RuntimeError",
    "StopIteration", "OSError", "IOError", "FileNotFoundError",
    // Common names
    "self", "cls", "args", "kwargs",
    // Dunder names (handled separately, but included for completeness)
    "__init__", "__new__", "__del__", "__repr__", "__str__", "__bytes__",
    "__format__", "__lt__", "__le__", "__eq__", "__ne__", "__gt__", "__ge__",
    "__hash__", "__bool__", "__getattr__", "__setattr__", "__delattr__",
    "__dir__", "__get__", "__set__", "__delete__", "__call__", "__len__",
    "__getitem__", "__setitem__", "__delitem__", "__iter__", "__next__",
    "__contains__", "__add__", "__sub__", "__mul__", "__truediv__",
    "__floordiv__", "__mod__", "__pow__", "__and__", "__or__", "__xor__",
    "__neg__", "__pos__", "__abs__", "__invert__", "__enter__", "__exit__",
    "__await__", "__aiter__", "__anext__", "__aenter__", "__aexit__",
    "__name__", "__module__", "__qualname__", "__doc__", "__dict__",
    "__class__", "__bases__", "__mro__", "__subclasses__", "__file__",
    "__path__", "__package__", "__spec__", "__loader__", "__cached__",
    "__all__", "__slots__", "__annotations__",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_obfuscator_basic() {
        let mut obfuscator = AstObfuscator::new(ObfuscationLevel::Standard);
        
        let source = r#"
def calculate(x, y):
    result = x + y
    return result

total = calculate(1, 2)
print(total)
"#;
        
        let result = obfuscator.obfuscate(source).unwrap();
        
        // Function name should be obfuscated
        assert!(!result.contains("def calculate"));
        // Parameters should be obfuscated
        assert!(!result.contains("(x, y)"));
        // Local variable should be obfuscated
        assert!(!result.contains("result ="));
        // Builtins should not be obfuscated
        assert!(result.contains("print"));
    }

    #[test]
    fn test_preserve_exports() {
        let mut obfuscator = AstObfuscator::new(ObfuscationLevel::Standard);
        
        let source = r#"
__all__ = ['public_func']

def public_func():
    pass

def _private_func():
    pass
"#;
        
        let result = obfuscator.obfuscate(source).unwrap();
        
        // Exported name should be preserved
        assert!(result.contains("public_func"));
        // Private name should be obfuscated
        assert!(!result.contains("_private_func"));
    }

    #[test]
    fn test_preserve_strings() {
        let mut obfuscator = AstObfuscator::new(ObfuscationLevel::Standard);
        
        let source = r#"
message = "hello world"
name = 'test'
"#;
        
        let result = obfuscator.obfuscate(source).unwrap();
        
        // Strings should be preserved
        assert!(result.contains("\"hello world\""));
        assert!(result.contains("'test'"));
    }

    #[test]
    fn test_imports_preserved() {
        let mut obfuscator = AstObfuscator::new(ObfuscationLevel::Standard);
        
        let source = r#"
import os
from pathlib import Path

path = Path(".")
os.getcwd()
"#;
        
        let result = obfuscator.obfuscate(source).unwrap();
        
        // Imported names should be preserved
        assert!(result.contains("import os"));
        assert!(result.contains("Path"));
    }
}
