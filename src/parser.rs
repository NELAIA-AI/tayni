//! NELAIA v0.4 Parser
//! Parses data flow graph syntax into IR

use crate::ir::*;

pub struct Parser;

impl Parser {
    pub fn parse(content: &str) -> Result<Graph, String> {
        let mut graph = Graph::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut current_function: Option<(String, Vec<String>, Vec<Node>)> = None;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("--") {
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
        
        // Node definition: .id: value/operation
        if line.starts_with('.') && line.contains(':') {
            return Self::parse_node_def(line);
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
    
    fn parse_node_def(line: &str) -> Result<Option<Node>, String> {
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
        
        // Check if it's a reference to another node
        if rest.starts_with('.') && !rest.contains(' ') {
            let target = rest[1..].to_string();
            return Ok(Some(Node::Reference { id, target }));
        }
        
        // Check if it's a literal
        if let Ok(value) = Self::parse_literal(rest) {
            return Ok(Some(Node::Literal { id, value }));
        }
        
        // Must be an operation
        Self::parse_operation(&id, rest)
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
    
    fn parse_operation(id: &str, rest: &str) -> Result<Option<Node>, String> {
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
            // LOOP deprecated - use cyclic flow >> instead
            "PRT" => Ok(Op::Prt),
            "INP" => Ok(Op::Inp),
            "OPN" => Ok(Op::Opn),
            "ACC" => Ok(Op::Acc),
            "GET" => Ok(Op::Get),
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
            s if s.starts_with('.') => Ok(Op::Call(s[1..].to_string())),
            _ => Err(format!("Unknown operation: {}", s)),
        }
    }
    
    fn parse_args(tokens: &[String]) -> Result<Vec<Arg>, String> {
        let mut args = Vec::new();
        let mut i = 0;
        
        while i < tokens.len() {
            let token = &tokens[i];
            
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
            let first = s.split(" > ").next().unwrap_or(s).split(" >> ").next().unwrap().trim();
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
}
