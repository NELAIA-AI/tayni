//! JSON Parser and Encoder for TAYNI
//! 
//! A complete, zero-dependency JSON implementation following RFC 8259.
//! Designed for AI-generated code with clear error messages.

use std::collections::HashMap;
use std::fmt;

// ============================================================================
// JSON Value Types
// ============================================================================

/// Represents any JSON value
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(JsonObject),
}

/// JSON number (can be integer or float)
#[derive(Debug, Clone, PartialEq)]
pub enum JsonNumber {
    Integer(i64),
    Float(f64),
}

/// JSON object (preserves insertion order)
pub type JsonObject = Vec<(String, JsonValue)>;

impl JsonValue {
    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }
    
    /// Get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Get as i64
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Number(JsonNumber::Integer(n)) => Some(*n),
            JsonValue::Number(JsonNumber::Float(f)) => Some(*f as i64),
            _ => None,
        }
    }
    
    /// Get as f64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(JsonNumber::Integer(n)) => Some(*n as f64),
            JsonValue::Number(JsonNumber::Float(f)) => Some(*f),
            _ => None,
        }
    }
    
    /// Get as string slice
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Get as array slice
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
    
    /// Get as object
    pub fn as_object(&self) -> Option<&JsonObject> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
    
    /// Get value by key (for objects)
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => {
                obj.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }
    
    /// Get value by index (for arrays)
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }
    
    /// Set value by key (for objects)
    pub fn set(&mut self, key: &str, value: JsonValue) -> bool {
        match self {
            JsonValue::Object(obj) => {
                for (k, v) in obj.iter_mut() {
                    if k == key {
                        *v = value;
                        return true;
                    }
                }
                obj.push((key.to_string(), value));
                true
            }
            _ => false,
        }
    }
    
    /// Remove key from object
    pub fn remove(&mut self, key: &str) -> Option<JsonValue> {
        match self {
            JsonValue::Object(obj) => {
                if let Some(pos) = obj.iter().position(|(k, _)| k == key) {
                    Some(obj.remove(pos).1)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", encode(self))
    }
}

impl From<bool> for JsonValue {
    fn from(b: bool) -> Self { JsonValue::Bool(b) }
}

impl From<i64> for JsonValue {
    fn from(n: i64) -> Self { JsonValue::Number(JsonNumber::Integer(n)) }
}

impl From<i32> for JsonValue {
    fn from(n: i32) -> Self { JsonValue::Number(JsonNumber::Integer(n as i64)) }
}

impl From<f64> for JsonValue {
    fn from(n: f64) -> Self { JsonValue::Number(JsonNumber::Float(n)) }
}

impl From<&str> for JsonValue {
    fn from(s: &str) -> Self { JsonValue::String(s.to_string()) }
}

impl From<String> for JsonValue {
    fn from(s: String) -> Self { JsonValue::String(s) }
}

impl<T: Into<JsonValue>> From<Vec<T>> for JsonValue {
    fn from(v: Vec<T>) -> Self {
        JsonValue::Array(v.into_iter().map(|x| x.into()).collect())
    }
}

// ============================================================================
// JSON Parser
// ============================================================================

/// Parse error with location information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON parse error at line {}, column {}: {}", 
            self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

/// JSON Parser state
struct Parser<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            input,
            chars: input.char_indices().peekable(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            line: self.line,
            column: self.column,
            position: self.position,
        }
    }
    
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }
    
    fn next(&mut self) -> Option<char> {
        if let Some((pos, c)) = self.chars.next() {
            self.position = pos;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(c)
        } else {
            None
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }
    
    fn expect(&mut self, expected: char) -> Result<(), ParseError> {
        match self.next() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(self.error(&format!("expected '{}', found '{}'", expected, c))),
            None => Err(self.error(&format!("expected '{}', found end of input", expected))),
        }
    }
    
    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        
        match self.peek() {
            None => Err(self.error("unexpected end of input")),
            Some('n') => self.parse_null(),
            Some('t') => self.parse_true(),
            Some('f') => self.parse_false(),
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(self.error(&format!("unexpected character '{}'", c))),
        }
    }
    
    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.expect('n')?;
        self.expect('u')?;
        self.expect('l')?;
        self.expect('l')?;
        Ok(JsonValue::Null)
    }
    
    fn parse_true(&mut self) -> Result<JsonValue, ParseError> {
        self.expect('t')?;
        self.expect('r')?;
        self.expect('u')?;
        self.expect('e')?;
        Ok(JsonValue::Bool(true))
    }
    
    fn parse_false(&mut self) -> Result<JsonValue, ParseError> {
        self.expect('f')?;
        self.expect('a')?;
        self.expect('l')?;
        self.expect('s')?;
        self.expect('e')?;
        Ok(JsonValue::Bool(false))
    }
    
    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect('"')?;
        let mut result = String::new();
        
        loop {
            match self.next() {
                None => return Err(self.error("unterminated string")),
                Some('"') => return Ok(result),
                Some('\\') => {
                    match self.next() {
                        None => return Err(self.error("unterminated escape sequence")),
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some('/') => result.push('/'),
                        Some('b') => result.push('\x08'),
                        Some('f') => result.push('\x0C'),
                        Some('n') => result.push('\n'),
                        Some('r') => result.push('\r'),
                        Some('t') => result.push('\t'),
                        Some('u') => {
                            let hex = self.parse_hex_escape()?;
                            if let Some(c) = char::from_u32(hex) {
                                result.push(c);
                            } else {
                                return Err(self.error("invalid unicode escape"));
                            }
                        }
                        Some(c) => return Err(self.error(&format!("invalid escape '\\{}'", c))),
                    }
                }
                Some(c) if c.is_control() => {
                    return Err(self.error("control characters not allowed in strings"));
                }
                Some(c) => result.push(c),
            }
        }
    }
    
    fn parse_hex_escape(&mut self) -> Result<u32, ParseError> {
        let mut value = 0u32;
        for _ in 0..4 {
            match self.next() {
                None => return Err(self.error("incomplete unicode escape")),
                Some(c) => {
                    let digit = c.to_digit(16)
                        .ok_or_else(|| self.error("invalid hex digit in unicode escape"))?;
                    value = value * 16 + digit;
                }
            }
        }
        Ok(value)
    }
    
    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        // Get start position from peek, not current position
        let start = self.chars.peek().map(|(i, _)| *i).unwrap_or(self.input.len());
        let mut is_float = false;
        
        // Optional minus
        if self.peek() == Some('-') {
            self.next();
        }
        
        // Integer part
        match self.peek() {
            Some('0') => {
                self.next();
            }
            Some(c) if c.is_ascii_digit() => {
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        self.next();
                    } else {
                        break;
                    }
                }
            }
            _ => return Err(self.error("invalid number")),
        }
        
        // Fractional part
        if self.peek() == Some('.') {
            is_float = true;
            self.next();
            
            if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                return Err(self.error("expected digit after decimal point"));
            }
            
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.next();
                } else {
                    break;
                }
            }
        }
        
        // Exponent part
        if matches!(self.peek(), Some('e') | Some('E')) {
            is_float = true;
            self.next();
            
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.next();
            }
            
            if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                return Err(self.error("expected digit in exponent"));
            }
            
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.next();
                } else {
                    break;
                }
            }
        }
        
        let end = self.chars.peek().map(|(i, _)| *i).unwrap_or(self.input.len());
        let num_str = &self.input[start..end];
        
        if is_float {
            num_str.parse::<f64>()
                .map(|f| JsonValue::Number(JsonNumber::Float(f)))
                .map_err(|_| self.error("invalid float"))
        } else {
            num_str.parse::<i64>()
                .map(|n| JsonValue::Number(JsonNumber::Integer(n)))
                .map_err(|_| self.error("integer overflow"))
        }
    }
    
    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect('[')?;
        self.skip_whitespace();
        
        let mut items = Vec::new();
        
        if self.peek() == Some(']') {
            self.next();
            return Ok(JsonValue::Array(items));
        }
        
        loop {
            items.push(self.parse_value()?);
            self.skip_whitespace();
            
            match self.peek() {
                Some(',') => {
                    self.next();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.next();
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err(self.error("expected ',' or ']' in array")),
            }
        }
    }
    
    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect('{')?;
        self.skip_whitespace();
        
        let mut items: JsonObject = Vec::new();
        
        if self.peek() == Some('}') {
            self.next();
            return Ok(JsonValue::Object(items));
        }
        
        loop {
            self.skip_whitespace();
            
            if self.peek() != Some('"') {
                return Err(self.error("expected string key in object"));
            }
            
            let key = self.parse_string()?;
            
            self.skip_whitespace();
            self.expect(':')?;
            
            let value = self.parse_value()?;
            items.push((key, value));
            
            self.skip_whitespace();
            
            match self.peek() {
                Some(',') => {
                    self.next();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.next();
                    return Ok(JsonValue::Object(items));
                }
                _ => return Err(self.error("expected ',' or '}' in object")),
            }
        }
    }
}

/// Parse a JSON string into a JsonValue
pub fn parse(input: &str) -> Result<JsonValue, ParseError> {
    let mut parser = Parser::new(input);
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    
    if parser.peek().is_some() {
        return Err(parser.error("unexpected content after JSON value"));
    }
    
    Ok(value)
}

// ============================================================================
// JSON Encoder
// ============================================================================

/// Encode a JsonValue to a JSON string
pub fn encode(value: &JsonValue) -> String {
    let mut output = String::new();
    encode_value(value, &mut output);
    output
}

/// Encode with pretty printing
pub fn encode_pretty(value: &JsonValue) -> String {
    let mut output = String::new();
    encode_value_pretty(value, &mut output, 0);
    output
}

fn encode_value(value: &JsonValue, output: &mut String) {
    match value {
        JsonValue::Null => output.push_str("null"),
        JsonValue::Bool(true) => output.push_str("true"),
        JsonValue::Bool(false) => output.push_str("false"),
        JsonValue::Number(JsonNumber::Integer(n)) => {
            output.push_str(&n.to_string());
        }
        JsonValue::Number(JsonNumber::Float(f)) => {
            if f.is_finite() {
                output.push_str(&f.to_string());
            } else {
                output.push_str("null"); // JSON doesn't support Infinity/NaN
            }
        }
        JsonValue::String(s) => encode_string(s, output),
        JsonValue::Array(arr) => {
            output.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                encode_value(item, output);
            }
            output.push(']');
        }
        JsonValue::Object(obj) => {
            output.push('{');
            for (i, (key, val)) in obj.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                encode_string(key, output);
                output.push(':');
                encode_value(val, output);
            }
            output.push('}');
        }
    }
}

fn encode_value_pretty(value: &JsonValue, output: &mut String, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let next_indent = "  ".repeat(indent + 1);
    
    match value {
        JsonValue::Null => output.push_str("null"),
        JsonValue::Bool(true) => output.push_str("true"),
        JsonValue::Bool(false) => output.push_str("false"),
        JsonValue::Number(JsonNumber::Integer(n)) => {
            output.push_str(&n.to_string());
        }
        JsonValue::Number(JsonNumber::Float(f)) => {
            if f.is_finite() {
                output.push_str(&f.to_string());
            } else {
                output.push_str("null");
            }
        }
        JsonValue::String(s) => encode_string(s, output),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                output.push_str("[]");
            } else {
                output.push_str("[\n");
                for (i, item) in arr.iter().enumerate() {
                    output.push_str(&next_indent);
                    encode_value_pretty(item, output, indent + 1);
                    if i < arr.len() - 1 {
                        output.push(',');
                    }
                    output.push('\n');
                }
                output.push_str(&indent_str);
                output.push(']');
            }
        }
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                output.push_str("{}");
            } else {
                output.push_str("{\n");
                for (i, (key, val)) in obj.iter().enumerate() {
                    output.push_str(&next_indent);
                    encode_string(key, output);
                    output.push_str(": ");
                    encode_value_pretty(val, output, indent + 1);
                    if i < obj.len() - 1 {
                        output.push(',');
                    }
                    output.push('\n');
                }
                output.push_str(&indent_str);
                output.push('}');
            }
        }
    }
}

fn encode_string(s: &str, output: &mut String) {
    output.push('"');
    for c in s.chars() {
        match c {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_control() => {
                output.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => output.push(c),
        }
    }
    output.push('"');
}

// ============================================================================
// JSON Builder (fluent API)
// ============================================================================

/// Builder for creating JSON objects
pub struct ObjectBuilder {
    items: JsonObject,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        ObjectBuilder { items: Vec::new() }
    }
    
    pub fn set<V: Into<JsonValue>>(mut self, key: &str, value: V) -> Self {
        self.items.push((key.to_string(), value.into()));
        self
    }
    
    pub fn set_if<V: Into<JsonValue>>(self, condition: bool, key: &str, value: V) -> Self {
        if condition {
            self.set(key, value)
        } else {
            self
        }
    }
    
    pub fn build(self) -> JsonValue {
        JsonValue::Object(self.items)
    }
}

/// Builder for creating JSON arrays
pub struct ArrayBuilder {
    items: Vec<JsonValue>,
}

impl ArrayBuilder {
    pub fn new() -> Self {
        ArrayBuilder { items: Vec::new() }
    }
    
    pub fn push<V: Into<JsonValue>>(mut self, value: V) -> Self {
        self.items.push(value.into());
        self
    }
    
    pub fn extend<I, V>(mut self, iter: I) -> Self 
    where
        I: IntoIterator<Item = V>,
        V: Into<JsonValue>,
    {
        self.items.extend(iter.into_iter().map(|v| v.into()));
        self
    }
    
    pub fn build(self) -> JsonValue {
        JsonValue::Array(self.items)
    }
}

/// Create a JSON object using the builder pattern
pub fn object() -> ObjectBuilder {
    ObjectBuilder::new()
}

/// Create a JSON array using the builder pattern
pub fn array() -> ArrayBuilder {
    ArrayBuilder::new()
}

// ============================================================================
// Macros for JSON literals
// ============================================================================

/// Create a JSON value from a literal
#[macro_export]
macro_rules! json {
    (null) => { $crate::json::JsonValue::Null };
    (true) => { $crate::json::JsonValue::Bool(true) };
    (false) => { $crate::json::JsonValue::Bool(false) };
    ($e:expr) => { $crate::json::JsonValue::from($e) };
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_null() {
        assert_eq!(parse("null").unwrap(), JsonValue::Null);
    }
    
    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse("false").unwrap(), JsonValue::Bool(false));
    }
    
    #[test]
    fn test_parse_numbers() {
        assert_eq!(parse("42").unwrap(), JsonValue::Number(JsonNumber::Integer(42)));
        assert_eq!(parse("-17").unwrap(), JsonValue::Number(JsonNumber::Integer(-17)));
        assert_eq!(parse("3.14").unwrap(), JsonValue::Number(JsonNumber::Float(3.14)));
        assert_eq!(parse("1e10").unwrap(), JsonValue::Number(JsonNumber::Float(1e10)));
        assert_eq!(parse("-2.5e-3").unwrap(), JsonValue::Number(JsonNumber::Float(-2.5e-3)));
    }
    
    #[test]
    fn test_parse_strings() {
        assert_eq!(parse(r#""hello""#).unwrap(), JsonValue::String("hello".to_string()));
        assert_eq!(parse(r#""hello\nworld""#).unwrap(), JsonValue::String("hello\nworld".to_string()));
        assert_eq!(parse(r#""tab\there""#).unwrap(), JsonValue::String("tab\there".to_string()));
        assert_eq!(parse(r#""quote\"here""#).unwrap(), JsonValue::String("quote\"here".to_string()));
        assert_eq!(parse(r#""\u0041""#).unwrap(), JsonValue::String("A".to_string()));
    }
    
    #[test]
    fn test_parse_arrays() {
        assert_eq!(parse("[]").unwrap(), JsonValue::Array(vec![]));
        assert_eq!(
            parse("[1, 2, 3]").unwrap(),
            JsonValue::Array(vec![
                JsonValue::Number(JsonNumber::Integer(1)),
                JsonValue::Number(JsonNumber::Integer(2)),
                JsonValue::Number(JsonNumber::Integer(3)),
            ])
        );
    }
    
    #[test]
    fn test_parse_objects() {
        assert_eq!(parse("{}").unwrap(), JsonValue::Object(vec![]));
        
        let obj = parse(r#"{"name": "TAYNI", "version": 1}"#).unwrap();
        assert_eq!(obj.get("name").unwrap().as_str(), Some("TAYNI"));
        assert_eq!(obj.get("version").unwrap().as_i64(), Some(1));
    }
    
    #[test]
    fn test_parse_nested() {
        let json = r#"{
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ],
            "count": 2
        }"#;
        
        let value = parse(json).unwrap();
        let users = value.get("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].get("name").unwrap().as_str(), Some("Alice"));
    }
    
    #[test]
    fn test_encode_roundtrip() {
        let original = r#"{"name":"test","values":[1,2,3],"nested":{"a":true}}"#;
        let parsed = parse(original).unwrap();
        let encoded = encode(&parsed);
        let reparsed = parse(&encoded).unwrap();
        assert_eq!(parsed, reparsed);
    }
    
    #[test]
    fn test_builder() {
        let value = object()
            .set("name", "TAYNI")
            .set("version", 1i64)
            .set("features", array()
                .push("fast")
                .push("safe")
                .build())
            .build();
        
        assert_eq!(value.get("name").unwrap().as_str(), Some("TAYNI"));
        assert_eq!(value.get("version").unwrap().as_i64(), Some(1));
    }
    
    #[test]
    fn test_error_messages() {
        let err = parse("{invalid}").unwrap_err();
        assert!(err.message.contains("expected string key"));
        
        let err = parse("[1, 2, ]").unwrap_err();
        assert!(err.message.contains("unexpected"));
    }
    
    #[test]
    fn test_pretty_print() {
        let value = object()
            .set("name", "test")
            .set("items", array().push(1i64).push(2i64).build())
            .build();
        
        let pretty = encode_pretty(&value);
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }
    
    #[test]
    fn test_modify_object() {
        let mut value = parse(r#"{"a": 1, "b": 2}"#).unwrap();
        value.set("c", JsonValue::Number(JsonNumber::Integer(3)));
        assert_eq!(value.get("c").unwrap().as_i64(), Some(3));
        
        value.set("a", JsonValue::Number(JsonNumber::Integer(10)));
        assert_eq!(value.get("a").unwrap().as_i64(), Some(10));
        
        let removed = value.remove("b");
        assert_eq!(removed.unwrap().as_i64(), Some(2));
        assert!(value.get("b").is_none());
    }
}
