//! TAYNI v1.5 Parser
//! Parses data flow graph syntax into IR
//! Supports both v1.0 and v1.5 syntax with automatic detection
//! v1.5 features: compound operators, direct jumps, for/loop/guard, destructuring

use crate::ir::*;
use std::collections::HashMap;

/// Detected syntax version
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyntaxVersion {
    V1_0,  // Original: @.x: ADD .a .b
    V1_5,  // New: x = add(a, b)
}

/// GEN definition: name, parameters, body lines (subgraph pattern)
#[derive(Clone, Debug)]
struct MacroDef {
    params: Vec<String>,
    body: Vec<String>,
}

pub struct Parser;

impl Parser {
    /// Detect syntax version from content
    pub fn detect_version(content: &str) -> SyntaxVersion {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("--") {
                continue;
            }
            // v1.5 indicators
            if line.starts_with("fn ") || line.starts_with("for ") || 
               line.starts_with("loop") || line.starts_with("guard ") ||
               line.contains(" += ") || line.contains(" -= ") ||
               line.contains(" *= ") || line.contains(" /= ") ||
               line.starts_with("use ") ||
               (line.contains(" = ") && !line.starts_with("@") && !line.starts_with(".")) {
                return SyntaxVersion::V1_5;
            }
            // v1.0 indicators
            if line.starts_with("@.") || line.starts_with(".") || line.starts_with("USE ") {
                return SyntaxVersion::V1_0;
            }
        }
        SyntaxVersion::V1_0 // Default to v1.0
    }

    pub fn parse(content: &str) -> Result<Graph, String> {
        let version = Self::detect_version(content);
        match version {
            SyntaxVersion::V1_0 => Self::parse_v1_0(content),
            SyntaxVersion::V1_5 => Self::parse_v1_5(content),
        }
    }
    
    /// Parse v1.0 syntax (original)
    fn parse_v1_0(content: &str) -> Result<Graph, String> {
        let mut graph = Graph::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_function: Option<(String, Vec<String>, Vec<Node>)> = None;
        let mut macros: HashMap<String, MacroDef> = HashMap::new();
        let mut gen_counter: u32 = 0;
        
        // First pass: collect macro definitions
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with('#') && line.contains(':') && !line.starts_with("#END") {
                let (name, def) = Self::parse_macro_def(&lines, &mut i)?;
                macros.insert(name, def);
                continue;
            }
            i += 1;
        }
        
        // Second pass: parse with macro expansion
        i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("--") {
                i += 1;
                continue;
            }
            
            // USE directive: USE module_name
            if line.starts_with("USE ") {
                let module = line[4..].trim().to_lowercase();
                if module.is_empty() {
                    return Err(format!("Line {}: USE requires a module name", i + 1));
                }
                if current_function.is_some() {
                    return Err(format!("Line {}: USE must be at top level, not inside functions", i + 1));
                }
                graph.add_node(Node::Use { module });
                i += 1;
                continue;
            }
            
            // Skip macro definitions (already processed)
            if line.starts_with('#') && !line.starts_with("#END") {
                while i < lines.len() && !lines[i].trim().starts_with("#END") {
                    i += 1;
                }
                i += 1; // skip #END
                continue;
            }
            
            // GEN call: !NAME args (generate and fuse subgraph)
            if line.starts_with('!') && !line.starts_with("!FUN") && line != "!" {
                let expanded = Self::expand_gen(line, &macros, &mut gen_counter)?;
                for exp_line in expanded {
                    match Self::parse_line(&exp_line, &mut vec![], &mut 0) {
                        Ok(Some(node)) => {
                            if let Some((_, _, ref mut body)) = current_function {
                                body.push(node);
                            } else {
                                graph.add_node(node);
                            }
                        }
                        Ok(None) => {},
                        Err(e) => return Err(format!("Macro expansion error: {}", e)),
                    }
                }
                i += 1;
                continue;
            }
            
            // End of program
            if line == "!" {
                if current_function.is_some() {
                    return Err(format!("Line {}: Unexpected '!' inside function", i + 1));
                }
                i += 1;
                continue;
            }
            
            // End of function
            if line == "!FUN" {
                if let Some((id, params, body)) = current_function.take() {
                    graph.add_node(Node::Function { id, params, body });
                } else {
                    return Err(format!("Line {}: !FUN without matching FUN", i + 1));
                }
                i += 1;
                continue;
            }
            
            // Parse the line
            let mut lines_mut = lines.clone();
            match Self::parse_line(line, &mut lines_mut, &mut i) {
                Ok(Some(node)) => {
                    // Check if this is a function definition start
                    if let Node::Function { id, params, body: _ } = &node {
                        current_function = Some((id.clone(), params.clone(), Vec::new()));
                    } else if let Some((_, _, ref mut body)) = current_function {
                        body.push(node);
                    } else {
                        graph.add_node(node);
                    }
                }
                Ok(None) => {},
                Err(e) => return Err(format!("Line {}: {}", i + 1, e)),
            }
            
            i += 1;
        }
        
        if current_function.is_some() {
            return Err("Unclosed function (missing !FUN)".to_string());
        }
        
        Ok(graph)
    }
    
    fn parse_line(line: &str, _lines: &mut Vec<&str>, _i: &mut usize) -> Result<Option<Node>, String> {
        let line = Self::strip_comment(line);
        
        // Flow expression: something > something
        if line.contains(" > ") {
            return Self::parse_flow(line);
        }
        
        // Runtime node definition: @.id: value/operation (NEW: @ prefix for runtime)
        if line.starts_with("@.") && line.contains(':') {
            return Self::parse_node_def(&line[1..], true); // skip @, pass runtime=true
        }
        
        // Node definition: .id: value/operation
        if line.starts_with('.') && line.contains(':') {
            return Self::parse_node_def(line, false); // runtime=false
        }
        
        // Standalone expression (like "Hello" > PRT)
        if line.contains('>') {
            return Self::parse_flow(line);
        }
        
        // Label definition: :name
        if line.starts_with(':') {
            let label = line[1..].trim().to_string();
            return Ok(Some(Node::Label(label)));
        }
        
        Err(format!("Unrecognized syntax: {}", line))
    }
    
    fn strip_comment(line: &str) -> &str {
        // Find -- that is not inside a string
        let mut in_string = false;
        let chars: Vec<char> = line.chars().collect();
        for i in 0..chars.len() {
            if chars[i] == '"' {
                in_string = !in_string;
            } else if !in_string && i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '-' {
                return line[..i].trim();
            }
        }
        line.trim()
    }
    
    fn parse_node_def(line: &str, runtime: bool) -> Result<Option<Node>, String> {
        // Find the colon that separates id from value
        let colon_pos = line.find(':').ok_or("Expected ':' in node definition")?;
        let id = line[1..colon_pos].trim().to_string();
        let rest = line[colon_pos + 1..].trim();
        
        // Check if it's a function definition: FUN .param1 .param2 ...
        if rest.starts_with("FUN ") || rest == "FUN" {
            let params: Vec<String> = if rest.len() > 4 {
                rest[4..].split_whitespace()
                    .filter(|s| s.starts_with('.'))
                    .map(|s| s[1..].to_string())
                    .collect()
            } else {
                Vec::new()
            };
            return Ok(Some(Node::Function { id, params, body: Vec::new() }));
        }
        
        // Check if it's a sub-graph
        if rest.starts_with('{') {
            return Self::parse_subgraph(&id, rest);
        }
        
        // Check if it's a REQUIRES declaration
        if rest.starts_with("REQUIRES") || rest.starts_with("REQ") {
            return Self::parse_requires(&id, rest);
        }
        
        // Check if it's a reference to another node
        if rest.starts_with('.') && !rest.contains(' ') {
            let target = rest[1..].to_string();
            return Ok(Some(Node::Reference { id, target }));
        }
        
        // Check if it's a literal
        if let Ok(value) = Self::parse_literal(rest) {
            return Ok(Some(Node::Literal { id, value, runtime }));
        }
        
        // Must be an operation
        Self::parse_operation(&id, rest, runtime)
    }
    
    fn parse_literal(s: &str) -> Result<Value, String> {
        let s = s.trim();
        
        // String literal
        if s.starts_with('"') && s.ends_with('"') {
            let inner = &s[1..s.len()-1];
            // Handle escape sequences
            let unescaped = inner
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
                .replace("\\0", "\0")
                .replace("\\\"", "\"");
            return Ok(Value::String(unescaped));
        }
        
        // Pair literal: (a b) or (a, b)
        if s.starts_with('(') && s.ends_with(')') {
            let inner = &s[1..s.len()-1];
            let parts: Vec<&str> = inner.split_whitespace().collect();
            if parts.len() == 2 {
                let a = Self::parse_literal(parts[0])?;
                let b = Self::parse_literal(parts[1])?;
                return Ok(Value::Pair(Box::new(a), Box::new(b)));
            }
        }
        
        // Float literal
        if s.contains('.') {
            if let Ok(f) = s.parse::<f64>() {
                return Ok(Value::Float(f));
            }
        }
        
        // Integer literal
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Int(i));
        }
        
        Err(format!("Cannot parse literal: {}", s))
    }
    
    fn parse_requires(id: &str, rest: &str) -> Result<Option<Node>, String> {
        // Parse: REQUIRES { http, sql, json } or REQUIRES http sql json
        let content = rest.trim_start_matches("REQUIRES").trim_start_matches("REQ").trim();
        
        let cap_strs: Vec<&str> = if content.starts_with('{') && content.ends_with('}') {
            // Brace syntax: { http, sql, json }
            content[1..content.len()-1]
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // Space-separated: http sql json
            content.split_whitespace().collect()
        };
        
        let mut capabilities = Vec::new();
        for cap_str in cap_strs {
            let cap = Self::parse_capability(cap_str)?;
            capabilities.push(cap);
        }
        
        Ok(Some(Node::Requires {
            id: id.to_string(),
            capabilities,
        }))
    }
    
    fn parse_capability(s: &str) -> Result<Capability, String> {
        match s.to_lowercase().as_str() {
            "math" => Ok(Capability::Math),
            "memory" | "mem" => Ok(Capability::Memory),
            "io" => Ok(Capability::IO),
            "http" | "http:server" | "http:client" => {
                if s.contains("client") {
                    Ok(Capability::HttpClient)
                } else if s.contains("server") {
                    Ok(Capability::HttpServer)
                } else {
                    Ok(Capability::HttpServer) // Default to server
                }
            }
            "http:server" => Ok(Capability::HttpServer),
            "http:client" => Ok(Capability::HttpClient),
            "tcp" | "tcp:raw" => Ok(Capability::TcpRaw),
            "json" => Ok(Capability::Json),
            "xml" => Ok(Capability::Xml),
            "sql" => Ok(Capability::Sql),
            "sql:readonly" | "sql:read" => Ok(Capability::SqlReadOnly),
            "filesystem" | "fs" => Ok(Capability::FileSystem),
            "filesystem:readonly" | "fs:readonly" | "fs:read" => Ok(Capability::FileSystemReadOnly),
            "threading" | "threads" => Ok(Capability::Threading),
            "gui" => Ok(Capability::Gui),
            other => Ok(Capability::Custom(other.to_string())),
        }
    }
    
    fn parse_operation(id: &str, rest: &str, runtime: bool) -> Result<Option<Node>, String> {
        let tokens = Self::tokenize(rest);
        if tokens.is_empty() {
            return Err("Empty operation".to_string());
        }
        
        let op = Self::parse_op(&tokens[0])?;
        let args = Self::parse_args(&tokens[1..])?;
        
        Ok(Some(Node::Operation {
            id: id.to_string(),
            op,
            args,
            runtime,
        }))
    }
    
    fn parse_op(s: &str) -> Result<Op, String> {
        match s.to_uppercase().as_str() {
            "ADD" => Ok(Op::Add),
            "SUB" => Ok(Op::Sub),
            "MUL" => Ok(Op::Mul),
            "DIV" => Ok(Op::Div),
            "MOD" => Ok(Op::Mod),
            "NEG" => Ok(Op::Neg),
            "EQ" => Ok(Op::Eq),
            "NE" => Ok(Op::Ne),
            "LT" => Ok(Op::Lt),
            "GT" => Ok(Op::Gt),
            "LE" => Ok(Op::Le),
            "GE" => Ok(Op::Ge),
            "AND" => Ok(Op::And),
            "OR" => Ok(Op::Or),
            "NOT" => Ok(Op::Not),
            "SEQ" => Ok(Op::Seq),
            "MAP" => Ok(Op::Map),
            "FLD" => Ok(Op::Fld),
            "FLT" => Ok(Op::Flt),
            "LEN" => Ok(Op::Len),
            "FST" => Ok(Op::Fst),
            "SND" => Ok(Op::Snd),
            "BRN" => Ok(Op::Brn),
            "JMP" => Ok(Op::Jmp),
            "JZ" => Ok(Op::Jz),
            "JNZ" => Ok(Op::Jnz),
            "WHL" => Ok(Op::Whl),
            "END" => Ok(Op::End),
            "TRN" => Ok(Op::Trn),
            "FSM" => Ok(Op::Fsm),
            "PSC" => Ok(Op::Psc),
            "AST" => Ok(Op::Ast),
            "EMT" => Ok(Op::Emt),
            // LOOP deprecated - use cyclic flow >> instead
            "PRT" => Ok(Op::Prt),
            "INP" => Ok(Op::Inp),
            "OPN" => Ok(Op::Opn),
            "ACC" => Ok(Op::Acc),
            "GET" => Ok(Op::Get),
            "GE8" => Ok(Op::Ge8),
            "PUT" => Ok(Op::Put),
            "CLS" => Ok(Op::Cls),
            "ERR" => Ok(Op::Err),
            // Network
            "TCP" => Ok(Op::Tcp),
            "UDP" => Ok(Op::Udp),
            "BND" => Ok(Op::Bnd),
            "LST" => Ok(Op::Lst),
            "CON" => Ok(Op::Con),
            "XMT" => Ok(Op::Xmt),  // Transmit (send)
            "RCV" => Ok(Op::Rcv),
            // Memory
            "ALC" => Ok(Op::Alc),
            "FRE" => Ok(Op::Fre),
            // Memory operations (self-hosting)
            "CPY" => Ok(Op::Cpy),
            "CMP" => Ok(Op::Cmp),
            "FND" => Ok(Op::Fnd),
            "SLN" => Ok(Op::Sln),
            // File I/O (self-hosting)
            "FOP" => Ok(Op::Fop),
            "FRD" => Ok(Op::Frd),
            "FWR" => Ok(Op::Fwr),
            "FCL" => Ok(Op::Fcl),
            // Command Line Arguments
            "ARGC" => Ok(Op::Argc),
            "ARGV" => Ok(Op::Argv),
            // Random Numbers
            "RND" => Ok(Op::Rnd),
            "RNG" => Ok(Op::Rng),
            // Logging
            "LOG" => Ok(Op::Log),
            // HTTP Routing
            "ROUTE" => Ok(Op::Route),
            // Environment Variables
            "GETENV" => Ok(Op::GetEnv),
            // Path Operations
            "PATH.JOIN" => Ok(Op::PathJoin),
            "PATH.DIR" => Ok(Op::PathDir),
            "PATH.BASE" => Ok(Op::PathBase),
            "PATH.EXT" => Ok(Op::PathExt),
            // Hash Operations
            "HASH.MD5" => Ok(Op::HashMd5),
            "HASH.SHA256" => Ok(Op::HashSha256),
            // Time Formatting Operations
            "TIME.FMT" => Ok(Op::TimeFmt),
            "TIME.YEAR" => Ok(Op::TimeYear),
            "TIME.MONTH" => Ok(Op::TimeMonth),
            "TIME.DAY" => Ok(Op::TimeDay),
            "TIME.HOUR" => Ok(Op::TimeHour),
            "TIME.MIN" => Ok(Op::TimeMin),
            "TIME.SEC" => Ok(Op::TimeSec),
            // Dynamic Vectors
            "VEC" => Ok(Op::Vec),
            "VPH" => Ok(Op::Vph),
            "VGT" => Ok(Op::Vgt),
            "VST" => Ok(Op::Vst),
            "VLN" => Ok(Op::Vln),
            "VCP" => Ok(Op::Vcp),
            // HashMap (self-hosting)
            "HMP" => Ok(Op::Hmp),
            "HPT" => Ok(Op::Hpt),
            "HGT" => Ok(Op::Hgt),
            "HHS" => Ok(Op::Hhs),
            // String operations (self-hosting)
            "CAT" => Ok(Op::Cat),
            "ITS" => Ok(Op::Its),
            "CHR" => Ok(Op::Chr),
            "SBS" => Ok(Op::Sbs),
            "SCM" => Ok(Op::Scm),
            "WRT" => Ok(Op::Wrt),
            "IFZ" => Ok(Op::Ifz),
            // Graph Transform (AI-native iteration)
            "TRN" => Ok(Op::Trn),
            "RED" => Ok(Op::Red),
            "MAP" => Ok(Op::Map),
            "FLT" => Ok(Op::Flt),
            // BRK/CNT deprecated - use cyclic flow >> instead
            // Error handling
            "CHK" => Ok(Op::Chk),
            // Async I/O
            "SEL" => Ok(Op::Sel),
            "RDY" => Ok(Op::Rdy),
            "NBK" => Ok(Op::Nbk),
            // Socket options (ultra-low-latency)
            "NDL" => Ok(Op::Ndl),
            "QCK" => Ok(Op::Qck),
            "SBF" => Ok(Op::Sbf),
            "KAL" => Ok(Op::Kal),
            // High-performance I/O
            "EPL" => Ok(Op::Epl),
            "EWA" => Ok(Op::Ewa),
            "ECT" => Ok(Op::Ect),
            // Threading
            "THR" => Ok(Op::Thr),
            "JON" => Ok(Op::Jon),
            "MTX" => Ok(Op::Mtx),
            "LCK" => Ok(Op::Lck),
            "ULK" => Ok(Op::Ulk),
            "TLK" => Ok(Op::Tlk),
            "YLD" => Ok(Op::Yld),
            // Atomic Operations
            "ATM.LD" => Ok(Op::AtmLd),
            "ATM.ST" => Ok(Op::AtmSt),
            "ATM.XCHG" => Ok(Op::AtmXchg),
            "ATM.CAS" => Ok(Op::AtmCas),
            "ATM.ADD" => Ok(Op::AtmAdd),
            "ATM.SUB" => Ok(Op::AtmSub),
            "FNC" => Ok(Op::Fence),
            // Channels
            "CHN" => Ok(Op::Chn),
            "CHN.SND" => Ok(Op::ChnSnd),
            "CHN.RCV" => Ok(Op::ChnRcv),
            "CHN.CLS" => Ok(Op::ChnCls),
            // Atomic Queue
            "QUE" => Ok(Op::Que),
            "PSH" => Ok(Op::Psh),
            "POP" => Ok(Op::Pop),
            // Functions
            "RET" => Ok(Op::Ret),
            // GUI - Window Management
            "WIN" => Ok(Op::Win),
            "SHW" => Ok(Op::Shw),
            "HID" => Ok(Op::Hid),
            "EVT" => Ok(Op::Evt),
            "RUN" => Ok(Op::Run),
            // GUI - Controls
            "LBL" => Ok(Op::Lbl),
            "TXB" => Ok(Op::Txb),
            "BTN" => Ok(Op::Btn),
            // GUI - Dialogs
            "DLG" => Ok(Op::Dlg),
            // GUI - Control Values
            "GVL" => Ok(Op::Gvl),
            "SVL" => Ok(Op::Svl),
            
            // === CAPABILITY SYSTEM (SCN) ===
            "REQ" | "REQUIRES" => Ok(Op::Req),
            
            // HTTP Capability Operations
            "HTTP.LISTEN" => Ok(Op::HttpListen),
            "HTTP.ACCEPT" => Ok(Op::HttpAccept),
            "HTTP.METHOD" => Ok(Op::HttpMethod),
            "HTTP.PATH" => Ok(Op::HttpPath),
            "HTTP.BODY" => Ok(Op::HttpBody),
            "HTTP.RESPOND" => Ok(Op::HttpRespond),
            "HTTP.GET" => Ok(Op::HttpGet),
            "HTTP.POST" => Ok(Op::HttpPost),
            
            // SQL Capability Operations
            "SQL.CONNECT" => Ok(Op::SqlConnect),
            "SQL.QUERY" => Ok(Op::SqlQuery),
            "SQL.EXEC" => Ok(Op::SqlExec),
            "SQL.NEXT" => Ok(Op::SqlNext),
            "SQL.GET" => Ok(Op::SqlGet),
            "SQL.CLOSE" => Ok(Op::SqlClose),
            
            // JSON Capability Operations
            "JSON.PARSE" => Ok(Op::JsonParse),
            "JSON.ENCODE" => Ok(Op::JsonEncode),
            "JSON.GET" => Ok(Op::JsonGet),
            "JSON.SET" => Ok(Op::JsonSet),
            
            // === PHASE 8: CONTRACTS & NEGOTIATION (IA-first) ===
            
            // Contract operations
            "CONTRACT" => Ok(Op::Contract),
            "GUARANTEE" | "GUARANTEES" => Ok(Op::Guarantee),
            "LIMIT" | "LIMITS" => Ok(Op::Limit),
            "SANDBOX" => Ok(Op::Sandbox),
            
            // Negotiation operations
            "PROVIDES" => Ok(Op::Provides),
            "NEGOTIATE" => Ok(Op::Negotiate),
            "BIND" => Ok(Op::Bind),
            
            // Custom capability operations
            "DEFINE_CAPABILITY" | "DEFCAP" => Ok(Op::DefCap),
            "EXTEND_CAPABILITY" | "EXTCAP" => Ok(Op::ExtendCap),
            "COMPOSE_CAPABILITIES" | "COMPOSE" => Ok(Op::ComposeCap),
            
            // === PHASE 10: TESTING (IA-first) ===
            
            // Property-based testing
            "PROPERTY" | "PROP" => Ok(Op::Property),
            "GENERATE_TESTS" | "GENTESTS" => Ok(Op::GenTests),
            "VERIFY" => Ok(Op::Verify),
            
            // === PHASE 9.2: INCREMENTAL COMPILATION (IA-first) ===
            
            // Content-Addressable Cache operations
            "HASH" => Ok(Op::Hash),
            "CACHE_GET" | "CGET" => Ok(Op::CacheGet),
            "CACHE_PUT" | "CPUT" => Ok(Op::CachePut),
            "CACHE_VERIFY" | "CVERIFY" => Ok(Op::CacheVerify),
            "CACHE_INVALIDATE" | "CINV" => Ok(Op::CacheInvalidate),
            
            // === PHASE 11: SEN - Sistema de Ecosistema TAYNI (IA-first) ===
            "DISCOVER" => Ok(Op::Discover),
            "CAPABILITY_INFO" | "CAP_INFO" => Ok(Op::CapInfo),
            "CAPABILITY_COST" | "CAP_COST" => Ok(Op::CapCost),
            "PUBLISH" => Ok(Op::CapPublish),
            "CAPABILITY_AVAILABLE" | "CAP_AVAIL" => Ok(Op::CapAvailable),
            "CAPABILITY_VERSION" | "CAP_VER" => Ok(Op::CapVersion),
            "CAPABILITY_DEPS" | "CAP_DEPS" => Ok(Op::CapDeps),
            
            // === STDLIB TIER 0: Core Operations ===
            
            // LOG module
            "LOG.INFO" => Ok(Op::LogInfo),
            "LOG.ERROR" => Ok(Op::LogError),
            "LOG.WARN" => Ok(Op::LogWarn),
            "LOG.DEBUG" => Ok(Op::LogDebug),
            
            // ROUTER module
            "ROUTER.MATCH" => Ok(Op::RouterMatch),
            "ROUTER.PARAM" => Ok(Op::RouterParam),
            
            // HTTP module (extended)
            "HTTP.IS_OPTIONS" => Ok(Op::HttpIsOptions),
            "HTTP.PATH_LEN" => Ok(Op::HttpPathLen),
            "HTTP.CLOSE" => Ok(Op::HttpClose),
            
            // CORS module
            "CORS.CONFIG_NEW" => Ok(Op::CorsConfigNew),
            "CORS.ALLOW_ORIGIN" => Ok(Op::CorsAllowOrigin),
            "CORS.ALLOW_ALL_ORIGINS" => Ok(Op::CorsAllowAllOrigins),
            "CORS.ALLOW_METHODS" => Ok(Op::CorsAllowMethods),
            "CORS.ALLOW_HEADERS" => Ok(Op::CorsAllowHeaders),
            "CORS.ALLOW_CREDENTIALS" => Ok(Op::CorsAllowCredentials),
            "CORS.HANDLE" => Ok(Op::CorsHandle),
            "CORS.HANDLE_PREFLIGHT" => Ok(Op::CorsHandlePreflight),
            "CORS.IS_PREFLIGHT" => Ok(Op::CorsPreflight),
            
            // === STDLIB TIER 1: Common Operations ===
            
            // TIME module
            "TIME.NOW" => Ok(Op::TimeNow),
            "TIME.NOW_MS" => Ok(Op::TimeNowMs),
            "TIME.SLEEP" => Ok(Op::TimeSleep),
            
            // UUID module
            "UUID.V4" => Ok(Op::UuidV4),
            "UUID.V7" => Ok(Op::UuidV7),
            
            // HASH module (aliases already defined above)
            
            // BASE64 module
            "BASE64.ENCODE" => Ok(Op::Base64Encode),
            "BASE64.DECODE" => Ok(Op::Base64Decode),
            
            // ENV module
            "ENV.GET" => Ok(Op::EnvGet),
            "ENV.SET" => Ok(Op::EnvSet),
            
            // PATH module (aliases)
            "PATH.DIRNAME" => Ok(Op::PathDirname),
            "PATH.BASENAME" => Ok(Op::PathBasename),
            
            // FORMAT module
            "FORMAT.INT" => Ok(Op::FormatInt),
            "FORMAT.HEX" => Ok(Op::FormatHex),
            
            // VALIDATION module
            "VALIDATE.EMAIL" => Ok(Op::ValidateEmail),
            "VALIDATE.URL" => Ok(Op::ValidateUrl),
            "VALIDATE.UUID" => Ok(Op::ValidateUuid),
            "VALIDATE.IPV4" => Ok(Op::ValidateIpv4),
            
            // TEST module
            "TEST.ASSERT" => Ok(Op::TestAssert),
            "TEST.ASSERT_EQ" => Ok(Op::TestAssertEq),
            "TEST.SUMMARY" => Ok(Op::TestSummary),
            
            // JWT module
            "JWT.ENCODE" => Ok(Op::JwtEncode),
            "JWT.DECODE" => Ok(Op::JwtDecode),
            "JWT.VERIFY" => Ok(Op::JwtVerify),
            
            // REGEX module
            "REGEX.MATCH" => Ok(Op::RegexMatch),
            "REGEX.FIND" => Ok(Op::RegexFind),
            
            // ASYNC module
            "ASYNC.SPAWN" => Ok(Op::AsyncSpawn),
            "ASYNC.AWAIT" => Ok(Op::AsyncAwait),
            
            // TIMEOUT module
            "TIMEOUT.SET" => Ok(Op::TimeoutSet),
            "TIMEOUT.CHECK" => Ok(Op::TimeoutCheck),
            
            // === STDLIB TIER 2: Specialized Operations ===
            
            // YAML module
            "YAML.PARSE" => Ok(Op::YamlParse),
            "YAML.GET" => Ok(Op::YamlGet),
            "YAML.ENCODE" => Ok(Op::YamlEncode),
            
            // CSV module
            "CSV.PARSE" => Ok(Op::CsvParse),
            "CSV.NEXT_ROW" => Ok(Op::CsvNextRow),
            "CSV.GET_FIELD" => Ok(Op::CsvGetField),
            "CSV.ENCODE" => Ok(Op::CsvEncode),
            
            // XML module
            "XML.PARSE" => Ok(Op::XmlParse),
            "XML.ROOT" => Ok(Op::XmlRoot),
            "XML.TAG" => Ok(Op::XmlTag),
            "XML.ATTR" => Ok(Op::XmlAttr),
            "XML.TEXT" => Ok(Op::XmlText),
            
            // CRYPTO module
            "AES.ENCRYPT" => Ok(Op::AesEncrypt),
            "AES.DECRYPT" => Ok(Op::AesDecrypt),
            "RSA.GENERATE" => Ok(Op::RsaGenerate),
            "RSA.ENCRYPT" => Ok(Op::RsaEncrypt),
            "RSA.DECRYPT" => Ok(Op::RsaDecrypt),
            "SHA1" => Ok(Op::Sha1),
            "MD5" => Ok(Op::Md5),
            "HMAC.SHA256" => Ok(Op::HmacSha256),
            
            // POSTGRES module
            "PG.CONNECT" => Ok(Op::PgConnect),
            "PG.QUERY" => Ok(Op::PgQuery),
            "PG.FETCH" => Ok(Op::PgFetch),
            "PG.CLOSE" => Ok(Op::PgClose),
            
            // REDIS module
            "REDIS.CONNECT" => Ok(Op::RedisConnect),
            "REDIS.SET" => Ok(Op::RedisSet),
            "REDIS.GET" => Ok(Op::RedisGet),
            "REDIS.DEL" => Ok(Op::RedisDel),
            "REDIS.CLOSE" => Ok(Op::RedisClose),
            
            // SQL/ODBC module - SQL.FETCH added (others already exist)
            "SQL.FETCH" => Ok(Op::SqlFetch),
            
            // WEBSOCKET module
            "WS.CONNECT" => Ok(Op::WsConnect),
            "WS.ACCEPT" => Ok(Op::WsAccept),
            "WS.SEND" => Ok(Op::WsSend),
            "WS.RECV" => Ok(Op::WsRecv),
            "WS.CLOSE" => Ok(Op::WsClose),
            
            // TLS module
            "TLS.CONNECT" => Ok(Op::TlsConnect),
            "TLS.ACCEPT" => Ok(Op::TlsAccept),
            "TLS.SEND" => Ok(Op::TlsSend),
            "TLS.RECV" => Ok(Op::TlsRecv),
            "TLS.CLOSE" => Ok(Op::TlsClose),
            
            // GRPC module
            "GRPC.CHANNEL" => Ok(Op::GrpcChannel),
            "GRPC.CALL" => Ok(Op::GrpcCall),
            "GRPC.SEND" => Ok(Op::GrpcSend),
            "GRPC.RECV" => Ok(Op::GrpcRecv),
            "GRPC.CLOSE" => Ok(Op::GrpcClose),
            
            // PQC (Post-Quantum Cryptography) module
            "KYBER.KEYGEN" => Ok(Op::KyberKeygen),
            "KYBER.ENCAPS" => Ok(Op::KyberEncaps),
            "KYBER.DECAPS" => Ok(Op::KyberDecaps),
            "DILITHIUM.KEYGEN" => Ok(Op::DilithiumKeygen),
            "DILITHIUM.SIGN" => Ok(Op::DilithiumSign),
            "DILITHIUM.VERIFY" => Ok(Op::DilithiumVerify),
            
            // GZIP module
            "GZIP.COMPRESS" => Ok(Op::GzipCompress),
            "GZIP.DECOMPRESS" => Ok(Op::GzipDecompress),
            
            // GUI module (Windows user32.dll)
            "GUI.WIN" => Ok(Op::GuiWin),
            "GUI.SHOW" => Ok(Op::GuiShow),
            "GUI.HIDE" => Ok(Op::GuiHide),
            "GUI.EVENT" => Ok(Op::GuiEvent),
            "GUI.RUN" => Ok(Op::GuiRun),
            "GUI.LABEL" => Ok(Op::GuiLabel),
            "GUI.TEXTBOX" => Ok(Op::GuiTextbox),
            "GUI.BUTTON" => Ok(Op::GuiButton),
            "GUI.GETVAL" => Ok(Op::GuiGetVal),
            "GUI.SETVAL" => Ok(Op::GuiSetVal),
            "GUI.MSGBOX" => Ok(Op::GuiMsgBox),
            "GUI.DLG" => Ok(Op::GuiDlg),
            
            // RETRY module
            "RETRY.CONFIG_NEW" => Ok(Op::RetryConfigNew),
            "RETRY.EXECUTE" => Ok(Op::RetryExecute),
            
            // COOKIE module
            "COOKIE.PARSE" => Ok(Op::CookieParse),
            "COOKIE.GET" => Ok(Op::CookieGet),
            
            // === PHASE 19: Quantum Computing (QIR) ===
            // Single-qubit gates
            "QH" => Ok(Op::Call("QH".to_string())),       // Hadamard
            "QX" => Ok(Op::Call("QX".to_string())),       // Pauli-X (NOT)
            "QY" => Ok(Op::Call("QY".to_string())),       // Pauli-Y
            "QZ" => Ok(Op::Call("QZ".to_string())),       // Pauli-Z
            "QS" => Ok(Op::Call("QS".to_string())),       // S gate
            "QT" => Ok(Op::Call("QT".to_string())),       // T gate
            "QRX" => Ok(Op::Call("QRX".to_string())),     // Rx rotation
            "QRY" => Ok(Op::Call("QRY".to_string())),     // Ry rotation
            "QRZ" => Ok(Op::Call("QRZ".to_string())),     // Rz rotation
            // Two-qubit gates
            "QCNOT" | "QCX" => Ok(Op::Call("QCNOT".to_string())),  // CNOT
            "QCZ" => Ok(Op::Call("QCZ".to_string())),     // CZ
            "QSWAP" => Ok(Op::Call("QSWAP".to_string())), // SWAP
            // Three-qubit gates
            "QCCX" | "QTOFFOLI" => Ok(Op::Call("QCCX".to_string())),  // Toffoli
            "QCSWAP" | "QFREDKIN" => Ok(Op::Call("QCSWAP".to_string())), // Fredkin
            // Measurement
            "QM" | "QMEASURE" => Ok(Op::Call("QM".to_string())),
            // Qubit management
            "QALLOC" => Ok(Op::Call("QALLOC".to_string())),
            "QFREE" => Ok(Op::Call("QFREE".to_string())),
            "QRESET" => Ok(Op::Call("QRESET".to_string())),
            
            s if s.starts_with('.') => Ok(Op::Call(s[1..].to_string())),
            _ => Err(format!("Unknown operation: {}", s)),
        }
    }
    
    fn parse_args(tokens: &[String]) -> Result<Vec<Arg>, String> {
        let mut args = Vec::new();
        let mut i = 0;
        
        while i < tokens.len() {
            let token = &tokens[i];
            
            // Label reference: :label_name (for JMP, JZ, JNZ)
            if token.starts_with(':') {
                args.push(Arg::Ref(format!(":{}", &token[1..]))); // Keep : prefix to identify as label
                i += 1;
                continue;
            }
            
            // Reference with @ prefix (runtime reference)
            if token.starts_with("@.") {
                args.push(Arg::Ref(token[2..].to_string())); // Skip @.
                i += 1;
                continue;
            }
            
            // Reference
            if token.starts_with('.') {
                args.push(Arg::Ref(token[1..].to_string()));
                i += 1;
                continue;
            }
            
            // Grouped expression: (OP args)
            if token == "(" {
                let (expr, consumed) = Self::parse_grouped_expr(&tokens[i..])?;
                args.push(expr);
                i += consumed;
                continue;
            }
            
            // Literal
            if let Ok(value) = Self::parse_literal(token) {
                args.push(Arg::Lit(value));
                i += 1;
                continue;
            }
            
            // String that spans multiple tokens (quoted)
            if token.starts_with('"') {
                // Find the end quote
                let mut full_string = token.clone();
                while !full_string.ends_with('"') && i + 1 < tokens.len() {
                    i += 1;
                    full_string.push(' ');
                    full_string.push_str(&tokens[i]);
                }
                if let Ok(value) = Self::parse_literal(&full_string) {
                    args.push(Arg::Lit(value));
                    i += 1;
                    continue;
                }
            }
            
            return Err(format!("Cannot parse argument: {}", token));
        }
        
        Ok(args)
    }
    
    fn parse_grouped_expr(tokens: &[String]) -> Result<(Arg, usize), String> {
        // tokens[0] should be "("
        if tokens.is_empty() || tokens[0] != "(" {
            return Err("Expected '('".to_string());
        }
        
        // Find matching ")"
        let mut depth = 1;
        let mut end = 1;
        while end < tokens.len() && depth > 0 {
            if tokens[end] == "(" {
                depth += 1;
            } else if tokens[end] == ")" {
                depth -= 1;
            }
            if depth > 0 {
                end += 1;
            }
        }
        
        if depth != 0 {
            return Err("Unmatched parenthesis".to_string());
        }
        
        // Parse the inner expression
        let inner = &tokens[1..end];
        if inner.is_empty() {
            return Err("Empty grouped expression".to_string());
        }
        
        let op = Self::parse_op(&inner[0])?;
        let args = Self::parse_args(&inner[1..].to_vec())?;
        
        Ok((Arg::Expr(op, args), end + 1))
    }
    
    fn parse_flow(line: &str) -> Result<Option<Node>, String> {
        // Check for cyclic flow operator >>
        let is_cyclic = line.contains(" >> ");
        let separator = if is_cyclic { " >> " } else { " > " };
        
        let parts: Vec<&str> = line.split(separator).collect();
        if parts.len() < 2 {
            return Err("Invalid flow syntax".to_string());
        }
        
        // Parse source
        let source = Self::parse_flow_source(parts[0].trim())?;
        
        // Parse destination
        let dest_str = parts[1..].join(separator);
        let dest = Self::parse_flow_dest(dest_str.trim(), is_cyclic)?;
        
        Ok(Some(Node::Flow { source, dest }))
    }
    
    fn parse_flow_source(s: &str) -> Result<Arg, String> {
        let s = s.trim();
        
        // Reference
        if s.starts_with('.') {
            return Ok(Arg::Ref(s[1..].to_string()));
        }
        
        // Literal string
        if s.starts_with('"') {
            let value = Self::parse_literal(s)?;
            return Ok(Arg::Lit(value));
        }
        
        // Literal number (check before operation)
        if let Ok(value) = Self::parse_literal(s) {
            return Ok(Arg::Lit(value));
        }
        
        // Operation expression
        let tokens = Self::tokenize(s);
        if !tokens.is_empty() {
            let op = Self::parse_op(&tokens[0])?;
            let args = Self::parse_args(&tokens[1..])?;
            return Ok(Arg::Expr(op, args));
        }
        
        // Literal number
        if let Ok(value) = Self::parse_literal(s) {
            return Ok(Arg::Lit(value));
        }
        
        Err(format!("Cannot parse flow source: {}", s))
    }
    
    fn parse_flow_dest(s: &str, is_cyclic: bool) -> Result<FlowDest, String> {
        let s = s.trim();
        
        // Check for effects
        match s.to_uppercase().as_str() {
            "PRT" => return Ok(FlowDest::Effect(Effect::Print)),
            _ => {}
        }
        
        // Check for chained flow (contains another >)
        if s.contains(" > ") || s.contains(" >> ") {
            // For now, treat as node reference to the first part
            let first = s.split(" > ").next().unwrap_or(s).split(" >> ").next().unwrap_or(s).trim();
            if first.starts_with('.') {
                return if is_cyclic {
                    Ok(FlowDest::CyclicNode(first[1..].to_string()))
                } else {
                    Ok(FlowDest::Node(first[1..].to_string()))
                };
            }
        }
        
        // Node reference
        if s.starts_with('.') {
            return if is_cyclic {
                Ok(FlowDest::CyclicNode(s[1..].to_string()))
            } else {
                Ok(FlowDest::Node(s[1..].to_string()))
            };
        }
        
        // Effect with arguments
        let tokens = Self::tokenize(s);
        if !tokens.is_empty() {
            match tokens[0].to_uppercase().as_str() {
                "PRT" => return Ok(FlowDest::Effect(Effect::Print)),
                "MAP" => {
                    // MAP is a transformation, not a final destination
                    // For now, treat as node
                    return Ok(FlowDest::Node(s.to_string()));
                }
                _ => {}
            }
        }
        
        Err(format!("Cannot parse flow destination: {}", s))
    }
    
    fn parse_subgraph(id: &str, rest: &str) -> Result<Option<Node>, String> {
        // Simple sub-graph parsing
        // Format: { IN .a .b ... OUT .result }
        
        let inner = rest.trim_start_matches('{').trim_end_matches('}').trim();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut nodes = Vec::new();
        
        // Split by lines or by major sections
        let parts: Vec<&str> = inner.split('|').collect();
        
        for part in parts {
            let part = part.trim();
            if part.starts_with("IN ") {
                // Parse inputs
                let args: Vec<&str> = part[3..].split_whitespace().collect();
                for arg in args {
                    if arg.starts_with('.') {
                        inputs.push(arg[1..].to_string());
                    }
                }
            } else if part.starts_with("OUT ") {
                // Parse outputs
                let args: Vec<&str> = part[4..].split_whitespace().collect();
                for arg in args {
                    if arg.starts_with('.') {
                        outputs.push(arg[1..].to_string());
                    }
                }
            } else if !part.is_empty() {
                // Parse as node
                if let Ok(Some(node)) = Self::parse_line(part, &mut vec![], &mut 0) {
                    nodes.push(node);
                }
            }
        }
        
        Ok(Some(Node::SubGraph {
            id: id.to_string(),
            inputs,
            outputs,
            nodes,
        }))
    }
    
    fn tokenize(s: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut chars = s.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '"' {
                in_string = !in_string;
                current.push(c);
            } else if in_string {
                current.push(c);
            } else if c == '(' || c == ')' {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(c.to_string());
            } else if c.is_whitespace() {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            } else {
                current.push(c);
            }
        }
        
        if !current.is_empty() {
            tokens.push(current);
        }
        
        tokens
    }
    
    /// Parse GEN definition: #NAME param1 param2:
    fn parse_macro_def(lines: &[&str], i: &mut usize) -> Result<(String, MacroDef), String> {
        let header = lines[*i].trim();
        
        // Parse header: #NAME param1 param2:
        let colon_pos = header.find(':').ok_or("GEN definition must end with ':'")?;
        let header_content = &header[1..colon_pos].trim();
        
        let tokens: Vec<&str> = header_content.split_whitespace().collect();
        if tokens.is_empty() {
            return Err("GEN name required".to_string());
        }
        
        let name = tokens[0].to_string();
        let params: Vec<String> = tokens[1..].iter().map(|s| s.to_string()).collect();
        
        // Collect body lines until #END
        let mut body = Vec::new();
        *i += 1;
        while *i < lines.len() {
            let line = lines[*i].trim();
            if line.starts_with("#END") {
                *i += 1;
                break;
            }
            if !line.is_empty() && !line.starts_with("--") {
                body.push(line.to_string());
            }
            *i += 1;
        }
        
        Ok((name, MacroDef { params, body }))
    }
    
    /// Generate and fuse subgraph: !NAME arg1 arg2
    fn expand_gen(line: &str, macros: &HashMap<String, MacroDef>, counter: &mut u32) -> Result<Vec<String>, String> {
        let tokens: Vec<&str> = line[1..].split_whitespace().collect();
        if tokens.is_empty() {
            return Err("GEN name required".to_string());
        }
        
        let name = tokens[0];
        let args: Vec<&str> = tokens[1..].to_vec();
        
        let macro_def = macros.get(name).ok_or(format!("Unknown GEN: {}", name))?;
        
        if args.len() != macro_def.params.len() {
            return Err(format!(
                "GEN {} expects {} args, got {}",
                name, macro_def.params.len(), args.len()
            ));
        }
        
        // Generate unique prefix for this expansion
        let prefix = format!("_g{}_{}_", counter, name.to_lowercase());
        *counter += 1;
        
        // Expand body with parameter substitution
        let mut expanded = Vec::new();
        for body_line in &macro_def.body {
            // First pass: replace .param references with arguments
            let mut line = body_line.clone();
            for (i, param) in macro_def.params.iter().enumerate() {
                let param_ref = format!(".{}", param);
                let arg = args[i];
                // Only replace exact .param matches (word boundary)
                line = Self::replace_word(&line, &param_ref, arg);
            }
            
            // Second pass: prefix internal node references
            let mut result = String::new();
            let mut chars = line.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '.' {
                    let mut node_name = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc.is_alphanumeric() || nc == '_' {
                            node_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if !node_name.is_empty() {
                        // Check if this is an argument (don't prefix)
                        let is_arg = args.iter().any(|a| a.trim_start_matches('.') == node_name);
                        if is_arg {
                            result.push('.');
                            result.push_str(&node_name);
                        } else {
                            result.push('.');
                            result.push_str(&prefix);
                            result.push_str(&node_name);
                        }
                    } else {
                        result.push('.');
                    }
                } else {
                    result.push(c);
                }
            }
            
            expanded.push(result);
        }
        
        Ok(expanded)
    }
    
    /// Replace word with exact match (not substring)
    fn replace_word(text: &str, word: &str, replacement: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();
        let word_chars: Vec<char> = word.chars().collect();
        
        while i < chars.len() {
            // Check if word matches at position i
            let mut matches = true;
            if i + word_chars.len() <= chars.len() {
                for (j, wc) in word_chars.iter().enumerate() {
                    if chars[i + j] != *wc {
                        matches = false;
                        break;
                    }
                }
                // Check word boundary after match
                if matches && i + word_chars.len() < chars.len() {
                    let next_char = chars[i + word_chars.len()];
                    if next_char.is_alphanumeric() || next_char == '_' {
                        matches = false;
                    }
                }
            } else {
                matches = false;
            }
            
            if matches {
                result.push_str(replacement);
                i += word_chars.len();
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        
        result
    }
    
    /// Parse v1.5 syntax (new)
    fn parse_v1_5(content: &str) -> Result<Graph, String> {
        let mut graph = Graph::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_function: Option<(String, Vec<String>, Vec<Node>)> = None;
        let mut for_counter: u32 = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("--") {
                i += 1;
                continue;
            }
            
            // End of program
            if line == "!" || line.starts_with("! ") {
                i += 1;
                continue;
            }
            
            // Parse v1.5 line
            match Self::parse_v1_5_line(line, &mut for_counter) {
                Ok(nodes) => {
                    for node in nodes {
                        if let Node::Function { id, params, body: _ } = &node {
                            if current_function.is_some() {
                                // Close previous function
                                if let Some((fid, fparams, fbody)) = current_function.take() {
                                    graph.add_node(Node::Function { id: fid, params: fparams, body: fbody });
                                }
                            }
                            current_function = Some((id.clone(), params.clone(), Vec::new()));
                        } else if let Some((_, _, ref mut body)) = current_function {
                            body.push(node);
                        } else {
                            graph.add_node(node);
                        }
                    }
                }
                Err(e) => return Err(format!("Line {}: {}", i + 1, e)),
            }
            
            i += 1;
        }
        
        // Close any open function
        if let Some((id, params, body)) = current_function {
            graph.add_node(Node::Function { id, params, body });
        }
        
        Ok(graph)
    }
    
    /// Parse a single v1.5 line, returns multiple nodes for desugaring
    fn parse_v1_5_line(line: &str, for_counter: &mut u32) -> Result<Vec<Node>, String> {
        let line = Self::strip_comment(line);
        
        // Label: :name
        if line.starts_with(':') && !line.contains(' ') {
            return Ok(vec![Node::Label(line[1..].to_string())]);
        }
        
        // use statement: use http, json
        if line.starts_with("use ") {
            let modules = line[4..].split(',').map(|s| s.trim());
            let mut nodes = Vec::new();
            for module in modules {
                nodes.push(Node::Use { module: module.to_string() });
            }
            return Ok(nodes);
        }
        
        // Function definition: fn name(args)
        if line.starts_with("fn ") {
            return Self::parse_v1_5_function(line);
        }
        
        // For loop: for i in 0..n
        if line.starts_with("for ") {
            return Self::parse_v1_5_for(line, for_counter);
        }
        
        // Guard: guard cond else :label
        if line.starts_with("guard ") {
            return Self::parse_v1_5_guard(line);
        }
        
        // Loop: loop (infinite)
        if line == "loop" {
            *for_counter += 1;
            let label = format!("_loop_{}", for_counter);
            return Ok(vec![Node::Label(label)]);
        }
        
        // Direct conditional jumps: jge, jgt, jle, jlt, jeq, jne
        if line.starts_with("jge ") || line.starts_with("jgt ") || 
           line.starts_with("jle ") || line.starts_with("jlt ") ||
           line.starts_with("jeq ") || line.starts_with("jne ") {
            return Self::parse_v1_5_cond_jump(line);
        }
        
        // Range jumps: jin, jnot
        if line.starts_with("jin ") || line.starts_with("jnot ") {
            return Self::parse_v1_5_range_jump(line);
        }
        
        // Simple jumps: jmp, jz, jnz
        if line.starts_with("jmp ") || line.starts_with("jz ") || line.starts_with("jnz ") {
            return Self::parse_v1_5_simple_jump(line);
        }
        
        // Return: ret value
        if line.starts_with("ret ") || line == "ret" {
            return Self::parse_v1_5_ret(line);
        }
        
        // Compound assignment: x += 1, x -= 1, etc.
        if line.contains(" += ") || line.contains(" -= ") || 
           line.contains(" *= ") || line.contains(" /= ") || line.contains(" %= ") {
            return Self::parse_v1_5_compound_assign(line);
        }
        
        // Assignment with error propagation: x = op() ?> :label
        if line.contains(" = ") && line.contains(" ?> ") {
            return Self::parse_v1_5_error_prop(line);
        }
        
        // Simple assignment: x = expr
        if line.contains(" = ") {
            return Self::parse_v1_5_assign(line);
        }
        
        // Function call without assignment: print(x)
        if line.contains('(') && line.contains(')') {
            return Self::parse_v1_5_call(line);
        }
        
        Err(format!("Unrecognized v1.5 syntax: {}", line))
    }
    
    /// Parse v1.5 function: fn name(a, b)
    fn parse_v1_5_function(line: &str) -> Result<Vec<Node>, String> {
        // fn name(a, b) or fn name(a, b) = expr
        let rest = &line[3..]; // skip "fn "
        let paren_start = rest.find('(').ok_or("Expected '(' in function")?;
        let paren_end = rest.find(')').ok_or("Expected ')' in function")?;
        
        let name = rest[..paren_start].trim().to_string();
        let params_str = &rest[paren_start + 1..paren_end];
        let params: Vec<String> = if params_str.trim().is_empty() {
            Vec::new()
        } else {
            params_str.split(',').map(|s| s.trim().to_string()).collect()
        };
        
        Ok(vec![Node::Function { id: name, params, body: Vec::new() }])
    }
    
    /// Parse v1.5 guard: guard cond else :label
    fn parse_v1_5_guard(line: &str) -> Result<Vec<Node>, String> {
        // guard cond else :label -> jz cond :label
        let rest = &line[6..]; // skip "guard "
        let else_pos = rest.find(" else ").ok_or("Expected 'else' in guard")?;
        let cond = rest[..else_pos].trim();
        let label = rest[else_pos + 6..].trim();
        
        if !label.starts_with(':') {
            return Err("Guard label must start with ':'".to_string());
        }
        
        // Desugar to: jz cond :label
        let tmp_id = format!("_guard_{}", cond.replace(' ', "_"));
        let cond_arg = Self::parse_v1_5_expr_to_arg(cond)?;
        
        Ok(vec![Node::Operation {
            id: tmp_id,
            op: Op::Jz,
            args: vec![cond_arg, Arg::Ref(label.to_string())],
            runtime: true,
        }])
    }
    
    /// Parse v1.5 for: for i in 0..n
    fn parse_v1_5_for(line: &str, counter: &mut u32) -> Result<Vec<Node>, String> {
        // for i in start..end
        let rest = &line[4..]; // skip "for "
        let in_pos = rest.find(" in ").ok_or("Expected 'in' in for loop")?;
        let var = rest[..in_pos].trim().to_string();
        let range = rest[in_pos + 4..].trim();
        
        let dots_pos = range.find("..").ok_or("Expected '..' in range")?;
        let start = range[..dots_pos].trim();
        let end = &range[dots_pos + 2..].trim();
        
        *counter += 1;
        let loop_label = format!("_for_{}", counter);
        let end_label = format!("_for_end_{}", counter);
        let tmp_done = format!("_for_done_{}", counter);
        
        // Desugar to:
        // var = start
        // :_for_N
        //   _done = ge(var, end)
        //   jnz _done :_for_end_N
        //   ... body ...
        //   var = add(var, 1)
        //   jmp :_for_N
        // :_for_end_N
        
        let start_arg = Self::parse_v1_5_expr_to_arg(start)?;
        let end_arg = Self::parse_v1_5_expr_to_arg(end)?;
        
        Ok(vec![
            // var = start
            Node::Literal { id: var.clone(), value: Value::Int(0), runtime: true },
            Node::Operation {
                id: var.clone(),
                op: Op::Add,
                args: vec![start_arg, Arg::Lit(Value::Int(0))],
                runtime: true,
            },
            // :_for_N
            Node::Label(loop_label.clone()),
            // _done = ge(var, end)
            Node::Operation {
                id: tmp_done.clone(),
                op: Op::Ge,
                args: vec![Arg::Ref(var.clone()), end_arg],
                runtime: true,
            },
            // jnz _done :_for_end_N
            Node::Operation {
                id: format!("_jnz_{}", counter),
                op: Op::Jnz,
                args: vec![Arg::Ref(tmp_done), Arg::Ref(format!(":{}", end_label))],
                runtime: true,
            },
        ])
    }
    
    /// Parse v1.5 conditional jump: jge a b :label
    fn parse_v1_5_cond_jump(line: &str) -> Result<Vec<Node>, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("Conditional jump requires: op a b :label".to_string());
        }
        
        let jump_op = parts[0];
        let a = parts[1];
        let b = parts[2];
        let label = parts[3];
        
        if !label.starts_with(':') {
            return Err("Jump label must start with ':'".to_string());
        }
        
        // Map jump op to comparison op
        let cmp_op = match jump_op {
            "jge" => Op::Ge,
            "jgt" => Op::Gt,
            "jle" => Op::Le,
            "jlt" => Op::Lt,
            "jeq" => Op::Eq,
            "jne" => Op::Ne,
            _ => return Err(format!("Unknown conditional jump: {}", jump_op)),
        };
        
        let a_arg = Self::parse_v1_5_expr_to_arg(a)?;
        let b_arg = Self::parse_v1_5_expr_to_arg(b)?;
        
        // Desugar to: _tmp = cmp(a, b); jnz _tmp :label
        let tmp_id = format!("_cmp_{}_{}", a, b).replace('.', "_");
        
        Ok(vec![
            Node::Operation {
                id: tmp_id.clone(),
                op: cmp_op,
                args: vec![a_arg, b_arg],
                runtime: true,
            },
            Node::Operation {
                id: format!("_jnz_{}", tmp_id),
                op: Op::Jnz,
                args: vec![Arg::Ref(tmp_id), Arg::Ref(label.to_string())],
                runtime: true,
            },
        ])
    }
    
    /// Parse v1.5 range jump: jin x 200..299 :label
    fn parse_v1_5_range_jump(line: &str) -> Result<Vec<Node>, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("Range jump requires: op x range :label".to_string());
        }
        
        let jump_op = parts[0];
        let x = parts[1];
        let range = parts[2];
        let label = parts[3];
        
        let dots_pos = range.find("..").ok_or("Expected '..' in range")?;
        let lo = &range[..dots_pos];
        let hi = &range[dots_pos + 2..];
        
        let x_arg = Self::parse_v1_5_expr_to_arg(x)?;
        let lo_arg = Self::parse_v1_5_expr_to_arg(lo)?;
        let hi_arg = Self::parse_v1_5_expr_to_arg(hi)?;
        
        // jin: jump if x in range (lo <= x <= hi)
        // jnot: jump if x not in range
        let tmp1 = format!("_rng1_{}", x);
        let tmp2 = format!("_rng2_{}", x);
        let tmp3 = format!("_rng3_{}", x);
        
        if jump_op == "jin" {
            // _t1 = ge(x, lo); _t2 = le(x, hi); _t3 = and(_t1, _t2); jnz _t3 :label
            Ok(vec![
                Node::Operation { id: tmp1.clone(), op: Op::Ge, args: vec![x_arg.clone(), lo_arg], runtime: true },
                Node::Operation { id: tmp2.clone(), op: Op::Le, args: vec![x_arg, hi_arg], runtime: true },
                Node::Operation { id: tmp3.clone(), op: Op::And, args: vec![Arg::Ref(tmp1), Arg::Ref(tmp2)], runtime: true },
                Node::Operation { id: format!("_jnz_{}", tmp3), op: Op::Jnz, args: vec![Arg::Ref(tmp3), Arg::Ref(label.to_string())], runtime: true },
            ])
        } else {
            // jnot: _t1 = lt(x, lo); _t2 = gt(x, hi); _t3 = or(_t1, _t2); jnz _t3 :label
            Ok(vec![
                Node::Operation { id: tmp1.clone(), op: Op::Lt, args: vec![x_arg.clone(), lo_arg], runtime: true },
                Node::Operation { id: tmp2.clone(), op: Op::Gt, args: vec![x_arg, hi_arg], runtime: true },
                Node::Operation { id: tmp3.clone(), op: Op::Or, args: vec![Arg::Ref(tmp1), Arg::Ref(tmp2)], runtime: true },
                Node::Operation { id: format!("_jnz_{}", tmp3), op: Op::Jnz, args: vec![Arg::Ref(tmp3), Arg::Ref(label.to_string())], runtime: true },
            ])
        }
    }
    
    /// Parse v1.5 simple jump: jmp :label, jz cond :label, jnz cond :label
    fn parse_v1_5_simple_jump(line: &str) -> Result<Vec<Node>, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        match parts[0] {
            "jmp" => {
                if parts.len() < 2 {
                    return Err("jmp requires :label".to_string());
                }
                Ok(vec![Node::Operation {
                    id: "_jmp".to_string(),
                    op: Op::Jmp,
                    args: vec![Arg::Ref(parts[1].to_string())],
                    runtime: true,
                }])
            }
            "jz" | "jnz" => {
                if parts.len() < 3 {
                    return Err(format!("{} requires cond :label", parts[0]));
                }
                let cond_arg = Self::parse_v1_5_expr_to_arg(parts[1])?;
                let op = if parts[0] == "jz" { Op::Jz } else { Op::Jnz };
                Ok(vec![Node::Operation {
                    id: format!("_{}", parts[0]),
                    op,
                    args: vec![cond_arg, Arg::Ref(parts[2].to_string())],
                    runtime: true,
                }])
            }
            _ => Err(format!("Unknown jump: {}", parts[0])),
        }
    }
    
    /// Parse v1.5 return: ret value
    fn parse_v1_5_ret(line: &str) -> Result<Vec<Node>, String> {
        if line == "ret" {
            Ok(vec![Node::Operation {
                id: "_ret".to_string(),
                op: Op::Ret,
                args: vec![Arg::Lit(Value::Int(0))],
                runtime: true,
            }])
        } else {
            let value = line[4..].trim();
            let arg = Self::parse_v1_5_expr_to_arg(value)?;
            Ok(vec![Node::Operation {
                id: "_ret".to_string(),
                op: Op::Ret,
                args: vec![arg],
                runtime: true,
            }])
        }
    }
    
    /// Parse v1.5 compound assignment: x += 1
    fn parse_v1_5_compound_assign(line: &str) -> Result<Vec<Node>, String> {
        let (var, op_str, expr) = if line.contains(" += ") {
            let parts: Vec<&str> = line.splitn(2, " += ").collect();
            (parts[0].trim(), "add", parts[1].trim())
        } else if line.contains(" -= ") {
            let parts: Vec<&str> = line.splitn(2, " -= ").collect();
            (parts[0].trim(), "sub", parts[1].trim())
        } else if line.contains(" *= ") {
            let parts: Vec<&str> = line.splitn(2, " *= ").collect();
            (parts[0].trim(), "mul", parts[1].trim())
        } else if line.contains(" /= ") {
            let parts: Vec<&str> = line.splitn(2, " /= ").collect();
            (parts[0].trim(), "div", parts[1].trim())
        } else if line.contains(" %= ") {
            let parts: Vec<&str> = line.splitn(2, " %= ").collect();
            (parts[0].trim(), "mod", parts[1].trim())
        } else {
            return Err("Unknown compound operator".to_string());
        };
        
        let op = Self::parse_v1_5_op(op_str)?;
        let expr_arg = Self::parse_v1_5_expr_to_arg(expr)?;
        
        // x += 1 -> @.x: ADD .x 1 (runtime operation that updates x)
        // Use runtime=true to indicate this is a runtime update
        Ok(vec![Node::Operation {
            id: var.to_string(),
            op,
            args: vec![Arg::Ref(var.to_string()), expr_arg],
            runtime: true,  // Mark as runtime to avoid false cycle detection
        }])
    }
    
    /// Parse v1.5 error propagation: x = op() ?> :label
    fn parse_v1_5_error_prop(line: &str) -> Result<Vec<Node>, String> {
        let prop_pos = line.find(" ?> ").ok_or("Expected '?>' in error propagation")?;
        let assign_part = &line[..prop_pos];
        let label = line[prop_pos + 4..].trim();
        
        // Parse the assignment part
        let mut nodes = Self::parse_v1_5_assign(assign_part)?;
        
        // Get the variable name from the assignment
        let var_name = if let Some(Node::Operation { id, .. }) = nodes.first() {
            id.clone()
        } else if let Some(Node::Literal { id, .. }) = nodes.first() {
            id.clone()
        } else {
            return Err("Could not determine variable for error propagation".to_string());
        };
        
        // Add: jlt var 0 :label
        nodes.push(Node::Operation {
            id: format!("_err_prop_{}", var_name),
            op: Op::Lt,
            args: vec![Arg::Ref(var_name.clone()), Arg::Lit(Value::Int(0))],
            runtime: true,
        });
        nodes.push(Node::Operation {
            id: format!("_err_jnz_{}", var_name),
            op: Op::Jnz,
            args: vec![Arg::Ref(format!("_err_prop_{}", var_name)), Arg::Ref(label.to_string())],
            runtime: true,
        });
        
        Ok(nodes)
    }
    
    /// Parse v1.5 assignment: x = expr
    fn parse_v1_5_assign(line: &str) -> Result<Vec<Node>, String> {
        let eq_pos = line.find(" = ").ok_or("Expected '=' in assignment")?;
        let var = line[..eq_pos].trim();
        let expr = line[eq_pos + 3..].trim();
        
        // Check if expr is a function call: func(args)
        if expr.contains('(') && expr.ends_with(')') {
            let paren_pos = expr.find('(').ok_or("Expected '(' in function call")?;
            let func_name = &expr[..paren_pos];
            let args_str = &expr[paren_pos + 1..expr.len() - 1];
            
            // Parse function name to operation
            let op = Self::parse_v1_5_op(func_name)?;
            
            // Parse arguments
            let args = Self::parse_v1_5_args(args_str)?;
            
            return Ok(vec![Node::Operation {
                id: var.to_string(),
                op,
                args,
                runtime: true,
            }]);
        }
        
        // Check if it's a literal
        if let Ok(value) = Self::parse_literal(expr) {
            return Ok(vec![Node::Literal {
                id: var.to_string(),
                value,
                runtime: false,
            }]);
        }
        
        // Check if it's a simple reference
        if !expr.contains(' ') && !expr.contains('(') {
            return Ok(vec![Node::Reference {
                id: var.to_string(),
                target: expr.to_string(),
            }]);
        }
        
        // Binary expression: a + b, a * b, etc.
        if let Some((left, op_str, right)) = Self::parse_v1_5_binary_expr(expr) {
            let op = Self::parse_v1_5_op(op_str)?;
            let left_arg = Self::parse_v1_5_expr_to_arg(left)?;
            let right_arg = Self::parse_v1_5_expr_to_arg(right)?;
            
            return Ok(vec![Node::Operation {
                id: var.to_string(),
                op,
                args: vec![left_arg, right_arg],
                runtime: true,
            }]);
        }
        
        Err(format!("Cannot parse expression: {}", expr))
    }
    
    /// Parse v1.5 function call without assignment: print(x)
    fn parse_v1_5_call(line: &str) -> Result<Vec<Node>, String> {
        let paren_pos = line.find('(').ok_or("Expected '(' in call")?;
        let func_name = &line[..paren_pos];
        let args_str = &line[paren_pos + 1..line.len() - 1];
        
        let op = Self::parse_v1_5_op(func_name)?;
        let args = Self::parse_v1_5_args(args_str)?;
        
        Ok(vec![Node::Operation {
            id: format!("_{}", func_name),
            op,
            args,
            runtime: true,
        }])
    }
    
    /// Parse v1.5 operation name to Op
    fn parse_v1_5_op(name: &str) -> Result<Op, String> {
        match name.to_lowercase().as_str() {
            // Arithmetic
            "add" | "+" => Ok(Op::Add),
            "sub" | "-" => Ok(Op::Sub),
            "mul" | "*" => Ok(Op::Mul),
            "div" | "/" => Ok(Op::Div),
            "mod" | "%" => Ok(Op::Mod),
            // Comparison
            "eq" | "==" => Ok(Op::Eq),
            "ne" | "!=" => Ok(Op::Ne),
            "lt" | "<" => Ok(Op::Lt),
            "gt" | ">" => Ok(Op::Gt),
            "le" | "<=" => Ok(Op::Le),
            "ge" | ">=" => Ok(Op::Ge),
            // Logic
            "and" | "&&" => Ok(Op::And),
            "or" | "||" => Ok(Op::Or),
            "not" | "!" => Ok(Op::Not),
            // Memory
            "alloc" => Ok(Op::Alc),
            "free" => Ok(Op::Fre),
            "get" => Ok(Op::Get),
            "put" => Ok(Op::Put),
            "copy" | "cpy" => Ok(Op::Cpy),
            "strlen" | "sln" => Ok(Op::Sln),
            // I/O
            "print" | "prt" => Ok(Op::Prt),
            "open" | "fop" => Ok(Op::Fop),
            "read" | "frd" => Ok(Op::Frd),
            "write" | "fwr" => Ok(Op::Fwr),
            "close" | "fcl" => Ok(Op::Fcl),
            // Network
            "tcp" => Ok(Op::Tcp),
            "bind" | "bnd" => Ok(Op::Bnd),
            "listen" | "lst" => Ok(Op::Lst),
            "accept" | "acc" => Ok(Op::Acc),
            "send" | "xmt" => Ok(Op::Xmt),
            "recv" | "rcv" => Ok(Op::Rcv),
            // Threading
            "spawn" | "thr" => Ok(Op::Thr),
            "join" | "jon" => Ok(Op::Jon),
            "mutex" | "mtx" => Ok(Op::Mtx),
            "lock" | "lck" => Ok(Op::Lck),
            "unlock" | "ulk" => Ok(Op::Ulk),
            // Channels
            "chan" | "chn" => Ok(Op::Chn),
            "chn_snd" => Ok(Op::ChnSnd),
            "chn_rcv" => Ok(Op::ChnRcv),
            "chn_cls" => Ok(Op::ChnCls),
            // Atomics
            "xadd" | "atm_add" => Ok(Op::AtmAdd),
            "cas" | "atm_cas" => Ok(Op::AtmCas),
            "fence" | "fnc" => Ok(Op::Fence),
            // String
            "cat" => Ok(Op::Cat),
            "itoa" | "its" => Ok(Op::Its),
            // Time
            "sleep" => Ok(Op::TimeSleep),
            "timestamp" | "time_now" => Ok(Op::TimeNow),
            // JSON
            "json_parse" => Ok(Op::JsonParse),
            "json_get" => Ok(Op::JsonGet),
            "json_set" => Ok(Op::JsonSet),
            "json_encode" => Ok(Op::JsonEncode),
            // HTTP
            "http_listen" => Ok(Op::HttpListen),
            "http_accept" => Ok(Op::HttpAccept),
            "http_method" => Ok(Op::HttpMethod),
            "http_path" => Ok(Op::HttpPath),
            "http_body" => Ok(Op::HttpBody),
            "http_respond" => Ok(Op::HttpRespond),
            "http_get" => Ok(Op::HttpGet),
            "http_post" => Ok(Op::HttpPost),
            // Other
            "env" => Ok(Op::EnvGet),
            "uuid" => Ok(Op::UuidV4),
            "min" => Ok(Op::Call("min".to_string())),
            "max" => Ok(Op::Call("max".to_string())),
            _ => Err(format!("Unknown v1.5 operation: {}", name)),
        }
    }
    
    /// Parse v1.5 arguments: "a, b, c" -> [Arg]
    fn parse_v1_5_args(args_str: &str) -> Result<Vec<Arg>, String> {
        if args_str.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let mut args = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut in_string = false;
        
        for c in args_str.chars() {
            if c == '"' {
                in_string = !in_string;
                current.push(c);
            } else if in_string {
                current.push(c);
            } else if c == '(' {
                depth += 1;
                current.push(c);
            } else if c == ')' {
                depth -= 1;
                current.push(c);
            } else if c == ',' && depth == 0 {
                args.push(Self::parse_v1_5_expr_to_arg(current.trim())?);
                current.clear();
            } else {
                current.push(c);
            }
        }
        
        if !current.trim().is_empty() {
            args.push(Self::parse_v1_5_expr_to_arg(current.trim())?);
        }
        
        Ok(args)
    }
    
    /// Parse v1.5 expression to Arg
    fn parse_v1_5_expr_to_arg(expr: &str) -> Result<Arg, String> {
        let expr = expr.trim();
        
        // Label reference
        if expr.starts_with(':') {
            return Ok(Arg::Ref(expr.to_string()));
        }
        
        // String literal
        if expr.starts_with('"') && expr.ends_with('"') {
            let inner = &expr[1..expr.len() - 1];
            let unescaped = inner
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
                .replace("\\0", "\0")
                .replace("\\\"", "\"");
            return Ok(Arg::Lit(Value::String(unescaped)));
        }
        
        // Number with suffix (K, M, G)
        if expr.ends_with('K') || expr.ends_with('M') || expr.ends_with('G') {
            let (num_str, mult) = if expr.ends_with('K') {
                (&expr[..expr.len() - 1], 1024i64)
            } else if expr.ends_with('M') {
                (&expr[..expr.len() - 1], 1024 * 1024)
            } else {
                (&expr[..expr.len() - 1], 1024 * 1024 * 1024)
            };
            if let Ok(n) = num_str.parse::<i64>() {
                return Ok(Arg::Lit(Value::Int(n * mult)));
            }
        }
        
        // Integer literal
        if let Ok(n) = expr.parse::<i64>() {
            return Ok(Arg::Lit(Value::Int(n)));
        }
        
        // Float literal
        if expr.contains('.') && !expr.contains('(') {
            if let Ok(f) = expr.parse::<f64>() {
                return Ok(Arg::Lit(Value::Float(f)));
            }
        }
        
        // Function call: func(args)
        if expr.contains('(') && expr.ends_with(')') {
            let paren_pos = expr.find('(').ok_or("Expected '(' in function call")?;
            let func_name = &expr[..paren_pos];
            let args_str = &expr[paren_pos + 1..expr.len() - 1];
            
            let op = Self::parse_v1_5_op(func_name)?;
            let args = Self::parse_v1_5_args(args_str)?;
            
            return Ok(Arg::Expr(op, args));
        }
        
        // Variable reference
        Ok(Arg::Ref(expr.to_string()))
    }
    
    /// Parse binary expression: "a + b" -> (a, "+", b)
    fn parse_v1_5_binary_expr(expr: &str) -> Option<(&str, &str, &str)> {
        let ops = [" + ", " - ", " * ", " / ", " % ", " == ", " != ", " <= ", " >= ", " < ", " > ", " && ", " || "];
        
        for op in ops {
            if let Some(pos) = expr.find(op) {
                let left = &expr[..pos];
                let right = &expr[pos + op.len()..];
                let op_str = op.trim();
                return Some((left.trim(), op_str, right.trim()));
            }
        }
        
        None
    }
}
