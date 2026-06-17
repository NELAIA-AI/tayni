//! TAYNI Structured Intent
//! Converts structured intent JSON to TAYNI code

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Intent {
    pub intent_type: String,
    pub requirements: HashMap<String, String>,
    pub constraints: Constraints,
}

#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub max_memory_bytes: Option<u64>,
    pub max_binary_size_bytes: Option<u64>,
    pub timeout_ms: Option<u64>,
}

/// Simple JSON value for parsing
#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(i64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

/// Parse a simple JSON string (minimal implementation)
fn parse_json_value(s: &str) -> Result<(JsonValue, &str), String> {
    let s = s.trim_start();
    if s.is_empty() {
        return Err("Unexpected end of input".to_string());
    }
    
    match s.chars().next().unwrap() {
        '"' => {
            let end = s[1..].find('"').ok_or("Unterminated string")?;
            let val = &s[1..end + 1];
            Ok((JsonValue::String(val.to_string()), &s[end + 2..]))
        }
        '{' => {
            let mut obj = HashMap::new();
            let mut rest = s[1..].trim_start();
            
            if rest.starts_with('}') {
                return Ok((JsonValue::Object(obj), &rest[1..]));
            }
            
            loop {
                let (key, r) = parse_json_value(rest)?;
                let key = match key {
                    JsonValue::String(s) => s,
                    _ => return Err("Object key must be string".to_string()),
                };
                
                rest = r.trim_start();
                if !rest.starts_with(':') {
                    return Err("Expected ':'".to_string());
                }
                rest = rest[1..].trim_start();
                
                let (val, r) = parse_json_value(rest)?;
                obj.insert(key, val);
                rest = r.trim_start();
                
                if rest.starts_with('}') {
                    return Ok((JsonValue::Object(obj), &rest[1..]));
                }
                if rest.starts_with(',') {
                    rest = rest[1..].trim_start();
                } else {
                    return Err("Expected ',' or '}'".to_string());
                }
            }
        }
        '[' => {
            let mut arr = Vec::new();
            let mut rest = s[1..].trim_start();
            
            if rest.starts_with(']') {
                return Ok((JsonValue::Array(arr), &rest[1..]));
            }
            
            loop {
                let (val, r) = parse_json_value(rest)?;
                arr.push(val);
                rest = r.trim_start();
                
                if rest.starts_with(']') {
                    return Ok((JsonValue::Array(arr), &rest[1..]));
                }
                if rest.starts_with(',') {
                    rest = rest[1..].trim_start();
                } else {
                    return Err("Expected ',' or ']'".to_string());
                }
            }
        }
        c if c.is_ascii_digit() || c == '-' => {
            let end = s.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(s.len());
            let num: i64 = s[..end].parse().map_err(|_| "Invalid number")?;
            Ok((JsonValue::Number(num), &s[end..]))
        }
        't' if s.starts_with("true") => Ok((JsonValue::Bool(true), &s[4..])),
        'f' if s.starts_with("false") => Ok((JsonValue::Bool(false), &s[5..])),
        'n' if s.starts_with("null") => Ok((JsonValue::Null, &s[4..])),
        _ => Err(format!("Unexpected character: {}", s.chars().next().unwrap())),
    }
}

/// Parse intent from JSON string
pub fn parse_intent(json: &str) -> Result<Intent, String> {
    let (value, _) = parse_json_value(json)?;
    
    let obj = match value {
        JsonValue::Object(o) => o,
        _ => return Err("Intent must be an object".to_string()),
    };
    
    let intent_type = match obj.get("intent") {
        Some(JsonValue::String(s)) => s.clone(),
        _ => return Err("Missing 'intent' field".to_string()),
    };
    
    let mut requirements = HashMap::new();
    if let Some(JsonValue::Object(reqs)) = obj.get("requirements") {
        for (k, v) in reqs {
            let val = match v {
                JsonValue::String(s) => s.clone(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::Bool(b) => b.to_string(),
                _ => continue,
            };
            requirements.insert(k.clone(), val);
        }
    }
    
    let mut constraints = Constraints::default();
    if let Some(JsonValue::Object(cons)) = obj.get("constraints") {
        if let Some(JsonValue::Number(n)) = cons.get("max_memory_bytes") {
            constraints.max_memory_bytes = Some(*n as u64);
        }
        if let Some(JsonValue::Number(n)) = cons.get("max_binary_size_bytes") {
            constraints.max_binary_size_bytes = Some(*n as u64);
        }
        if let Some(JsonValue::Number(n)) = cons.get("timeout_ms") {
            constraints.timeout_ms = Some(*n as u64);
        }
    }
    
    Ok(Intent { intent_type, requirements, constraints })
}

/// Generate TAYNI code from intent
pub fn generate_tayni_from_intent(intent: &Intent) -> Result<String, String> {
    match intent.intent_type.as_str() {
        "http_server" => generate_http_server(intent),
        "cli_tool" => generate_cli_tool(intent),
        "file_processor" => generate_file_processor(intent),
        "hello_world" => generate_hello_world(intent),
        _ => Err(format!("Unknown intent type: {}", intent.intent_type)),
    }
}

fn generate_http_server(intent: &Intent) -> Result<String, String> {
    let port = intent.requirements.get("port")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(8080);
    
    let mut code = String::new();
    code.push_str("-- TAYNI HTTP Server (auto-generated from intent)\n");
    code.push_str("USE http\n");
    code.push_str("USE json\n\n");
    
    code.push_str(&format!(".port: {}\n", port));
    code.push_str(".server: HTTP.LISTEN .port\n\n");
    
    code.push_str(".path0: \"/\"\n");
    code.push_str(".resp0: \"Hello from TAYNI!\"\n");
    
    code.push_str("\n#LOOP\n");
    code.push_str("  .client: HTTP.ACCEPT .server\n");
    code.push_str("  .req_buf: ALC 4096\n");
    code.push_str("  .req_len: RCV .client .req_buf 4096\n");
    code.push_str("  .status: 200\n");
    code.push_str("  .body: \"OK\"\n");
    code.push_str("  .send: HTTP.RESPOND .client .status .body\n");
    code.push_str("  .close: CLS .client\n");
    code.push_str("#END\n\n");
    code.push_str("!\n");
    
    Ok(code)
}

fn generate_cli_tool(intent: &Intent) -> Result<String, String> {
    let name = intent.requirements.get("name")
        .map(|s| s.as_str())
        .unwrap_or("tool");
    
    let mut code = String::new();
    code.push_str(&format!("-- TAYNI CLI Tool: {} (auto-generated from intent)\n", name));
    code.push_str("USE args\n");
    code.push_str("USE file\n\n");
    
    code.push_str(".argc: ARGS.COUNT\n");
    code.push_str(".help_msg: \"Usage: ");
    code.push_str(name);
    code.push_str(" [options]\\n\"\n");
    code.push_str(".help_len: ");
    code.push_str(&format!("{}\n", name.len() + 20));
    
    code.push_str("\n.check_help: IFZ .argc 1 0\n");
    code.push_str(".show_help: BRN .check_help .help_msg .empty\n");
    code.push_str(".out: PRT .show_help .help_len\n\n");
    code.push_str("!\n");
    
    Ok(code)
}

fn generate_file_processor(_intent: &Intent) -> Result<String, String> {
    let mut code = String::new();
    code.push_str("-- TAYNI File Processor (auto-generated from intent)\n");
    code.push_str("USE file\n");
    code.push_str("USE args\n\n");
    
    code.push_str(".input_file: ARGS.GET 1\n");
    code.push_str(".content: FILE.READ .input_file\n");
    code.push_str(".out: PRT .content\n\n");
    code.push_str("!\n");
    
    Ok(code)
}

fn generate_hello_world(_intent: &Intent) -> Result<String, String> {
    let mut code = String::new();
    code.push_str("-- TAYNI Hello World (auto-generated from intent)\n\n");
    code.push_str(".msg: \"Hello, World!\\n\"\n");
    code.push_str(".len: 14\n");
    code.push_str(".out: PRT .msg .len\n\n");
    code.push_str("!\n");
    
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_http_server_intent() {
        let json = r#"{"intent": "http_server", "requirements": {"port": "8080"}, "constraints": {"max_binary_size_bytes": 10240}}"#;
        
        let intent = parse_intent(json).unwrap();
        assert_eq!(intent.intent_type, "http_server");
        assert_eq!(intent.constraints.max_binary_size_bytes, Some(10240));
    }
    
    #[test]
    fn test_generate_hello_world() {
        let intent = Intent {
            intent_type: "hello_world".to_string(),
            requirements: HashMap::new(),
            constraints: Constraints::default(),
        };
        
        let code = generate_tayni_from_intent(&intent).unwrap();
        assert!(code.contains("Hello, World!"));
    }
}
