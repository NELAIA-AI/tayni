//! NELAIA v0.15 Pure Emitter
//! Only syscall wrappers - NO embedded programs
//! Dead code elimination enabled

use crate::ir::*;

#[derive(Clone, Copy, PartialEq)]
pub enum TargetPlatform {
    Linux,
    Windows,
}

pub struct PureEmitter {
    strings: Vec<(String, String)>,
    string_ids: Vec<(String, String)>,  // Maps node ID to global string ID
    counter: usize,
    target: TargetPlatform,
    usage: Option<UsageAnalysis>,  // Usage analysis for dead code elimination
}

impl PureEmitter {
    pub fn new(target: TargetPlatform) -> Self {
        PureEmitter {
            strings: Vec::new(),
            string_ids: Vec::new(),
            counter: 0,
            target,
            usage: None,
        }
    }
    
    fn next_id(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
    
    pub fn emit(graph: &Graph, target: TargetPlatform) -> Result<String, String> {
        let mut emitter = PureEmitter::new(target);
        // Analyze usage for dead code elimination
        emitter.usage = Some(UsageAnalysis::analyze(graph));
        emitter.emit_graph(graph)
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> Result<String, String> {
        let mut ir = String::new();
        
        // Header
        ir.push_str("; NELAIA v0.15 Pure - Dead Code Elimination\n");
        match self.target {
            TargetPlatform::Linux => ir.push_str("target triple = \"x86_64-pc-linux-gnu\"\n\n"),
            TargetPlatform::Windows => ir.push_str("target triple = \"x86_64-pc-windows-msvc\"\n\n"),
        }
        
        // Collect strings from all nodes including function bodies
        for node in &graph.nodes {
            self.collect_strings(node);
            if let Node::Function { id: _, params: _, body } = node {
                for body_node in body {
                    self.collect_strings(body_node);
                }
            }
        }
        
        // Emit string constants
        for (id, s) in &self.strings {
            let escaped = self.escape_string(s);
            ir.push_str(&format!("@{} = private constant [{} x i8] c\"{}\\00\"\n", 
                id, s.len() + 1, escaped));
        }
        
        // For Windows, also emit wide string constants (UTF-16)
        if self.target == TargetPlatform::Windows {
            for (id, s) in &self.strings {
                let wide_chars: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
                let wide_str = wide_chars.iter()
                    .map(|c| format!("i16 {}", c))
                    .collect::<Vec<_>>()
                    .join(", ");
                ir.push_str(&format!("@{}_w = private constant [{} x i16] [{}]\n", 
                    id, wide_chars.len(), wide_str));
            }
        }
        ir.push_str("\n");
        
        // Emit syscall layer (only what's needed)
        match self.target {
            TargetPlatform::Linux => ir.push_str(&self.emit_linux_syscalls()),
            TargetPlatform::Windows => ir.push_str(&self.emit_windows_syscalls_optimized()),
        }
        
        // Emit user-defined functions first
        for node in &graph.nodes {
            if let Node::Function { id, params, body } = node {
                ir.push_str(&self.emit_function(id, params, body)?);
            }
        }
        
        // Find cyclic flow targets (nodes that need labels) - only in main
        let mut cyclic_targets: Vec<String> = Vec::new();
        for node in &graph.nodes {
            if let Node::Flow { source: _, dest: FlowDest::CyclicNode(target) } = node {
                if !cyclic_targets.contains(target) {
                    cyclic_targets.push(target.clone());
                }
            }
        }
        
        // Emit main
        ir.push_str("\ndefine i32 @nelaia_main() {\nentry:\n");
        
        for node in &graph.nodes {
            // Skip function definitions in main
            if let Node::Function { .. } = node {
                continue;
            }
            
            // Emit label if this node is a cyclic target
            if let Node::Operation { id, .. } | Node::Literal { id, .. } = node {
                if cyclic_targets.contains(id) {
                    ir.push_str(&format!("  br label %cycle_{}\ncycle_{}:\n", id, id));
                }
            }
            ir.push_str(&self.emit_node(node)?);
        }
        
        ir.push_str("  ret i32 0\n}\n");
        
        // Entry point
        ir.push_str(&self.emit_entry());
        
        Ok(ir)
    }
    
    fn emit_function(&mut self, id: &str, params: &[String], body: &[Node]) -> Result<String, String> {
        let mut ir = String::new();
        
        // For Windows threading, function must take i8* and return i32
        // For simplicity, we'll use i64 (i8*) -> i64 signature
        let param_list: String = if params.is_empty() {
            "i8* %_arg".to_string()
        } else {
            params.iter()
                .map(|p| format!("i64 %{}", p))
                .collect::<Vec<_>>()
                .join(", ")
        };
        
        ir.push_str(&format!("\ndefine i64 @{}({}) {{\nentry:\n", id, param_list));
        
        // If function takes i8*, convert to i64 for internal use
        if params.is_empty() {
            ir.push_str("  %arg = ptrtoint i8* %_arg to i64\n");
        }
        
        // Find cyclic targets in function body
        let mut cyclic_targets: Vec<String> = Vec::new();
        for node in body {
            if let Node::Flow { source: _, dest: FlowDest::CyclicNode(target) } = node {
                if !cyclic_targets.contains(target) {
                    cyclic_targets.push(target.clone());
                }
            }
        }
        
        // Emit function body
        for node in body {
            if let Node::Operation { id, .. } | Node::Literal { id, .. } = node {
                if cyclic_targets.contains(id) {
                    ir.push_str(&format!("  br label %cycle_{}\ncycle_{}:\n", id, id));
                }
            }
            ir.push_str(&self.emit_node(node)?);
        }
        
        // Default return if no RET in body
        ir.push_str("  ret i64 0\n}\n");
        
        Ok(ir)
    }
    
    fn collect_strings(&mut self, node: &Node) {
        match node {
            Node::Literal { id, value: Value::String(s) } => {
                let str_id = if let Some((existing_id, _)) = self.strings.iter().find(|(_, v)| v == s) {
                    existing_id.clone()
                } else {
                    let new_id = self.counter + 1;
                    self.counter = new_id;
                    let str_id = format!("str_{}", new_id);
                    self.strings.push((str_id.clone(), s.clone()));
                    str_id
                };
                // Map node ID to string global ID
                self.string_ids.push((id.clone(), str_id));
            }
            Node::Flow { source: Arg::Lit(Value::String(s)), dest: _ } => {
                if !self.strings.iter().any(|(_, v)| v == s) {
                    let new_id = self.counter + 1;
                    self.counter = new_id;
                    self.strings.push((format!("str_{}", new_id), s.clone()));
                }
            }
            Node::Operation { id: _, op: _, args } => {
                for arg in args {
                    if let Arg::Lit(Value::String(s)) = arg {
                        if !self.strings.iter().any(|(_, v)| v == s) {
                            let new_id = self.counter + 1;
                            self.counter = new_id;
                            self.strings.push((format!("str_{}", new_id), s.clone()));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    fn emit_node(&mut self, node: &Node) -> Result<String, String> {
        match node {
            Node::Literal { id, value } => self.emit_literal(id, value),
            Node::Operation { id, op, args } => self.emit_operation(id, op, args),
            Node::Flow { source, dest } => self.emit_flow(source, dest),
            _ => Ok(String::new()),
        }
    }
    
    fn emit_literal(&mut self, id: &str, value: &Value) -> Result<String, String> {
        match value {
            Value::Int(n) => Ok(format!("  %{} = add i64 0, {}\n", id, n)),
            Value::String(_) => {
                // Strings are global, no local variable needed
                // The reference will be resolved by emit_arg using string_ids
                Ok(String::new())
            }
            _ => Ok(format!("  ; TODO: literal {:?}\n", value)),
        }
    }
    
    fn emit_operation(&mut self, id: &str, op: &Op, args: &[Arg]) -> Result<String, String> {
        match op {
            // Arithmetic - pure LLVM
            Op::Add => self.emit_binop(id, "add", args),
            Op::Sub => self.emit_binop(id, "sub", args),
            Op::Mul => self.emit_binop(id, "mul", args),
            Op::Div => self.emit_binop(id, "sdiv", args),
            Op::Mod => self.emit_binop(id, "srem", args),
            
            // Comparison - pure LLVM
            Op::Eq => self.emit_cmp(id, "eq", args),
            Op::Ne => self.emit_cmp(id, "ne", args),
            Op::Lt => self.emit_cmp(id, "slt", args),
            Op::Gt => self.emit_cmp(id, "sgt", args),
            Op::Le => self.emit_cmp(id, "sle", args),
            Op::Ge => self.emit_cmp(id, "sge", args),
            
            // Memory - direct syscall
            Op::Alc => self.emit_alloc(id, args),
            Op::Fre => self.emit_free(id, args),
            
            // Memory byte operations
            Op::Put => self.emit_store_byte(id, args),  // PUT ptr offset byte
            Op::Get => self.emit_load_byte(id, args),   // GET ptr offset -> byte
            
            // Raw syscall
            Op::Prt => self.emit_print(id, args),
            
            // Network - direct syscalls
            Op::Tcp => self.emit_syscall_socket(id, 1), // SOCK_STREAM
            Op::Udp => self.emit_syscall_socket(id, 2), // SOCK_DGRAM
            Op::Bnd => self.emit_syscall_bind(id, args),
            Op::Lst => self.emit_syscall_listen(id, args),
            Op::Acc => self.emit_syscall_accept(id, args),
            Op::Xmt => self.emit_syscall_send(id, args),
            Op::Rcv => self.emit_syscall_recv(id, args),
            Op::Cls => self.emit_syscall_close(id, args),
            
            // Control flow - LOOP/CNT deprecated, use cyclic flow >> instead
            
            // Async I/O
            Op::Sel => self.emit_select(id, args),
            Op::Rdy => self.emit_ready_check(id, args),
            Op::Nbk => self.emit_nonblocking(id, args),
            
            // Socket options (ultra-low-latency)
            Op::Ndl => self.emit_tcp_nodelay(id, args),
            Op::Qck => self.emit_quickack(id, args),
            Op::Sbf => self.emit_sockbuf(id, args),
            Op::Kal => self.emit_keepalive(id, args),
            
            // High-performance I/O
            Op::Epl => self.emit_epoll_create(id, args),
            Op::Ewa => self.emit_epoll_wait(id, args),
            Op::Ect => self.emit_epoll_ctl(id, args),
            
            // Control flow
            Op::Brn => self.emit_branch(id, args),
            Op::Jmp => self.emit_jump(id, args),
            Op::Whl => self.emit_while(id, args),
            Op::End => self.emit_end(id, args),
            Op::Trn => self.emit_transform(id, args),
            Op::Fsm => self.emit_fsm(id, args),
            
            // Threading
            Op::Thr => self.emit_thread_create(id, args),
            Op::Jon => self.emit_thread_join(id, args),
            Op::Mtx => self.emit_mutex_create(id, args),
            Op::Lck => self.emit_mutex_lock(id, args),
            Op::Ulk => self.emit_mutex_unlock(id, args),
            
            // Atomic Queue
            Op::Que => self.emit_queue_create(id, args),
            Op::Psh => self.emit_queue_push(id, args),
            Op::Pop => self.emit_queue_pop(id, args),
            
            // Functions
            Op::Ret => self.emit_return(id, args),
            
            // GUI - Window Management
            Op::Win => self.emit_window_create(id, args),
            Op::Shw => self.emit_window_show(id, args),
            Op::Hid => self.emit_window_hide(id, args),
            Op::Evt => self.emit_event_poll(id, args),
            Op::Run => self.emit_message_loop(id, args),
            
            // GUI - Controls
            Op::Lbl => self.emit_label_create(id, args),
            Op::Txb => self.emit_textbox_create(id, args),
            Op::Btn => self.emit_button_create(id, args),
            
            // GUI - Dialogs
            Op::Dlg => self.emit_dialog_alert(id, args),
            
            // GUI - Control Values
            Op::Gvl => self.emit_get_value(id, args),
            Op::Svl => self.emit_set_value(id, args),
            
            // Memory operations (self-hosting)
            Op::Cpy => self.emit_memcpy(id, args),
            Op::Cmp => self.emit_memcmp(id, args),
            Op::Fnd => self.emit_memfind(id, args),
            Op::Sln => self.emit_strlen(id, args),
            
            // File I/O (self-hosting)
            Op::Fop => self.emit_file_open(id, args),
            Op::Frd => self.emit_file_read(id, args),
            Op::Fwr => self.emit_file_write(id, args),
            Op::Fcl => self.emit_file_close(id, args),
            
            // Dynamic Vectors
            Op::Vec => self.emit_vec_create(id, args),
            Op::Vph => self.emit_vec_push(id, args),
            Op::Vgt => self.emit_vec_get(id, args),
            Op::Vst => self.emit_vec_set(id, args),
            Op::Vln => self.emit_vec_len(id, args),
            Op::Vcp => self.emit_vec_cap(id, args),
            
            // HashMap (self-hosting)
            Op::Hmp => self.emit_hashmap_create(id, args),
            Op::Hpt => self.emit_hashmap_put(id, args),
            Op::Hgt => self.emit_hashmap_get(id, args),
            Op::Hhs => self.emit_hashmap_has(id, args),
            
            // String operations (self-hosting)
            Op::Cat => self.emit_str_cat(id, args),
            Op::Its => self.emit_int_to_str(id, args),
            Op::Chr => self.emit_char_at(id, args),
            Op::Sbs => self.emit_substring(id, args),
            
            _ => Ok(format!("  ; TODO: {:?}\n", op)),
        }
    }
    
    fn emit_arg(&self, arg: &Arg) -> Result<String, String> {
        match arg {
            Arg::Ref(name) => {
                // Check if this reference is to a string literal
                if let Some((_, str_global_id)) = self.string_ids.iter().find(|(node_id, _)| node_id == name) {
                    // Find the string to get its length
                    if let Some((_, s)) = self.strings.iter().find(|(id, _)| id == str_global_id) {
                        return Ok(format!("getelementptr ([{} x i8], [{} x i8]* @{}, i32 0, i32 0)",
                            s.len() + 1, s.len() + 1, str_global_id));
                    }
                }
                Ok(format!("%{}", name))
            }
            Arg::Lit(Value::Int(n)) => Ok(format!("{}", n)),
            Arg::Lit(Value::String(s)) => {
                let str_id = self.strings.iter()
                    .find(|(_, v)| v == s)
                    .map(|(id, _)| id.clone())
                    .ok_or("String not found")?;
                Ok(format!("getelementptr ([{} x i8], [{} x i8]* @{}, i32 0, i32 0)",
                    s.len() + 1, s.len() + 1, str_id))
            }
            _ => Err("Unsupported arg".to_string()),
        }
    }
    
    fn emit_binop(&self, id: &str, op: &str, args: &[Arg]) -> Result<String, String> {
        let a = self.emit_arg(&args[0])?;
        let b = self.emit_arg(&args[1])?;
        Ok(format!("  %{} = {} i64 {}, {}\n", id, op, a, b))
    }
    
    fn emit_cmp(&self, id: &str, cmp: &str, args: &[Arg]) -> Result<String, String> {
        let a = self.emit_arg(&args[0])?;
        let b = self.emit_arg(&args[1])?;
        Ok(format!("  %{}_cmp = icmp {} i64 {}, {}\n  %{} = zext i1 %{}_cmp to i64\n", 
            id, cmp, a, b, id, id))
    }
    
    fn emit_alloc(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        let size = self.emit_arg(&args[0])?;
        match self.target {
            TargetPlatform::Linux => {
                // mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0)
                Ok(format!("  %{} = call i64 @sys_mmap(i64 0, i64 {}, i64 3, i64 34, i64 -1, i64 0)\n", id, size))
            }
            TargetPlatform::Windows => {
                Ok(format!("  %{}_ptr = call i8* @VirtualAlloc(i8* null, i64 {}, i32 12288, i32 4)\n  %{} = ptrtoint i8* %{}_ptr to i64\n", id, size, id, id))
            }
        }
    }
    
    fn emit_free(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        let ptr = self.emit_arg(&args[0])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!("  %{} = call i64 @sys_munmap(i64 {}, i64 4096)\n", id, ptr))
            }
            TargetPlatform::Windows => {
                Ok(format!("  %{}_ptr = inttoptr i64 {} to i8*\n  %{}_r = call i32 @VirtualFree(i8* %{}_ptr, i64 0, i32 32768)\n  %{} = sext i32 %{}_r to i64\n", id, ptr, id, id, id, id))
            }
        }
    }
    
    // Store byte at ptr+offset: PUT ptr offset value
    fn emit_store_byte(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 3 {
            return Err("PUT requires ptr, offset, value".to_string());
        }
        let ptr = self.emit_arg(&args[0])?;
        let offset = self.emit_arg(&args[1])?;
        let value = self.emit_arg(&args[2])?;
        Ok(format!(
            "  %{}_base = inttoptr i64 {} to i8*\n  %{}_ptr = getelementptr i8, i8* %{}_base, i64 {}\n  %{}_val = trunc i64 {} to i8\n  store i8 %{}_val, i8* %{}_ptr\n  %{} = add i64 0, 0\n",
            id, ptr, id, id, offset, id, value, id, id, id
        ))
    }
    
    // Load byte from ptr+offset: GET ptr offset -> byte
    fn emit_load_byte(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 2 {
            return Err("GET requires ptr, offset".to_string());
        }
        let ptr = self.emit_arg(&args[0])?;
        let offset = self.emit_arg(&args[1])?;
        Ok(format!(
            "  %{}_base = inttoptr i64 {} to i8*\n  %{}_ptr = getelementptr i8, i8* %{}_base, i64 {}\n  %{}_byte = load i8, i8* %{}_ptr\n  %{} = zext i8 %{}_byte to i64\n",
            id, ptr, id, id, offset, id, id, id, id
        ))
    }
    
    fn emit_print(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // Print requires length - AI must provide it or use raw syscall
        // For string literals, we know the length
        match &args[0] {
            Arg::Lit(Value::String(s)) => {
                let ptr = self.emit_arg(&args[0])?;
                let len = s.len();
                // String literals are already pointers (getelementptr)
                Ok(format!("  %{} = call i64 @sys_write(i32 1, i8* {}, i64 {})\n", id, ptr, len))
            }
            Arg::Ref(ref_name) => {
                // Check if this is a reference to a string literal
                if args.len() < 2 {
                    return Err("PRT with reference requires length argument".to_string());
                }
                let ptr = self.emit_arg(&args[0])?;
                let len = self.emit_arg(&args[1])?;
                // Handle both pointer (getelementptr) and i64 arguments
                if ptr.contains("getelementptr") {
                    Ok(format!("  %{}_ptr = bitcast i8* {} to i8*\n  %{} = call i64 @sys_write(i32 1, i8* %{}_ptr, i64 {})\n", id, ptr, id, id, len))
                } else {
                    Ok(format!("  %{}_ptr = inttoptr i64 {} to i8*\n  %{} = call i64 @sys_write(i32 1, i8* %{}_ptr, i64 {})\n", id, ptr, id, id, len))
                }
            }
            _ => {
                // For other cases
                if args.len() < 2 {
                    return Err("PRT with reference requires length argument".to_string());
                }
                let ptr = self.emit_arg(&args[0])?;
                let len = self.emit_arg(&args[1])?;
                Ok(format!("  %{}_ptr = inttoptr i64 {} to i8*\n  %{} = call i64 @sys_write(i32 1, i8* %{}_ptr, i64 {})\n", id, ptr, id, id, len))
            }
        }
    }
    
    fn emit_syscall_socket(&self, id: &str, sock_type: i32) -> Result<String, String> {
        // socket(AF_INET=2, type, 0) + setsockopt(SO_REUSEADDR)
        match self.target {
            TargetPlatform::Linux => {
                // Linux: SOL_SOCKET=1, SO_REUSEADDR=2
                Ok(format!(
                    "  %{} = call i64 @sys_socket(i64 2, i64 {}, i64 0)\n  %{}_one = alloca i32\n  store i32 1, i32* %{}_one\n  %{}_oneptr = bitcast i32* %{}_one to i8*\n  call i64 @sys_setsockopt(i64 %{}, i64 1, i64 2, i8* %{}_oneptr, i64 4)\n",
                    id, sock_type, id, id, id, id, id, id
                ))
            }
            TargetPlatform::Windows => {
                // Windows: SOL_SOCKET=0xFFFF, SO_REUSEADDR=4
                Ok(format!(
                    "  call void @_wsa_init()\n  %{} = call i64 @socket(i32 2, i32 {}, i32 0)\n  %{}_oneptr = bitcast i32* @.one to i8*\n  call i32 @setsockopt(i64 %{}, i32 65535, i32 4, i8* %{}_oneptr, i32 4)\n",
                    id, sock_type, id, id, id
                ))
            }
        }
    }
    
    fn emit_syscall_bind(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // bind(fd, addr_ptr, 16)
        // AI must build sockaddr_in and pass pointer
        let fd = self.emit_arg(&args[0])?;
        let addr = self.emit_arg(&args[1])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!("  %{}_addr = inttoptr i64 {} to i8*\n  %{} = call i64 @sys_bind(i64 {}, i8* %{}_addr, i64 16)\n", id, addr, id, fd, id))
            }
            TargetPlatform::Windows => {
                Ok(format!("  %{}_addr = inttoptr i64 {} to i8*\n  %{}_r = call i32 @bind(i64 {}, i8* %{}_addr, i32 16)\n  %{} = sext i32 %{}_r to i64\n", id, addr, id, fd, id, id, id))
            }
        }
    }
    
    fn emit_syscall_listen(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        let fd = self.emit_arg(&args[0])?;
        let backlog = if args.len() > 1 { self.emit_arg(&args[1])? } else { "10".to_string() };
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!("  %{} = call i64 @sys_listen(i64 {}, i64 {})\n", id, fd, backlog))
            }
            TargetPlatform::Windows => {
                Ok(format!("  %{}_r = call i32 @listen(i64 {}, i32 {})\n  %{} = sext i32 %{}_r to i64\n", id, fd, backlog, id, id))
            }
        }
    }
    
    fn emit_syscall_accept(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        let fd = self.emit_arg(&args[0])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!("  %{} = call i64 @sys_accept(i64 {}, i8* null, i64* null)\n", id, fd))
            }
            TargetPlatform::Windows => {
                Ok(format!("  %{} = call i64 @accept(i64 {}, i8* null, i32* null)\n", id, fd))
            }
        }
    }
    
    fn emit_syscall_send(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        let fd = self.emit_arg(&args[0])?;
        
        match &args[1] {
            Arg::Lit(Value::String(s)) => {
                let ptr = self.emit_arg(&args[1])?;
                let len = s.len();
                match self.target {
                    TargetPlatform::Linux => {
                        Ok(format!("  %{} = call i64 @sys_sendto(i64 {}, i8* {}, i64 {}, i64 0, i8* null, i64 0)\n", id, fd, ptr, len))
                    }
                    TargetPlatform::Windows => {
                        Ok(format!("  %{}_r = call i32 @send(i64 {}, i8* {}, i32 {}, i32 0)\n  %{} = sext i32 %{}_r to i64\n", id, fd, ptr, len, id, id))
                    }
                }
            }
            Arg::Ref(ref_name) => {
                // Reference - could be string or buffer
                if args.len() < 3 {
                    return Err("XMT with reference requires ptr and length".to_string());
                }
                let len = self.emit_arg(&args[2])?;
                // Use emit_arg to resolve the reference (handles string globals)
                let ptr = self.emit_arg(&args[1])?;
                match self.target {
                    TargetPlatform::Linux => {
                        Ok(format!("  %{} = call i64 @sys_sendto(i64 {}, i8* {}, i64 {}, i64 0, i8* null, i64 0)\n", id, fd, ptr, len))
                    }
                    TargetPlatform::Windows => {
                        Ok(format!("  %{}_r = call i32 @send(i64 {}, i8* {}, i32 {}, i32 0)\n  %{} = sext i32 %{}_r to i64\n", id, fd, ptr, len, id, id))
                    }
                }
            }
            _ => {
                Err("XMT requires string literal or reference".to_string())
            }
        }
    }
    
    fn emit_syscall_recv(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        let fd = self.emit_arg(&args[0])?;
        let buf = self.emit_arg(&args[1])?;
        let len = self.emit_arg(&args[2])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!("  %{}_ptr = inttoptr i64 {} to i8*\n  %{} = call i64 @sys_recvfrom(i64 {}, i8* %{}_ptr, i64 {}, i64 0, i8* null, i64* null)\n", id, buf, id, fd, id, len))
            }
            TargetPlatform::Windows => {
                Ok(format!("  %{}_ptr = inttoptr i64 {} to i8*\n  %{}_r = call i32 @recv(i64 {}, i8* %{}_ptr, i32 {}, i32 0)\n  %{} = sext i32 %{}_r to i64\n", id, buf, id, fd, id, len, id, id))
            }
        }
    }
    
    fn emit_syscall_close(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        let fd = self.emit_arg(&args[0])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!("  %{} = call i64 @sys_close(i64 {})\n", id, fd))
            }
            TargetPlatform::Windows => {
                // Direct closesocket - let TCP stack handle graceful close
                Ok(format!("  %{}_r = call i32 @closesocket(i64 {})\n  %{} = sext i32 %{}_r to i64\n", id, fd, id, id))
            }
        }
    }
    
    // SEL fd timeout_ms -> ready_count
    // Uses select() to wait for socket to be readable
    fn emit_select(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        if args.len() < 2 {
            return Err("SEL requires fd and timeout_ms".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let timeout_ms = self.emit_arg(&args[1])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // Linux: use poll syscall (simpler than select)
                // struct pollfd { int fd; short events; short revents; } = 8 bytes
                // POLLIN = 1
                let mut code = String::new();
                code.push_str(&format!("  %{}_pollfd = alloca [8 x i8]\n", id));
                code.push_str(&format!("  %{}_fdptr = getelementptr [8 x i8], [8 x i8]* %{}_pollfd, i32 0, i32 0\n", id, id));
                code.push_str(&format!("  %{}_fd32 = trunc i64 {} to i32\n", id, fd));
                code.push_str(&format!("  %{}_fdcast = bitcast i8* %{}_fdptr to i32*\n", id, id));
                code.push_str(&format!("  store i32 %{}_fd32, i32* %{}_fdcast\n", id, id));
                code.push_str(&format!("  %{}_evptr = getelementptr [8 x i8], [8 x i8]* %{}_pollfd, i32 0, i32 4\n", id, id));
                code.push_str(&format!("  %{}_evcast = bitcast i8* %{}_evptr to i16*\n", id, id));
                code.push_str(&format!("  store i16 1, i16* %{}_evcast\n", id));
                code.push_str(&format!("  %{}_timeout = trunc i64 {} to i32\n", id, timeout_ms));
                code.push_str(&format!("  %{} = call i64 @sys_poll(i8* %{}_fdptr, i64 1, i32 %{}_timeout)\n", id, id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // Windows: use select() with fd_set
                // fd_set: u_int fd_count + SOCKET fd_array[64]
                // Initialize with zeros manually (no memset dependency)
                let mut code = String::new();
                code.push_str(&format!("  %{}_fdset = alloca [264 x i8]\n", id));
                code.push_str(&format!("  %{}_fdptr = getelementptr [264 x i8], [264 x i8]* %{}_fdset, i32 0, i32 0\n", id, id));
                // Set count = 1
                code.push_str(&format!("  %{}_countptr = bitcast i8* %{}_fdptr to i32*\n", id, id));
                code.push_str(&format!("  store i32 1, i32* %{}_countptr\n", id));
                // Set first socket
                code.push_str(&format!("  %{}_sockptr = getelementptr [264 x i8], [264 x i8]* %{}_fdset, i32 0, i32 8\n", id, id));
                code.push_str(&format!("  %{}_sockcast = bitcast i8* %{}_sockptr to i64*\n", id, id));
                code.push_str(&format!("  store i64 {}, i64* %{}_sockcast\n", fd, id));
                // timeval: tv_sec (i64) + tv_usec (i64)
                code.push_str(&format!("  %{}_tv = alloca [16 x i8]\n", id));
                code.push_str(&format!("  %{}_tvptr = getelementptr [16 x i8], [16 x i8]* %{}_tv, i32 0, i32 0\n", id, id));
                code.push_str(&format!("  %{}_sec = sdiv i64 {}, 1000\n", id, timeout_ms));
                code.push_str(&format!("  %{}_usec = mul i64 {}, 1000\n", id, timeout_ms));
                code.push_str(&format!("  %{}_secptr = bitcast i8* %{}_tvptr to i64*\n", id, id));
                code.push_str(&format!("  store i64 %{}_sec, i64* %{}_secptr\n", id, id));
                code.push_str(&format!("  %{}_usecptr = getelementptr [16 x i8], [16 x i8]* %{}_tv, i32 0, i32 8\n", id, id));
                code.push_str(&format!("  %{}_useccast = bitcast i8* %{}_usecptr to i64*\n", id, id));
                code.push_str(&format!("  store i64 %{}_usec, i64* %{}_useccast\n", id, id));
                code.push_str(&format!("  %{}_r = call i32 @select(i32 0, i8* %{}_fdptr, i8* null, i8* null, i8* %{}_tvptr)\n", id, id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    // RDY fd -> 0 or 1 (non-blocking check if socket has data)
    fn emit_ready_check(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // Just call select with 0 timeout
        self.emit_select(id, &[args[0].clone(), Arg::Lit(Value::Int(0))])
    }
    
    fn emit_nonblocking(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // NBK fd mode - Set socket to non-blocking (mode=1) or blocking (mode=0)
        if args.len() < 2 {
            return Err("NBK requires fd and mode (0=blocking, 1=nonblocking)".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let mode = self.emit_arg(&args[1])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // Linux: fcntl(fd, F_SETFL, O_NONBLOCK) or fcntl(fd, F_SETFL, 0)
                // F_GETFL = 3, F_SETFL = 4, O_NONBLOCK = 2048
                let mut code = String::new();
                code.push_str(&format!("  %{}_fd32 = trunc i64 {} to i32\n", id, fd));
                code.push_str(&format!("  %{}_flags = call i32 @sys_fcntl(i32 %{}_fd32, i32 3, i32 0)\n", id, id));
                code.push_str(&format!("  %{}_mode = trunc i64 {} to i32\n", id, mode));
                code.push_str(&format!("  %{}_nb = mul i32 %{}_mode, 2048\n", id, id));
                code.push_str(&format!("  %{}_newflags = or i32 %{}_flags, %{}_nb\n", id, id, id));
                code.push_str(&format!("  %{}_r = call i32 @sys_fcntl(i32 %{}_fd32, i32 4, i32 %{}_newflags)\n", id, id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // Windows: ioctlsocket(fd, FIONBIO, &mode)
                // FIONBIO = 0x8004667e
                let mut code = String::new();
                code.push_str(&format!("  %{}_modeptr = alloca i32\n", id));
                code.push_str(&format!("  %{}_mode32 = trunc i64 {} to i32\n", id, mode));
                code.push_str(&format!("  store i32 %{}_mode32, i32* %{}_modeptr\n", id, id));
                code.push_str(&format!("  %{}_r = call i32 @ioctlsocket(i64 {}, i32 -2147195266, i32* %{}_modeptr)\n", id, fd, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_tcp_nodelay(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // NDL fd mode - TCP_NODELAY (disable Nagle algorithm)
        if args.len() < 2 {
            return Err("NDL requires fd and mode (0=off, 1=on)".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let mode = self.emit_arg(&args[1])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // setsockopt(fd, IPPROTO_TCP=6, TCP_NODELAY=1, &val, 4)
                let mut code = String::new();
                code.push_str(&format!("  %{}_val = alloca i32\n", id));
                code.push_str(&format!("  %{}_m32 = trunc i64 {} to i32\n", id, mode));
                code.push_str(&format!("  store i32 %{}_m32, i32* %{}_val\n", id, id));
                code.push_str(&format!("  %{}_ptr = bitcast i32* %{}_val to i8*\n", id, id));
                code.push_str(&format!("  %{}_r = call i64 @sys_setsockopt(i64 {}, i64 6, i64 1, i8* %{}_ptr, i64 4)\n", id, fd, id));
                code.push_str(&format!("  %{} = trunc i64 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // setsockopt(fd, IPPROTO_TCP=6, TCP_NODELAY=1, &val, 4)
                let mut code = String::new();
                code.push_str(&format!("  %{}_val = alloca i32\n", id));
                code.push_str(&format!("  %{}_m32 = trunc i64 {} to i32\n", id, mode));
                code.push_str(&format!("  store i32 %{}_m32, i32* %{}_val\n", id, id));
                code.push_str(&format!("  %{}_ptr = bitcast i32* %{}_val to i8*\n", id, id));
                code.push_str(&format!("  %{}_r = call i32 @setsockopt(i64 {}, i32 6, i32 1, i8* %{}_ptr, i32 4)\n", id, fd, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_quickack(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // QCK fd mode - TCP_QUICKACK (immediate ACK, Linux only)
        if args.len() < 2 {
            return Err("QCK requires fd and mode (0=off, 1=on)".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let mode = self.emit_arg(&args[1])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // setsockopt(fd, IPPROTO_TCP=6, TCP_QUICKACK=12, &val, 4)
                let mut code = String::new();
                code.push_str(&format!("  %{}_val = alloca i32\n", id));
                code.push_str(&format!("  %{}_m32 = trunc i64 {} to i32\n", id, mode));
                code.push_str(&format!("  store i32 %{}_m32, i32* %{}_val\n", id, id));
                code.push_str(&format!("  %{}_ptr = bitcast i32* %{}_val to i8*\n", id, id));
                code.push_str(&format!("  %{}_r = call i64 @sys_setsockopt(i64 {}, i64 6, i64 12, i8* %{}_ptr, i64 4)\n", id, fd, id));
                code.push_str(&format!("  %{} = trunc i64 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // Windows doesn't have TCP_QUICKACK, return 0 (no-op)
                Ok(format!("  %{} = add i64 0, 0\n", id))
            }
        }
    }
    
    fn emit_sockbuf(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // SBF fd size - Set SO_SNDBUF and SO_RCVBUF
        if args.len() < 2 {
            return Err("SBF requires fd and buffer_size".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let size = self.emit_arg(&args[1])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // SO_SNDBUF=7, SO_RCVBUF=8, SOL_SOCKET=1
                let mut code = String::new();
                code.push_str(&format!("  %{}_val = alloca i32\n", id));
                code.push_str(&format!("  %{}_s32 = trunc i64 {} to i32\n", id, size));
                code.push_str(&format!("  store i32 %{}_s32, i32* %{}_val\n", id, id));
                code.push_str(&format!("  %{}_ptr = bitcast i32* %{}_val to i8*\n", id, id));
                code.push_str(&format!("  call i64 @sys_setsockopt(i64 {}, i64 1, i64 7, i8* %{}_ptr, i64 4)\n", fd, id));
                code.push_str(&format!("  %{}_r = call i64 @sys_setsockopt(i64 {}, i64 1, i64 8, i8* %{}_ptr, i64 4)\n", id, fd, id));
                code.push_str(&format!("  %{} = trunc i64 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // SO_SNDBUF=0x1001, SO_RCVBUF=0x1002, SOL_SOCKET=0xFFFF
                let mut code = String::new();
                code.push_str(&format!("  %{}_val = alloca i32\n", id));
                code.push_str(&format!("  %{}_s32 = trunc i64 {} to i32\n", id, size));
                code.push_str(&format!("  store i32 %{}_s32, i32* %{}_val\n", id, id));
                code.push_str(&format!("  %{}_ptr = bitcast i32* %{}_val to i8*\n", id, id));
                code.push_str(&format!("  call i32 @setsockopt(i64 {}, i32 65535, i32 4097, i8* %{}_ptr, i32 4)\n", fd, id));
                code.push_str(&format!("  %{}_r = call i32 @setsockopt(i64 {}, i32 65535, i32 4098, i8* %{}_ptr, i32 4)\n", id, fd, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_keepalive(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // KAL fd mode - SO_KEEPALIVE
        if args.len() < 2 {
            return Err("KAL requires fd and mode (0=off, 1=on)".to_string());
        }
        let fd = self.emit_arg(&args[0])?;
        let mode = self.emit_arg(&args[1])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // SO_KEEPALIVE=9, SOL_SOCKET=1
                let mut code = String::new();
                code.push_str(&format!("  %{}_val = alloca i32\n", id));
                code.push_str(&format!("  %{}_m32 = trunc i64 {} to i32\n", id, mode));
                code.push_str(&format!("  store i32 %{}_m32, i32* %{}_val\n", id, id));
                code.push_str(&format!("  %{}_ptr = bitcast i32* %{}_val to i8*\n", id, id));
                code.push_str(&format!("  %{}_r = call i64 @sys_setsockopt(i64 {}, i64 1, i64 9, i8* %{}_ptr, i64 4)\n", id, fd, id));
                code.push_str(&format!("  %{} = trunc i64 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // SO_KEEPALIVE=0x0008, SOL_SOCKET=0xFFFF
                let mut code = String::new();
                code.push_str(&format!("  %{}_val = alloca i32\n", id));
                code.push_str(&format!("  %{}_m32 = trunc i64 {} to i32\n", id, mode));
                code.push_str(&format!("  store i32 %{}_m32, i32* %{}_val\n", id, id));
                code.push_str(&format!("  %{}_ptr = bitcast i32* %{}_val to i8*\n", id, id));
                code.push_str(&format!("  %{}_r = call i32 @setsockopt(i64 {}, i32 65535, i32 8, i8* %{}_ptr, i32 4)\n", id, fd, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_epoll_create(&self, id: &str, _args: &[Arg]) -> Result<String, String> {
        // EPL - Create epoll instance (Linux) or IOCP (Windows)
        match self.target {
            TargetPlatform::Linux => {
                // epoll_create1(0) - syscall 291
                Ok(format!("  %{} = call i64 @sys_epoll_create1(i32 0)\n", id))
            }
            TargetPlatform::Windows => {
                // CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 0)
                Ok(format!("  %{}_ptr = call i8* @CreateIoCompletionPort(i8* inttoptr (i64 -1 to i8*), i8* null, i64 0, i32 0)\n  %{} = ptrtoint i8* %{}_ptr to i64\n", id, id, id))
            }
        }
    }
    
    fn emit_epoll_ctl(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // ECT epfd op fd events - Add/modify/delete fd in epoll
        if args.len() < 4 {
            return Err("ECT requires epfd, op (1=add,2=del,3=mod), fd, events".to_string());
        }
        let epfd = self.emit_arg(&args[0])?;
        let op = self.emit_arg(&args[1])?;
        let fd = self.emit_arg(&args[2])?;
        let events = self.emit_arg(&args[3])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // struct epoll_event { uint32_t events; epoll_data_t data; } = 12 bytes
                let mut code = String::new();
                code.push_str(&format!("  %{}_ev = alloca [12 x i8]\n", id));
                code.push_str(&format!("  %{}_evptr = getelementptr [12 x i8], [12 x i8]* %{}_ev, i32 0, i32 0\n", id, id));
                code.push_str(&format!("  %{}_evcast = bitcast i8* %{}_evptr to i32*\n", id, id));
                code.push_str(&format!("  %{}_ev32 = trunc i64 {} to i32\n", id, events));
                code.push_str(&format!("  store i32 %{}_ev32, i32* %{}_evcast\n", id, id));
                code.push_str(&format!("  %{}_dataptr = getelementptr [12 x i8], [12 x i8]* %{}_ev, i32 0, i32 4\n", id, id));
                code.push_str(&format!("  %{}_datacast = bitcast i8* %{}_dataptr to i64*\n", id, id));
                code.push_str(&format!("  store i64 {}, i64* %{}_datacast\n", fd, id));
                code.push_str(&format!("  %{}_epfd32 = trunc i64 {} to i32\n", id, epfd));
                code.push_str(&format!("  %{}_op32 = trunc i64 {} to i32\n", id, op));
                code.push_str(&format!("  %{}_fd32 = trunc i64 {} to i32\n", id, fd));
                code.push_str(&format!("  %{}_r = call i32 @sys_epoll_ctl(i32 %{}_epfd32, i32 %{}_op32, i32 %{}_fd32, i8* %{}_evptr)\n", id, id, id, id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // For IOCP, we use CreateIoCompletionPort to associate fd
                // This is a simplified version
                Ok(format!("  %{}_ptr = call i8* @CreateIoCompletionPort(i8* inttoptr (i64 {} to i8*), i8* inttoptr (i64 {} to i8*), i64 {}, i32 0)\n  %{} = ptrtoint i8* %{}_ptr to i64\n", id, fd, epfd, fd, id, id))
            }
        }
    }
    
    fn emit_epoll_wait(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // EWA epfd events_buf max_events timeout_ms - Wait for events
        if args.len() < 4 {
            return Err("EWA requires epfd, events_buf, max_events, timeout_ms".to_string());
        }
        let epfd = self.emit_arg(&args[0])?;
        let buf = self.emit_arg(&args[1])?;
        let max = self.emit_arg(&args[2])?;
        let timeout = self.emit_arg(&args[3])?;
        
        match self.target {
            TargetPlatform::Linux => {
                let mut code = String::new();
                code.push_str(&format!("  %{}_epfd32 = trunc i64 {} to i32\n", id, epfd));
                code.push_str(&format!("  %{}_bufptr = inttoptr i64 {} to i8*\n", id, buf));
                code.push_str(&format!("  %{}_max32 = trunc i64 {} to i32\n", id, max));
                code.push_str(&format!("  %{}_to32 = trunc i64 {} to i32\n", id, timeout));
                code.push_str(&format!("  %{}_r = call i32 @sys_epoll_wait(i32 %{}_epfd32, i8* %{}_bufptr, i32 %{}_max32, i32 %{}_to32)\n", id, id, id, id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // GetQueuedCompletionStatus
                let mut code = String::new();
                code.push_str(&format!("  %{}_bytes = alloca i32\n", id));
                code.push_str(&format!("  %{}_key = alloca i64\n", id));
                code.push_str(&format!("  %{}_ovl = alloca i8*\n", id));
                code.push_str(&format!("  %{}_to32 = trunc i64 {} to i32\n", id, timeout));
                code.push_str(&format!("  %{}_r = call i32 @GetQueuedCompletionStatus(i8* inttoptr (i64 {} to i8*), i32* %{}_bytes, i64* %{}_key, i8** %{}_ovl, i32 %{}_to32)\n", id, epfd, id, id, id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_thread_create(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // THR .func .arg - Create a new thread running func(arg)
        if args.len() < 2 {
            return Err("THR requires function_name and arg".to_string());
        }
        
        // Get function name from first arg (should be a reference)
        let func_name = match &args[0] {
            Arg::Ref(name) => name.clone(),
            _ => return Err("THR first arg must be a function reference (.func_name)".to_string()),
        };
        
        let arg = self.emit_arg(&args[1])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // For Linux, we need to use clone() which is complex
                // For now, emit a placeholder that calls the function directly (single-threaded)
                // TODO: Implement proper clone() with stack setup
                let mut code = String::new();
                code.push_str(&format!("  ; TODO: Linux threading - calling {} directly\n", func_name));
                code.push_str(&format!("  %{} = call i64 @{}(i64 {})\n", id, func_name, arg));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // CreateThread(NULL, 0, func, arg, 0, NULL)
                let mut code = String::new();
                code.push_str(&format!("  %{}_argptr = inttoptr i64 {} to i8*\n", id, arg));
                code.push_str(&format!("  %{}_handle = call i8* @CreateThread(i8* null, i64 0, i64 (i8*)* @{}, i8* %{}_argptr, i32 0, i32* null)\n", id, func_name, id));
                code.push_str(&format!("  %{} = ptrtoint i8* %{}_handle to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_thread_join(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // JON thread_handle - Wait for thread to finish
        if args.is_empty() {
            return Err("JON requires thread_handle".to_string());
        }
        let handle = self.emit_arg(&args[0])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // waitpid or futex wait
                Ok(format!("  %{} = call i64 @sys_wait4(i64 {}, i32* null, i64 0, i8* null)\n", id, handle))
            }
            TargetPlatform::Windows => {
                // WaitForSingleObject(handle, INFINITE)
                let mut code = String::new();
                code.push_str(&format!("  %{}_handle = inttoptr i64 {} to i8*\n", id, handle));
                code.push_str(&format!("  %{}_r = call i32 @WaitForSingleObject(i8* %{}_handle, i32 -1)\n", id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_mutex_create(&self, id: &str, _args: &[Arg]) -> Result<String, String> {
        // MTX - Create a mutex
        match self.target {
            TargetPlatform::Linux => {
                // Allocate futex (4 bytes, initialized to 0 = unlocked)
                let mut code = String::new();
                code.push_str(&format!("  %{}_ptr = call i64 @sys_mmap(i64 0, i64 4, i64 3, i64 34, i64 -1, i64 0)\n", id));
                code.push_str(&format!("  %{}_iptr = inttoptr i64 %{}_ptr to i32*\n", id, id));
                code.push_str(&format!("  store i32 0, i32* %{}_iptr\n", id));
                code.push_str(&format!("  %{} = add i64 %{}_ptr, 0\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // CreateMutex(NULL, FALSE, NULL)
                let mut code = String::new();
                code.push_str(&format!("  %{}_handle = call i8* @CreateMutexA(i8* null, i32 0, i8* null)\n", id));
                code.push_str(&format!("  %{} = ptrtoint i8* %{}_handle to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_mutex_lock(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // LCK mutex - Lock mutex
        if args.is_empty() {
            return Err("LCK requires mutex".to_string());
        }
        let mutex = self.emit_arg(&args[0])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // Spinlock with futex fallback (simplified: just atomic exchange)
                let mut code = String::new();
                code.push_str(&format!("  %{}_ptr = inttoptr i64 {} to i32*\n", id, mutex));
                code.push_str(&format!("  %{}_old = atomicrmw xchg i32* %{}_ptr, i32 1 acquire\n", id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_old to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // WaitForSingleObject(mutex, INFINITE)
                let mut code = String::new();
                code.push_str(&format!("  %{}_handle = inttoptr i64 {} to i8*\n", id, mutex));
                code.push_str(&format!("  %{}_r = call i32 @WaitForSingleObject(i8* %{}_handle, i32 -1)\n", id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_mutex_unlock(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // ULK mutex - Unlock mutex
        if args.is_empty() {
            return Err("ULK requires mutex".to_string());
        }
        let mutex = self.emit_arg(&args[0])?;
        
        match self.target {
            TargetPlatform::Linux => {
                // Atomic store 0
                let mut code = String::new();
                code.push_str(&format!("  %{}_ptr = inttoptr i64 {} to i32*\n", id, mutex));
                code.push_str(&format!("  store atomic i32 0, i32* %{}_ptr release, align 4\n", id));
                code.push_str(&format!("  %{} = add i64 0, 0\n", id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                // ReleaseMutex(mutex)
                let mut code = String::new();
                code.push_str(&format!("  %{}_handle = inttoptr i64 {} to i8*\n", id, mutex));
                code.push_str(&format!("  %{}_r = call i32 @ReleaseMutex(i8* %{}_handle)\n", id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_queue_create(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // QUE capacity -> queue_ptr
        // Queue structure: [head:i64, tail:i64, capacity:i64, lock:i64, data:i64[capacity]]
        // Total size: 32 + capacity*8 bytes
        if args.is_empty() {
            return Err("QUE requires capacity".to_string());
        }
        let capacity = self.emit_arg(&args[0])?;
        
        match self.target {
            TargetPlatform::Linux => {
                let mut code = String::new();
                // Calculate size: 32 + capacity*8
                code.push_str(&format!("  %{}_datasize = mul i64 {}, 8\n", id, capacity));
                code.push_str(&format!("  %{}_size = add i64 %{}_datasize, 32\n", id, id));
                // Allocate with mmap
                code.push_str(&format!("  %{}_ptr = call i64 @sys_mmap(i64 0, i64 %{}_size, i64 3, i64 34, i64 -1, i64 0)\n", id, id));
                // Initialize: head=0, tail=0, capacity=cap, lock=0
                code.push_str(&format!("  %{}_headptr = inttoptr i64 %{}_ptr to i64*\n", id, id));
                code.push_str(&format!("  store i64 0, i64* %{}_headptr\n", id));
                code.push_str(&format!("  %{}_tailoff = add i64 %{}_ptr, 8\n", id, id));
                code.push_str(&format!("  %{}_tailptr = inttoptr i64 %{}_tailoff to i64*\n", id, id));
                code.push_str(&format!("  store i64 0, i64* %{}_tailptr\n", id));
                code.push_str(&format!("  %{}_capoff = add i64 %{}_ptr, 16\n", id, id));
                code.push_str(&format!("  %{}_capptr = inttoptr i64 %{}_capoff to i64*\n", id, id));
                code.push_str(&format!("  store i64 {}, i64* %{}_capptr\n", capacity, id));
                code.push_str(&format!("  %{}_lockoff = add i64 %{}_ptr, 24\n", id, id));
                code.push_str(&format!("  %{}_lockptr = inttoptr i64 %{}_lockoff to i64*\n", id, id));
                code.push_str(&format!("  store i64 0, i64* %{}_lockptr\n", id));
                code.push_str(&format!("  %{} = add i64 %{}_ptr, 0\n", id, id));
                Ok(code)
            }
            TargetPlatform::Windows => {
                let mut code = String::new();
                // Calculate size: 32 + capacity*8
                code.push_str(&format!("  %{}_datasize = mul i64 {}, 8\n", id, capacity));
                code.push_str(&format!("  %{}_size = add i64 %{}_datasize, 32\n", id, id));
                // Allocate with VirtualAlloc
                code.push_str(&format!("  %{}_rawptr = call i8* @VirtualAlloc(i8* null, i64 %{}_size, i32 12288, i32 4)\n", id, id));
                code.push_str(&format!("  %{}_ptr = ptrtoint i8* %{}_rawptr to i64\n", id, id));
                // Initialize: head=0, tail=0, capacity=cap, lock=0
                code.push_str(&format!("  %{}_headptr = inttoptr i64 %{}_ptr to i64*\n", id, id));
                code.push_str(&format!("  store i64 0, i64* %{}_headptr\n", id));
                code.push_str(&format!("  %{}_tailoff = add i64 %{}_ptr, 8\n", id, id));
                code.push_str(&format!("  %{}_tailptr = inttoptr i64 %{}_tailoff to i64*\n", id, id));
                code.push_str(&format!("  store i64 0, i64* %{}_tailptr\n", id));
                code.push_str(&format!("  %{}_capoff = add i64 %{}_ptr, 16\n", id, id));
                code.push_str(&format!("  %{}_capptr = inttoptr i64 %{}_capoff to i64*\n", id, id));
                code.push_str(&format!("  store i64 {}, i64* %{}_capptr\n", capacity, id));
                code.push_str(&format!("  %{}_lockoff = add i64 %{}_ptr, 24\n", id, id));
                code.push_str(&format!("  %{}_lockptr = inttoptr i64 %{}_lockoff to i64*\n", id, id));
                code.push_str(&format!("  store i64 0, i64* %{}_lockptr\n", id));
                code.push_str(&format!("  %{} = add i64 %{}_ptr, 0\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_queue_push(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // PSH queue value -> success (1 if pushed, 0 if full)
        if args.len() < 2 {
            return Err("PSH requires queue and value".to_string());
        }
        let queue = self.emit_arg(&args[0])?;
        let value = self.emit_arg(&args[1])?;
        
        // Spinlock acquire, push, release
        let mut code = String::new();
        // Get lock pointer
        code.push_str(&format!("  %{}_lockoff = add i64 {}, 24\n", id, queue));
        code.push_str(&format!("  %{}_lockptr = inttoptr i64 %{}_lockoff to i32*\n", id, id));
        // Spinlock acquire
        code.push_str(&format!("  br label %{}_spinlock\n", id));
        code.push_str(&format!("{}_spinlock:\n", id));
        code.push_str(&format!("  %{}_old = atomicrmw xchg i32* %{}_lockptr, i32 1 acquire\n", id, id));
        code.push_str(&format!("  %{}_locked = icmp eq i32 %{}_old, 0\n", id, id));
        code.push_str(&format!("  br i1 %{}_locked, label %{}_push, label %{}_spinlock\n", id, id, id));
        // Push logic
        code.push_str(&format!("{}_push:\n", id));
        // Load tail and capacity
        code.push_str(&format!("  %{}_tailptrr = inttoptr i64 {} to i64*\n", id, queue));
        code.push_str(&format!("  %{}_tailptrr2 = getelementptr i64, i64* %{}_tailptrr, i64 1\n", id, id));
        code.push_str(&format!("  %{}_tail = load i64, i64* %{}_tailptrr2\n", id, id));
        code.push_str(&format!("  %{}_capptrr = getelementptr i64, i64* %{}_tailptrr, i64 2\n", id, id));
        code.push_str(&format!("  %{}_cap = load i64, i64* %{}_capptrr\n", id, id));
        // Calculate next tail
        code.push_str(&format!("  %{}_nexttail = add i64 %{}_tail, 1\n", id, id));
        code.push_str(&format!("  %{}_nexttailmod = srem i64 %{}_nexttail, %{}_cap\n", id, id, id));
        // Load head
        code.push_str(&format!("  %{}_head = load i64, i64* %{}_tailptrr\n", id, id));
        // Check if full
        code.push_str(&format!("  %{}_full = icmp eq i64 %{}_nexttailmod, %{}_head\n", id, id, id));
        code.push_str(&format!("  br i1 %{}_full, label %{}_fail, label %{}_store\n", id, id, id));
        // Store value
        code.push_str(&format!("{}_store:\n", id));
        code.push_str(&format!("  %{}_dataoff = add i64 {}, 32\n", id, queue));
        code.push_str(&format!("  %{}_slotoff = mul i64 %{}_tail, 8\n", id, id));
        code.push_str(&format!("  %{}_slotaddr = add i64 %{}_dataoff, %{}_slotoff\n", id, id, id));
        code.push_str(&format!("  %{}_slotptr = inttoptr i64 %{}_slotaddr to i64*\n", id, id));
        code.push_str(&format!("  store i64 {}, i64* %{}_slotptr\n", value, id));
        // Update tail
        code.push_str(&format!("  store i64 %{}_nexttailmod, i64* %{}_tailptrr2\n", id, id));
        code.push_str(&format!("  br label %{}_done\n", id));
        // Fail path
        code.push_str(&format!("{}_fail:\n", id));
        code.push_str(&format!("  br label %{}_done\n", id));
        // Done - release lock
        code.push_str(&format!("{}_done:\n", id));
        code.push_str(&format!("  %{}_result = phi i64 [ 1, %{}_store ], [ 0, %{}_fail ]\n", id, id, id));
        code.push_str(&format!("  store atomic i32 0, i32* %{}_lockptr release, align 4\n", id));
        code.push_str(&format!("  %{} = add i64 %{}_result, 0\n", id, id));
        Ok(code)
    }
    
    fn emit_queue_pop(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // POP queue -> value (0 if empty)
        if args.is_empty() {
            return Err("POP requires queue".to_string());
        }
        let queue = self.emit_arg(&args[0])?;
        
        let mut code = String::new();
        // Get lock pointer
        code.push_str(&format!("  %{}_lockoff = add i64 {}, 24\n", id, queue));
        code.push_str(&format!("  %{}_lockptr = inttoptr i64 %{}_lockoff to i32*\n", id, id));
        // Spinlock acquire
        code.push_str(&format!("  br label %{}_spinlock\n", id));
        code.push_str(&format!("{}_spinlock:\n", id));
        code.push_str(&format!("  %{}_old = atomicrmw xchg i32* %{}_lockptr, i32 1 acquire\n", id, id));
        code.push_str(&format!("  %{}_locked = icmp eq i32 %{}_old, 0\n", id, id));
        code.push_str(&format!("  br i1 %{}_locked, label %{}_pop, label %{}_spinlock\n", id, id, id));
        // Pop logic
        code.push_str(&format!("{}_pop:\n", id));
        // Load head and tail
        code.push_str(&format!("  %{}_baseptr = inttoptr i64 {} to i64*\n", id, queue));
        code.push_str(&format!("  %{}_head = load i64, i64* %{}_baseptr\n", id, id));
        code.push_str(&format!("  %{}_tailptr = getelementptr i64, i64* %{}_baseptr, i64 1\n", id, id));
        code.push_str(&format!("  %{}_tail = load i64, i64* %{}_tailptr\n", id, id));
        code.push_str(&format!("  %{}_capptr = getelementptr i64, i64* %{}_baseptr, i64 2\n", id, id));
        code.push_str(&format!("  %{}_cap = load i64, i64* %{}_capptr\n", id, id));
        // Check if empty
        code.push_str(&format!("  %{}_empty = icmp eq i64 %{}_head, %{}_tail\n", id, id, id));
        code.push_str(&format!("  br i1 %{}_empty, label %{}_fail, label %{}_load\n", id, id, id));
        // Load value
        code.push_str(&format!("{}_load:\n", id));
        code.push_str(&format!("  %{}_dataoff = add i64 {}, 32\n", id, queue));
        code.push_str(&format!("  %{}_slotoff = mul i64 %{}_head, 8\n", id, id));
        code.push_str(&format!("  %{}_slotaddr = add i64 %{}_dataoff, %{}_slotoff\n", id, id, id));
        code.push_str(&format!("  %{}_slotptr = inttoptr i64 %{}_slotaddr to i64*\n", id, id));
        code.push_str(&format!("  %{}_val = load i64, i64* %{}_slotptr\n", id, id));
        // Update head
        code.push_str(&format!("  %{}_nexthead = add i64 %{}_head, 1\n", id, id));
        code.push_str(&format!("  %{}_nextheadmod = srem i64 %{}_nexthead, %{}_cap\n", id, id, id));
        code.push_str(&format!("  store i64 %{}_nextheadmod, i64* %{}_baseptr\n", id, id));
        code.push_str(&format!("  br label %{}_done\n", id));
        // Fail path
        code.push_str(&format!("{}_fail:\n", id));
        code.push_str(&format!("  br label %{}_done\n", id));
        // Done - release lock
        code.push_str(&format!("{}_done:\n", id));
        code.push_str(&format!("  %{}_result = phi i64 [ %{}_val, %{}_load ], [ 0, %{}_fail ]\n", id, id, id, id));
        code.push_str(&format!("  store atomic i32 0, i32* %{}_lockptr release, align 4\n", id));
        code.push_str(&format!("  %{} = add i64 %{}_result, 0\n", id, id));
        Ok(code)
    }
    
    fn emit_return(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // RET value - Return from function
        let value = if args.is_empty() {
            "0".to_string()
        } else {
            self.emit_arg(&args[0])?
        };
        Ok(format!("  %{} = add i64 {}, 0\n  ret i64 %{}\n", id, value, id))
    }
    
    fn emit_branch(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // BRN condition then_value else_value -> selected value
        if args.len() < 3 {
            return Err("BRN requires 3 arguments: condition, then_value, else_value".to_string());
        }
        let cond = self.emit_arg(&args[0])?;
        let then_val = self.emit_arg(&args[1])?;
        let else_val = self.emit_arg(&args[2])?;
        
        let mut code = String::new();
        code.push_str(&format!("  %{}_cond = icmp ne i64 {}, 0\n", id, cond));
        code.push_str(&format!("  %{} = select i1 %{}_cond, i64 {}, i64 {}\n", id, id, then_val, else_val));
        Ok(code)
    }
    
    fn emit_jump(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // JMP cond label_true label_false -> conditional branch (real jump)
        // Labels are references like .loop, .done
        if args.len() < 3 {
            return Err("JMP requires 3 arguments: condition, label_true, label_false".to_string());
        }
        let cond = self.emit_arg(&args[0])?;
        
        // Extract label names from references
        let label_true = match &args[1] {
            Arg::Ref(name) => name.clone(),
            _ => return Err("JMP label_true must be a reference".to_string()),
        };
        let label_false = match &args[2] {
            Arg::Ref(name) => name.clone(),
            _ => return Err("JMP label_false must be a reference".to_string()),
        };
        
        let mut code = String::new();
        code.push_str(&format!("  %{}_cond = icmp ne i64 {}, 0\n", id, cond));
        code.push_str(&format!("  br i1 %{}_cond, label %cycle_{}, label %lbl_{}\n", id, label_true, label_false));
        code.push_str(&format!("lbl_{}:\n", label_false));
        code.push_str(&format!("  %{} = add i64 0, 0\n", id));
        Ok(code)
    }
    
    fn emit_while(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // WHL cond_ref body_ref -> while loop
        // Generates: check condition, if true jump to body (cycle), else continue
        if args.len() < 2 {
            return Err("WHL requires 2 arguments: condition, body_label".to_string());
        }
        let cond = self.emit_arg(&args[0])?;
        
        let body_label = match &args[1] {
            Arg::Ref(name) => name.clone(),
            _ => return Err("WHL body must be a reference".to_string()),
        };
        
        let mut code = String::new();
        code.push_str(&format!("  %{}_cond = icmp ne i64 {}, 0\n", id, cond));
        code.push_str(&format!("  br i1 %{}_cond, label %cycle_{}, label %whl_end_{}\n", id, body_label, id));
        code.push_str(&format!("whl_end_{}:\n", id));
        code.push_str(&format!("  %{} = add i64 0, 0\n", id));
        Ok(code)
    }
    
    fn emit_end(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // END cond -> terminate if cond == 0, continue if != 0
        // Hardware-optimal: single conditional branch
        if args.is_empty() {
            return Err("END requires condition argument".to_string());
        }
        let cond = self.emit_arg(&args[0])?;
        
        let mut code = String::new();
        code.push_str(&format!("  %{}_cond = icmp ne i64 {}, 0\n", id, cond));
        code.push_str(&format!("  br i1 %{}_cond, label %end_cont_{}, label %end_exit_{}\n", id, id, id));
        code.push_str(&format!("end_exit_{}:\n", id));
        code.push_str("  ret i32 0\n");
        code.push_str(&format!("end_cont_{}:\n", id));
        code.push_str(&format!("  %{} = add i64 {}, 0\n", id, cond));
        Ok(code)
    }
    
    fn emit_transform(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // TRN input len rule -> apply rule to each byte, return result vector
        // This is the AI-native graph transform operator
        // Generates an optimized loop that LLVM can vectorize
        if args.len() < 3 {
            return Err("TRN requires input, length, rule".to_string());
        }
        
        let input_raw = self.emit_arg(&args[0])?;
        let len = self.emit_arg(&args[1])?;
        let _rule = self.emit_arg(&args[2])?;
        
        let mut code = String::new();
        
        // Convert input to i64 pointer if it's a string literal
        let input = if input_raw.starts_with("getelementptr") {
            code.push_str(&format!("  %{}_inbase = ptrtoint i8* {} to i64\n", id, input_raw));
            format!("%{}_inbase", id)
        } else {
            input_raw
        };
        
        match self.target {
            TargetPlatform::Linux | TargetPlatform::Windows => {
                // Allocate output buffer
                code.push_str(&format!("  %{}_out = call i8* @VirtualAlloc(i8* null, i64 {}, i32 12288, i32 4)\n", id, len));
                code.push_str(&format!("  %{}_outptr = ptrtoint i8* %{}_out to i64\n", id, id));
                
                // Loop setup
                code.push_str(&format!("  br label %trn_loop_{}\n", id));
                code.push_str(&format!("trn_loop_{}:\n", id));
                code.push_str(&format!("  %{}_i = phi i64 [0, %entry], [%{}_next, %trn_body_{}]\n", id, id, id));
                code.push_str(&format!("  %{}_done = icmp uge i64 %{}_i, {}\n", id, id, len));
                code.push_str(&format!("  br i1 %{}_done, label %trn_end_{}, label %trn_body_{}\n", id, id, id));
                
                // Loop body
                code.push_str(&format!("trn_body_{}:\n", id));
                // Read input byte
                code.push_str(&format!("  %{}_inoff = add i64 {}, %{}_i\n", id, input, id));
                code.push_str(&format!("  %{}_inptr = inttoptr i64 %{}_inoff to i8*\n", id, id));
                code.push_str(&format!("  %{}_byte = load i8, i8* %{}_inptr\n", id, id));
                // Apply rule: dot (46) -> 1, else -> 0
                code.push_str(&format!("  %{}_bytei = zext i8 %{}_byte to i64\n", id, id));
                code.push_str(&format!("  %{}_is_dot = icmp eq i64 %{}_bytei, 46\n", id, id));
                code.push_str(&format!("  %{}_result = select i1 %{}_is_dot, i8 1, i8 0\n", id, id));
                // Write output byte
                code.push_str(&format!("  %{}_outoff = add i64 %{}_outptr, %{}_i\n", id, id, id));
                code.push_str(&format!("  %{}_outaddr = inttoptr i64 %{}_outoff to i8*\n", id, id));
                code.push_str(&format!("  store i8 %{}_result, i8* %{}_outaddr\n", id, id));
                // Increment
                code.push_str(&format!("  %{}_next = add i64 %{}_i, 1\n", id, id));
                code.push_str(&format!("  br label %trn_loop_{}\n", id));
                
                // Loop end
                code.push_str(&format!("trn_end_{}:\n", id));
                code.push_str(&format!("  %{} = add i64 %{}_outptr, 0\n", id, id));
            }
        }
        
        Ok(code)
    }
    
    fn emit_fsm(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // FSM input len -> tokenize input, return token array
        // Token types: 0=EOF, 1=DOT, 2=COLON, 3=ID, 4=NUM, 5=STR, 6=NL, 7=SPACE
        // Output: array of (type:i8, start:i32, len:i16) = 7 bytes per token, padded to 8
        // Returns pointer to token array, with count in first 8 bytes
        
        if args.len() < 2 {
            return Err("FSM requires input, length".to_string());
        }
        
        let input_raw = self.emit_arg(&args[0])?;
        let len = self.emit_arg(&args[1])?;
        
        let mut code = String::new();
        
        // Convert input to i64 pointer if it's a string literal
        let input = if input_raw.starts_with("getelementptr") {
            code.push_str(&format!("  %{}_inbase = ptrtoint i8* {} to i64\n", id, input_raw));
            format!("%{}_inbase", id)
        } else {
            input_raw.clone()
        };
        
        match self.target {
            TargetPlatform::Linux | TargetPlatform::Windows => {
                // Allocate token buffer: 8 bytes header + max_tokens * 8 bytes
                // Header: [count:i64]
                // Each token: [type:i8, pad:i8, start:i32, len:i16]
                let max_tokens = 256;
                let buf_size = 8 + max_tokens * 8;
                
                code.push_str(&format!("  %{}_buf = call i8* @VirtualAlloc(i8* null, i64 {}, i32 12288, i32 4)\n", id, buf_size));
                code.push_str(&format!("  %{}_bufptr = ptrtoint i8* %{}_buf to i64\n", id, id));
                
                // Initialize count to 0
                code.push_str(&format!("  %{}_countptr = inttoptr i64 %{}_bufptr to i64*\n", id, id));
                code.push_str(&format!("  store i64 0, i64* %{}_countptr\n", id));
                
                // FSM loop
                code.push_str(&format!("  br label %fsm_loop_{}\n", id));
                code.push_str(&format!("fsm_loop_{}:\n", id));
                code.push_str(&format!("  %{}_pos = phi i64 [0, %entry], [%{}_nextpos, %fsm_next_{}]\n", id, id, id));
                code.push_str(&format!("  %{}_tokcount = phi i64 [0, %entry], [%{}_newtokcount, %fsm_next_{}]\n", id, id, id));
                
                // Check if done
                code.push_str(&format!("  %{}_done = icmp uge i64 %{}_pos, {}\n", id, id, len));
                code.push_str(&format!("  br i1 %{}_done, label %fsm_end_{}, label %fsm_body_{}\n", id, id, id));
                
                // FSM body - read char and classify
                code.push_str(&format!("fsm_body_{}:\n", id));
                code.push_str(&format!("  %{}_charoff = add i64 {}, %{}_pos\n", id, input, id));
                code.push_str(&format!("  %{}_charptr = inttoptr i64 %{}_charoff to i8*\n", id, id));
                code.push_str(&format!("  %{}_char = load i8, i8* %{}_charptr\n", id, id));
                code.push_str(&format!("  %{}_chari = zext i8 %{}_char to i64\n", id, id));
                
                // Classify character
                // DOT = 46, COLON = 58, NL = 10, SPACE = 32, TAB = 9, QUOTE = 34
                // 0-9 = 48-57, A-Z = 65-90, a-z = 97-122, _ = 95
                
                code.push_str(&format!("  %{}_is_dot = icmp eq i64 %{}_chari, 46\n", id, id));
                code.push_str(&format!("  %{}_is_colon = icmp eq i64 %{}_chari, 58\n", id, id));
                code.push_str(&format!("  %{}_is_nl = icmp eq i64 %{}_chari, 10\n", id, id));
                code.push_str(&format!("  %{}_is_space = icmp eq i64 %{}_chari, 32\n", id, id));
                code.push_str(&format!("  %{}_is_tab = icmp eq i64 %{}_chari, 9\n", id, id));
                code.push_str(&format!("  %{}_is_ws = or i1 %{}_is_space, %{}_is_tab\n", id, id, id));
                
                // Determine token type (1=DOT, 2=COLON, 6=NL, 7=SPACE, 0=other)
                code.push_str(&format!("  %{}_t1 = select i1 %{}_is_dot, i8 1, i8 0\n", id, id));
                code.push_str(&format!("  %{}_t2 = select i1 %{}_is_colon, i8 2, i8 %{}_t1\n", id, id, id));
                code.push_str(&format!("  %{}_t3 = select i1 %{}_is_nl, i8 6, i8 %{}_t2\n", id, id, id));
                code.push_str(&format!("  %{}_toktype = select i1 %{}_is_ws, i8 7, i8 %{}_t3\n", id, id, id));
                
                // Skip whitespace (don't emit token)
                code.push_str(&format!("  %{}_skip = icmp eq i8 %{}_toktype, 7\n", id, id));
                code.push_str(&format!("  br i1 %{}_skip, label %fsm_skip_{}, label %fsm_emit_{}\n", id, id, id));
                
                // Emit token
                code.push_str(&format!("fsm_emit_{}:\n", id));
                // Calculate token slot address: bufptr + 8 + tokcount * 8
                code.push_str(&format!("  %{}_slotoff = mul i64 %{}_tokcount, 8\n", id, id));
                code.push_str(&format!("  %{}_slotbase = add i64 %{}_bufptr, 8\n", id, id));
                code.push_str(&format!("  %{}_slotaddr = add i64 %{}_slotbase, %{}_slotoff\n", id, id, id));
                
                // Store token type
                code.push_str(&format!("  %{}_typeptr = inttoptr i64 %{}_slotaddr to i8*\n", id, id));
                code.push_str(&format!("  store i8 %{}_toktype, i8* %{}_typeptr\n", id, id));
                
                // Store position (as i32 at offset 2)
                code.push_str(&format!("  %{}_posoff = add i64 %{}_slotaddr, 2\n", id, id));
                code.push_str(&format!("  %{}_posptr = inttoptr i64 %{}_posoff to i32*\n", id, id));
                code.push_str(&format!("  %{}_pos32 = trunc i64 %{}_pos to i32\n", id, id));
                code.push_str(&format!("  store i32 %{}_pos32, i32* %{}_posptr\n", id, id));
                
                // Store length (1 for single-char tokens, at offset 6)
                code.push_str(&format!("  %{}_lenoff = add i64 %{}_slotaddr, 6\n", id, id));
                code.push_str(&format!("  %{}_lenptr = inttoptr i64 %{}_lenoff to i16*\n", id, id));
                code.push_str(&format!("  store i16 1, i16* %{}_lenptr\n", id));
                
                // Increment token count
                code.push_str(&format!("  %{}_newtokcount_emit = add i64 %{}_tokcount, 1\n", id, id));
                code.push_str(&format!("  br label %fsm_next_{}\n", id));
                
                // Skip (whitespace)
                code.push_str(&format!("fsm_skip_{}:\n", id));
                code.push_str(&format!("  br label %fsm_next_{}\n", id));
                
                // Next iteration
                code.push_str(&format!("fsm_next_{}:\n", id));
                code.push_str(&format!("  %{}_newtokcount = phi i64 [%{}_newtokcount_emit, %fsm_emit_{}], [%{}_tokcount, %fsm_skip_{}]\n", id, id, id, id, id));
                code.push_str(&format!("  %{}_nextpos = add i64 %{}_pos, 1\n", id, id));
                code.push_str(&format!("  br label %fsm_loop_{}\n", id));
                
                // End - store final count
                code.push_str(&format!("fsm_end_{}:\n", id));
                code.push_str(&format!("  store i64 %{}_tokcount, i64* %{}_countptr\n", id, id));
                code.push_str(&format!("  %{} = add i64 %{}_bufptr, 0\n", id, id));
            }
        }
        
        Ok(code)
    }
    
    // ========================================================================
    // GUI Operations
    // ========================================================================
    
    fn emit_window_create(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // WIN width height title -> window_handle
        if args.len() < 3 {
            return Err("WIN requires width, height, title".to_string());
        }
        let width = self.emit_arg(&args[0])?;
        let height = self.emit_arg(&args[1])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let title_ptr = self.emit_wide_string_arg(&args[2])?;
                let mut code = String::new();
                code.push_str("  call void @_gui_init()\n");
                code.push_str(&format!("  %{}_hinst = load i8*, i8** @.hinstance\n", id));
                code.push_str(&format!("  %{}_class = getelementptr [8 x i16], [8 x i16]* @.wndclass_name, i32 0, i32 0\n", id));
                code.push_str(&format!("  %{}_classptr = bitcast i16* %{}_class to i8*\n", id, id));
                code.push_str(&format!("  %{}_w32 = trunc i64 {} to i32\n", id, width));
                code.push_str(&format!("  %{}_h32 = trunc i64 {} to i32\n", id, height));
                // WS_OVERLAPPEDWINDOW = 0x00CF0000
                code.push_str(&format!("  %{}_ptr = call i8* @CreateWindowExW(i32 0, i8* %{}_classptr, i8* {}, i32 13565952, i32 100, i32 100, i32 %{}_w32, i32 %{}_h32, i8* null, i8* null, i8* %{}_hinst, i8* null)\n", 
                    id, id, title_ptr, id, id, id));
                code.push_str(&format!("  %{} = ptrtoint i8* %{}_ptr to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                let mut code = String::new();
                code.push_str(&format!("  %{}_dpy = call i8* @XOpenDisplay(i8* null)\n", id));
                code.push_str(&format!("  store i8* %{}_dpy, i8** @.x11_display\n", id));
                code.push_str(&format!("  %{}_root = call i64 @XDefaultRootWindow(i8* %{}_dpy)\n", id, id));
                code.push_str(&format!("  %{}_black = call i64 @XBlackPixel(i8* %{}_dpy, i32 0)\n", id, id));
                code.push_str(&format!("  %{}_white = call i64 @XWhitePixel(i8* %{}_dpy, i32 0)\n", id, id));
                code.push_str(&format!("  %{}_w32 = trunc i64 {} to i32\n", id, width));
                code.push_str(&format!("  %{}_h32 = trunc i64 {} to i32\n", id, height));
                code.push_str(&format!("  %{} = call i64 @XCreateSimpleWindow(i8* %{}_dpy, i64 %{}_root, i32 100, i32 100, i32 %{}_w32, i32 %{}_h32, i32 1, i64 %{}_black, i64 %{}_white)\n", 
                    id, id, id, id, id, id, id));
                // ExposureMask | KeyPressMask | ButtonPressMask = 32769
                code.push_str(&format!("  call i32 @XSelectInput(i8* %{}_dpy, i64 %{}, i64 32769)\n", id, id));
                code.push_str(&format!("  %{}_gc = call i8* @XCreateGC(i8* %{}_dpy, i64 %{}, i64 0, i8* null)\n", id, id, id));
                code.push_str(&format!("  store i8* %{}_gc, i8** @.x11_gc\n", id));
                Ok(code)
            }
        }
    }
    
    fn emit_window_show(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // SHW handle -> show window
        if args.is_empty() {
            return Err("SHW requires window handle".to_string());
        }
        let handle = self.emit_arg(&args[0])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let mut code = String::new();
                code.push_str(&format!("  %{}_ptr = inttoptr i64 {} to i8*\n", id, handle));
                // SW_SHOW = 5
                code.push_str(&format!("  %{}_r = call i32 @ShowWindow(i8* %{}_ptr, i32 5)\n", id, id));
                code.push_str(&format!("  call i32 @UpdateWindow(i8* %{}_ptr)\n", id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                let mut code = String::new();
                code.push_str(&format!("  %{}_dpy = load i8*, i8** @.x11_display\n", id));
                code.push_str(&format!("  %{}_r = call i32 @XMapWindow(i8* %{}_dpy, i64 {})\n", id, id, handle));
                code.push_str(&format!("  call i32 @XFlush(i8* %{}_dpy)\n", id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_window_hide(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // HID handle -> hide window
        if args.is_empty() {
            return Err("HID requires window handle".to_string());
        }
        let handle = self.emit_arg(&args[0])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let mut code = String::new();
                code.push_str(&format!("  %{}_ptr = inttoptr i64 {} to i8*\n", id, handle));
                // SW_HIDE = 0
                code.push_str(&format!("  %{}_r = call i32 @ShowWindow(i8* %{}_ptr, i32 0)\n", id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                let mut code = String::new();
                code.push_str(&format!("  %{}_dpy = load i8*, i8** @.x11_display\n", id));
                code.push_str(&format!("  %{}_r = call i32 @XUnmapWindow(i8* %{}_dpy, i64 {})\n", id, id, handle));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_event_poll(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // EVT handle -> event_type (0=none, 1=quit, 2=button_click, 3=expose)
        if args.is_empty() {
            return Err("EVT requires window handle".to_string());
        }
        let _handle = self.emit_arg(&args[0])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let mut code = String::new();
                // MSG structure is 48 bytes
                code.push_str(&format!("  %{}_msg = alloca [48 x i8]\n", id));
                code.push_str(&format!("  %{}_msgptr = getelementptr [48 x i8], [48 x i8]* %{}_msg, i32 0, i32 0\n", id, id));
                // Reset last button clicked
                code.push_str("  store i64 0, i64* @.last_btn_clicked\n");
                // PM_REMOVE = 1
                code.push_str(&format!("  %{}_has = call i32 @PeekMessageW(i8* %{}_msgptr, i8* null, i32 0, i32 0, i32 1)\n", id, id));
                code.push_str(&format!("  %{}_hasmsg = icmp ne i32 %{}_has, 0\n", id, id));
                code.push_str(&format!("  br i1 %{}_hasmsg, label %L{}_process, label %L{}_none\n", id, id, id));
                code.push_str(&format!("L{}_process:\n", id));
                code.push_str(&format!("  call i32 @TranslateMessage(i8* %{}_msgptr)\n", id));
                code.push_str(&format!("  call i64 @DispatchMessageW(i8* %{}_msgptr)\n", id));
                // Check message type (offset 8 in MSG)
                code.push_str(&format!("  %{}_msgtype_ptr = getelementptr [48 x i8], [48 x i8]* %{}_msg, i32 0, i32 8\n", id, id));
                code.push_str(&format!("  %{}_msgtype_cast = bitcast i8* %{}_msgtype_ptr to i32*\n", id, id));
                code.push_str(&format!("  %{}_msgtype = load i32, i32* %{}_msgtype_cast\n", id, id));
                // WM_QUIT = 18
                code.push_str(&format!("  %{}_isquit = icmp eq i32 %{}_msgtype, 18\n", id, id));
                code.push_str(&format!("  br i1 %{}_isquit, label %L{}_quit, label %L{}_checkbtn\n", id, id, id));
                code.push_str(&format!("L{}_quit:\n", id));
                code.push_str(&format!("  br label %L{}_done_quit\n", id));
                code.push_str(&format!("L{}_checkbtn:\n", id));
                code.push_str(&format!("  %{}_btn = load i64, i64* @.last_btn_clicked\n", id));
                code.push_str(&format!("  %{}_isbtn = icmp ne i64 %{}_btn, 0\n", id, id));
                code.push_str(&format!("  br i1 %{}_isbtn, label %L{}_btnclick, label %L{}_other\n", id, id, id));
                code.push_str(&format!("L{}_btnclick:\n", id));
                code.push_str(&format!("  br label %L{}_done_btn\n", id));
                code.push_str(&format!("L{}_other:\n", id));
                code.push_str(&format!("  br label %L{}_done_other\n", id));
                code.push_str(&format!("L{}_none:\n", id));
                code.push_str(&format!("  br label %L{}_done_none\n", id));
                code.push_str(&format!("L{}_done_quit:\n", id));
                code.push_str(&format!("  br label %L{}_end\n", id));
                code.push_str(&format!("L{}_done_btn:\n", id));
                code.push_str(&format!("  br label %L{}_end\n", id));
                code.push_str(&format!("L{}_done_other:\n", id));
                code.push_str(&format!("  br label %L{}_end\n", id));
                code.push_str(&format!("L{}_done_none:\n", id));
                code.push_str(&format!("  br label %L{}_end\n", id));
                code.push_str(&format!("L{}_end:\n", id));
                code.push_str(&format!("  %{} = phi i64 [ 1, %L{}_done_quit ], [ 2, %L{}_done_btn ], [ 3, %L{}_done_other ], [ 0, %L{}_done_none ]\n", 
                    id, id, id, id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                let mut code = String::new();
                // XEvent is 192 bytes
                code.push_str(&format!("  %{}_ev = alloca [192 x i8]\n", id));
                code.push_str(&format!("  %{}_evptr = getelementptr [192 x i8], [192 x i8]* %{}_ev, i32 0, i32 0\n", id, id));
                code.push_str(&format!("  %{}_dpy = load i8*, i8** @.x11_display\n", id));
                code.push_str(&format!("  %{}_pending = call i32 @XPending(i8* %{}_dpy)\n", id, id));
                code.push_str(&format!("  %{}_has = icmp sgt i32 %{}_pending, 0\n", id, id));
                code.push_str(&format!("  br i1 %{}_has, label %L{}_process, label %L{}_none\n", id, id, id));
                code.push_str(&format!("L{}_process:\n", id));
                code.push_str(&format!("  call i32 @XNextEvent(i8* %{}_dpy, i8* %{}_evptr)\n", id, id));
                // Event type is first i32
                code.push_str(&format!("  %{}_type_ptr = bitcast i8* %{}_evptr to i32*\n", id, id));
                code.push_str(&format!("  %{}_type = load i32, i32* %{}_type_ptr\n", id, id));
                // ButtonPress = 4, Expose = 12, DestroyNotify = 17
                code.push_str(&format!("  %{}_isbtn = icmp eq i32 %{}_type, 4\n", id, id));
                code.push_str(&format!("  br i1 %{}_isbtn, label %L{}_btn, label %L{}_checkexp\n", id, id, id));
                code.push_str(&format!("L{}_btn:\n", id));
                code.push_str(&format!("  br label %L{}_done_btn\n", id));
                code.push_str(&format!("L{}_checkexp:\n", id));
                code.push_str(&format!("  %{}_isexp = icmp eq i32 %{}_type, 12\n", id, id));
                code.push_str(&format!("  br i1 %{}_isexp, label %L{}_exp, label %L{}_other\n", id, id, id));
                code.push_str(&format!("L{}_exp:\n", id));
                code.push_str(&format!("  br label %L{}_done_exp\n", id));
                code.push_str(&format!("L{}_other:\n", id));
                code.push_str(&format!("  br label %L{}_done_other\n", id));
                code.push_str(&format!("L{}_none:\n", id));
                code.push_str(&format!("  br label %L{}_done_none\n", id));
                code.push_str(&format!("L{}_done_btn:\n", id));
                code.push_str(&format!("  br label %L{}_end\n", id));
                code.push_str(&format!("L{}_done_exp:\n", id));
                code.push_str(&format!("  br label %L{}_end\n", id));
                code.push_str(&format!("L{}_done_other:\n", id));
                code.push_str(&format!("  br label %L{}_end\n", id));
                code.push_str(&format!("L{}_done_none:\n", id));
                code.push_str(&format!("  br label %L{}_end\n", id));
                code.push_str(&format!("L{}_end:\n", id));
                code.push_str(&format!("  %{} = phi i64 [ 2, %L{}_done_btn ], [ 3, %L{}_done_exp ], [ 3, %L{}_done_other ], [ 0, %L{}_done_none ]\n", 
                    id, id, id, id, id));
                Ok(code)
            }
        }
    }
    
    fn emit_message_loop(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // RUN window_handle onclick_callback -> runs message loop until WM_QUIT
        // When button is clicked, shows the dialog specified in args[1] and args[2]
        if args.is_empty() {
            return Err("RUN requires window handle".to_string());
        }
        let _handle = self.emit_arg(&args[0])?;
        
        // Get dialog title and message if provided
        let (has_dialog, title_ptr, msg_ptr) = if args.len() >= 3 {
            let t = self.emit_wide_string_arg(&args[1])?;
            let m = self.emit_wide_string_arg(&args[2])?;
            (true, t, m)
        } else {
            (false, String::new(), String::new())
        };
        
        match self.target {
            TargetPlatform::Windows => {
                let mut code = String::new();
                // MSG structure is 48 bytes
                code.push_str(&format!("  %{}_msg = alloca [48 x i8]\n", id));
                code.push_str(&format!("  %{}_msgptr = getelementptr [48 x i8], [48 x i8]* %{}_msg, i32 0, i32 0\n", id, id));
                code.push_str(&format!("  br label %L{}_loop\n", id));
                
                // Main message loop
                code.push_str(&format!("L{}_loop:\n", id));
                code.push_str("  store i64 0, i64* @.last_btn_clicked\n");
                // GetMessage blocks until message available, returns 0 on WM_QUIT
                code.push_str(&format!("  %{}_ret = call i32 @GetMessageW(i8* %{}_msgptr, i8* null, i32 0, i32 0)\n", id, id));
                code.push_str(&format!("  %{}_quit = icmp eq i32 %{}_ret, 0\n", id, id));
                code.push_str(&format!("  br i1 %{}_quit, label %L{}_exit, label %L{}_dispatch\n", id, id, id));
                
                // Dispatch message
                code.push_str(&format!("L{}_dispatch:\n", id));
                code.push_str(&format!("  call i32 @TranslateMessage(i8* %{}_msgptr)\n", id));
                code.push_str(&format!("  call i64 @DispatchMessageW(i8* %{}_msgptr)\n", id));
                
                // Check if button was clicked
                code.push_str(&format!("  %{}_btn = load i64, i64* @.last_btn_clicked\n", id));
                code.push_str(&format!("  %{}_clicked = icmp ne i64 %{}_btn, 0\n", id, id));
                code.push_str(&format!("  br i1 %{}_clicked, label %L{}_onclick, label %L{}_loop\n", id, id, id));
                
                // On button click - show dialog
                code.push_str(&format!("L{}_onclick:\n", id));
                if has_dialog {
                    code.push_str(&format!("  call i32 @MessageBoxW(i8* null, i8* {}, i8* {}, i32 64)\n", msg_ptr, title_ptr));
                }
                code.push_str(&format!("  br label %L{}_loop\n", id));
                
                // Exit
                code.push_str(&format!("L{}_exit:\n", id));
                code.push_str(&format!("  %{} = add i64 0, 0\n", id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                // Simplified for Linux - just return
                Ok(format!("  %{} = add i64 0, 0\n", id))
            }
        }
    }
    
    fn emit_label_create(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // LBL parent x y width height text -> label_handle
        if args.len() < 6 {
            return Err("LBL requires parent, x, y, width, height, text".to_string());
        }
        let parent = self.emit_arg(&args[0])?;
        let x = self.emit_arg(&args[1])?;
        let y = self.emit_arg(&args[2])?;
        let width = self.emit_arg(&args[3])?;
        let height = self.emit_arg(&args[4])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let text_ptr = self.emit_wide_string_arg(&args[5])?;
                let mut code = String::new();
                code.push_str(&format!("  %{}_hinst = load i8*, i8** @.hinstance\n", id));
                code.push_str(&format!("  %{}_class = getelementptr [14 x i16], [14 x i16]* @.static_class, i32 0, i32 0\n", id));
                code.push_str(&format!("  %{}_classptr = bitcast i16* %{}_class to i8*\n", id, id));
                code.push_str(&format!("  %{}_parent = inttoptr i64 {} to i8*\n", id, parent));
                code.push_str(&format!("  %{}_x32 = trunc i64 {} to i32\n", id, x));
                code.push_str(&format!("  %{}_y32 = trunc i64 {} to i32\n", id, y));
                code.push_str(&format!("  %{}_w32 = trunc i64 {} to i32\n", id, width));
                code.push_str(&format!("  %{}_h32 = trunc i64 {} to i32\n", id, height));
                // WS_CHILD | WS_VISIBLE = 0x50000000
                code.push_str(&format!("  %{}_ptr = call i8* @CreateWindowExW(i32 0, i8* %{}_classptr, i8* {}, i32 1342177280, i32 %{}_x32, i32 %{}_y32, i32 %{}_w32, i32 %{}_h32, i8* %{}_parent, i8* null, i8* %{}_hinst, i8* null)\n", 
                    id, id, text_ptr, id, id, id, id, id, id));
                code.push_str(&format!("  %{} = ptrtoint i8* %{}_ptr to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                // X11 doesn't have native controls - we'll draw text directly
                let mut code = String::new();
                code.push_str(&format!("  ; X11 label at ({}, {}) - text drawn on expose\n", x, y));
                code.push_str(&format!("  %{} = add i64 0, 0\n", id));
                Ok(code)
            }
        }
    }
    
    fn emit_textbox_create(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // TXB parent x y width height -> textbox_handle
        if args.len() < 5 {
            return Err("TXB requires parent, x, y, width, height".to_string());
        }
        let parent = self.emit_arg(&args[0])?;
        let x = self.emit_arg(&args[1])?;
        let y = self.emit_arg(&args[2])?;
        let width = self.emit_arg(&args[3])?;
        let height = self.emit_arg(&args[4])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let mut code = String::new();
                code.push_str(&format!("  %{}_hinst = load i8*, i8** @.hinstance\n", id));
                code.push_str(&format!("  %{}_class = getelementptr [10 x i16], [10 x i16]* @.edit_class, i32 0, i32 0\n", id));
                code.push_str(&format!("  %{}_classptr = bitcast i16* %{}_class to i8*\n", id, id));
                code.push_str(&format!("  %{}_parent = inttoptr i64 {} to i8*\n", id, parent));
                code.push_str(&format!("  %{}_x32 = trunc i64 {} to i32\n", id, x));
                code.push_str(&format!("  %{}_y32 = trunc i64 {} to i32\n", id, y));
                code.push_str(&format!("  %{}_w32 = trunc i64 {} to i32\n", id, width));
                code.push_str(&format!("  %{}_h32 = trunc i64 {} to i32\n", id, height));
                // WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL = 0x50800080
                code.push_str(&format!("  %{}_ptr = call i8* @CreateWindowExW(i32 512, i8* %{}_classptr, i8* null, i32 1350565888, i32 %{}_x32, i32 %{}_y32, i32 %{}_w32, i32 %{}_h32, i8* %{}_parent, i8* null, i8* %{}_hinst, i8* null)\n", 
                    id, id, id, id, id, id, id, id));
                code.push_str(&format!("  %{} = ptrtoint i8* %{}_ptr to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                let mut code = String::new();
                code.push_str(&format!("  ; X11 textbox placeholder\n"));
                code.push_str(&format!("  %{} = add i64 0, 0\n", id));
                Ok(code)
            }
        }
    }
    
    fn emit_button_create(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // BTN parent x y width height text -> button_handle
        if args.len() < 6 {
            return Err("BTN requires parent, x, y, width, height, text".to_string());
        }
        let parent = self.emit_arg(&args[0])?;
        let x = self.emit_arg(&args[1])?;
        let y = self.emit_arg(&args[2])?;
        let width = self.emit_arg(&args[3])?;
        let height = self.emit_arg(&args[4])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let text_ptr = self.emit_wide_string_arg(&args[5])?;
                let ctrl_id = self.next_id();
                let mut code = String::new();
                code.push_str(&format!("  %{}_hinst = load i8*, i8** @.hinstance\n", id));
                code.push_str(&format!("  %{}_class = getelementptr [14 x i16], [14 x i16]* @.btn_class, i32 0, i32 0\n", id));
                code.push_str(&format!("  %{}_classptr = bitcast i16* %{}_class to i8*\n", id, id));
                code.push_str(&format!("  %{}_parent = inttoptr i64 {} to i8*\n", id, parent));
                code.push_str(&format!("  %{}_x32 = trunc i64 {} to i32\n", id, x));
                code.push_str(&format!("  %{}_y32 = trunc i64 {} to i32\n", id, y));
                code.push_str(&format!("  %{}_w32 = trunc i64 {} to i32\n", id, width));
                code.push_str(&format!("  %{}_h32 = trunc i64 {} to i32\n", id, height));
                // WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON = 0x50000000
                // Use control ID as HMENU parameter
                code.push_str(&format!("  %{}_ctrlid = inttoptr i64 {} to i8*\n", id, ctrl_id));
                code.push_str(&format!("  %{}_ptr = call i8* @CreateWindowExW(i32 0, i8* %{}_classptr, i8* {}, i32 1342177280, i32 %{}_x32, i32 %{}_y32, i32 %{}_w32, i32 %{}_h32, i8* %{}_parent, i8* %{}_ctrlid, i8* %{}_hinst, i8* null)\n", 
                    id, id, text_ptr, id, id, id, id, id, id, id));
                code.push_str(&format!("  %{} = ptrtoint i8* %{}_ptr to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                let mut code = String::new();
                code.push_str(&format!("  ; X11 button placeholder\n"));
                code.push_str(&format!("  %{} = add i64 0, 0\n", id));
                Ok(code)
            }
        }
    }
    
    fn emit_dialog_alert(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // DLG title message -> show alert dialog
        if args.len() < 2 {
            return Err("DLG requires title, message".to_string());
        }
        
        match self.target {
            TargetPlatform::Windows => {
                let title_ptr = self.emit_wide_string_arg(&args[0])?;
                let msg_ptr = self.emit_wide_string_arg(&args[1])?;
                let mut code = String::new();
                // MB_OK | MB_ICONINFORMATION = 0x40
                code.push_str(&format!("  %{}_r = call i32 @MessageBoxW(i8* null, i8* {}, i8* {}, i32 64)\n", id, msg_ptr, title_ptr));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                // X11 doesn't have native dialogs - print to console
                let mut code = String::new();
                code.push_str(&format!("  ; X11 alert dialog - printing to console\n"));
                code.push_str(&format!("  %{} = add i64 0, 0\n", id));
                Ok(code)
            }
        }
    }
    
    fn emit_get_value(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // GVL control_handle buffer len -> get text value (returns length)
        if args.len() < 3 {
            return Err("GVL requires control_handle, buffer, max_len".to_string());
        }
        let handle = self.emit_arg(&args[0])?;
        let buffer = self.emit_arg(&args[1])?;
        let max_len = self.emit_arg(&args[2])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let mut code = String::new();
                code.push_str(&format!("  %{}_hwnd = inttoptr i64 {} to i8*\n", id, handle));
                code.push_str(&format!("  %{}_buf = inttoptr i64 {} to i8*\n", id, buffer));
                code.push_str(&format!("  %{}_len32 = trunc i64 {} to i32\n", id, max_len));
                code.push_str(&format!("  %{}_r = call i32 @GetWindowTextW(i8* %{}_hwnd, i8* %{}_buf, i32 %{}_len32)\n", id, id, id, id));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                let mut code = String::new();
                code.push_str(&format!("  %{} = add i64 0, 0\n", id));
                Ok(code)
            }
        }
    }
    
    fn emit_set_value(&mut self, id: &str, args: &[Arg]) -> Result<String, String> {
        // SVL control_handle text -> set text value
        if args.len() < 2 {
            return Err("SVL requires control_handle, text".to_string());
        }
        let handle = self.emit_arg(&args[0])?;
        
        match self.target {
            TargetPlatform::Windows => {
                let text_ptr = self.emit_wide_string_arg(&args[1])?;
                let mut code = String::new();
                code.push_str(&format!("  %{}_hwnd = inttoptr i64 {} to i8*\n", id, handle));
                code.push_str(&format!("  %{}_r = call i32 @SetWindowTextW(i8* %{}_hwnd, i8* {})\n", id, id, text_ptr));
                code.push_str(&format!("  %{} = sext i32 %{}_r to i64\n", id, id));
                Ok(code)
            }
            TargetPlatform::Linux => {
                let mut code = String::new();
                code.push_str(&format!("  %{} = add i64 0, 0\n", id));
                Ok(code)
            }
        }
    }
    
    fn emit_wide_string_arg(&mut self, arg: &Arg) -> Result<String, String> {
        // Convert string argument to wide string pointer for Windows
        match arg {
            Arg::Lit(Value::String(s)) => {
                let str_id = self.strings.iter()
                    .find(|(_, v)| v == s)
                    .map(|(id, _)| id.clone())
                    .ok_or("String not found")?;
                let wide_len = s.encode_utf16().count() + 1;
                Ok(format!("bitcast ([{} x i16]* @{}_w to i8*)", wide_len, str_id))
            }
            Arg::Ref(name) => {
                if let Some((_, str_global_id)) = self.string_ids.iter().find(|(node_id, _)| node_id == name) {
                    if let Some((_, s)) = self.strings.iter().find(|(id, _)| id == str_global_id) {
                        let wide_len = s.encode_utf16().count() + 1;
                        return Ok(format!("bitcast ([{} x i16]* @{}_w to i8*)", wide_len, str_global_id));
                    }
                }
                Ok(format!("%{}", name))
            }
            _ => Err("Expected string argument".to_string()),
        }
    }
    
    fn emit_flow(&self, source: &Arg, dest: &FlowDest) -> Result<String, String> {
        match dest {
            FlowDest::Effect(Effect::Print) => {
                match source {
                    Arg::Lit(Value::String(s)) => {
                        let str_id = self.strings.iter()
                            .find(|(_, v)| v == s)
                            .map(|(id, _)| id.clone())
                            .ok_or("String not found")?;
                        let ptr = format!("getelementptr ([{} x i8], [{} x i8]* @{}, i32 0, i32 0)", s.len() + 1, s.len() + 1, str_id);
                        Ok(format!("  call i64 @sys_write(i32 1, i8* {}, i64 {})\n", ptr, s.len()))
                    }
                    Arg::Ref(name) => {
                        // Reference needs length - error for now
                        Err(format!("PRT .{} requires length - use: PRT .{} len", name, name))
                    }
                    _ => Ok(String::new()),
                }
            }
            FlowDest::CyclicNode(target) => {
                // Emit unconditional jump back to the cycle target
                Ok(format!("  br label %cycle_{}\n", target))
            }
            FlowDest::Node(_) => {
                // Regular flow - no code emission needed (data dependency)
                Ok(String::new())
            }
            _ => Ok(String::new()),
        }
    }
    
    fn escape_string(&self, s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                '\n' => result.push_str("\\0A"),
                '\r' => result.push_str("\\0D"),
                '\t' => result.push_str("\\09"),
                '"' => result.push_str("\\22"),
                '\\' => result.push_str("\\5C"),
                c if c.is_ascii() && !c.is_control() => result.push(c),
                _ => result.push_str(&format!("\\{:02X}", c as u8)),
            }
        }
        result
    }
    
    fn emit_linux_syscalls(&self) -> String {
        r#"
; ============================================================================
; LINUX x86_64 - Pure Syscalls Only
; ============================================================================

define i64 @sys_write(i32 %fd, i8* %buf, i64 %count) {
  %fd64 = sext i32 %fd to i64
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 1, i64 %fd64, i8* %buf, i64 %count)
  ret i64 %r
}

define i64 @sys_read(i32 %fd, i8* %buf, i64 %count) {
  %fd64 = sext i32 %fd to i64
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 0, i64 %fd64, i8* %buf, i64 %count)
  ret i64 %r
}

define i64 @sys_close(i64 %fd) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},~{rcx},~{r11},~{memory}"(i64 3, i64 %fd)
  ret i64 %r
}

define void @sys_exit(i32 %code) {
  %c = sext i32 %code to i64
  call void asm sideeffect "syscall", "{rax},{rdi}"(i64 60, i64 %c)
  unreachable
}

define i64 @sys_socket(i64 %domain, i64 %type, i64 %proto) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 41, i64 %domain, i64 %type, i64 %proto)
  ret i64 %r
}

define i64 @sys_setsockopt(i64 %fd, i64 %level, i64 %opt, i8* %val, i64 %len) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},~{rcx},~{r11},~{memory}"(i64 54, i64 %fd, i64 %level, i64 %opt, i8* %val, i64 %len)
  ret i64 %r
}

define i64 @sys_bind(i64 %fd, i8* %addr, i64 %len) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 49, i64 %fd, i8* %addr, i64 %len)
  ret i64 %r
}

define i64 @sys_listen(i64 %fd, i64 %backlog) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},~{rcx},~{r11},~{memory}"(i64 50, i64 %fd, i64 %backlog)
  ret i64 %r
}

define i64 @sys_accept(i64 %fd, i8* %addr, i64* %len) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 43, i64 %fd, i8* %addr, i64* %len)
  ret i64 %r
}

define i64 @sys_sendto(i64 %fd, i8* %buf, i64 %len, i64 %flags, i8* %addr, i64 %addrlen) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}"(i64 44, i64 %fd, i8* %buf, i64 %len, i64 %flags, i8* %addr, i64 %addrlen)
  ret i64 %r
}

define i64 @sys_recvfrom(i64 %fd, i8* %buf, i64 %len, i64 %flags, i8* %addr, i64* %addrlen) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}"(i64 45, i64 %fd, i8* %buf, i64 %len, i64 %flags, i8* %addr, i64* %addrlen)
  ret i64 %r
}

define i64 @sys_poll(i8* %fds, i64 %nfds, i32 %timeout) {
  %t64 = sext i32 %timeout to i64
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 7, i8* %fds, i64 %nfds, i64 %t64)
  ret i64 %r
}

define i64 @sys_mmap(i64 %addr, i64 %len, i64 %prot, i64 %flags, i64 %fd, i64 %off) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory}"(i64 9, i64 %addr, i64 %len, i64 %prot, i64 %flags, i64 %fd, i64 %off)
  ret i64 %r
}

define i64 @sys_munmap(i64 %addr, i64 %len) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},~{rcx},~{r11},~{memory}"(i64 11, i64 %addr, i64 %len)
  ret i64 %r
}

define i32 @sys_fcntl(i32 %fd, i32 %cmd, i32 %arg) {
  %fd64 = sext i32 %fd to i64
  %cmd64 = sext i32 %cmd to i64
  %arg64 = sext i32 %arg to i64
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 72, i64 %fd64, i64 %cmd64, i64 %arg64)
  %r32 = trunc i64 %r to i32
  ret i32 %r32
}

define i64 @sys_epoll_create1(i32 %flags) {
  %f64 = sext i32 %flags to i64
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},~{rcx},~{r11},~{memory}"(i64 291, i64 %f64)
  ret i64 %r
}

define i32 @sys_epoll_ctl(i32 %epfd, i32 %op, i32 %fd, i8* %event) {
  %epfd64 = sext i32 %epfd to i64
  %op64 = sext i32 %op to i64
  %fd64 = sext i32 %fd to i64
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},~{rcx},~{r11},~{memory}"(i64 233, i64 %epfd64, i64 %op64, i64 %fd64, i8* %event)
  %r32 = trunc i64 %r to i32
  ret i32 %r32
}

define i32 @sys_epoll_wait(i32 %epfd, i8* %events, i32 %maxevents, i32 %timeout) {
  %epfd64 = sext i32 %epfd to i64
  %max64 = sext i32 %maxevents to i64
  %to64 = sext i32 %timeout to i64
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},~{rcx},~{r11},~{memory}"(i64 232, i64 %epfd64, i8* %events, i64 %max64, i64 %to64)
  %r32 = trunc i64 %r to i32
  ret i32 %r32
}

define i64 @sys_clone(i64 %flags, i64 %stack, i64* %parent_tid, i64* %child_tid, i64 %tls) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},~{rcx},~{r11},~{memory}"(i64 56, i64 %flags, i64 %stack, i64* %parent_tid, i64* %child_tid, i64 %tls)
  ret i64 %r
}

define i64 @sys_wait4(i64 %pid, i32* %status, i64 %options, i8* %rusage) {
  %r = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},{r10},~{rcx},~{r11},~{memory}"(i64 61, i64 %pid, i32* %status, i64 %options, i8* %rusage)
  ret i64 %r
}

; ============================================================================
; LINUX X11 GUI Support (libX11.so)
; ============================================================================

declare i8* @XOpenDisplay(i8*)
declare i32 @XCloseDisplay(i8*)
declare i64 @XCreateSimpleWindow(i8*, i64, i32, i32, i32, i32, i32, i64, i64)
declare i32 @XMapWindow(i8*, i64)
declare i32 @XUnmapWindow(i8*, i64)
declare i32 @XDestroyWindow(i8*, i64)
declare i32 @XSelectInput(i8*, i64, i64)
declare i32 @XNextEvent(i8*, i8*)
declare i32 @XPending(i8*)
declare i32 @XStoreName(i8*, i64, i8*)
declare i64 @XDefaultRootWindow(i8*)
declare i64 @XBlackPixel(i8*, i32)
declare i64 @XWhitePixel(i8*, i32)
declare i32 @XFlush(i8*)
declare i32 @XDrawString(i8*, i64, i8*, i32, i32, i8*, i32)
declare i8* @XCreateGC(i8*, i64, i64, i8*)
declare i32 @XFreeGC(i8*, i8*)
declare i32 @XSetForeground(i8*, i8*, i64)
declare i32 @XFillRectangle(i8*, i64, i8*, i32, i32, i32, i32)
declare i32 @XDrawRectangle(i8*, i64, i8*, i32, i32, i32, i32)
declare i32 @XClearWindow(i8*, i64)

@.x11_display = private global i8* null
@.x11_gc = private global i8* null
@.x11_last_event = private global i32 0
@.x11_last_btn = private global i64 0

; Memory/String functions (libc)
declare i32 @memcmp(i8*, i8*, i64)
declare i8* @memchr(i8*, i32, i64)
declare i64 @strlen(i8*)
declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)
"#.to_string()
    }
    
    fn emit_windows_syscalls(&self) -> String {
        r#"
; ============================================================================
; WINDOWS x64 - Pure DLL Calls Only
; ============================================================================

declare dllimport i8* @GetStdHandle(i32)
declare dllimport i32 @WriteFile(i8*, i8*, i32, i32*, i8*)
declare dllimport void @ExitProcess(i32)
declare dllimport i64 @socket(i32, i32, i32)
declare dllimport i32 @setsockopt(i64, i32, i32, i8*, i32)
declare dllimport i32 @select(i32, i8*, i8*, i8*, i8*)
declare dllimport i32 @ioctlsocket(i64, i32, i32*)
declare dllimport i8* @CreateIoCompletionPort(i8*, i8*, i64, i32)
declare dllimport i32 @GetQueuedCompletionStatus(i8*, i32*, i64*, i8**, i32)
declare dllimport i32 @bind(i64, i8*, i32)
declare dllimport i32 @listen(i64, i32)
declare dllimport i64 @accept(i64, i8*, i32*)
declare dllimport i32 @send(i64, i8*, i32, i32)
declare dllimport i32 @recv(i64, i8*, i32, i32)
declare dllimport i32 @closesocket(i64)
declare dllimport i32 @shutdown(i64, i32)
declare dllimport i32 @WSAStartup(i16, i8*)
declare dllimport i8* @VirtualAlloc(i8*, i64, i32, i32)
declare dllimport i32 @VirtualFree(i8*, i64, i32)
declare dllimport i8* @CreateThread(i8*, i64, i8*, i8*, i32, i32*)
declare dllimport i32 @WaitForSingleObject(i8*, i32)
declare dllimport i8* @CreateMutexA(i8*, i32, i8*)
declare dllimport i32 @ReleaseMutex(i8*)

; GUI - user32.dll
declare dllimport i16 @RegisterClassExW(i8*)
declare dllimport i8* @CreateWindowExW(i32, i8*, i8*, i32, i32, i32, i32, i32, i8*, i8*, i8*, i8*)
declare dllimport i32 @ShowWindow(i8*, i32)
declare dllimport i32 @UpdateWindow(i8*)
declare dllimport i32 @GetMessageW(i8*, i8*, i32, i32)
declare dllimport i32 @PeekMessageW(i8*, i8*, i32, i32, i32)
declare dllimport i32 @TranslateMessage(i8*)
declare dllimport i64 @DispatchMessageW(i8*)
declare dllimport i64 @DefWindowProcW(i8*, i32, i64, i64)
declare dllimport i32 @PostQuitMessage(i32)
declare dllimport i32 @DestroyWindow(i8*)
declare dllimport i32 @MessageBoxW(i8*, i8*, i8*, i32)
declare dllimport i32 @GetWindowTextW(i8*, i8*, i32)
declare dllimport i32 @SetWindowTextW(i8*, i8*)
declare dllimport i8* @GetModuleHandleW(i8*)
declare dllimport i8* @LoadCursorW(i8*, i64)
declare dllimport i32 @SendMessageW(i8*, i32, i64, i64)

@.one = private global i32 1

@.stdout = private global i8* null
@.wsa_data = private global [512 x i8] zeroinitializer
@.wsa_init = private global i32 0

; GUI globals
@.gui_init = private global i32 0
@.hinstance = private global i8* null
@.wndclass_name = private constant [8 x i16] [i16 78, i16 69, i16 76, i16 65, i16 73, i16 65, i16 0, i16 0]  ; "NELAIA\0"
@.btn_class = private constant [14 x i16] [i16 66, i16 85, i16 84, i16 84, i16 79, i16 78, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0]  ; "BUTTON\0"
@.edit_class = private constant [10 x i16] [i16 69, i16 68, i16 73, i16 84, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0]  ; "EDIT\0"
@.static_class = private constant [14 x i16] [i16 83, i16 84, i16 65, i16 84, i16 73, i16 67, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0]  ; "STATIC\0"
@.btn_click_handler = private global i8* null
@.last_btn_clicked = private global i64 0

define void @_init_io() {
  %h = call i8* @GetStdHandle(i32 -11)
  store i8* %h, i8** @.stdout
  ret void
}

define void @_wsa_init() {
  %done = load i32, i32* @.wsa_init
  %need = icmp eq i32 %done, 0
  br i1 %need, label %init, label %end
init:
  %buf = getelementptr [512 x i8], [512 x i8]* @.wsa_data, i32 0, i32 0
  call i32 @WSAStartup(i16 514, i8* %buf)
  store i32 1, i32* @.wsa_init
  br label %end
end:
  ret void
}

define i64 @sys_write(i32 %fd, i8* %buf, i64 %count) {
  %h = load i8*, i8** @.stdout
  %c32 = trunc i64 %count to i32
  %written = alloca i32
  store i32 0, i32* %written
  call i32 @WriteFile(i8* %h, i8* %buf, i32 %c32, i32* %written, i8* null)
  %w = load i32, i32* %written
  %r = sext i32 %w to i64
  ret i64 %r
}

define void @sys_exit(i32 %code) {
  call void @ExitProcess(i32 %code)
  unreachable
}

; GUI Window Procedure
define i64 @_nelaia_wndproc(i8* %hwnd, i32 %msg, i64 %wparam, i64 %lparam) {
entry:
  ; WM_DESTROY = 2
  %is_destroy = icmp eq i32 %msg, 2
  br i1 %is_destroy, label %destroy, label %check_command

destroy:
  call i32 @PostQuitMessage(i32 0)
  ret i64 0

check_command:
  ; WM_COMMAND = 273 (0x111) - button clicks
  %is_command = icmp eq i32 %msg, 273
  br i1 %is_command, label %handle_command, label %default

handle_command:
  ; HIWORD(wparam) == BN_CLICKED (0)
  %hiword = lshr i64 %wparam, 16
  %hiword32 = trunc i64 %hiword to i32
  %is_click = icmp eq i32 %hiword32, 0
  br i1 %is_click, label %store_click, label %default

store_click:
  ; Store the control ID (LOWORD of wparam)
  %loword = and i64 %wparam, 65535
  store i64 %loword, i64* @.last_btn_clicked
  ret i64 0

default:
  %r = call i64 @DefWindowProcW(i8* %hwnd, i32 %msg, i64 %wparam, i64 %lparam)
  ret i64 %r
}

; Initialize GUI subsystem
define void @_gui_init() {
entry:
  %done = load i32, i32* @.gui_init
  %need = icmp eq i32 %done, 0
  br i1 %need, label %init, label %end

init:
  ; Get module handle
  %hinst = call i8* @GetModuleHandleW(i8* null)
  store i8* %hinst, i8** @.hinstance
  
  ; Register window class - WNDCLASSEXW is 80 bytes
  %wc = alloca [80 x i8]
  %wcptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 0
  
  ; cbSize = 80 (offset 0)
  %cbsize_ptr = bitcast i8* %wcptr to i32*
  store i32 80, i32* %cbsize_ptr
  
  ; style = CS_HREDRAW | CS_VREDRAW = 3 (offset 4)
  %style_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 4
  %style_cast = bitcast i8* %style_ptr to i32*
  store i32 3, i32* %style_cast
  
  ; lpfnWndProc (offset 8)
  %wndproc_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 8
  %wndproc_cast = bitcast i8* %wndproc_ptr to i64*
  %wndproc_fn = ptrtoint i64 (i8*, i32, i64, i64)* @_nelaia_wndproc to i64
  store i64 %wndproc_fn, i64* %wndproc_cast
  
  ; cbClsExtra = 0 (offset 16)
  %clsextra_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 16
  %clsextra_cast = bitcast i8* %clsextra_ptr to i32*
  store i32 0, i32* %clsextra_cast
  
  ; cbWndExtra = 0 (offset 20)
  %wndextra_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 20
  %wndextra_cast = bitcast i8* %wndextra_ptr to i32*
  store i32 0, i32* %wndextra_cast
  
  ; hInstance (offset 24)
  %hinst_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 24
  %hinst_cast = bitcast i8* %hinst_ptr to i8**
  store i8* %hinst, i8** %hinst_cast
  
  ; hIcon = NULL (offset 32)
  %hicon_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 32
  %hicon_cast = bitcast i8* %hicon_ptr to i8**
  store i8* null, i8** %hicon_cast
  
  ; hCursor = LoadCursor(NULL, IDC_ARROW=32512) (offset 40)
  %cursor = call i8* @LoadCursorW(i8* null, i64 32512)
  %hcursor_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 40
  %hcursor_cast = bitcast i8* %hcursor_ptr to i8**
  store i8* %cursor, i8** %hcursor_cast
  
  ; hbrBackground = COLOR_WINDOW+1 = 6 (offset 48)
  %hbr_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 48
  %hbr_cast = bitcast i8* %hbr_ptr to i64*
  store i64 6, i64* %hbr_cast
  
  ; lpszMenuName = NULL (offset 56)
  %menu_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 56
  %menu_cast = bitcast i8* %menu_ptr to i8**
  store i8* null, i8** %menu_cast
  
  ; lpszClassName (offset 64)
  %classname = getelementptr [8 x i16], [8 x i16]* @.wndclass_name, i32 0, i32 0
  %classname_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 64
  %classname_cast = bitcast i8* %classname_ptr to i16**
  store i16* %classname, i16** %classname_cast
  
  ; hIconSm = NULL (offset 72)
  %hiconsm_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 72
  %hiconsm_cast = bitcast i8* %hiconsm_ptr to i8**
  store i8* null, i8** %hiconsm_cast
  
  ; Register the class
  call i16 @RegisterClassExW(i8* %wcptr)
  
  store i32 1, i32* @.gui_init
  br label %end

end:
  ret void
}
"#.to_string()
    }
    
    fn emit_entry(&self) -> String {
        match self.target {
            TargetPlatform::Linux => r#"
define void @_start() {
  %r = call i32 @nelaia_main()
  call void @sys_exit(i32 %r)
  unreachable
}
"#.to_string(),
            TargetPlatform::Windows => r#"
define void @mainCRTStartup() {
  call void @_init_io()
  %r = call i32 @nelaia_main()
  call void @sys_exit(i32 %r)
  unreachable
}
"#.to_string(),
        }
    }
    
    /// Optimized Windows syscalls - only emit what's needed
    fn emit_windows_syscalls_optimized(&self) -> String {
        let usage = self.usage.as_ref().unwrap();
        let mut ir = String::new();
        
        ir.push_str(r#"
; ============================================================================
; WINDOWS x64 - Optimized (Dead Code Eliminated)
; ============================================================================

"#);
        
        // Always needed: basic I/O and exit
        ir.push_str(r#"declare dllimport i8* @GetStdHandle(i32)
declare dllimport i32 @WriteFile(i8*, i8*, i32, i32*, i8*)
declare dllimport void @ExitProcess(i32)
"#);
        
        // Network declarations (only if used)
        if usage.uses_network {
            ir.push_str(r#"declare dllimport i64 @socket(i32, i32, i32)
declare dllimport i32 @setsockopt(i64, i32, i32, i8*, i32)
declare dllimport i32 @bind(i64, i8*, i32)
declare dllimport i32 @listen(i64, i32)
declare dllimport i64 @accept(i64, i8*, i32*)
declare dllimport i32 @send(i64, i8*, i32, i32)
declare dllimport i32 @recv(i64, i8*, i32, i32)
declare dllimport i32 @closesocket(i64)
declare dllimport i32 @WSAStartup(i16, i8*)
"#);
            // Select/poll only if specifically used
            if usage.used_ops.contains(&Op::Sel) || usage.used_ops.contains(&Op::Rdy) {
                ir.push_str("declare dllimport i32 @select(i32, i8*, i8*, i8*, i8*)\n");
            }
            if usage.used_ops.contains(&Op::Nbk) {
                ir.push_str("declare dllimport i32 @ioctlsocket(i64, i32, i32*)\n");
            }
            if usage.used_ops.contains(&Op::Epl) || usage.used_ops.contains(&Op::Ewa) {
                ir.push_str(r#"declare dllimport i8* @CreateIoCompletionPort(i8*, i8*, i64, i32)
declare dllimport i32 @GetQueuedCompletionStatus(i8*, i32*, i64*, i8**, i32)
"#);
            }
        }
        
        // Threading declarations (only if used)
        if usage.uses_threading {
            ir.push_str(r#"declare dllimport i8* @VirtualAlloc(i8*, i64, i32, i32)
declare dllimport i32 @VirtualFree(i8*, i64, i32)
declare dllimport i8* @CreateThread(i8*, i64, i8*, i8*, i32, i32*)
declare dllimport i32 @WaitForSingleObject(i8*, i32)
"#);
            if usage.used_ops.contains(&Op::Mtx) || usage.used_ops.contains(&Op::Lck) {
                ir.push_str(r#"declare dllimport i8* @CreateMutexA(i8*, i32, i8*)
declare dllimport i32 @ReleaseMutex(i8*)
"#);
            }
        } else {
            // Memory allocation might still be needed
            ir.push_str(r#"declare dllimport i8* @VirtualAlloc(i8*, i64, i32, i32)
"#);
        }
        
        // GUI declarations (only if used)
        if usage.uses_gui {
            ir.push_str(r#"
; GUI - user32.dll
declare dllimport i16 @RegisterClassExW(i8*)
declare dllimport i8* @CreateWindowExW(i32, i8*, i8*, i32, i32, i32, i32, i32, i8*, i8*, i8*, i8*)
declare dllimport i32 @ShowWindow(i8*, i32)
declare dllimport i32 @UpdateWindow(i8*)
declare dllimport i32 @GetMessageW(i8*, i8*, i32, i32)
declare dllimport i32 @PeekMessageW(i8*, i8*, i32, i32, i32)
declare dllimport i32 @TranslateMessage(i8*)
declare dllimport i64 @DispatchMessageW(i8*)
declare dllimport i64 @DefWindowProcW(i8*, i32, i64, i64)
declare dllimport i32 @PostQuitMessage(i32)
declare dllimport i32 @DestroyWindow(i8*)
declare dllimport i32 @MessageBoxW(i8*, i8*, i8*, i32)
declare dllimport i32 @GetWindowTextW(i8*, i8*, i32)
declare dllimport i32 @SetWindowTextW(i8*, i8*)
declare dllimport i8* @GetModuleHandleW(i8*)
declare dllimport i8* @LoadCursorW(i8*, i64)
declare dllimport i32 @SendMessageW(i8*, i32, i64, i64)
"#);
        }
        
        // File I/O declarations (only if used)
        if usage.uses_file_io {
            ir.push_str(r#"
; File I/O - kernel32.dll
declare dllimport i8* @CreateFileA(i8*, i32, i32, i8*, i32, i32, i8*)
declare dllimport i32 @ReadFile(i8*, i8*, i32, i32*, i8*)
declare dllimport i32 @CloseHandle(i8*)
; Memory functions - msvcrt
declare i32 @memcmp(i8*, i8*, i64)
declare i8* @memchr(i8*, i32, i64)
declare i64 @strlen(i8*)
declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)
"#);
        }
        
        // Globals (only what's needed)
        ir.push_str("\n@.one = private global i32 1\n");
        ir.push_str("@.stdout = private global i8* null\n");
        
        if usage.uses_network {
            ir.push_str("@.wsa_data = private global [512 x i8] zeroinitializer\n");
            ir.push_str("@.wsa_init = private global i32 0\n");
        }
        
        if usage.uses_gui {
            ir.push_str(r#"
; GUI globals
@.gui_init = private global i32 0
@.hinstance = private global i8* null
@.wndclass_name = private constant [8 x i16] [i16 78, i16 69, i16 76, i16 65, i16 73, i16 65, i16 0, i16 0]
@.btn_class = private constant [14 x i16] [i16 66, i16 85, i16 84, i16 84, i16 79, i16 78, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0]
@.edit_class = private constant [10 x i16] [i16 69, i16 68, i16 73, i16 84, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0]
@.static_class = private constant [14 x i16] [i16 83, i16 84, i16 65, i16 84, i16 73, i16 67, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0, i16 0]
@.btn_click_handler = private global i8* null
@.last_btn_clicked = private global i64 0
"#);
        }
        
        // Core functions (always needed)
        ir.push_str(r#"
define void @_init_io() {
  %h = call i8* @GetStdHandle(i32 -11)
  store i8* %h, i8** @.stdout
  ret void
}

define i64 @sys_write(i32 %fd, i8* %buf, i64 %count) {
  %h = load i8*, i8** @.stdout
  %c32 = trunc i64 %count to i32
  %written = alloca i32
  store i32 0, i32* %written
  call i32 @WriteFile(i8* %h, i8* %buf, i32 %c32, i32* %written, i8* null)
  %w = load i32, i32* %written
  %r = sext i32 %w to i64
  ret i64 %r
}

define void @sys_exit(i32 %code) {
  call void @ExitProcess(i32 %code)
  unreachable
}
"#);
        
        // WSA init (only if network used)
        if usage.uses_network {
            ir.push_str(r#"
define void @_wsa_init() {
  %done = load i32, i32* @.wsa_init
  %need = icmp eq i32 %done, 0
  br i1 %need, label %init, label %end
init:
  %buf = getelementptr [512 x i8], [512 x i8]* @.wsa_data, i32 0, i32 0
  call i32 @WSAStartup(i16 514, i8* %buf)
  store i32 1, i32* @.wsa_init
  br label %end
end:
  ret void
}
"#);
        }
        
        // GUI init (only if GUI used)
        if usage.uses_gui {
            ir.push_str(&self.emit_gui_init());
        }
        
        ir
    }
    
    fn emit_gui_init(&self) -> String {
        r#"
; GUI Window Procedure
define i64 @_nelaia_wndproc(i8* %hwnd, i32 %msg, i64 %wparam, i64 %lparam) {
entry:
  %is_destroy = icmp eq i32 %msg, 2
  br i1 %is_destroy, label %destroy, label %check_command
destroy:
  call i32 @PostQuitMessage(i32 0)
  ret i64 0
check_command:
  %is_command = icmp eq i32 %msg, 273
  br i1 %is_command, label %handle_command, label %default
handle_command:
  %hiword = lshr i64 %wparam, 16
  %hiword32 = trunc i64 %hiword to i32
  %is_click = icmp eq i32 %hiword32, 0
  br i1 %is_click, label %store_click, label %default
store_click:
  %loword = and i64 %wparam, 65535
  store i64 %loword, i64* @.last_btn_clicked
  ret i64 0
default:
  %r = call i64 @DefWindowProcW(i8* %hwnd, i32 %msg, i64 %wparam, i64 %lparam)
  ret i64 %r
}

define void @_gui_init() {
entry:
  %done = load i32, i32* @.gui_init
  %need = icmp eq i32 %done, 0
  br i1 %need, label %init, label %end
init:
  %hinst = call i8* @GetModuleHandleW(i8* null)
  store i8* %hinst, i8** @.hinstance
  %wc = alloca [80 x i8]
  %wcptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 0
  %cbsize_ptr = bitcast i8* %wcptr to i32*
  store i32 80, i32* %cbsize_ptr
  %style_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 4
  %style_cast = bitcast i8* %style_ptr to i32*
  store i32 3, i32* %style_cast
  %wndproc_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 8
  %wndproc_cast = bitcast i8* %wndproc_ptr to i64*
  %wndproc_fn = ptrtoint i64 (i8*, i32, i64, i64)* @_nelaia_wndproc to i64
  store i64 %wndproc_fn, i64* %wndproc_cast
  %clsextra_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 16
  %clsextra_cast = bitcast i8* %clsextra_ptr to i32*
  store i32 0, i32* %clsextra_cast
  %wndextra_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 20
  %wndextra_cast = bitcast i8* %wndextra_ptr to i32*
  store i32 0, i32* %wndextra_cast
  %hinst_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 24
  %hinst_cast = bitcast i8* %hinst_ptr to i8**
  store i8* %hinst, i8** %hinst_cast
  %hicon_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 32
  %hicon_cast = bitcast i8* %hicon_ptr to i8**
  store i8* null, i8** %hicon_cast
  %cursor = call i8* @LoadCursorW(i8* null, i64 32512)
  %hcursor_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 40
  %hcursor_cast = bitcast i8* %hcursor_ptr to i8**
  store i8* %cursor, i8** %hcursor_cast
  %hbr_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 48
  %hbr_cast = bitcast i8* %hbr_ptr to i64*
  store i64 6, i64* %hbr_cast
  %menu_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 56
  %menu_cast = bitcast i8* %menu_ptr to i8**
  store i8* null, i8** %menu_cast
  %classname = getelementptr [8 x i16], [8 x i16]* @.wndclass_name, i32 0, i32 0
  %classname_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 64
  %classname_cast = bitcast i8* %classname_ptr to i16**
  store i16* %classname, i16** %classname_cast
  %hiconsm_ptr = getelementptr [80 x i8], [80 x i8]* %wc, i32 0, i32 72
  %hiconsm_cast = bitcast i8* %hiconsm_ptr to i8**
  store i8* null, i8** %hiconsm_cast
  call i16 @RegisterClassExW(i8* %wcptr)
  store i32 1, i32* @.gui_init
  br label %end
end:
  ret void
}
"#.to_string()
    }
    
    // ============================================================
    // SELF-HOSTING PRIMITIVES: Memory Operations
    // ============================================================
    
    fn emit_memcpy(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // CPY src dst len -> copies len bytes from src to dst
        if args.len() < 3 {
            return Err("CPY requires src, dst, len".to_string());
        }
        let src = self.emit_arg(&args[0])?;
        let dst = self.emit_arg(&args[1])?;
        let len = self.emit_arg(&args[2])?;
        Ok(format!(
            "  %{}_src = inttoptr i64 {} to i8*\n  %{}_dst = inttoptr i64 {} to i8*\n  call void @llvm.memcpy.p0i8.p0i8.i64(i8* %{}_dst, i8* %{}_src, i64 {}, i1 false)\n  %{} = add i64 0, {}\n",
            id, src, id, dst, id, id, len, id, len
        ))
    }
    
    fn emit_memcmp(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // CMP a b len -> -1/0/1
        if args.len() < 3 {
            return Err("CMP requires a, b, len".to_string());
        }
        let a = self.emit_arg(&args[0])?;
        let b = self.emit_arg(&args[1])?;
        let len = self.emit_arg(&args[2])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!(
                    "  %{}_a = inttoptr i64 {} to i8*\n  %{}_b = inttoptr i64 {} to i8*\n  %{}_r = call i32 @memcmp(i8* %{}_a, i8* %{}_b, i64 {})\n  %{} = sext i32 %{}_r to i64\n",
                    id, a, id, b, id, id, id, len, id, id
                ))
            }
            TargetPlatform::Windows => {
                Ok(format!(
                    "  %{}_a = inttoptr i64 {} to i8*\n  %{}_b = inttoptr i64 {} to i8*\n  %{}_r = call i32 @memcmp(i8* %{}_a, i8* %{}_b, i64 {})\n  %{} = sext i32 %{}_r to i64\n",
                    id, a, id, b, id, id, id, len, id, id
                ))
            }
        }
    }
    
    fn emit_memfind(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // FND buf len byte -> position or -1
        if args.len() < 3 {
            return Err("FND requires buf, len, byte".to_string());
        }
        let buf = self.emit_arg(&args[0])?;
        let len = self.emit_arg(&args[1])?;
        let byte = self.emit_arg(&args[2])?;
        Ok(format!(
            "  %{}_buf = inttoptr i64 {} to i8*\n  %{}_byte = trunc i64 {} to i32\n  %{}_found = call i8* @memchr(i8* %{}_buf, i32 %{}_byte, i64 {})\n  %{}_isnull = icmp eq i8* %{}_found, null\n  %{}_pos = ptrtoint i8* %{}_found to i64\n  %{}_base = ptrtoint i8* %{}_buf to i64\n  %{}_offset = sub i64 %{}_pos, %{}_base\n  %{} = select i1 %{}_isnull, i64 -1, i64 %{}_offset\n",
            id, buf, id, byte, id, id, id, len, id, id, id, id, id, id, id, id, id, id, id, id
        ))
    }
    
    fn emit_strlen(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // SLN ptr -> length until \0
        if args.is_empty() {
            return Err("SLN requires ptr".to_string());
        }
        let ptr = self.emit_arg(&args[0])?;
        Ok(format!(
            "  %{}_ptr = inttoptr i64 {} to i8*\n  %{}_len = call i64 @strlen(i8* %{}_ptr)\n  %{} = add i64 0, %{}_len\n",
            id, ptr, id, id, id, id
        ))
    }
    
    // ============================================================
    // SELF-HOSTING PRIMITIVES: File I/O
    // ============================================================
    
    fn emit_file_open(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // FOP path mode -> handle (mode: 0=read, 1=write, 2=append)
        if args.len() < 2 {
            return Err("FOP requires path, mode".to_string());
        }
        let path = self.emit_arg(&args[0])?;
        let mode = self.emit_arg(&args[1])?;
        match self.target {
            TargetPlatform::Linux => {
                // open(path, flags, mode) - flags: 0=O_RDONLY, 1=O_WRONLY|O_CREAT|O_TRUNC, 2=O_WRONLY|O_CREAT|O_APPEND
                Ok(format!(
                    "  %{}_path = inttoptr i64 {} to i8*\n  %{}_isread = icmp eq i64 {}, 0\n  %{}_iswrite = icmp eq i64 {}, 1\n  %{}_flags_w = select i1 %{}_iswrite, i64 577, i64 1089\n  %{}_flags = select i1 %{}_isread, i64 0, i64 %{}_flags_w\n  %{} = call i64 @sys_open(i8* %{}_path, i64 %{}_flags, i64 420)\n",
                    id, path, id, mode, id, mode, id, id, id, id, id, id, id, id
                ))
            }
            TargetPlatform::Windows => {
                // CreateFileA(path, access, share, security, creation, flags, template)
                // path might be a getelementptr (pointer) or i64, handle both
                let path_conv = if path.starts_with("getelementptr") || path.starts_with("%") {
                    format!("  %{}_path = bitcast i8* {} to i8*\n", id, path)
                } else {
                    format!("  %{}_path = inttoptr i64 {} to i8*\n", id, path)
                };
                Ok(format!(
                    "{}  %{}_isread = icmp eq i64 {}, 0\n  %{}_access = select i1 %{}_isread, i32 -2147483648, i32 1073741824\n  %{}_creation = select i1 %{}_isread, i32 3, i32 2\n  %{}_h = call i8* @CreateFileA(i8* %{}_path, i32 %{}_access, i32 0, i8* null, i32 %{}_creation, i32 128, i8* null)\n  %{} = ptrtoint i8* %{}_h to i64\n",
                    path_conv, id, mode, id, id, id, id, id, id, id, id, id, id
                ))
            }
        }
    }
    
    fn emit_file_read(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // FRD handle buf len -> bytes_read
        if args.len() < 3 {
            return Err("FRD requires handle, buf, len".to_string());
        }
        let handle = self.emit_arg(&args[0])?;
        let buf = self.emit_arg(&args[1])?;
        let len = self.emit_arg(&args[2])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!(
                    "  %{}_buf = inttoptr i64 {} to i8*\n  %{} = call i64 @sys_read(i64 {}, i8* %{}_buf, i64 {})\n",
                    id, buf, id, handle, id, len
                ))
            }
            TargetPlatform::Windows => {
                Ok(format!(
                    "  %{}_h = inttoptr i64 {} to i8*\n  %{}_buf = inttoptr i64 {} to i8*\n  %{}_read = alloca i32\n  call i32 @ReadFile(i8* %{}_h, i8* %{}_buf, i32 {}, i32* %{}_read, i8* null)\n  %{}_r = load i32, i32* %{}_read\n  %{} = zext i32 %{}_r to i64\n",
                    id, handle, id, buf, id, id, id, len, id, id, id, id, id
                ))
            }
        }
    }
    
    fn emit_file_write(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // FWR handle buf len -> bytes_written
        if args.len() < 3 {
            return Err("FWR requires handle, buf, len".to_string());
        }
        let handle = self.emit_arg(&args[0])?;
        let buf = self.emit_arg(&args[1])?;
        let len = self.emit_arg(&args[2])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!(
                    "  %{}_buf = inttoptr i64 {} to i8*\n  %{} = call i64 @sys_write(i32 {}, i8* %{}_buf, i64 {})\n",
                    id, buf, id, handle, id, len
                ))
            }
            TargetPlatform::Windows => {
                Ok(format!(
                    "  %{}_h = inttoptr i64 {} to i8*\n  %{}_buf = inttoptr i64 {} to i8*\n  %{}_written = alloca i32\n  call i32 @WriteFile(i8* %{}_h, i8* %{}_buf, i32 {}, i32* %{}_written, i8* null)\n  %{}_r = load i32, i32* %{}_written\n  %{} = zext i32 %{}_r to i64\n",
                    id, handle, id, buf, id, id, id, len, id, id, id, id, id
                ))
            }
        }
    }
    
    fn emit_file_close(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // FCL handle -> 0
        if args.is_empty() {
            return Err("FCL requires handle".to_string());
        }
        let handle = self.emit_arg(&args[0])?;
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!(
                    "  %{} = call i64 @sys_close(i64 {})\n",
                    id, handle
                ))
            }
            TargetPlatform::Windows => {
                Ok(format!(
                    "  %{}_h = inttoptr i64 {} to i8*\n  %{}_r = call i32 @CloseHandle(i8* %{}_h)\n  %{} = zext i32 %{}_r to i64\n",
                    id, handle, id, id, id, id
                ))
            }
        }
    }
    
    // ============================================================
    // DYNAMIC VECTORS (AI-native data structures)
    // Vector layout: [capacity: i64][length: i64][data: i64*]
    // ============================================================
    
    fn emit_vec_create(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // VEC capacity -> vec_ptr
        let cap = if args.is_empty() { "16".to_string() } else { self.emit_arg(&args[0])? };
        // Allocate: 16 bytes header + capacity * 8 bytes data
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!(
                    "  %{}_size = mul i64 {}, 8\n  %{}_total = add i64 %{}_size, 16\n  %{} = call i64 @sys_mmap(i64 0, i64 %{}_total, i64 3, i64 34, i64 -1, i64 0)\n  %{}_cap_ptr = inttoptr i64 %{} to i64*\n  store i64 {}, i64* %{}_cap_ptr\n  %{}_len_ptr = getelementptr i64, i64* %{}_cap_ptr, i32 1\n  store i64 0, i64* %{}_len_ptr\n",
                    id, cap, id, id, id, id, id, id, cap, id, id, id, id
                ))
            }
            TargetPlatform::Windows => {
                Ok(format!(
                    "  %{}_size = mul i64 {}, 8\n  %{}_total = add i64 %{}_size, 16\n  %{}_ptr = call i8* @VirtualAlloc(i8* null, i64 %{}_total, i32 12288, i32 4)\n  %{} = ptrtoint i8* %{}_ptr to i64\n  %{}_cap_ptr = inttoptr i64 %{} to i64*\n  store i64 {}, i64* %{}_cap_ptr\n  %{}_len_ptr = getelementptr i64, i64* %{}_cap_ptr, i32 1\n  store i64 0, i64* %{}_len_ptr\n",
                    id, cap, id, id, id, id, id, id, id, id, cap, id, id, id, id
                ))
            }
        }
    }
    
    fn emit_vec_push(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // VPH vec value -> new_length
        if args.len() < 2 {
            return Err("VPH requires vec, value".to_string());
        }
        let vec = self.emit_arg(&args[0])?;
        let value = self.emit_arg(&args[1])?;
        Ok(format!(
            "  %{}_base = inttoptr i64 {} to i64*\n  %{}_len_ptr = getelementptr i64, i64* %{}_base, i32 1\n  %{}_len = load i64, i64* %{}_len_ptr\n  %{}_data_base = getelementptr i64, i64* %{}_base, i32 2\n  %{}_slot = getelementptr i64, i64* %{}_data_base, i64 %{}_len\n  store i64 {}, i64* %{}_slot\n  %{}_newlen = add i64 %{}_len, 1\n  store i64 %{}_newlen, i64* %{}_len_ptr\n  %{} = add i64 0, %{}_newlen\n",
            id, vec, id, id, id, id, id, id, id, id, id, value, id, id, id, id, id, id, id
        ))
    }
    
    fn emit_vec_get(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // VGT vec index -> value
        if args.len() < 2 {
            return Err("VGT requires vec, index".to_string());
        }
        let vec = self.emit_arg(&args[0])?;
        let index = self.emit_arg(&args[1])?;
        Ok(format!(
            "  %{}_base = inttoptr i64 {} to i64*\n  %{}_data_base = getelementptr i64, i64* %{}_base, i32 2\n  %{}_slot = getelementptr i64, i64* %{}_data_base, i64 {}\n  %{} = load i64, i64* %{}_slot\n",
            id, vec, id, id, id, id, index, id, id
        ))
    }
    
    fn emit_vec_set(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // VST vec index value -> 0
        if args.len() < 3 {
            return Err("VST requires vec, index, value".to_string());
        }
        let vec = self.emit_arg(&args[0])?;
        let index = self.emit_arg(&args[1])?;
        let value = self.emit_arg(&args[2])?;
        Ok(format!(
            "  %{}_base = inttoptr i64 {} to i64*\n  %{}_data_base = getelementptr i64, i64* %{}_base, i32 2\n  %{}_slot = getelementptr i64, i64* %{}_data_base, i64 {}\n  store i64 {}, i64* %{}_slot\n  %{} = add i64 0, 0\n",
            id, vec, id, id, id, id, index, value, id, id
        ))
    }
    
    fn emit_vec_len(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // VLN vec -> length
        if args.is_empty() {
            return Err("VLN requires vec".to_string());
        }
        let vec = self.emit_arg(&args[0])?;
        Ok(format!(
            "  %{}_base = inttoptr i64 {} to i64*\n  %{}_len_ptr = getelementptr i64, i64* %{}_base, i32 1\n  %{} = load i64, i64* %{}_len_ptr\n",
            id, vec, id, id, id, id
        ))
    }
    
    fn emit_vec_cap(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // VCP vec -> capacity
        if args.is_empty() {
            return Err("VCP requires vec".to_string());
        }
        let vec = self.emit_arg(&args[0])?;
        Ok(format!(
            "  %{}_base = inttoptr i64 {} to i64*\n  %{} = load i64, i64* %{}_base\n",
            id, vec, id, id
        ))
    }
    
    // ============================================================
    // HASHMAP (Self-hosting - Symbol Tables)
    // Layout: [capacity: i64][count: i64][buckets: (hash, key_ptr, key_len, value)*]
    // Simple open addressing with linear probing
    // ============================================================
    
    fn emit_hashmap_create(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // HMP capacity -> hashmap_ptr
        let cap = if args.is_empty() { "64".to_string() } else { self.emit_arg(&args[0])? };
        // Each bucket: 32 bytes (hash:8 + key_ptr:8 + key_len:8 + value:8)
        // Header: 16 bytes (capacity:8 + count:8)
        match self.target {
            TargetPlatform::Linux => {
                Ok(format!(
                    "  %{}_bsize = mul i64 {}, 32\n  %{}_total = add i64 %{}_bsize, 16\n  %{} = call i64 @sys_mmap(i64 0, i64 %{}_total, i64 3, i64 34, i64 -1, i64 0)\n  %{}_cap_ptr = inttoptr i64 %{} to i64*\n  store i64 {}, i64* %{}_cap_ptr\n  %{}_cnt_ptr = getelementptr i64, i64* %{}_cap_ptr, i32 1\n  store i64 0, i64* %{}_cnt_ptr\n",
                    id, cap, id, id, id, id, id, id, cap, id, id, id, id
                ))
            }
            TargetPlatform::Windows => {
                Ok(format!(
                    "  %{}_bsize = mul i64 {}, 32\n  %{}_total = add i64 %{}_bsize, 16\n  %{}_ptr = call i8* @VirtualAlloc(i8* null, i64 %{}_total, i32 12288, i32 4)\n  %{} = ptrtoint i8* %{}_ptr to i64\n  %{}_cap_ptr = inttoptr i64 %{} to i64*\n  store i64 {}, i64* %{}_cap_ptr\n  %{}_cnt_ptr = getelementptr i64, i64* %{}_cap_ptr, i32 1\n  store i64 0, i64* %{}_cnt_ptr\n",
                    id, cap, id, id, id, id, id, id, id, id, cap, id, id, id, id
                ))
            }
        }
    }
    
    fn emit_hashmap_put(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // HPT map key_ptr key_len value -> 0
        // Simple implementation: append to linear list
        if args.len() < 4 {
            return Err("HPT requires map, key_ptr, key_len, value".to_string());
        }
        let map = self.emit_arg(&args[0])?;
        let key_ptr = self.emit_arg(&args[1])?;
        let key_len = self.emit_arg(&args[2])?;
        let value = self.emit_arg(&args[3])?;
        
        // Handle key_ptr that might be a getelementptr (string literal)
        let key_conv = if key_ptr.contains("getelementptr") {
            format!("  %{}_key = ptrtoint i8* {} to i64\n", id, key_ptr)
        } else {
            format!("  %{}_key = add i64 {}, 0\n", id, key_ptr)
        };
        
        // Simplified: just store at next slot (3 i64s per entry: key_ptr, key_len, value)
        Ok(format!(concat!(
            "  ; HPT: store key-value\n",
            "{key_conv}",
            "  %{id}_base = inttoptr i64 {map} to i64*\n",
            "  %{id}_cnt_ptr = getelementptr i64, i64* %{id}_base, i32 1\n",
            "  %{id}_cnt = load i64, i64* %{id}_cnt_ptr\n",
            "  %{id}_off = mul i64 %{id}_cnt, 3\n",
            "  %{id}_off2 = add i64 %{id}_off, 2\n",
            "  %{id}_slot = getelementptr i64, i64* %{id}_base, i64 %{id}_off2\n",
            "  store i64 %{id}_key, i64* %{id}_slot\n",
            "  %{id}_s1 = getelementptr i64, i64* %{id}_slot, i32 1\n",
            "  store i64 {key_len}, i64* %{id}_s1\n",
            "  %{id}_s2 = getelementptr i64, i64* %{id}_slot, i32 2\n",
            "  store i64 {value}, i64* %{id}_s2\n",
            "  %{id}_newcnt = add i64 %{id}_cnt, 1\n",
            "  store i64 %{id}_newcnt, i64* %{id}_cnt_ptr\n",
            "  %{id} = add i64 0, 0\n"),
            id=id, map=map, key_conv=key_conv, key_len=key_len, value=value
        ))
    }
    
    fn emit_hashmap_get(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // HGT map key_ptr key_len -> value (linear search)
        if args.len() < 3 {
            return Err("HGT requires map, key_ptr, key_len".to_string());
        }
        let map = self.emit_arg(&args[0])?;
        let key_ptr = self.emit_arg(&args[1])?;
        let key_len = self.emit_arg(&args[2])?;
        
        // Linear search with memcmp
        Ok(format!(concat!(
            "  ; HGT: linear search lookup\n",
            "  %{id}_base = inttoptr i64 {map} to i64*\n",
            "  %{id}_cnt_ptr = getelementptr i64, i64* %{id}_base, i32 1\n",
            "  %{id}_cnt = load i64, i64* %{id}_cnt_ptr\n",
            "  %{id}_key = inttoptr i64 {key_ptr} to i8*\n",
            "  br label %{id}_loop\n",
            "{id}_loop:\n",
            "  %{id}_i = phi i64 [0, %entry], [%{id}_i_next, %{id}_next]\n",
            "  %{id}_done = icmp uge i64 %{id}_i, %{id}_cnt\n",
            "  br i1 %{id}_done, label %{id}_notfound, label %{id}_check\n",
            "{id}_check:\n",
            "  %{id}_off = mul i64 %{id}_i, 4\n",
            "  %{id}_off2 = add i64 %{id}_off, 2\n",
            "  %{id}_slot = getelementptr i64, i64* %{id}_base, i64 %{id}_off2\n",
            "  %{id}_stored_ptr = load i64, i64* %{id}_slot\n",
            "  %{id}_s1 = getelementptr i64, i64* %{id}_slot, i32 1\n",
            "  %{id}_stored_len = load i64, i64* %{id}_s1\n",
            "  %{id}_len_eq = icmp eq i64 %{id}_stored_len, {key_len}\n",
            "  br i1 %{id}_len_eq, label %{id}_cmp, label %{id}_next\n",
            "{id}_cmp:\n",
            "  %{id}_sp = inttoptr i64 %{id}_stored_ptr to i8*\n",
            "  %{id}_cmp_r = call i32 @memcmp(i8* %{id}_sp, i8* %{id}_key, i64 {key_len})\n",
            "  %{id}_eq = icmp eq i32 %{id}_cmp_r, 0\n",
            "  br i1 %{id}_eq, label %{id}_found, label %{id}_next\n",
            "{id}_next:\n",
            "  %{id}_i_next = add i64 %{id}_i, 1\n",
            "  br label %{id}_loop\n",
            "{id}_found:\n",
            "  %{id}_s2 = getelementptr i64, i64* %{id}_slot, i32 2\n",
            "  %{id}_val = load i64, i64* %{id}_s2\n",
            "  br label %{id}_end\n",
            "{id}_notfound:\n",
            "  br label %{id}_end\n",
            "{id}_end:\n",
            "  %{id} = phi i64 [%{id}_val, %{id}_found], [0, %{id}_notfound]\n"),
            id=id, map=map, key_ptr=key_ptr, key_len=key_len
        ))
    }
    
    fn emit_hashmap_has(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // HHS map key_ptr key_len -> 1 if exists, 0 if not
        if args.len() < 3 {
            return Err("HHS requires map, key_ptr, key_len".to_string());
        }
        let map = self.emit_arg(&args[0])?;
        
        // Simplified placeholder
        Ok(format!(
            "  ; HHS: check exists (simplified)\n  %{}_base = inttoptr i64 {} to i64*\n  %{} = add i64 0, 0\n",
            id, map, id
        ))
    }
    
    // ============================================================
    // STRING OPERATIONS (Self-hosting - Code Generation)
    // ============================================================
    
    fn emit_str_cat(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // CAT dst src1 src2 -> total_len
        if args.len() < 3 {
            return Err("CAT requires dst, src1, src2".to_string());
        }
        let dst = self.emit_arg(&args[0])?;
        let src1 = self.emit_arg(&args[1])?;
        let src2 = self.emit_arg(&args[2])?;
        
        // Handle both pointer (getelementptr) and i64 arguments
        let s1_conv = if src1.contains("getelementptr") {
            format!("  %{}_s1 = bitcast i8* {} to i8*\n", id, src1)
        } else {
            format!("  %{}_s1 = inttoptr i64 {} to i8*\n", id, src1)
        };
        let s2_conv = if src2.contains("getelementptr") {
            format!("  %{}_s2 = bitcast i8* {} to i8*\n", id, src2)
        } else {
            format!("  %{}_s2 = inttoptr i64 {} to i8*\n", id, src2)
        };
        
        Ok(format!(
            "  ; CAT: concatenate strings\n  %{}_dst = inttoptr i64 {} to i8*\n{}{}  %{}_len1 = call i64 @strlen(i8* %{}_s1)\n  %{}_len2 = call i64 @strlen(i8* %{}_s2)\n  call void @llvm.memcpy.p0i8.p0i8.i64(i8* %{}_dst, i8* %{}_s1, i64 %{}_len1, i1 false)\n  %{}_dst2 = getelementptr i8, i8* %{}_dst, i64 %{}_len1\n  call void @llvm.memcpy.p0i8.p0i8.i64(i8* %{}_dst2, i8* %{}_s2, i64 %{}_len2, i1 false)\n  %{} = add i64 %{}_len1, %{}_len2\n",
            id, dst, s1_conv, s2_conv, id, id, id, id, id, id, id, id, id, id, id, id, id, id, id, id
        ))
    }
    
    fn emit_int_to_str(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // ITS num buf -> length
        // Pure implementation without libc
        if args.len() < 2 {
            return Err("ITS requires num, buf".to_string());
        }
        let num = self.emit_arg(&args[0])?;
        let buf = self.emit_arg(&args[1])?;
        
        // Simple single-digit for now (0-9), handles negative
        // For full numbers, would need inline loop
        Ok(format!(concat!(
            "  ; ITS: int to string (single digit)\n",
            "  %{id}_buf = inttoptr i64 {buf} to i8*\n",
            "  %{id}_is_neg = icmp slt i64 {num}, 0\n",
            "  %{id}_abs = select i1 %{id}_is_neg, i64 0, i64 {num}\n",
            "  %{id}_mod = urem i64 %{id}_abs, 10\n",
            "  %{id}_digit = add i64 %{id}_mod, 48\n",
            "  %{id}_char = trunc i64 %{id}_digit to i8\n",
            "  store i8 %{id}_char, i8* %{id}_buf\n",
            "  %{id}_next = getelementptr i8, i8* %{id}_buf, i32 1\n",
            "  store i8 0, i8* %{id}_next\n",
            "  %{id} = add i64 0, 1\n"),
            id=id, buf=buf, num=num
        ))
    }
    
    fn emit_char_at(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // CHR str index -> char_code
        if args.len() < 2 {
            return Err("CHR requires str, index".to_string());
        }
        let str_ptr = self.emit_arg(&args[0])?;
        let index = self.emit_arg(&args[1])?;
        
        // Handle both pointer (getelementptr) and i64 arguments
        let str_conv = if str_ptr.contains("getelementptr") {
            format!("  %{}_str = bitcast i8* {} to i8*\n", id, str_ptr)
        } else {
            format!("  %{}_str = inttoptr i64 {} to i8*\n", id, str_ptr)
        };
        
        Ok(format!(
            "{}  %{}_ptr = getelementptr i8, i8* %{}_str, i64 {}\n  %{}_char = load i8, i8* %{}_ptr\n  %{} = zext i8 %{}_char to i64\n",
            str_conv, id, id, index, id, id, id, id
        ))
    }
    
    fn emit_substring(&self, id: &str, args: &[Arg]) -> Result<String, String> {
        // SBS dst src start len -> len
        if args.len() < 4 {
            return Err("SBS requires dst, src, start, len".to_string());
        }
        let dst = self.emit_arg(&args[0])?;
        let src = self.emit_arg(&args[1])?;
        let start = self.emit_arg(&args[2])?;
        let len = self.emit_arg(&args[3])?;
        
        // Handle both pointer (getelementptr) and i64 arguments
        let src_conv = if src.contains("getelementptr") {
            format!("  %{}_src = bitcast i8* {} to i8*\n", id, src)
        } else {
            format!("  %{}_src = inttoptr i64 {} to i8*\n", id, src)
        };
        
        Ok(format!(
            "  %{}_dst = inttoptr i64 {} to i8*\n{}  %{}_srcoff = getelementptr i8, i8* %{}_src, i64 {}\n  call void @llvm.memcpy.p0i8.p0i8.i64(i8* %{}_dst, i8* %{}_srcoff, i64 {}, i1 false)\n  %{}_term = getelementptr i8, i8* %{}_dst, i64 {}\n  store i8 0, i8* %{}_term\n  %{} = add i64 0, {}\n",
            id, dst, src_conv, id, id, start, id, id, len, id, id, len, id, id, len
        ))
    }
}
