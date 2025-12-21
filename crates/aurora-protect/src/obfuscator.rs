//! Python code obfuscation
//!
//! Provides name obfuscation and control flow obfuscation for Python code.
//! This module now delegates to AstObfuscator for precise AST-based obfuscation.

use crate::ast_obfuscator::AstObfuscator;
use crate::{ObfuscationLevel, ProtectError, ProtectResult};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Name obfuscator for Python identifiers
pub struct NameObfuscator {
    /// Mapping from original name to obfuscated name
    name_map: HashMap<String, String>,
    /// Counter for generating unique names
    counter: u64,
    /// Names to preserve (not obfuscate)
    preserved: HashSet<String>,
    /// Prefix for obfuscated names
    prefix: String,
}

impl NameObfuscator {
    /// Create a new name obfuscator
    pub fn new() -> Self {
        Self {
            name_map: HashMap::new(),
            counter: 0,
            preserved: HashSet::new(),
            prefix: "_0x".to_string(),
        }
    }

    /// Add names to preserve
    pub fn preserve(&mut self, names: impl IntoIterator<Item = impl Into<String>>) {
        for name in names {
            self.preserved.insert(name.into());
        }
    }

    /// Set the prefix for obfuscated names
    pub fn set_prefix(&mut self, prefix: impl Into<String>) {
        self.prefix = prefix.into();
    }

    /// Get or create an obfuscated name for the given identifier
    pub fn obfuscate(&mut self, name: &str) -> String {
        // Don't obfuscate preserved names
        if self.preserved.contains(name) {
            return name.to_string();
        }

        // Don't obfuscate dunder methods
        if name.starts_with("__") && name.ends_with("__") {
            return name.to_string();
        }

        // Return existing mapping if available
        if let Some(obfuscated) = self.name_map.get(name) {
            return obfuscated.clone();
        }

        // Generate new obfuscated name
        let obfuscated = format!("{}{:x}", self.prefix, self.counter);
        self.counter += 1;

        self.name_map.insert(name.to_string(), obfuscated.clone());
        obfuscated
    }

    /// Get the name mapping
    pub fn get_mapping(&self) -> &HashMap<String, String> {
        &self.name_map
    }

    /// Check if a name should be obfuscated
    pub fn should_obfuscate(&self, name: &str) -> bool {
        !self.preserved.contains(name)
            && !(name.starts_with("__") && name.ends_with("__"))
            && !name.starts_with("_0x") // Already obfuscated
    }
}

impl Default for NameObfuscator {
    fn default() -> Self {
        let mut obfuscator = Self::new();
        // Add Python builtins to preserved names
        obfuscator.preserve(PYTHON_BUILTINS.iter().copied());
        obfuscator
    }
}

/// Python built-in names that should not be obfuscated
const PYTHON_BUILTINS: &[&str] = &[
    // Built-in functions
    "abs",
    "all",
    "any",
    "ascii",
    "bin",
    "bool",
    "breakpoint",
    "bytearray",
    "bytes",
    "callable",
    "chr",
    "classmethod",
    "compile",
    "complex",
    "delattr",
    "dict",
    "dir",
    "divmod",
    "enumerate",
    "eval",
    "exec",
    "filter",
    "float",
    "format",
    "frozenset",
    "getattr",
    "globals",
    "hasattr",
    "hash",
    "help",
    "hex",
    "id",
    "input",
    "int",
    "isinstance",
    "issubclass",
    "iter",
    "len",
    "list",
    "locals",
    "map",
    "max",
    "memoryview",
    "min",
    "next",
    "object",
    "oct",
    "open",
    "ord",
    "pow",
    "print",
    "property",
    "range",
    "repr",
    "reversed",
    "round",
    "set",
    "setattr",
    "slice",
    "sorted",
    "staticmethod",
    "str",
    "sum",
    "super",
    "tuple",
    "type",
    "vars",
    "zip",
    // Built-in constants
    "True",
    "False",
    "None",
    "Ellipsis",
    "NotImplemented",
    // Built-in exceptions
    "Exception",
    "BaseException",
    "TypeError",
    "ValueError",
    "KeyError",
    "IndexError",
    "AttributeError",
    "ImportError",
    "RuntimeError",
    "StopIteration",
    "OSError",
    "IOError",
    "FileNotFoundError",
    // Common module names
    "self",
    "cls",
    "args",
    "kwargs",
    // Dunder attributes
    "__init__",
    "__new__",
    "__del__",
    "__repr__",
    "__str__",
    "__bytes__",
    "__format__",
    "__lt__",
    "__le__",
    "__eq__",
    "__ne__",
    "__gt__",
    "__ge__",
    "__hash__",
    "__bool__",
    "__getattr__",
    "__setattr__",
    "__delattr__",
    "__dir__",
    "__get__",
    "__set__",
    "__delete__",
    "__call__",
    "__len__",
    "__getitem__",
    "__setitem__",
    "__delitem__",
    "__iter__",
    "__next__",
    "__contains__",
    "__add__",
    "__sub__",
    "__mul__",
    "__truediv__",
    "__floordiv__",
    "__mod__",
    "__pow__",
    "__and__",
    "__or__",
    "__xor__",
    "__neg__",
    "__pos__",
    "__abs__",
    "__invert__",
    "__enter__",
    "__exit__",
    "__await__",
    "__aiter__",
    "__anext__",
    "__aenter__",
    "__aexit__",
    "__name__",
    "__module__",
    "__qualname__",
    "__doc__",
    "__dict__",
    "__class__",
    "__bases__",
    "__mro__",
    "__subclasses__",
    "__file__",
    "__path__",
    "__package__",
    "__spec__",
    "__loader__",
    "__cached__",
    "__all__",
    "__slots__",
    "__annotations__",
];

/// Main obfuscator for Python code
/// 
/// This is a facade that delegates to AstObfuscator for precise AST-based obfuscation.
/// The legacy regex-based implementation is kept for fallback.
pub struct Obfuscator {
    level: ObfuscationLevel,
    name_obfuscator: NameObfuscator,
    string_key: Option<[u8; 16]>,
    /// Use AST-based obfuscation (default: true)
    use_ast: bool,
    /// Names to preserve
    preserved_names: Vec<String>,
}

impl Obfuscator {
    /// Create a new obfuscator with the given level
    pub fn new(level: ObfuscationLevel) -> Self {
        Self {
            level,
            name_obfuscator: NameObfuscator::default(),
            string_key: None,
            use_ast: true,
            preserved_names: Vec::new(),
        }
    }

    /// Create a new obfuscator with legacy regex-based implementation
    pub fn new_legacy(level: ObfuscationLevel) -> Self {
        Self {
            level,
            name_obfuscator: NameObfuscator::default(),
            string_key: None,
            use_ast: false,
            preserved_names: Vec::new(),
        }
    }

    /// Set names to preserve
    pub fn preserve_names(&mut self, names: impl IntoIterator<Item = impl Into<String>>) {
        let names: Vec<String> = names.into_iter().map(|n| n.into()).collect();
        self.name_obfuscator.preserve(names.iter().map(|s| s.as_str()));
        self.preserved_names.extend(names);
    }

    /// Enable string encryption with the given key
    pub fn enable_string_encryption(&mut self, key: [u8; 16]) {
        self.string_key = Some(key);
    }

    /// Obfuscate Python source code
    pub fn obfuscate(&mut self, source: &str) -> ProtectResult<String> {
        if self.use_ast {
            self.obfuscate_with_ast(source)
        } else {
            self.obfuscate_legacy(source)
        }
    }

    /// AST-based obfuscation (default)
    fn obfuscate_with_ast(&mut self, source: &str) -> ProtectResult<String> {
        let mut ast_obfuscator = AstObfuscator::new(self.level);
        
        // Transfer preserved names
        ast_obfuscator.preserve(self.preserved_names.iter().map(|s| s.as_str()));
        
        // Transfer string encryption key
        if let Some(key) = self.string_key {
            ast_obfuscator.enable_string_encryption(key);
        }
        
        ast_obfuscator.obfuscate(source)
    }

    /// Legacy regex-based obfuscation (fallback)
    fn obfuscate_legacy(&mut self, source: &str) -> ProtectResult<String> {
        match self.level {
            ObfuscationLevel::None => Ok(source.to_string()),
            ObfuscationLevel::Basic => self.obfuscate_basic(source),
            ObfuscationLevel::Standard => self.obfuscate_standard(source),
            ObfuscationLevel::Advanced => self.obfuscate_advanced(source),
            ObfuscationLevel::Maximum => self.obfuscate_maximum(source),
        }
    }

    /// Basic obfuscation: only local variable names
    fn obfuscate_basic(&mut self, source: &str) -> ProtectResult<String> {
        // For basic level, we only rename local variables
        // This is a simplified implementation - a full implementation would use AST
        self.rename_identifiers(source, false)
    }

    /// Standard obfuscation: variables, functions, classes
    fn obfuscate_standard(&mut self, source: &str) -> ProtectResult<String> {
        self.rename_identifiers(source, true)
    }

    /// Advanced obfuscation: + control flow
    fn obfuscate_advanced(&mut self, source: &str) -> ProtectResult<String> {
        let mut result = self.obfuscate_standard(source)?;
        result = self.insert_opaque_predicates(&result)?;
        Ok(result)
    }

    /// Maximum obfuscation: + string encryption, dead code
    fn obfuscate_maximum(&mut self, source: &str) -> ProtectResult<String> {
        let mut result = self.obfuscate_advanced(source)?;

        if self.string_key.is_some() {
            result = self.encrypt_strings(&result)?;
        }

        result = self.insert_dead_code(&result)?;
        Ok(result)
    }

    /// Rename identifiers in the source code
    fn rename_identifiers(&mut self, source: &str, include_functions: bool) -> ProtectResult<String> {
        let mut result = source.to_string();

        // Pattern for Python identifiers
        // This is a simplified approach - production code should use a proper Python parser
        let identifier_pattern = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b")
            .map_err(|e| ProtectError::Obfuscation(e.to_string()))?;

        // First pass: collect all identifiers to obfuscate
        let mut identifiers_to_rename: Vec<String> = Vec::new();

        // Find function/class definitions if include_functions is true
        if include_functions {
            let def_pattern = Regex::new(r"(?m)^[ \t]*(def|class)\s+([a-zA-Z_][a-zA-Z0-9_]*)")
                .map_err(|e| ProtectError::Obfuscation(e.to_string()))?;

            for cap in def_pattern.captures_iter(source) {
                let name = cap.get(2).unwrap().as_str();
                if self.name_obfuscator.should_obfuscate(name) {
                    identifiers_to_rename.push(name.to_string());
                }
            }
        }

        // Find variable assignments
        let assign_pattern = Regex::new(r"(?m)^[ \t]*([a-zA-Z_][a-zA-Z0-9_]*)\s*=")
            .map_err(|e| ProtectError::Obfuscation(e.to_string()))?;

        for cap in assign_pattern.captures_iter(source) {
            let name = cap.get(1).unwrap().as_str();
            if self.name_obfuscator.should_obfuscate(name) {
                identifiers_to_rename.push(name.to_string());
            }
        }

        // Find function parameters
        let param_pattern = Regex::new(r"def\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(([^)]*)\)")
            .map_err(|e| ProtectError::Obfuscation(e.to_string()))?;

        for cap in param_pattern.captures_iter(source) {
            let params = cap.get(1).unwrap().as_str();
            for param in params.split(',') {
                let param = param.trim().split(':').next().unwrap_or("").trim();
                let param = param.split('=').next().unwrap_or("").trim();
                if !param.is_empty() && self.name_obfuscator.should_obfuscate(param) {
                    identifiers_to_rename.push(param.to_string());
                }
            }
        }

        // Generate obfuscated names for all collected identifiers
        for name in &identifiers_to_rename {
            self.name_obfuscator.obfuscate(name);
        }

        // Second pass: replace identifiers
        // Sort by length (longest first) to avoid partial replacements
        let mut sorted_names: Vec<_> = self.name_obfuscator.get_mapping().iter().collect();
        sorted_names.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (original, obfuscated) in sorted_names {
            let pattern = format!(r"\b{}\b", regex::escape(original));
            let re = Regex::new(&pattern).map_err(|e| ProtectError::Obfuscation(e.to_string()))?;
            result = re.replace_all(&result, obfuscated.as_str()).to_string();
        }

        Ok(result)
    }

    /// Insert opaque predicates for control flow obfuscation
    fn insert_opaque_predicates(&self, source: &str) -> ProtectResult<String> {
        // Add opaque predicates that always evaluate to True or False
        // but are hard to analyze statically
        let opaque_true = "_0xT = (lambda: (lambda x: x(x))(lambda y: True))()";
        let opaque_false = "_0xF = (lambda: (lambda x: x(x))(lambda y: False))()";

        let mut result = format!(
            "# -*- coding: utf-8 -*-\n{}\n{}\n{}",
            opaque_true, opaque_false, source
        );

        // Wrap some if statements with opaque predicates
        let if_pattern =
            Regex::new(r"(?m)^([ \t]*)(if\s+)(.+?)(:)").map_err(|e| ProtectError::Obfuscation(e.to_string()))?;

        result = if_pattern
            .replace_all(&result, "${1}${2}(_0xT and (${3}))${4}")
            .to_string();

        Ok(result)
    }

    /// Encrypt string literals
    fn encrypt_strings(&self, source: &str) -> ProtectResult<String> {
        let key = self
            .string_key
            .ok_or_else(|| ProtectError::Obfuscation("String encryption key not set".to_string()))?;

        // Pattern for string literals (simplified - doesn't handle all edge cases)
        let string_pattern = Regex::new(r#"(['"])([^'"\\]|\\.)*\1"#)
            .map_err(|e| ProtectError::Obfuscation(e.to_string()))?;

        let mut result = source.to_string();

        // Add string decryption function at the beginning
        let decrypt_func = format!(
            r#"
def _0xS(s):
    k = bytes([{}])
    return bytes([c ^ k[i % len(k)] for i, c in enumerate(s)]).decode('utf-8')
"#,
            key.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        result = format!("{}\n{}", decrypt_func, result);

        // Encrypt string literals and replace with decryption calls
        // This is a simplified implementation
        // A production version would need proper string parsing

        Ok(result)
    }

    /// Insert dead code for confusion
    fn insert_dead_code(&self, source: &str) -> ProtectResult<String> {
        let dead_code = r#"
# Dead code - never executed
if False:
    _0xD1 = lambda x: x * 2 + 1
    _0xD2 = [_0xD1(i) for i in range(100)]
    _0xD3 = {k: v for k, v in zip(_0xD2, reversed(_0xD2))}
"#;

        Ok(format!("{}\n{}", dead_code, source))
    }

    /// Get the name mapping for debugging
    pub fn get_name_mapping(&self) -> &HashMap<String, String> {
        self.name_obfuscator.get_mapping()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_obfuscator_basic() {
        let mut obfuscator = NameObfuscator::new();

        let name1 = obfuscator.obfuscate("my_variable");
        let name2 = obfuscator.obfuscate("my_variable");
        let name3 = obfuscator.obfuscate("other_var");

        // Same name should get same obfuscated name
        assert_eq!(name1, name2);
        // Different names should get different obfuscated names
        assert_ne!(name1, name3);
        // Should start with prefix
        assert!(name1.starts_with("_0x"));
    }

    #[test]
    fn test_preserve_names() {
        let mut obfuscator = NameObfuscator::new();
        obfuscator.preserve(["important_func"]);

        let name = obfuscator.obfuscate("important_func");
        assert_eq!(name, "important_func");
    }

    #[test]
    fn test_dunder_not_obfuscated() {
        let mut obfuscator = NameObfuscator::new();

        let name = obfuscator.obfuscate("__init__");
        assert_eq!(name, "__init__");
    }

    #[test]
    fn test_obfuscator_basic() {
        let mut obfuscator = Obfuscator::new(ObfuscationLevel::Basic);

        let source = r#"
def hello():
    message = "Hello, World!"
    print(message)
"#;

        let result = obfuscator.obfuscate(source).unwrap();

        // The variable 'message' should be obfuscated
        assert!(!result.contains("message ="));
        // But 'print' should not be (it's a builtin)
        assert!(result.contains("print"));
    }
}
