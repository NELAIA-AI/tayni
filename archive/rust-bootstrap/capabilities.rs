//! TAYNI Capability System (SCN)
//! Resolves high-level capability operations to platform-specific code
//! 
//! Principle: Programs declare WHAT they need, compiler decides HOW

use crate::ir::{Capability, Op, Graph, Node, Arg, Value};
use std::collections::HashSet;

/// Capability resolver - transforms high-level ops to low-level implementations
pub struct CapabilityResolver {
    pub platform: Platform,
    pub required: HashSet<Capability>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Windows,
    Linux,
}

impl CapabilityResolver {
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            required: HashSet::new(),
        }
    }
    
    /// Extract required capabilities from graph
    pub fn analyze_requirements(&mut self, graph: &Graph) {
        self.required = graph.requirements.capabilities.clone();
    }
    
    /// Check if a capability is available
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.required.contains(cap)
    }
    
    /// Generate LLVM IR for HTTP.LISTEN
    pub fn emit_http_listen(&self, port_ref: &str) -> String {
        match self.platform {
            Platform::Windows => self.emit_http_listen_windows(port_ref),
            Platform::Linux => self.emit_http_listen_linux(port_ref),
        }
    }
    
    fn emit_http_listen_windows(&self, port_ref: &str) -> String {
        // Windows: WSAStartup + socket + bind + listen
        format!(r#"
  ; HTTP.LISTEN - Windows Winsock
  %wsa_data = alloca [408 x i8], align 8
  %wsa_ptr = bitcast [408 x i8]* %wsa_data to i8*
  call i32 @WSAStartup(i32 514, i8* %wsa_ptr)
  
  %http_sock = call i64 @socket(i32 2, i32 1, i32 6)
  
  %addr = alloca [16 x i8], align 8
  %addr_ptr = bitcast [16 x i8]* %addr to i8*
  call void @llvm.memset.p0i8.i64(i8* %addr_ptr, i8 0, i64 16, i1 false)
  %family_ptr = getelementptr [16 x i8], [16 x i8]* %addr, i64 0, i64 0
  store i8 2, i8* %family_ptr
  %port_hi = lshr i64 %{port_ref}, 8
  %port_lo = and i64 %{port_ref}, 255
  %port_be = or i64 %port_lo, %port_hi
  %port_ptr = getelementptr [16 x i8], [16 x i8]* %addr, i64 0, i64 2
  %port_i8 = bitcast i8* %port_ptr to i16*
  store i16 %port_be, i16* %port_i8
  
  call i32 @bind(i64 %http_sock, i8* %addr_ptr, i32 16)
  call i32 @listen(i64 %http_sock, i32 128)
"#, port_ref = port_ref)
    }
    
    fn emit_http_listen_linux(&self, port_ref: &str) -> String {
        // Linux: socket + bind + listen (no WSAStartup)
        format!(r#"
  ; HTTP.LISTEN - Linux sockets
  %http_sock = call i32 @socket(i32 2, i32 1, i32 0)
  
  %addr = alloca [16 x i8], align 8
  %addr_ptr = bitcast [16 x i8]* %addr to i8*
  call void @llvm.memset.p0i8.i64(i8* %addr_ptr, i8 0, i64 16, i1 false)
  %family_ptr = getelementptr [16 x i8], [16 x i8]* %addr, i64 0, i64 0
  store i8 2, i8* %family_ptr
  %port_ptr = getelementptr [16 x i8], [16 x i8]* %addr, i64 0, i64 2
  %port_i8 = bitcast i8* %port_ptr to i16*
  %port_be = call i16 @htons(i16 %{port_ref})
  store i16 %port_be, i16* %port_i8
  
  call i32 @bind(i32 %http_sock, i8* %addr_ptr, i32 16)
  call i32 @listen(i32 %http_sock, i32 128)
"#, port_ref = port_ref)
    }
    
    /// Generate LLVM IR for HTTP.ACCEPT
    pub fn emit_http_accept(&self, server_ref: &str) -> String {
        match self.platform {
            Platform::Windows => format!(r#"
  ; HTTP.ACCEPT - Windows
  %client_sock = call i64 @accept(i64 %{server}, i8* null, i32* null)
"#, server = server_ref),
            Platform::Linux => format!(r#"
  ; HTTP.ACCEPT - Linux
  %client_sock = call i32 @accept(i32 %{server}, i8* null, i32* null)
"#, server = server_ref),
        }
    }
    
    /// Generate LLVM IR for HTTP.RESPOND
    pub fn emit_http_respond(&self, client_ref: &str, status: i64, body_ref: &str) -> String {
        let status_text = match status {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "OK",
        };
        
        match self.platform {
            Platform::Windows => format!(r#"
  ; HTTP.RESPOND - Windows
  %resp_hdr = alloca [128 x i8], align 8
  %resp_ptr = bitcast [128 x i8]* %resp_hdr to i8*
  call i32 @send(i64 %{client}, i8* %{body}, i32 %body_len, i32 0)
  call i32 @closesocket(i64 %{client})
"#, client = client_ref, body = body_ref),
            Platform::Linux => format!(r#"
  ; HTTP.RESPOND - Linux
  call i64 @send(i32 %{client}, i8* %{body}, i64 %body_len, i32 0)
  call i32 @close(i32 %{client})
"#, client = client_ref, body = body_ref),
        }
    }
    
    /// Generate LLVM IR for SQL.CONNECT (ODBC)
    pub fn emit_sql_connect(&self, conn_str_ref: &str) -> String {
        // ODBC is cross-platform
        format!(r#"
  ; SQL.CONNECT via ODBC
  %sql_env = alloca i8*, align 8
  %sql_conn = alloca i8*, align 8
  
  call i16 @SQLAllocHandle(i16 1, i8* null, i8** %sql_env)
  %env_ptr = load i8*, i8** %sql_env
  call i16 @SQLSetEnvAttr(i8* %env_ptr, i32 200, i8* inttoptr (i64 3 to i8*), i32 0)
  call i16 @SQLAllocHandle(i16 2, i8* %env_ptr, i8** %sql_conn)
  %conn_ptr = load i8*, i8** %sql_conn
  
  call i16 @SQLDriverConnect(i8* %conn_ptr, i8* null, i8* %{conn_str}, i16 -3, i8* null, i16 0, i16* null, i16 0)
"#, conn_str = conn_str_ref)
    }
    
    /// Generate LLVM IR for SQL.QUERY
    pub fn emit_sql_query(&self, conn_ref: &str, query_ref: &str) -> String {
        format!(r#"
  ; SQL.QUERY
  %sql_stmt = alloca i8*, align 8
  call i16 @SQLAllocHandle(i16 3, i8* %{conn}, i8** %sql_stmt)
  %stmt_ptr = load i8*, i8** %sql_stmt
  call i16 @SQLExecDirect(i8* %stmt_ptr, i8* %{query}, i32 -3)
"#, conn = conn_ref, query = query_ref)
    }
    
    /// Generate LLVM IR for SQL.CLOSE
    pub fn emit_sql_close(&self, conn_ref: &str) -> String {
        format!(r#"
  ; SQL.CLOSE
  call i16 @SQLDisconnect(i8* %{conn})
  call i16 @SQLFreeHandle(i16 2, i8* %{conn})
"#, conn = conn_ref)
    }
    
    /// Generate LLVM IR for JSON.PARSE (simple implementation)
    pub fn emit_json_parse(&self, input_ref: &str) -> String {
        // Simple JSON parser - returns pointer to parsed structure
        format!(r#"
  ; JSON.PARSE - allocate and parse
  %json_obj = call i8* @malloc(i64 4096)
  call void @TAYNI_json_parse(i8* %{input}, i8* %json_obj)
"#, input = input_ref)
    }
    
    /// Generate LLVM IR for JSON.ENCODE
    pub fn emit_json_encode(&self, obj_ref: &str) -> String {
        format!(r#"
  ; JSON.ENCODE
  %json_out = call i8* @malloc(i64 4096)
  call i64 @TAYNI_json_encode(i8* %{obj}, i8* %json_out)
"#, obj = obj_ref)
    }
    
    /// Generate external declarations for capabilities
    pub fn emit_declarations(&self) -> String {
        let mut decls = String::new();
        
        if self.required.contains(&Capability::HttpServer) || 
           self.required.contains(&Capability::HttpClient) {
            decls.push_str(&self.emit_http_declarations());
        }
        
        if self.required.contains(&Capability::Sql) || 
           self.required.contains(&Capability::SqlReadOnly) {
            decls.push_str(&self.emit_sql_declarations());
        }
        
        if self.required.contains(&Capability::Json) {
            decls.push_str(&self.emit_json_declarations());
        }
        
        decls
    }
    
    fn emit_http_declarations(&self) -> String {
        match self.platform {
            Platform::Windows => r#"
; Windows Winsock declarations
declare i32 @WSAStartup(i32, i8*)
declare i32 @WSACleanup()
declare i64 @socket(i32, i32, i32)
declare i32 @bind(i64, i8*, i32)
declare i32 @listen(i64, i32)
declare i64 @accept(i64, i8*, i32*)
declare i32 @send(i64, i8*, i32, i32)
declare i32 @recv(i64, i8*, i32, i32)
declare i32 @closesocket(i64)
"#.to_string(),
            Platform::Linux => r#"
; Linux socket declarations
declare i32 @socket(i32, i32, i32)
declare i32 @bind(i32, i8*, i32)
declare i32 @listen(i32, i32)
declare i32 @accept(i32, i8*, i32*)
declare i64 @send(i32, i8*, i64, i32)
declare i64 @recv(i32, i8*, i64, i32)
declare i32 @close(i32)
declare i16 @htons(i16)
"#.to_string(),
        }
    }
    
    fn emit_sql_declarations(&self) -> String {
        // ODBC declarations (cross-platform)
        r#"
; ODBC declarations
declare i16 @SQLAllocHandle(i16, i8*, i8**)
declare i16 @SQLFreeHandle(i16, i8*)
declare i16 @SQLSetEnvAttr(i8*, i32, i8*, i32)
declare i16 @SQLDriverConnect(i8*, i8*, i8*, i16, i8*, i16, i16*, i16)
declare i16 @SQLExecDirect(i8*, i8*, i32)
declare i16 @SQLFetch(i8*)
declare i16 @SQLGetData(i8*, i16, i16, i8*, i64, i64*)
declare i16 @SQLDisconnect(i8*)
"#.to_string()
    }
    
    fn emit_json_declarations(&self) -> String {
        r#"
; JSON helper declarations (TAYNI runtime)
declare void @TAYNI_json_parse(i8*, i8*)
declare i64 @TAYNI_json_encode(i8*, i8*)
declare i8* @TAYNI_json_get(i8*, i8*)
declare void @TAYNI_json_set(i8*, i8*, i8*)
"#.to_string()
    }
}

/// Verify that required capabilities are satisfied
pub fn verify_capabilities(graph: &Graph) -> Result<(), String> {
    let required = &graph.requirements.capabilities;
    
    // Check for conflicting capabilities
    if required.contains(&Capability::Sql) && required.contains(&Capability::SqlReadOnly) {
        return Err("Cannot require both 'sql' and 'sql:readonly'".to_string());
    }
    
    if required.contains(&Capability::FileSystem) && required.contains(&Capability::FileSystemReadOnly) {
        return Err("Cannot require both 'filesystem' and 'filesystem:readonly'".to_string());
    }
    
    Ok(())
}
