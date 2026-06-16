//! TAYNI v0.8 LLVM IR Emitter
//! Generates pure LLVM IR with direct syscalls (no libc)
//! Supports Linux (direct syscalls) and Windows (kernel32.dll)

use crate::ir::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum TargetPlatform {
    Linux,
    Windows,
}

/// Stored sub-graph for later emission as function
#[derive(Clone)]
struct SubGraphDef {
    inputs: Vec<String>,
    outputs: Vec<String>,
    nodes: Vec<Node>,
}

pub struct Emitter {
    /// String constants
    strings: Vec<(String, String)>,
    /// Counter for generating unique IDs
    counter: usize,
    /// Variable types
    var_types: HashMap<String, VarType>,
    /// Known constant values for folding
    constants: HashMap<String, i64>,
    /// Target platform
    target: TargetPlatform,
    /// Sub-graph definitions (to emit as functions)
    subgraphs: HashMap<String, SubGraphDef>,
    /// Current loop nesting depth
    loop_depth: usize,
}

#[derive(Clone, Debug)]
enum VarType {
    Int,
    Float,
    String,
    Fd,
}

impl Emitter {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let target = TargetPlatform::Windows;
        #[cfg(not(target_os = "windows"))]
        let target = TargetPlatform::Linux;
        
        Emitter {
            strings: Vec::new(),
            counter: 0,
            var_types: HashMap::new(),
            constants: HashMap::new(),
            target,
            subgraphs: HashMap::new(),
            loop_depth: 0,
        }
    }
    
    pub fn new_with_target(target: TargetPlatform) -> Self {
        Emitter {
            strings: Vec::new(),
            counter: 0,
            var_types: HashMap::new(),
            constants: HashMap::new(),
            target,
            subgraphs: HashMap::new(),
            loop_depth: 0,
        }
    }
    
    fn next_id(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
    
    pub fn emit(graph: &Graph) -> Result<String, String> {
        let mut emitter = Emitter::new();
        emitter.emit_graph(graph)
    }
    
    pub fn emit_for_target(graph: &Graph, target: TargetPlatform) -> Result<String, String> {
        let mut emitter = Emitter::new_with_target(target);
        emitter.emit_graph(graph)
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> Result<String, String> {
        let mut main_body = String::new();
        
        // First pass: collect constants for folding
        for node in &graph.nodes {
            self.collect_constants(node);
        }
        
        // Second pass: collect string literals and analyze types
        for node in &graph.nodes {
            self.analyze_node(node);
        }
        
        // Third pass: generate code
        for node in &graph.nodes {
            main_body.push_str(&self.emit_node(node)?);
        }
        
        // Build the complete LLVM IR
        let mut ir = String::new();
        
        // Header
        ir.push_str("; TAYNI v0.6 Auto-Generated LLVM IR\n");
        ir.push_str("; Pure syscalls - NO libc dependency\n");
        
        match self.target {
            TargetPlatform::Windows => {
                ir.push_str("target triple = \"x86_64-pc-windows-msvc\"\n\n");
            }
            TargetPlatform::Linux => {
                ir.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");
            }
        }
        
        // String constants
        for (id, value) in &self.strings {
            let escaped = self.escape_string(value);
            let len = value.len() + 1;
            ir.push_str(&format!(
                "@{} = private constant [{} x i8] c\"{}\\00\"\n",
                id, len, escaped
            ));
        }
        ir.push_str("\n");
        
        // Platform-specific syscall layer
        match self.target {
            TargetPlatform::Linux => ir.push_str(&self.emit_linux_syscalls()),
            TargetPlatform::Windows => ir.push_str(&self.emit_windows_syscalls()),
        }
        ir.push_str("\n");
        
        // Common runtime (itoa, strlen, etc)
        ir.push_str(&self.emit_common_runtime());
        ir.push_str("\n");
        
        // Sub-graph functions (before main)
        ir.push_str(&self.emit_subgraph_functions()?);
        
        // Main function
        ir.push_str("define i32 @TAYNI_main() {\n");
        ir.push_str("entry:\n");
        ir.push_str(&main_body);
        ir.push_str("  ret i32 0\n");
        ir.push_str("}\n\n");
        
        // Entry point
        match self.target {
            TargetPlatform::Linux => ir.push_str(&self.emit_linux_entry()),
            TargetPlatform::Windows => ir.push_str(&self.emit_windows_entry()),
        }
        
        Ok(ir)
    }
    
    /// Collect constant values for folding
    fn collect_constants(&mut self, node: &Node) {
        match node {
            Node::Literal { id, value: Value::Int(n) } => {
                self.constants.insert(id.clone(), *n);
            }
            Node::Operation { id, op, args } => {
                if let Some(result) = self.try_fold(op, args) {
                    self.constants.insert(id.clone(), result);
                }
            }
            _ => {}
        }
    }
    
    /// Try to fold a constant expression
    fn try_fold(&self, op: &Op, args: &[Arg]) -> Option<i64> {
        let get_const = |arg: &Arg| -> Option<i64> {
            match arg {
                Arg::Lit(Value::Int(n)) => Some(*n),
                Arg::Ref(name) => self.constants.get(name).copied(),
                _ => None,
            }
        };
        
        match op {
            Op::Add if args.len() >= 2 => {
                Some(get_const(&args[0])? + get_const(&args[1])?)
            }
            Op::Sub if args.len() >= 2 => {
                Some(get_const(&args[0])? - get_const(&args[1])?)
            }
            Op::Mul if args.len() >= 2 => {
                Some(get_const(&args[0])? * get_const(&args[1])?)
            }
            Op::Div if args.len() >= 2 => {
                let b = get_const(&args[1])?;
                if b != 0 { Some(get_const(&args[0])? / b) } else { None }
            }
            Op::Mod if args.len() >= 2 => {
                let b = get_const(&args[1])?;
                if b != 0 { Some(get_const(&args[0])? % b) } else { None }
            }
            Op::Neg if !args.is_empty() => {
                Some(-get_const(&args[0])?)
            }
            _ => None,
        }
    }
    
    fn analyze_node(&mut self, node: &Node) {
        match node {
            Node::Literal { id, value } => {
                match value {
                    Value::String(s) => {
                        let str_id = format!("str_{}", self.next_id());
                        self.strings.push((str_id, s.clone()));
                        self.var_types.insert(id.clone(), VarType::String);
                    }
                    Value::Int(_) => {
                        self.var_types.insert(id.clone(), VarType::Int);
                    }
                    Value::Float(_) => {
                        self.var_types.insert(id.clone(), VarType::Float);
                    }
                    _ => {}
                }
            }
            Node::Operation { id, op, .. } => {
                // Arithmetic and comparison operations produce integers
                match op {
                    Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod | Op::Neg |
                    Op::Eq | Op::Ne | Op::Lt | Op::Gt | Op::Le | Op::Ge |
                    Op::And | Op::Or | Op::Not | Op::Len => {
                        self.var_types.insert(id.clone(), VarType::Int);
                    }
                    _ => {}
                }
            }
            Node::Flow { source, .. } => {
                if let Arg::Lit(Value::String(s)) = source {
                    let str_id = format!("str_{}", self.next_id());
                    self.strings.push((str_id, s.clone()));
                }
            }
            _ => {}
        }
    }
    
    fn emit_node(&mut self, node: &Node) -> Result<String, String> {
        match node {
            Node::Literal { id, value } => self.emit_literal(id, value),
            Node::Reference { id, target } => self.emit_reference(id, target),
            Node::Operation { id, op, args } => self.emit_operation(id, op, args),
            Node::Flow { source, dest } => self.emit_flow(source, dest),
            Node::SubGraph { id, inputs, outputs, nodes } => {
                self.emit_subgraph(id, inputs, outputs, nodes)
            }
            Node::Label(_) => Ok(String::new()),
        }
    }
    
    fn emit_subgraph(&mut self, id: &str, inputs: &[String], outputs: &[String], nodes: &[Node]) -> Result<String, String> {
        // Store sub-graph for later emission as a function
        self.subgraphs.insert(id.to_string(), SubGraphDef {
            inputs: inputs.to_vec(),
            outputs: outputs.to_vec(),
            nodes: nodes.to_vec(),
        });
        
        // No inline code - function will be emitted separately
        Ok(format!("  ; subgraph {} registered as function\n", id))
    }
    
    /// Emit all stored sub-graphs as LLVM functions
    fn emit_subgraph_functions(&mut self) -> Result<String, String> {
        let mut code = String::new();
        
        // Clone subgraphs to avoid borrow issues
        let subgraphs: Vec<(String, SubGraphDef)> = self.subgraphs.clone().into_iter().collect();
        
        for (name, def) in subgraphs {
            code.push_str(&format!("\n; Sub-graph function: {}\n", name));
            
            // Build parameter list
            let params: Vec<String> = def.inputs.iter()
                .map(|p| format!("i64 %{}", p))
                .collect();
            
            code.push_str(&format!("define i64 @TAYNI_func_{}({}) {{\n", name, params.join(", ")));
            code.push_str("entry:\n");
            
            // Emit nodes inside the function
            for node in &def.nodes {
                code.push_str(&self.emit_node(node)?);
            }
            
            // Return the first output (or 0 if none)
            if let Some(out) = def.outputs.first() {
                code.push_str(&format!("  ret i64 %{}\n", out));
            } else {
                code.push_str("  ret i64 0\n");
            }
            
            code.push_str("}\n");
        }
        
        Ok(code)
    }
    
    fn emit_literal(&mut self, id: &str, value: &Value) -> Result<String, String> {
        match value {
            Value::Int(n) => {
                // Use folded constant if available
                let val = self.constants.get(id).copied().unwrap_or(*n);
                Ok(format!("  %{} = add i64 0, {}\n", id, val))
            }
            Value::Float(f) => {
                Ok(format!("  %{} = fadd double 0.0, {}\n", id, f))
            }
            Value::String(s) => {
                let str_id = self.strings.iter()
                    .find(|(_, v)| v == s)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| {
                        let new_id = format!("str_{}", self.next_id());
                        self.strings.push((new_id.clone(), s.clone()));
                        new_id
                    });
                
                let len = s.len() + 1;
                Ok(format!(
                    "  %{} = getelementptr [{} x i8], [{} x i8]* @{}, i32 0, i32 0\n",
                    id, len, len, str_id
                ))
            }
            Value::Pair(a, b) => {
                let a_id = format!("{}_a", id);
                let b_id = format!("{}_b", id);
                let mut code = self.emit_literal(&a_id, a)?;
                code.push_str(&self.emit_literal(&b_id, b)?);
                Ok(code)
            }
        }
    }
    
    fn emit_reference(&self, id: &str, target: &str) -> Result<String, String> {
        // Just create an alias
        Ok(format!("  ; %{} = %{} (reference)\n", id, target))
    }
    
    fn emit_operation(&mut self, id: &str, op: &Op, args: &[Arg]) -> Result<String, String> {
        // Check if this operation was folded to a constant
        if let Some(&folded_value) = self.constants.get(id) {
            return Ok(format!("  %{} = add i64 0, {}  ; folded constant\n", id, folded_value));
        }
        
        match op {
            Op::Add => self.emit_binary_op(id, "add i64", args),
            Op::Sub => self.emit_binary_op(id, "sub i64", args),
            Op::Mul => self.emit_binary_op(id, "mul i64", args),
            Op::Div => self.emit_binary_op(id, "sdiv i64", args),
            Op::Mod => self.emit_binary_op(id, "srem i64", args),
            Op::Neg => self.emit_unary_op(id, "sub i64 0,", args),
            Op::Eq => self.emit_cmp_op(id, "icmp eq i64", args),
            Op::Ne => self.emit_cmp_op(id, "icmp ne i64", args),
            Op::Lt => self.emit_cmp_op(id, "icmp slt i64", args),
            Op::Gt => self.emit_cmp_op(id, "icmp sgt i64", args),
            Op::Le => self.emit_cmp_op(id, "icmp sle i64", args),
            Op::Ge => self.emit_cmp_op(id, "icmp sge i64", args),
            Op::And => self.emit_binary_op(id, "and i64", args),
            Op::Or => self.emit_binary_op(id, "or i64", args),
            Op::Not => {
                let mut code = String::new();
                let arg = self.emit_arg(&args[0])?;
                code.push_str(&format!("  %{}_cmp = icmp eq i64 {}, 0\n", id, arg));
                code.push_str(&format!("  %{} = zext i1 %{}_cmp to i64\n", id, id));
                Ok(code)
            }
            Op::Brn => self.emit_branch(id, args),
            Op::Prt => {
                let arg = self.emit_arg(&args[0])?;
                Ok(format!("  call i64 @TAYNI_println(i8* {})\n", arg))
            }
            Op::Opn => self.emit_file_open(id, args),
            Op::Get => self.emit_file_read(id, args),
            Op::Put => self.emit_file_write(id, args),
            Op::Cls => self.emit_file_close(id, args),
            // Network operations
            Op::Tcp => self.emit_tcp_socket(id, args),
            Op::Udp => self.emit_udp_socket(id, args),
            Op::Bnd => self.emit_tcp_bind(id, args),
            Op::Lst => self.emit_tcp_listen(id, args),
            Op::Acc => self.emit_tcp_accept(id, args),
            Op::Con => self.emit_tcp_connect(id, args),
            Op::Xmt => self.emit_tcp_send(id, args),
            Op::Rcv => self.emit_tcp_recv(id, args),
            // Memory operations
            Op::Alc => self.emit_alloc(id, args),
            Op::Fre => self.emit_free(id, args),
            // Control flow
            Op::Loop => self.emit_loop(id, args),
            Op::Brk => Ok(format!("  br label %loop_end_{}\n", self.loop_depth)),
            Op::Cnt => Ok(format!("  br label %loop_start_{}\n", self.loop_depth)),
            // Error handling
            Op::Chk => self.emit_check(id, args),
            Op::Call(func_name) => self.emit_call(id, func_name, args),
            _ => Ok(format!("  ; TODO: {} operation\n", format!("{:?}", op))),
        }
    }
    
    // ========== NETWORK OPERATIONS ==========
    
    /// TCP socket: TCP -> fd
    fn emit_tcp_socket(&mut self, id: &str, _args: &[Arg]) -> Result<String, String> {
        Ok(format!("  %{} = call i64 @TAYNI_tcp_socket()\n", id))
    }
    
    /// UDP socket: UDP -> fd
    fn emit_udp_socket(&mut self, id: &str, _args: &[Arg]) -> Result<String, String> {
        Ok(format!("  %{} = call i64 @TAYNI_udp_socket()\n", id))
    }
    
    /// TCP bind: BND .fd port
    fn emit_tcp_bind(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 2 {
            return Err("BND requires fd and port arguments".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let port = self.emit_arg(&args[1])?;
        Ok(format!("  %{} = call i64 @TAYNI_tcp_bind(i64 {}, i64 {})\n", id, fd, port))
    }
    
    /// TCP listen: LST .fd backlog
    fn emit_tcp_listen(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("LST requires fd argument".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let backlog = if args.len() > 1 {
            self.emit_arg(&args[1])?
        } else {
            "10".to_string()
        };
        Ok(format!("  %{} = call i64 @TAYNI_tcp_listen(i64 {}, i64 {})\n", id, fd, backlog))
    }
    
    /// TCP accept: ACC .fd -> client_fd
    fn emit_tcp_accept(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("ACC requires fd argument".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        Ok(format!("  %{} = call i64 @TAYNI_tcp_accept(i64 {})\n", id, fd))
    }
    
    /// TCP connect: CON .fd "host" port
    fn emit_tcp_connect(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 3 {
            return Err("CON requires fd, host, port arguments".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let host = self.emit_arg(&args[1])?;
        let port = self.emit_arg(&args[2])?;
        Ok(format!("  %{} = call i64 @TAYNI_tcp_connect(i64 {}, i8* {}, i64 {})\n", id, fd, host, port))
    }
    
    /// TCP send: SND .fd "data" or SND .fd .buf .len
    fn emit_tcp_send(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 2 {
            return Err("SND requires fd and data arguments".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        
        match &args[1] {
            Arg::Lit(Value::String(s)) => {
                let str_id = self.strings.iter()
                    .find(|(_, v)| v == s)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| {
                        let new_id = format!("str_{}", self.next_id());
                        self.strings.push((new_id.clone(), s.clone()));
                        new_id
                    });
                let len = s.len();
                Ok(format!(
                    "  %{} = call i64 @TAYNI_tcp_send(i64 {}, i8* getelementptr ([{} x i8], [{} x i8]* @{}, i32 0, i32 0), i64 {})\n",
                    id, fd, len + 1, len + 1, str_id, len
                ))
            }
            Arg::Ref(ref_name) => {
                // Reference to a string - need to calculate length at runtime
                let buf = self.emit_arg(&args[1])?;
                Ok(format!(
                    "  %{}_len = call i64 @_TAYNI_str_len(i8* {})\n  %{} = call i64 @TAYNI_tcp_send(i64 {}, i8* {}, i64 %{}_len)\n",
                    id, buf, id, fd, buf, id
                ))
            }
            _ => {
                let buf = self.emit_arg(&args[1])?;
                let len = if args.len() > 2 {
                    self.emit_arg(&args[2])?
                } else {
                    "0".to_string()
                };
                Ok(format!("  %{} = call i64 @TAYNI_tcp_send(i64 {}, i8* {}, i64 {})\n", id, fd, buf, len))
            }
        }
    }
    
    /// TCP recv: RCV .fd .buf .maxlen -> bytes_read
    fn emit_tcp_recv(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 3 {
            return Err("RCV requires fd, buffer, maxlen arguments".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let buf = self.emit_arg(&args[1])?;
        let maxlen = self.emit_arg(&args[2])?;
        Ok(format!("  %{} = call i64 @TAYNI_tcp_recv(i64 {}, i8* {}, i64 {})\n", id, fd, buf, maxlen))
    }
    
    // ========== MEMORY OPERATIONS ==========
    
    /// Allocate memory: ALC size -> ptr
    fn emit_alloc(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("ALC requires size argument".to_string());
        }
        let size = self.emit_arg(&args[0])?;
        Ok(format!("  %{}_ptr = call i8* @TAYNI_alloc(i64 {})\n  %{} = ptrtoint i8* %{}_ptr to i64\n", id, size, id, id))
    }
    
    /// Free memory: FRE .ptr
    fn emit_free(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("FRE requires ptr argument".to_string());
        }
        let ptr = self.emit_arg(&args[0])?;
        Ok(format!("  %{}_ptr = inttoptr i64 {} to i8*\n  %{} = call i64 @TAYNI_free(i8* %{}_ptr)\n", id, ptr, id, id))
    }
    
    // ========== CONTROL FLOW ==========
    
    /// Loop: LOOP condition body
    fn emit_loop(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        self.loop_depth += 1;
        let depth = self.loop_depth;
        
        // Get max iterations (default 1000 to prevent infinite loops)
        let max_iter = if !args.is_empty() {
            self.emit_arg(&args[0])?
        } else {
            "1000".to_string()
        };
        
        let code = format!(
            r#"  br label %loop_start_{depth}
loop_start_{depth}:
  %loop_i_{depth} = phi i64 [ 0, %entry ], [ %loop_next_{depth}, %loop_body_{depth} ]
  %loop_cond_{depth} = icmp slt i64 %loop_i_{depth}, {max_iter}
  br i1 %loop_cond_{depth}, label %loop_body_{depth}, label %loop_end_{depth}
loop_body_{depth}:
  %{id} = add i64 %loop_i_{depth}, 0
  %loop_next_{depth} = add i64 %loop_i_{depth}, 1
  br label %loop_start_{depth}
loop_end_{depth}:
"#,
            depth = depth,
            max_iter = max_iter,
            id = id
        );
        
        self.loop_depth -= 1;
        Ok(code)
    }
    
    /// Check error: CHK value error_label
    fn emit_check(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("CHK requires value argument".to_string());
        }
        
        let value = self.emit_arg(&args[0])?;
        let error_code = if args.len() > 1 {
            self.emit_arg(&args[1])?
        } else {
            "-1".to_string()
        };
        
        Ok(format!(
            r#"  %{id}_cmp = icmp slt i64 {value}, 0
  %{id} = select i1 %{id}_cmp, i64 {error_code}, i64 {value}
"#,
            id = id,
            value = value,
            error_code = error_code
        ))
    }
    
    /// Emit file open: OPN "filename" mode
    fn emit_file_open(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("OPN requires filename argument".to_string());
        }
        
        let filename = self.emit_arg(&args[0])?;
        // Mode: 0 = read, 1 = write, 2 = append (default: read)
        let mode = if args.len() > 1 {
            self.emit_arg(&args[1])?
        } else {
            "0".to_string()
        };
        
        Ok(format!("  %{} = call i64 @TAYNI_file_open(i8* {}, i64 {})\n", id, filename, mode))
    }
    
    /// Emit file read: GET .fd .buffer .size
    fn emit_file_read(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 3 {
            return Err("GET requires fd, buffer, size arguments".to_string());
        }
        
        let fd = self.emit_arg(&args[0])?;
        let buf = self.emit_arg(&args[1])?;
        let size = self.emit_arg(&args[2])?;
        
        Ok(format!("  %{} = call i64 @TAYNI_file_read(i64 {}, i8* {}, i64 {})\n", id, fd, buf, size))
    }
    
    /// Emit file write: PUT .fd "data" or PUT .fd .buffer .size
    fn emit_file_write(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("PUT requires fd argument".to_string());
        }
        
        let fd = self.emit_arg(&args[0])?;
        
        if args.len() >= 2 {
            match &args[1] {
                Arg::Lit(Value::String(s)) => {
                    let str_id = self.strings.iter()
                        .find(|(_, v)| v == s)
                        .map(|(id, _)| id.clone())
                        .unwrap_or_else(|| {
                            let new_id = format!("str_{}", self.next_id());
                            self.strings.push((new_id.clone(), s.clone()));
                            new_id
                        });
                    let len = s.len();
                    Ok(format!(
                        "  %{} = call i64 @TAYNI_file_write(i64 {}, i8* getelementptr ([{} x i8], [{} x i8]* @{}, i32 0, i32 0), i64 {})\n",
                        id, fd, len + 1, len + 1, str_id, len
                    ))
                }
                _ => {
                    let buf = self.emit_arg(&args[1])?;
                    let size = if args.len() > 2 {
                        self.emit_arg(&args[2])?
                    } else {
                        "0".to_string()
                    };
                    Ok(format!("  %{} = call i64 @TAYNI_file_write(i64 {}, i8* {}, i64 {})\n", id, fd, buf, size))
                }
            }
        } else {
            Err("PUT requires data argument".to_string())
        }
    }
    
    /// Emit file close: CLS .fd
    fn emit_file_close(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("CLS requires fd argument".to_string());
        }
        
        let fd = self.emit_arg(&args[0])?;
        // Use socket_close for network fds (works for both sockets and files on most cases)
        match self.target {
            TargetPlatform::Windows => {
                // Try closesocket first (for sockets), then CloseHandle (for files)
                Ok(format!("  %{} = call i64 @TAYNI_socket_close(i64 {})\n", id, fd))
            }
            TargetPlatform::Linux => {
                Ok(format!("  %{} = call i64 @TAYNI_file_close(i64 {})\n", id, fd))
            }
        }
    }
    
    /// Emit branch operation: BRN .cond .then_val .else_val
    fn emit_branch(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 3 {
            return Err("BRN requires 3 arguments: condition, then_value, else_value".to_string());
        }
        
        let cond = self.emit_arg(&args[0])?;
        let then_val = self.emit_arg(&args[1])?;
        let else_val = self.emit_arg(&args[2])?;
        
        let mut code = String::new();
        code.push_str(&format!("  %{}_cond = icmp ne i64 {}, 0\n", id, cond));
        code.push_str(&format!("  %{} = select i1 %{}_cond, i64 {}, i64 {}\n", 
            id, id, then_val, else_val));
        Ok(code)
    }
    
    /// Emit function call
    fn emit_call(&mut self, id: &str, func_name: &str, args: &[Arg]) -> Result<String, String> {
        let mut code = String::new();
        let mut arg_strs = Vec::new();
        
        for arg in args {
            arg_strs.push(format!("i64 {}", self.emit_arg(arg)?));
        }
        
        code.push_str(&format!("  %{} = call i64 @TAYNI_func_{}({})\n", 
            id, func_name, arg_strs.join(", ")));
        Ok(code)
    }
    
    fn emit_binary_op(&mut self, id: &str, op: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 2 {
            return Err(format!("Binary operation requires 2 arguments, got {}", args.len()));
        }
        let a = self.emit_arg(&args[0])?;
        let b = self.emit_arg(&args[1])?;
        Ok(format!("  %{} = {} {}, {}\n", id, op, a, b))
    }
    
    fn emit_unary_op(&mut self, id: &str, op: &str, args: &[Arg]) -> Result<String, String> {
        if args.is_empty() {
            return Err("Unary operation requires 1 argument".to_string());
        }
        let a = self.emit_arg(&args[0])?;
        Ok(format!("  %{} = {} {}\n", id, op, a))
    }
    
    fn emit_cmp_op(&mut self, id: &str, op: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 2 {
            return Err("Comparison requires 2 arguments".to_string());
        }
        let a = self.emit_arg(&args[0])?;
        let b = self.emit_arg(&args[1])?;
        let mut code = String::new();
        code.push_str(&format!("  %{}_cmp = {} {}, {}\n", id, op, a, b));
        code.push_str(&format!("  %{} = zext i1 %{}_cmp to i64\n", id, id));
        Ok(code)
    }
    
    fn emit_arg(&mut self, arg: &Arg) -> Result<String, String> {
        match arg {
            Arg::Ref(name) => Ok(format!("%{}", name)),
            Arg::Lit(Value::Int(n)) => Ok(format!("{}", n)),
            Arg::Lit(Value::Float(f)) => Ok(format!("{}", f)),
            Arg::Lit(Value::String(s)) => {
                // Find or create string constant
                let str_id = self.strings.iter()
                    .find(|(_, v)| v == s)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| {
                        let new_id = format!("str_{}", self.next_id());
                        self.strings.push((new_id.clone(), s.clone()));
                        new_id
                    });
                let len = s.len() + 1;
                Ok(format!("getelementptr ([{} x i8], [{} x i8]* @{}, i32 0, i32 0)", len, len, str_id))
            }
            Arg::Expr(op, inner_args) => {
                // Generate a temporary for the expression
                let tmp_id = format!("tmp_{}", self.next_id());
                // This is a simplification - in reality we'd need to emit the code first
                match op {
                    Op::Add => {
                        if inner_args.len() >= 2 {
                            let a = self.emit_arg(&inner_args[0])?;
                            let b = self.emit_arg(&inner_args[1])?;
                            Ok(format!("add (i64 {}, i64 {})", a, b))
                        } else {
                            Err("ADD requires 2 arguments".to_string())
                        }
                    }
                    _ => Ok(format!("%{}", tmp_id))
                }
            }
            _ => Err("Unsupported argument type".to_string()),
        }
    }
    
    fn emit_flow(&mut self, source: &Arg, dest: &FlowDest) -> Result<String, String> {
        match dest {
            FlowDest::Effect(Effect::Print) => {
                match source {
                    Arg::Lit(Value::String(s)) => {
                        let str_id = self.strings.iter()
                            .find(|(_, v)| v == s)
                            .map(|(id, _)| id.clone())
                            .unwrap_or_else(|| {
                                let new_id = format!("str_{}", self.next_id());
                                self.strings.push((new_id.clone(), s.clone()));
                                new_id
                            });
                        let len = s.len() + 1;
                        Ok(format!(
                            "  call i64 @TAYNI_println(i8* getelementptr ([{} x i8], [{} x i8]* @{}, i32 0, i32 0))\n",
                            len, len, str_id
                        ))
                    }
                    Arg::Lit(Value::Int(n)) => {
                        Ok(format!("  call i64 @TAYNI_print_int(i64 {})\n", n))
                    }
                    Arg::Ref(name) => {
                        // Check if we know the type
                        if let Some(var_type) = self.var_types.get(name) {
                            match var_type {
                                VarType::String => {
                                    Ok(format!("  call i64 @TAYNI_println(i8* %{})\n", name))
                                }
                                VarType::Int => {
                                    Ok(format!("  call i64 @TAYNI_print_int(i64 %{})\n", name))
                                }
                                VarType::Float => {
                                    // TODO: implement float printing
                                    Ok(format!("  ; TODO: print float %{}\n", name))
                                }
                                VarType::Fd => {
                                    Ok(format!("  ; TODO: print fd %{}\n", name))
                                }
                            }
                        } else {
                            // Default to integer for operations
                            Ok(format!("  call i64 @TAYNI_print_int(i64 %{})\n", name))
                        }
                    }
                    Arg::Expr(op, args) => {
                        // Emit the expression first, then print the result
                        let tmp_id = format!("flow_tmp_{}", self.next_id());
                        let mut code = self.emit_operation(&tmp_id, op, args)?;
                        code.push_str(&format!("  call i64 @TAYNI_print_int(i64 %{})\n", tmp_id));
                        Ok(code)
                    }
                    _ => Ok("  ; TODO: print unknown type\n".to_string())
                }
            }
            FlowDest::Node(target) => {
                Ok(format!("  ; flow to {}\n", target))
            }
            _ => Ok("  ; TODO: flow\n".to_string()),
        }
    }
    
    fn escape_string(&self, s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                '\n' => result.push_str("\\0A"),
                '\r' => result.push_str("\\0D"),
                '\t' => result.push_str("\\09"),
                '\0' => result.push_str("\\00"),
                '"' => result.push_str("\\22"),
                '\\' => result.push_str("\\5C"),
                c if c.is_ascii() && !c.is_control() => result.push(c),
                c => {
                    for b in c.to_string().as_bytes() {
                        result.push_str(&format!("\\{:02X}", b));
                    }
                }
            }
        }
        result
    }
    
    fn emit_linux_syscalls(&self) -> String {
        r#"
; ============================================================================
; LINUX x86_64 SYSCALL LAYER - Direct syscalls, No libc
; ============================================================================

; sys_write(fd, buf, count) -> bytes_written (syscall #1)
define i64 @sys_write(i32 %fd, i8* %buf, i64 %count) {
entry:
  %fd64 = sext i32 %fd to i64
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 1, i64 %fd64, i8* %buf, i64 %count)
  ret i64 %result
}

; sys_read(fd, buf, count) -> bytes_read (syscall #0)
define i64 @sys_read(i32 %fd, i8* %buf, i64 %count) {
entry:
  %fd64 = sext i32 %fd to i64
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 0, i64 %fd64, i8* %buf, i64 %count)
  ret i64 %result
}

; sys_open(filename, flags, mode) -> fd (syscall #2)
define i64 @sys_open(i8* %filename, i64 %flags, i64 %mode) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 2, i8* %filename, i64 %flags, i64 %mode)
  ret i64 %result
}

; sys_close(fd) -> result (syscall #3)
define i64 @sys_close(i64 %fd) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},~{rcx},~{r11},~{memory}"(
      i64 3, i64 %fd)
  ret i64 %result
}

; sys_exit(code) (syscall #60)
define void @sys_exit(i32 %code) {
entry:
  %code64 = sext i32 %code to i64
  call void asm sideeffect "syscall", "{rax},{rdi}"(i64 60, i64 %code64)
  unreachable
}

; ============================================================================
; FILE I/O WRAPPERS (Linux)
; ============================================================================

; Open file: mode 0=read, 1=write, 2=append
define i64 @TAYNI_file_open(i8* %filename, i64 %mode) {
entry:
  %is_read = icmp eq i64 %mode, 0
  br i1 %is_read, label %open_read, label %check_write

open_read:
  ; O_RDONLY = 0
  %fd_read = call i64 @sys_open(i8* %filename, i64 0, i64 0)
  ret i64 %fd_read

check_write:
  %is_write = icmp eq i64 %mode, 1
  br i1 %is_write, label %open_write, label %open_append

open_write:
  ; O_WRONLY | O_CREAT | O_TRUNC = 1 | 64 | 512 = 577
  %fd_write = call i64 @sys_open(i8* %filename, i64 577, i64 420)
  ret i64 %fd_write

open_append:
  ; O_WRONLY | O_CREAT | O_APPEND = 1 | 64 | 1024 = 1089
  %fd_append = call i64 @sys_open(i8* %filename, i64 1089, i64 420)
  ret i64 %fd_append
}

; Read from file
define i64 @TAYNI_file_read(i64 %fd, i8* %buf, i64 %size) {
entry:
  %fd32 = trunc i64 %fd to i32
  %result = call i64 @sys_read(i32 %fd32, i8* %buf, i64 %size)
  ret i64 %result
}

; Write to file
define i64 @TAYNI_file_write(i64 %fd, i8* %buf, i64 %size) {
entry:
  %fd32 = trunc i64 %fd to i32
  %result = call i64 @sys_write(i32 %fd32, i8* %buf, i64 %size)
  ret i64 %result
}

; Close file
define i64 @TAYNI_file_close(i64 %fd) {
entry:
  %result = call i64 @sys_close(i64 %fd)
  ret i64 %result
}

; ============================================================================
; NETWORK SYSCALLS (Linux)
; ============================================================================

; sys_socket(domain, type, protocol) -> fd (syscall #41)
define i64 @sys_socket(i64 %domain, i64 %type, i64 %protocol) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 41, i64 %domain, i64 %type, i64 %protocol)
  ret i64 %result
}

; sys_bind(fd, addr, addrlen) -> result (syscall #49)
define i64 @sys_bind(i64 %fd, i8* %addr, i64 %addrlen) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 49, i64 %fd, i8* %addr, i64 %addrlen)
  ret i64 %result
}

; sys_listen(fd, backlog) -> result (syscall #50)
define i64 @sys_listen(i64 %fd, i64 %backlog) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},~{rcx},~{r11},~{memory}"(
      i64 50, i64 %fd, i64 %backlog)
  ret i64 %result
}

; sys_accept(fd, addr, addrlen) -> client_fd (syscall #43)
define i64 @sys_accept(i64 %fd, i8* %addr, i64* %addrlen) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 43, i64 %fd, i8* %addr, i64* %addrlen)
  ret i64 %result
}

; sys_connect(fd, addr, addrlen) -> result (syscall #42)
define i64 @sys_connect(i64 %fd, i8* %addr, i64 %addrlen) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(
      i64 42, i64 %fd, i8* %addr, i64 %addrlen)
  ret i64 %result
}

; sys_sendto(fd, buf, len, flags, addr, addrlen) -> bytes_sent (syscall #44)
define i64 @sys_sendto(i64 %fd, i8* %buf, i64 %len, i64 %flags, i8* %addr, i64 %addrlen) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}"(
      i64 44, i64 %fd, i8* %buf, i64 %len, i64 %flags, i8* %addr, i64 %addrlen)
  ret i64 %result
}

; sys_recvfrom(fd, buf, len, flags, addr, addrlen) -> bytes_recv (syscall #45)
define i64 @sys_recvfrom(i64 %fd, i8* %buf, i64 %len, i64 %flags, i8* %addr, i64* %addrlen) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}"(
      i64 45, i64 %fd, i8* %buf, i64 %len, i64 %flags, i8* %addr, i64* %addrlen)
  ret i64 %result
}

; sys_mmap(addr, len, prot, flags, fd, offset) -> ptr (syscall #9)
define i8* @sys_mmap(i8* %addr, i64 %len, i64 %prot, i64 %flags, i64 %fd, i64 %offset) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}"(
      i64 9, i8* %addr, i64 %len, i64 %prot, i64 %flags, i64 %fd, i64 %offset)
  %ptr = inttoptr i64 %result to i8*
  ret i8* %ptr
}

; sys_munmap(addr, len) -> result (syscall #11)
define i64 @sys_munmap(i8* %addr, i64 %len) {
entry:
  %result = call i64 asm sideeffect "syscall",
    "={rax},{rax},{rdi},{rsi},~{rcx},~{r11},~{memory}"(
      i64 11, i8* %addr, i64 %len)
  ret i64 %result
}

; ============================================================================
; NETWORK WRAPPERS (Linux)
; ============================================================================

; sockaddr_in structure for IPv4
; struct sockaddr_in { sin_family (2), sin_port (2), sin_addr (4), padding (8) }
@.sockaddr_buf = private global [16 x i8] zeroinitializer

; Create TCP socket
define i64 @TAYNI_tcp_socket() {
entry:
  ; AF_INET = 2, SOCK_STREAM = 1, protocol = 0
  %fd = call i64 @sys_socket(i64 2, i64 1, i64 0)
  ret i64 %fd
}

; Create UDP socket
define i64 @TAYNI_udp_socket() {
entry:
  ; AF_INET = 2, SOCK_DGRAM = 2, protocol = 0
  %fd = call i64 @sys_socket(i64 2, i64 2, i64 0)
  ret i64 %fd
}

; Bind socket to port
define i64 @TAYNI_tcp_bind(i64 %fd, i64 %port) {
entry:
  ; Build sockaddr_in: AF_INET (2), port (big-endian), INADDR_ANY (0)
  %addr = getelementptr [16 x i8], [16 x i8]* @.sockaddr_buf, i32 0, i32 0
  ; sin_family = AF_INET = 2
  store i8 2, i8* %addr
  %addr1 = getelementptr i8, i8* %addr, i32 1
  store i8 0, i8* %addr1
  ; sin_port (big-endian)
  %port_hi = lshr i64 %port, 8
  %port_hi8 = trunc i64 %port_hi to i8
  %port_lo8 = trunc i64 %port to i8
  %addr2 = getelementptr i8, i8* %addr, i32 2
  store i8 %port_hi8, i8* %addr2
  %addr3 = getelementptr i8, i8* %addr, i32 3
  store i8 %port_lo8, i8* %addr3
  ; sin_addr = INADDR_ANY = 0
  %addr4 = getelementptr i8, i8* %addr, i32 4
  store i8 0, i8* %addr4
  %addr5 = getelementptr i8, i8* %addr, i32 5
  store i8 0, i8* %addr5
  %addr6 = getelementptr i8, i8* %addr, i32 6
  store i8 0, i8* %addr6
  %addr7 = getelementptr i8, i8* %addr, i32 7
  store i8 0, i8* %addr7
  ; Call bind
  %result = call i64 @sys_bind(i64 %fd, i8* %addr, i64 16)
  ret i64 %result
}

; Listen on socket
define i64 @TAYNI_tcp_listen(i64 %fd, i64 %backlog) {
entry:
  %result = call i64 @sys_listen(i64 %fd, i64 %backlog)
  ret i64 %result
}

; Accept connection
define i64 @TAYNI_tcp_accept(i64 %fd) {
entry:
  %result = call i64 @sys_accept(i64 %fd, i8* null, i64* null)
  ret i64 %result
}

; Connect to host:port (simplified - only supports IP addresses)
define i64 @TAYNI_tcp_connect(i64 %fd, i8* %host, i64 %port) {
entry:
  ; Parse IP address from string "x.x.x.x"
  %ip = call i32 @TAYNI_parse_ip(i8* %host)
  
  ; Build sockaddr_in
  %addr = getelementptr [16 x i8], [16 x i8]* @.sockaddr_buf, i32 0, i32 0
  store i8 2, i8* %addr
  %addr1 = getelementptr i8, i8* %addr, i32 1
  store i8 0, i8* %addr1
  %port_hi = lshr i64 %port, 8
  %port_hi8 = trunc i64 %port_hi to i8
  %port_lo8 = trunc i64 %port to i8
  %addr2 = getelementptr i8, i8* %addr, i32 2
  store i8 %port_hi8, i8* %addr2
  %addr3 = getelementptr i8, i8* %addr, i32 3
  store i8 %port_lo8, i8* %addr3
  
  ; Store parsed IP (already in network byte order from parse_ip)
  %addr4 = getelementptr i8, i8* %addr, i32 4
  %ip_ptr = bitcast i8* %addr4 to i32*
  store i32 %ip, i32* %ip_ptr
  
  %result = call i64 @sys_connect(i64 %fd, i8* %addr, i64 16)
  ret i64 %result
}

; Parse IP address string "x.x.x.x" to 32-bit integer (network byte order)
define i32 @TAYNI_parse_ip(i8* %str) #1 {
entry:
  %octet1 = alloca i32
  %octet2 = alloca i32
  %octet3 = alloca i32
  %octet4 = alloca i32
  store i32 0, i32* %octet1
  store i32 0, i32* %octet2
  store i32 0, i32* %octet3
  store i32 0, i32* %octet4
  
  %current = alloca i32*
  store i32* %octet1, i32** %current
  
  br label %loop

loop:
  %i = phi i64 [ 0, %entry ], [ %next_i, %continue ]
  %ptr = getelementptr i8, i8* %str, i64 %i
  %char = load i8, i8* %ptr
  %is_null = icmp eq i8 %char, 0
  br i1 %is_null, label %done, label %process

process:
  %is_dot = icmp eq i8 %char, 46  ; '.'
  br i1 %is_dot, label %next_octet, label %digit

digit:
  %is_digit = icmp uge i8 %char, 48  ; >= '0'
  %is_digit2 = icmp ule i8 %char, 57  ; <= '9'
  %valid = and i1 %is_digit, %is_digit2
  br i1 %valid, label %add_digit, label %continue

add_digit:
  %cur_ptr = load i32*, i32** %current
  %cur_val = load i32, i32* %cur_ptr
  %mul = mul i32 %cur_val, 10
  %digit_val = sub i8 %char, 48
  %digit_i32 = zext i8 %digit_val to i32
  %new_val = add i32 %mul, %digit_i32
  store i32 %new_val, i32* %cur_ptr
  br label %continue

next_octet:
  %cur_ptr2 = load i32*, i32** %current
  %is_o1 = icmp eq i32* %cur_ptr2, %octet1
  br i1 %is_o1, label %set_o2, label %check_o2

set_o2:
  store i32* %octet2, i32** %current
  br label %continue

check_o2:
  %is_o2 = icmp eq i32* %cur_ptr2, %octet2
  br i1 %is_o2, label %set_o3, label %set_o4

set_o3:
  store i32* %octet3, i32** %current
  br label %continue

set_o4:
  store i32* %octet4, i32** %current
  br label %continue

continue:
  %next_i = add i64 %i, 1
  br label %loop

done:
  ; Combine octets into 32-bit IP (network byte order = big endian)
  %o1 = load i32, i32* %octet1
  %o2 = load i32, i32* %octet2
  %o3 = load i32, i32* %octet3
  %o4 = load i32, i32* %octet4
  
  %o1_8 = trunc i32 %o1 to i8
  %o2_8 = trunc i32 %o2 to i8
  %o3_8 = trunc i32 %o3 to i8
  %o4_8 = trunc i32 %o4 to i8
  
  ; Network byte order: first octet at lowest address
  %r1 = zext i8 %o1_8 to i32
  %r2 = zext i8 %o2_8 to i32
  %r3 = zext i8 %o3_8 to i32
  %r4 = zext i8 %o4_8 to i32
  
  %s2 = shl i32 %r2, 8
  %s3 = shl i32 %r3, 16
  %s4 = shl i32 %r4, 24
  
  %t1 = or i32 %r1, %s2
  %t2 = or i32 %t1, %s3
  %result = or i32 %t2, %s4
  
  ret i32 %result
}

; Send data
define i64 @TAYNI_tcp_send(i64 %fd, i8* %buf, i64 %len) {
entry:
  %result = call i64 @sys_sendto(i64 %fd, i8* %buf, i64 %len, i64 0, i8* null, i64 0)
  ret i64 %result
}

; Receive data
define i64 @TAYNI_tcp_recv(i64 %fd, i8* %buf, i64 %maxlen) {
entry:
  %result = call i64 @sys_recvfrom(i64 %fd, i8* %buf, i64 %maxlen, i64 0, i8* null, i64* null)
  ret i64 %result
}

; ============================================================================
; MEMORY WRAPPERS (Linux)
; ============================================================================

; Allocate memory using mmap
define i8* @TAYNI_alloc(i64 %size) {
entry:
  ; PROT_READ | PROT_WRITE = 3, MAP_PRIVATE | MAP_ANONYMOUS = 34
  %ptr = call i8* @sys_mmap(i8* null, i64 %size, i64 3, i64 34, i64 -1, i64 0)
  ret i8* %ptr
}

; Free memory using munmap
define i64 @TAYNI_free(i8* %ptr) {
entry:
  ; Note: munmap needs the size, but we don't track it. Use a page size.
  %result = call i64 @sys_munmap(i8* %ptr, i64 4096)
  ret i64 %result
}
"#.to_string()
    }
    
    fn emit_windows_syscalls(&self) -> String {
        r#"
; ============================================================================
; WINDOWS x64 SYSCALL LAYER - Via kernel32.dll
; ============================================================================

; External declarations for kernel32 functions
declare dllimport i8* @GetStdHandle(i32) #0
declare dllimport i32 @WriteFile(i8*, i8*, i32, i32*, i8*) #0
declare dllimport i32 @ReadFile(i8*, i8*, i32, i32*, i8*) #0
declare dllimport i8* @CreateFileA(i8*, i32, i32, i8*, i32, i32, i8*) #0
declare dllimport i32 @CloseHandle(i8*) #0
declare dllimport void @ExitProcess(i32) #0

attributes #0 = { nounwind }

; Global handle for stdout (initialized at startup)
@.stdout_handle = private global i8* null

; Initialize stdout handle
define void @TAYNI_init_io() {
entry:
  ; STD_OUTPUT_HANDLE = -11
  %handle = call i8* @GetStdHandle(i32 -11)
  store i8* %handle, i8** @.stdout_handle
  ret void
}

; sys_write compatible wrapper for Windows
define i64 @sys_write(i32 %fd, i8* %buf, i64 %count) {
entry:
  %handle = load i8*, i8** @.stdout_handle
  %count32 = trunc i64 %count to i32
  %written = alloca i32
  store i32 0, i32* %written
  %result = call i32 @WriteFile(i8* %handle, i8* %buf, i32 %count32, i32* %written, i8* null)
  %written_val = load i32, i32* %written
  %written64 = sext i32 %written_val to i64
  ret i64 %written64
}

; sys_exit compatible wrapper for Windows
define void @sys_exit(i32 %code) {
entry:
  call void @ExitProcess(i32 %code)
  unreachable
}

; ============================================================================
; FILE I/O WRAPPERS (Windows)
; ============================================================================

; Open file: mode 0=read, 1=write, 2=append
define i64 @TAYNI_file_open(i8* %filename, i64 %mode) {
entry:
  %is_read = icmp eq i64 %mode, 0
  br i1 %is_read, label %open_read, label %check_write

open_read:
  ; GENERIC_READ = 0x80000000, OPEN_EXISTING = 3
  %handle_read = call i8* @CreateFileA(i8* %filename, i32 -2147483648, i32 1, i8* null, i32 3, i32 128, i8* null)
  %fd_read = ptrtoint i8* %handle_read to i64
  ret i64 %fd_read

check_write:
  %is_write = icmp eq i64 %mode, 1
  br i1 %is_write, label %open_write, label %open_append

open_write:
  ; GENERIC_WRITE = 0x40000000, CREATE_ALWAYS = 2
  %handle_write = call i8* @CreateFileA(i8* %filename, i32 1073741824, i32 0, i8* null, i32 2, i32 128, i8* null)
  %fd_write = ptrtoint i8* %handle_write to i64
  ret i64 %fd_write

open_append:
  ; GENERIC_WRITE = 0x40000000, OPEN_ALWAYS = 4
  %handle_append = call i8* @CreateFileA(i8* %filename, i32 1073741824, i32 0, i8* null, i32 4, i32 128, i8* null)
  %fd_append = ptrtoint i8* %handle_append to i64
  ret i64 %fd_append
}

; Read from file
define i64 @TAYNI_file_read(i64 %fd, i8* %buf, i64 %size) {
entry:
  %handle = inttoptr i64 %fd to i8*
  %size32 = trunc i64 %size to i32
  %bytes_read = alloca i32
  store i32 0, i32* %bytes_read
  call i32 @ReadFile(i8* %handle, i8* %buf, i32 %size32, i32* %bytes_read, i8* null)
  %result = load i32, i32* %bytes_read
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; Write to file
define i64 @TAYNI_file_write(i64 %fd, i8* %buf, i64 %size) {
entry:
  %handle = inttoptr i64 %fd to i8*
  %size32 = trunc i64 %size to i32
  %bytes_written = alloca i32
  store i32 0, i32* %bytes_written
  call i32 @WriteFile(i8* %handle, i8* %buf, i32 %size32, i32* %bytes_written, i8* null)
  %result = load i32, i32* %bytes_written
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; Close file
define i64 @TAYNI_file_close(i64 %fd) {
entry:
  %handle = inttoptr i64 %fd to i8*
  %result = call i32 @CloseHandle(i8* %handle)
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; ============================================================================
; NETWORK (Windows - Winsock2)
; ============================================================================

; Winsock2 declarations
declare dllimport i32 @WSAStartup(i16, i8*) #0
declare dllimport i64 @socket(i32, i32, i32) #0
declare dllimport i32 @bind(i64, i8*, i32) #0
declare dllimport i32 @listen(i64, i32) #0
declare dllimport i64 @accept(i64, i8*, i32*) #0
declare dllimport i32 @connect(i64, i8*, i32) #0
declare dllimport i32 @send(i64, i8*, i32, i32) #0
declare dllimport i32 @recv(i64, i8*, i32, i32) #0
declare dllimport i32 @closesocket(i64) #0
declare dllimport i32 @shutdown(i64, i32) #0
declare dllimport i32 @WSACleanup() #0

; Memory declarations
declare dllimport i8* @VirtualAlloc(i8*, i64, i32, i32) #0
declare dllimport i32 @VirtualFree(i8*, i64, i32) #0

; WSA data buffer
@.wsa_data = private global [512 x i8] zeroinitializer
@.wsa_initialized = private global i32 0

; sockaddr_in buffer for Windows
@.win_sockaddr = private global [16 x i8] zeroinitializer

; Initialize Winsock
define void @TAYNI_wsa_init() {
entry:
  %initialized = load i32, i32* @.wsa_initialized
  %need_init = icmp eq i32 %initialized, 0
  br i1 %need_init, label %do_init, label %done

do_init:
  %wsa_data = getelementptr [512 x i8], [512 x i8]* @.wsa_data, i32 0, i32 0
  ; MAKEWORD(2, 2) = 0x0202
  call i32 @WSAStartup(i16 514, i8* %wsa_data)
  store i32 1, i32* @.wsa_initialized
  br label %done

done:
  ret void
}

; Create TCP socket
define i64 @TAYNI_tcp_socket() {
entry:
  call void @TAYNI_wsa_init()
  ; AF_INET = 2, SOCK_STREAM = 1, IPPROTO_TCP = 6
  %fd = call i64 @socket(i32 2, i32 1, i32 6)
  ret i64 %fd
}

; Create UDP socket
define i64 @TAYNI_udp_socket() {
entry:
  call void @TAYNI_wsa_init()
  ; AF_INET = 2, SOCK_DGRAM = 2, IPPROTO_UDP = 17
  %fd = call i64 @socket(i32 2, i32 2, i32 17)
  ret i64 %fd
}

; Bind socket to port
define i64 @TAYNI_tcp_bind(i64 %fd, i64 %port) {
entry:
  %addr = getelementptr [16 x i8], [16 x i8]* @.win_sockaddr, i32 0, i32 0
  ; sin_family = AF_INET = 2
  store i8 2, i8* %addr
  %addr1 = getelementptr i8, i8* %addr, i32 1
  store i8 0, i8* %addr1
  ; sin_port (big-endian)
  %port_hi = lshr i64 %port, 8
  %port_hi8 = trunc i64 %port_hi to i8
  %port_lo8 = trunc i64 %port to i8
  %addr2 = getelementptr i8, i8* %addr, i32 2
  store i8 %port_hi8, i8* %addr2
  %addr3 = getelementptr i8, i8* %addr, i32 3
  store i8 %port_lo8, i8* %addr3
  ; sin_addr = INADDR_ANY = 0
  %addr4 = getelementptr i8, i8* %addr, i32 4
  store i8 0, i8* %addr4
  %addr5 = getelementptr i8, i8* %addr, i32 5
  store i8 0, i8* %addr5
  %addr6 = getelementptr i8, i8* %addr, i32 6
  store i8 0, i8* %addr6
  %addr7 = getelementptr i8, i8* %addr, i32 7
  store i8 0, i8* %addr7
  %result = call i32 @bind(i64 %fd, i8* %addr, i32 16)
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; Listen on socket
define i64 @TAYNI_tcp_listen(i64 %fd, i64 %backlog) {
entry:
  %backlog32 = trunc i64 %backlog to i32
  %result = call i32 @listen(i64 %fd, i32 %backlog32)
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; Accept connection
define i64 @TAYNI_tcp_accept(i64 %fd) {
entry:
  %client_fd = call i64 @accept(i64 %fd, i8* null, i32* null)
  ret i64 %client_fd
}

; Connect to host:port
define i64 @TAYNI_tcp_connect(i64 %fd, i8* %host, i64 %port) {
entry:
  ; Parse IP address from string
  %ip = call i32 @TAYNI_parse_ip(i8* %host)
  
  %addr = getelementptr [16 x i8], [16 x i8]* @.win_sockaddr, i32 0, i32 0
  store i8 2, i8* %addr
  %addr1 = getelementptr i8, i8* %addr, i32 1
  store i8 0, i8* %addr1
  %port_hi = lshr i64 %port, 8
  %port_hi8 = trunc i64 %port_hi to i8
  %port_lo8 = trunc i64 %port to i8
  %addr2 = getelementptr i8, i8* %addr, i32 2
  store i8 %port_hi8, i8* %addr2
  %addr3 = getelementptr i8, i8* %addr, i32 3
  store i8 %port_lo8, i8* %addr3
  
  ; Store parsed IP
  %addr4 = getelementptr i8, i8* %addr, i32 4
  %ip_ptr = bitcast i8* %addr4 to i32*
  store i32 %ip, i32* %ip_ptr
  
  %result = call i32 @connect(i64 %fd, i8* %addr, i32 16)
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; Parse IP address string "x.x.x.x" to 32-bit integer (network byte order)
define i32 @TAYNI_parse_ip(i8* %str) #1 {
entry:
  %octet1 = alloca i32
  %octet2 = alloca i32
  %octet3 = alloca i32
  %octet4 = alloca i32
  store i32 0, i32* %octet1
  store i32 0, i32* %octet2
  store i32 0, i32* %octet3
  store i32 0, i32* %octet4
  
  %current = alloca i32*
  store i32* %octet1, i32** %current
  
  br label %loop

loop:
  %i = phi i64 [ 0, %entry ], [ %next_i, %continue ]
  %ptr = getelementptr i8, i8* %str, i64 %i
  %char = load i8, i8* %ptr
  %is_null = icmp eq i8 %char, 0
  br i1 %is_null, label %done, label %process

process:
  %is_dot = icmp eq i8 %char, 46  ; '.'
  br i1 %is_dot, label %next_octet, label %digit

digit:
  %is_digit = icmp uge i8 %char, 48  ; >= '0'
  %is_digit2 = icmp ule i8 %char, 57  ; <= '9'
  %valid = and i1 %is_digit, %is_digit2
  br i1 %valid, label %add_digit, label %continue

add_digit:
  %cur_ptr = load i32*, i32** %current
  %cur_val = load i32, i32* %cur_ptr
  %mul = mul i32 %cur_val, 10
  %digit_val = sub i8 %char, 48
  %digit_i32 = zext i8 %digit_val to i32
  %new_val = add i32 %mul, %digit_i32
  store i32 %new_val, i32* %cur_ptr
  br label %continue

next_octet:
  %cur_ptr2 = load i32*, i32** %current
  %is_o1 = icmp eq i32* %cur_ptr2, %octet1
  br i1 %is_o1, label %set_o2, label %check_o2

set_o2:
  store i32* %octet2, i32** %current
  br label %continue

check_o2:
  %is_o2 = icmp eq i32* %cur_ptr2, %octet2
  br i1 %is_o2, label %set_o3, label %set_o4

set_o3:
  store i32* %octet3, i32** %current
  br label %continue

set_o4:
  store i32* %octet4, i32** %current
  br label %continue

continue:
  %next_i = add i64 %i, 1
  br label %loop

done:
  ; Combine octets into 32-bit IP (network byte order)
  %o1 = load i32, i32* %octet1
  %o2 = load i32, i32* %octet2
  %o3 = load i32, i32* %octet3
  %o4 = load i32, i32* %octet4
  
  %o1_8 = trunc i32 %o1 to i8
  %o2_8 = trunc i32 %o2 to i8
  %o3_8 = trunc i32 %o3 to i8
  %o4_8 = trunc i32 %o4 to i8
  
  %r1 = zext i8 %o1_8 to i32
  %r2 = zext i8 %o2_8 to i32
  %r3 = zext i8 %o3_8 to i32
  %r4 = zext i8 %o4_8 to i32
  
  %s2 = shl i32 %r2, 8
  %s3 = shl i32 %r3, 16
  %s4 = shl i32 %r4, 24
  
  %t1 = or i32 %r1, %s2
  %t2 = or i32 %t1, %s3
  %result = or i32 %t2, %s4
  
  ret i32 %result
}

; Send data
define i64 @TAYNI_tcp_send(i64 %fd, i8* %buf, i64 %len) {
entry:
  %len32 = trunc i64 %len to i32
  %result = call i32 @send(i64 %fd, i8* %buf, i32 %len32, i32 0)
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; Receive data
define i64 @TAYNI_tcp_recv(i64 %fd, i8* %buf, i64 %maxlen) {
entry:
  %maxlen32 = trunc i64 %maxlen to i32
  %result = call i32 @recv(i64 %fd, i8* %buf, i32 %maxlen32, i32 0)
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; Close socket (use closesocket, not CloseHandle)
define i64 @TAYNI_socket_close(i64 %fd) {
entry:
  ; Shutdown send direction first (SD_SEND = 1)
  call i32 @shutdown(i64 %fd, i32 1)
  ; Then close
  %result = call i32 @closesocket(i64 %fd)
  %result64 = sext i32 %result to i64
  ret i64 %result64
}

; ============================================================================
; MEMORY (Windows)
; ============================================================================

; Allocate memory using VirtualAlloc
define i8* @TAYNI_alloc(i64 %size) {
entry:
  ; MEM_COMMIT | MEM_RESERVE = 0x3000, PAGE_READWRITE = 0x04
  %ptr = call i8* @VirtualAlloc(i8* null, i64 %size, i32 12288, i32 4)
  ret i8* %ptr
}

; Free memory using VirtualFree
define i64 @TAYNI_free(i8* %ptr) {
entry:
  ; MEM_RELEASE = 0x8000
  %result = call i32 @VirtualFree(i8* %ptr, i64 0, i32 32768)
  %result64 = sext i32 %result to i64
  ret i64 %result64
}
"#.to_string()
    }
    
    fn emit_common_runtime(&self) -> String {
        r#"
; ============================================================================
; COMMON RUNTIME - Platform independent
; ============================================================================

; String length (internal, noinline to prevent optimization to libc strlen)
define i64 @_TAYNI_str_len(i8* %str) #1 {
entry:
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %next, %loop ]
  %ptr = getelementptr i8, i8* %str, i64 %i
  %char = load volatile i8, i8* %ptr
  %is_null = icmp eq i8 %char, 0
  %next = add i64 %i, 1
  br i1 %is_null, label %done, label %loop
done:
  ret i64 %i
}

attributes #1 = { noinline nounwind optnone }

; Print string (no newline)
define i64 @TAYNI_print(i8* %str) {
entry:
  %len = call i64 @_TAYNI_str_len(i8* %str)
  %result = call i64 @sys_write(i32 1, i8* %str, i64 %len)
  ret i64 %result
}

; Print string with newline
@.newline = private constant [2 x i8] c"\0A\00"
define i64 @TAYNI_println(i8* %str) {
entry:
  %len = call i64 @_TAYNI_str_len(i8* %str)
  call i64 @sys_write(i32 1, i8* %str, i64 %len)
  %nl = getelementptr [2 x i8], [2 x i8]* @.newline, i32 0, i32 0
  %result = call i64 @sys_write(i32 1, i8* %nl, i64 1)
  ret i64 %result
}

; ============================================================================
; PURE ITOA - Integer to ASCII (No libc)
; ============================================================================
@.itoa_buf = private global [21 x i8] zeroinitializer

define i8* @TAYNI_itoa(i64 %num) #1 {
entry:
  %buf_end = getelementptr [21 x i8], [21 x i8]* @.itoa_buf, i32 0, i32 20
  store i8 0, i8* %buf_end
  %is_zero = icmp eq i64 %num, 0
  br i1 %is_zero, label %zero_case, label %check_neg

zero_case:
  %zero_ptr = getelementptr [21 x i8], [21 x i8]* @.itoa_buf, i32 0, i32 19
  store i8 48, i8* %zero_ptr
  ret i8* %zero_ptr

check_neg:
  %is_neg = icmp slt i64 %num, 0
  %neg_num = sub i64 0, %num
  %work_num = select i1 %is_neg, i64 %neg_num, i64 %num
  br label %convert_loop

convert_loop:
  %pos = phi i64 [ 19, %check_neg ], [ %next_pos, %convert_loop ]
  %n = phi i64 [ %work_num, %check_neg ], [ %next_n, %convert_loop ]
  %digit = urem i64 %n, 10
  %char = add i64 %digit, 48
  %char8 = trunc i64 %char to i8
  %ptr = getelementptr [21 x i8], [21 x i8]* @.itoa_buf, i32 0, i64 %pos
  store i8 %char8, i8* %ptr
  %next_n = udiv i64 %n, 10
  %next_pos = sub i64 %pos, 1
  %done = icmp eq i64 %next_n, 0
  br i1 %done, label %maybe_sign, label %convert_loop

maybe_sign:
  %final_pos = phi i64 [ %next_pos, %convert_loop ]
  br i1 %is_neg, label %add_sign, label %return_ptr

add_sign:
  %sign_ptr = getelementptr [21 x i8], [21 x i8]* @.itoa_buf, i32 0, i64 %final_pos
  store i8 45, i8* %sign_ptr
  %result_signed = getelementptr [21 x i8], [21 x i8]* @.itoa_buf, i32 0, i64 %final_pos
  ret i8* %result_signed

return_ptr:
  %adj_pos = add i64 %final_pos, 1
  %result_ptr = getelementptr [21 x i8], [21 x i8]* @.itoa_buf, i32 0, i64 %adj_pos
  ret i8* %result_ptr
}

; Print integer with newline
define i64 @TAYNI_print_int(i64 %num) {
entry:
  %str = call i8* @TAYNI_itoa(i64 %num)
  %len = call i64 @_TAYNI_str_len(i8* %str)
  call i64 @sys_write(i32 1, i8* %str, i64 %len)
  %nl = getelementptr [2 x i8], [2 x i8]* @.newline, i32 0, i32 0
  %result = call i64 @sys_write(i32 1, i8* %nl, i64 1)
  ret i64 %result
}
"#.to_string()
    }
    
    fn emit_linux_entry(&self) -> String {
        r#"
; ============================================================================
; LINUX ENTRY POINT (no libc _start)
; ============================================================================

define void @_start() {
entry:
  %result = call i32 @TAYNI_main()
  call void @sys_exit(i32 %result)
  unreachable
}
"#.to_string()
    }
    
    fn emit_windows_entry(&self) -> String {
        r#"
; ============================================================================
; WINDOWS ENTRY POINT (mainCRTStartup for console apps)
; ============================================================================

define void @mainCRTStartup() {
entry:
  ; Initialize I/O handles
  call void @TAYNI_init_io()
  ; Run main
  %result = call i32 @TAYNI_main()
  call void @sys_exit(i32 %result)
  unreachable
}
"#.to_string()
    }
}
