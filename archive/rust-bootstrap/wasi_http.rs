//! WASI HTTP (wasi-http) Implementation for TAYNI
//!
//! Implements the wasi-http proposal for serverless HTTP handling.
//! This enables TAYNI programs to run on Cloudflare Workers, Deno Deploy,
//! Fermyon Spin, and other WASI-compatible serverless platforms.
//!
//! Spec: https://github.com/WebAssembly/wasi-http

use crate::target::format::wasm::{
    encode_uleb128, encode_sleb128, encode_string, encode_section,
    WASM_MAGIC, WASM_VERSION,
    SECTION_TYPE, SECTION_IMPORT, SECTION_FUNCTION, SECTION_MEMORY,
    SECTION_EXPORT, SECTION_CODE, SECTION_DATA,
    TYPE_I32, TYPE_I64, TYPE_FUNC,
    OP_END, OP_CALL, OP_LOCAL_GET, OP_LOCAL_SET,
    OP_I32_CONST, OP_I64_CONST,
};

// ============================================================================
// WASI HTTP Interface Names (WIT-based)
// ============================================================================

pub mod wit {
    // wasi-http types
    pub const HTTP_TYPES: &str = "wasi:http/types@0.2.0";
    
    // Incoming handler (for servers)
    pub const HTTP_INCOMING_HANDLER: &str = "wasi:http/incoming-handler@0.2.0";
    
    // Outgoing handler (for clients)
    pub const HTTP_OUTGOING_HANDLER: &str = "wasi:http/outgoing-handler@0.2.0";
}

// ============================================================================
// HTTP Types
// ============================================================================

/// HTTP Method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get = 0,
    Head = 1,
    Post = 2,
    Put = 3,
    Delete = 4,
    Connect = 5,
    Options = 6,
    Trace = 7,
    Patch = 8,
}

impl Method {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Method::Get),
            1 => Some(Method::Head),
            2 => Some(Method::Post),
            3 => Some(Method::Put),
            4 => Some(Method::Delete),
            5 => Some(Method::Connect),
            6 => Some(Method::Options),
            7 => Some(Method::Trace),
            8 => Some(Method::Patch),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
            Method::Patch => "PATCH",
        }
    }
}

/// HTTP Scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http = 0,
    Https = 1,
    Other = 2,
}

/// HTTP Status Code categories
#[derive(Debug, Clone, Copy)]
pub enum StatusClass {
    Informational, // 1xx
    Success,       // 2xx
    Redirection,   // 3xx
    ClientError,   // 4xx
    ServerError,   // 5xx
}

impl StatusClass {
    pub fn from_code(code: u16) -> Self {
        match code {
            100..=199 => StatusClass::Informational,
            200..=299 => StatusClass::Success,
            300..=399 => StatusClass::Redirection,
            400..=499 => StatusClass::ClientError,
            _ => StatusClass::ServerError,
        }
    }
}

/// HTTP Header
#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: Vec<u8>,
}

impl Header {
    pub fn new(name: &str, value: &str) -> Self {
        Header {
            name: name.to_string(),
            value: value.as_bytes().to_vec(),
        }
    }
    
    pub fn value_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.value).ok()
    }
}

/// HTTP Headers collection
#[derive(Debug, Clone, Default)]
pub struct Headers {
    pub entries: Vec<Header>,
}

impl Headers {
    pub fn new() -> Self {
        Headers { entries: Vec::new() }
    }
    
    pub fn add(&mut self, name: &str, value: &str) {
        self.entries.push(Header::new(name, value));
    }
    
    pub fn get(&self, name: &str) -> Option<&Header> {
        let name_lower = name.to_lowercase();
        self.entries.iter().find(|h| h.name.to_lowercase() == name_lower)
    }
    
    pub fn content_type(&self) -> Option<&str> {
        self.get("content-type").and_then(|h| h.value_str())
    }
    
    pub fn content_length(&self) -> Option<usize> {
        self.get("content-length")
            .and_then(|h| h.value_str())
            .and_then(|s| s.parse().ok())
    }
}

/// Incoming HTTP Request (for handlers)
#[derive(Debug, Clone)]
pub struct IncomingRequest {
    pub method: Method,
    pub scheme: Option<Scheme>,
    pub authority: Option<String>,
    pub path_with_query: String,
    pub headers: Headers,
}

impl IncomingRequest {
    pub fn path(&self) -> &str {
        self.path_with_query.split('?').next().unwrap_or("/")
    }
    
    pub fn query(&self) -> Option<&str> {
        self.path_with_query.split('?').nth(1)
    }
    
    pub fn query_param(&self, key: &str) -> Option<String> {
        self.query().and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with(&format!("{}=", key)))
                .map(|p| p.split('=').nth(1).unwrap_or("").to_string())
        })
    }
}

/// Outgoing HTTP Response (from handlers)
#[derive(Debug, Clone)]
pub struct OutgoingResponse {
    pub status: u16,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
}

impl OutgoingResponse {
    pub fn new(status: u16) -> Self {
        OutgoingResponse {
            status,
            headers: Headers::new(),
            body: None,
        }
    }
    
    pub fn ok() -> Self {
        Self::new(200)
    }
    
    pub fn not_found() -> Self {
        Self::new(404)
    }
    
    pub fn bad_request() -> Self {
        Self::new(400)
    }
    
    pub fn internal_error() -> Self {
        Self::new(500)
    }
    
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.add(name, value);
        self
    }
    
    pub fn body(mut self, data: Vec<u8>) -> Self {
        self.body = Some(data);
        self
    }
    
    pub fn text(self, text: &str) -> Self {
        self.header("Content-Type", "text/plain; charset=utf-8")
            .body(text.as_bytes().to_vec())
    }
    
    pub fn html(self, html: &str) -> Self {
        self.header("Content-Type", "text/html; charset=utf-8")
            .body(html.as_bytes().to_vec())
    }
    
    pub fn json(self, json: &str) -> Self {
        self.header("Content-Type", "application/json")
            .body(json.as_bytes().to_vec())
    }
}

/// Outgoing HTTP Request (for clients)
#[derive(Debug, Clone)]
pub struct OutgoingRequest {
    pub method: Method,
    pub scheme: Scheme,
    pub authority: String,
    pub path_with_query: String,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
}

impl OutgoingRequest {
    pub fn get(url: &str) -> Result<Self, String> {
        Self::new(Method::Get, url)
    }
    
    pub fn post(url: &str) -> Result<Self, String> {
        Self::new(Method::Post, url)
    }
    
    pub fn new(method: Method, url: &str) -> Result<Self, String> {
        // Parse URL
        let (scheme, rest) = if url.starts_with("https://") {
            (Scheme::Https, &url[8..])
        } else if url.starts_with("http://") {
            (Scheme::Http, &url[7..])
        } else {
            return Err("Invalid URL scheme".to_string());
        };
        
        let (authority, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };
        
        Ok(OutgoingRequest {
            method,
            scheme,
            authority: authority.to_string(),
            path_with_query: path.to_string(),
            headers: Headers::new(),
            body: None,
        })
    }
    
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.add(name, value);
        self
    }
    
    pub fn body(mut self, data: Vec<u8>) -> Self {
        self.body = Some(data);
        self
    }
}

/// Incoming HTTP Response (from clients)
#[derive(Debug, Clone)]
pub struct IncomingResponse {
    pub status: u16,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
}

impl IncomingResponse {
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
    
    pub fn body_text(&self) -> Option<String> {
        self.body.as_ref().and_then(|b| String::from_utf8(b.clone()).ok())
    }
}

// ============================================================================
// WASI HTTP Error Codes
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpErrorCode {
    Success = 0,
    InvalidUrl = 1,
    InvalidMethod = 2,
    InvalidHeader = 3,
    InvalidBody = 4,
    Timeout = 5,
    ConnectionRefused = 6,
    ConnectionReset = 7,
    NetworkError = 8,
    TlsError = 9,
    ProtocolError = 10,
    InternalError = 11,
}

// ============================================================================
// WASI HTTP Type Section Generation
// ============================================================================

fn generate_http_type_section() -> Vec<u8> {
    let mut types = Vec::new();
    
    // 8 types
    types.extend(encode_uleb128(8));
    
    // Type 0: handle (incoming-request) -> (outgoing-response)
    // Simplified as (i32) -> (i32) for resource handles
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32); // incoming-request handle
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32); // outgoing-response handle
    
    // Type 1: method (request) -> i32
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 2: path-with-query (request) -> (ptr, len)
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(2));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    
    // Type 3: headers (request) -> headers-handle
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 4: body (request) -> body-handle
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 5: new-response (status) -> response-handle
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    types.extend(encode_uleb128(1));
    types.push(TYPE_I32);
    
    // Type 6: set-body (response, body-ptr, body-len) -> ()
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(3));
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.push(TYPE_I32);
    types.extend(encode_uleb128(0));
    
    // Type 7: () -> () for _start
    types.push(TYPE_FUNC);
    types.extend(encode_uleb128(0));
    types.extend(encode_uleb128(0));
    
    types
}

fn generate_http_import_section() -> Vec<u8> {
    let mut imports = Vec::new();
    
    // 6 imports
    imports.extend(encode_uleb128(6));
    
    // Import 0: method
    imports.extend(encode_string(wit::HTTP_TYPES));
    imports.extend(encode_string("[method]incoming-request.method"));
    imports.push(0x00);
    imports.extend(encode_uleb128(1));
    
    // Import 1: path-with-query
    imports.extend(encode_string(wit::HTTP_TYPES));
    imports.extend(encode_string("[method]incoming-request.path-with-query"));
    imports.push(0x00);
    imports.extend(encode_uleb128(2));
    
    // Import 2: headers
    imports.extend(encode_string(wit::HTTP_TYPES));
    imports.extend(encode_string("[method]incoming-request.headers"));
    imports.push(0x00);
    imports.extend(encode_uleb128(3));
    
    // Import 3: consume (body)
    imports.extend(encode_string(wit::HTTP_TYPES));
    imports.extend(encode_string("[method]incoming-request.consume"));
    imports.push(0x00);
    imports.extend(encode_uleb128(4));
    
    // Import 4: new outgoing-response
    imports.extend(encode_string(wit::HTTP_TYPES));
    imports.extend(encode_string("[constructor]outgoing-response"));
    imports.push(0x00);
    imports.extend(encode_uleb128(5));
    
    // Import 5: set-body
    imports.extend(encode_string(wit::HTTP_TYPES));
    imports.extend(encode_string("[method]outgoing-response.set-body"));
    imports.push(0x00);
    imports.extend(encode_uleb128(6));
    
    imports
}

// ============================================================================
// HTTP Handler Generation
// ============================================================================

/// Route definition for HTTP handler
#[derive(Debug, Clone)]
pub struct Route {
    pub method: Option<Method>,
    pub path: String,
    pub response_status: u16,
    pub response_body: Vec<u8>,
    pub content_type: String,
}

impl Route {
    pub fn get(path: &str) -> RouteBuilder {
        RouteBuilder::new(Some(Method::Get), path)
    }
    
    pub fn post(path: &str) -> RouteBuilder {
        RouteBuilder::new(Some(Method::Post), path)
    }
    
    pub fn any(path: &str) -> RouteBuilder {
        RouteBuilder::new(None, path)
    }
}

pub struct RouteBuilder {
    method: Option<Method>,
    path: String,
    status: u16,
    body: Vec<u8>,
    content_type: String,
}

impl RouteBuilder {
    fn new(method: Option<Method>, path: &str) -> Self {
        RouteBuilder {
            method,
            path: path.to_string(),
            status: 200,
            body: Vec::new(),
            content_type: "text/plain".to_string(),
        }
    }
    
    pub fn status(mut self, code: u16) -> Self {
        self.status = code;
        self
    }
    
    pub fn text(mut self, text: &str) -> Self {
        self.body = text.as_bytes().to_vec();
        self.content_type = "text/plain; charset=utf-8".to_string();
        self
    }
    
    pub fn json(mut self, json: &str) -> Self {
        self.body = json.as_bytes().to_vec();
        self.content_type = "application/json".to_string();
        self
    }
    
    pub fn html(mut self, html: &str) -> Self {
        self.body = html.as_bytes().to_vec();
        self.content_type = "text/html; charset=utf-8".to_string();
        self
    }
    
    pub fn build(self) -> Route {
        Route {
            method: self.method,
            path: self.path,
            response_status: self.status,
            response_body: self.body,
            content_type: self.content_type,
        }
    }
}

/// Generate a WASI-http handler module
pub fn generate_wasi_http_handler(routes: &[Route], not_found_body: &str) -> Vec<u8> {
    let mut wasm = Vec::new();
    
    // Magic and version
    wasm.extend(&WASM_MAGIC);
    wasm.extend(&WASM_VERSION);
    
    // Type section
    let types = generate_http_type_section();
    wasm.extend(encode_section(SECTION_TYPE, &types));
    
    // Import section
    let imports = generate_http_import_section();
    wasm.extend(encode_section(SECTION_IMPORT, &imports));
    
    // Function section - declare handle function
    let mut funcs = Vec::new();
    funcs.extend(encode_uleb128(1));
    funcs.extend(encode_uleb128(0)); // type 0: handle
    wasm.extend(encode_section(SECTION_FUNCTION, &funcs));
    
    // Memory section
    let mut mem = Vec::new();
    mem.extend(encode_uleb128(1));
    mem.push(0x00);
    mem.extend(encode_uleb128(1)); // 1 page
    wasm.extend(encode_section(SECTION_MEMORY, &mem));
    
    // Export section
    let mut exports = Vec::new();
    exports.extend(encode_uleb128(2));
    
    // Export handle function
    exports.extend(encode_string("wasi:http/incoming-handler@0.2.0#handle"));
    exports.push(0x00);
    exports.extend(encode_uleb128(6)); // func index after 6 imports
    
    // Export memory
    exports.extend(encode_string("memory"));
    exports.push(0x02);
    exports.extend(encode_uleb128(0));
    
    wasm.extend(encode_section(SECTION_EXPORT, &exports));
    
    // Code section
    let mut code = Vec::new();
    code.extend(encode_uleb128(1));
    
    // Handler function body
    let mut body = Vec::new();
    
    // Locals: path_ptr, path_len, response_handle
    body.extend(encode_uleb128(1));
    body.extend(encode_uleb128(3));
    body.push(TYPE_I32);
    
    // Get path from request
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(0)); // request handle (param 0)
    body.push(OP_CALL);
    body.extend(encode_uleb128(1)); // path-with-query
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(2)); // path_len
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(1)); // path_ptr
    
    // Create response with status 200
    body.push(OP_I32_CONST);
    body.extend(encode_sleb128(200));
    body.push(OP_CALL);
    body.extend(encode_uleb128(4)); // new outgoing-response
    body.push(OP_LOCAL_SET);
    body.extend(encode_uleb128(3)); // response_handle
    
    // Return response handle
    body.push(OP_LOCAL_GET);
    body.extend(encode_uleb128(3));
    
    body.push(OP_END);
    
    code.extend(encode_uleb128(body.len() as u64));
    code.extend(body);
    wasm.extend(encode_section(SECTION_CODE, &code));
    
    // Data section - route data
    let mut data = Vec::new();
    let mut data_segments = Vec::new();
    let mut offset = 0usize;
    
    // Add route paths and bodies
    for route in routes {
        data_segments.push((offset, route.path.as_bytes().to_vec()));
        offset += route.path.len();
        data_segments.push((offset, route.response_body.clone()));
        offset += route.response_body.len();
    }
    
    // Add 404 body
    data_segments.push((offset, not_found_body.as_bytes().to_vec()));
    
    if !data_segments.is_empty() {
        data.extend(encode_uleb128(data_segments.len() as u64));
        
        for (off, bytes) in &data_segments {
            data.push(0x00);
            data.push(OP_I32_CONST);
            data.extend(encode_sleb128(*off as i64));
            data.push(OP_END);
            data.extend(encode_uleb128(bytes.len() as u64));
            data.extend(bytes);
        }
        
        wasm.extend(encode_section(SECTION_DATA, &data));
    }
    
    wasm
}

/// Generate a simple JSON API handler
pub fn generate_json_api_handler(endpoints: &[(&str, &str)]) -> Vec<u8> {
    let routes: Vec<Route> = endpoints.iter()
        .map(|(path, json)| Route::get(path).json(json).build())
        .collect();
    
    generate_wasi_http_handler(&routes, r#"{"error":"Not Found"}"#)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_method() {
        assert_eq!(Method::Get.as_str(), "GET");
        assert_eq!(Method::from_u8(2), Some(Method::Post));
    }
    
    #[test]
    fn test_headers() {
        let mut headers = Headers::new();
        headers.add("Content-Type", "application/json");
        headers.add("X-Custom", "value");
        
        assert_eq!(headers.content_type(), Some("application/json"));
        assert!(headers.get("x-custom").is_some()); // case insensitive
    }
    
    #[test]
    fn test_incoming_request() {
        let req = IncomingRequest {
            method: Method::Get,
            scheme: Some(Scheme::Https),
            authority: Some("api.example.com".to_string()),
            path_with_query: "/users?page=1&limit=10".to_string(),
            headers: Headers::new(),
        };
        
        assert_eq!(req.path(), "/users");
        assert_eq!(req.query(), Some("page=1&limit=10"));
        assert_eq!(req.query_param("page"), Some("1".to_string()));
        assert_eq!(req.query_param("limit"), Some("10".to_string()));
    }
    
    #[test]
    fn test_outgoing_response() {
        let resp = OutgoingResponse::ok()
            .header("X-Custom", "value")
            .json(r#"{"status":"ok"}"#);
        
        assert_eq!(resp.status, 200);
        assert_eq!(resp.headers.content_type(), Some("application/json"));
        assert!(resp.body.is_some());
    }
    
    #[test]
    fn test_outgoing_request() {
        let req = OutgoingRequest::get("https://api.example.com/users?page=1").unwrap();
        
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.scheme, Scheme::Https);
        assert_eq!(req.authority, "api.example.com");
        assert_eq!(req.path_with_query, "/users?page=1");
    }
    
    #[test]
    fn test_route_builder() {
        let route = Route::get("/api/status")
            .json(r#"{"status":"ok"}"#)
            .build();
        
        assert_eq!(route.path, "/api/status");
        assert_eq!(route.method, Some(Method::Get));
        assert_eq!(route.content_type, "application/json");
    }
    
    #[test]
    fn test_generate_handler() {
        let routes = vec![
            Route::get("/").json(r#"{"service":"test"}"#).build(),
            Route::get("/health").json(r#"{"status":"ok"}"#).build(),
        ];
        
        let wasm = generate_wasi_http_handler(&routes, r#"{"error":"Not Found"}"#);
        
        assert!(wasm.len() > 0);
        assert_eq!(&wasm[0..4], &WASM_MAGIC);
    }
    
    #[test]
    fn test_json_api_handler() {
        let endpoints = vec![
            ("/", r#"{"name":"TAYNI API"}"#),
            ("/health", r#"{"status":"healthy"}"#),
        ];
        
        let wasm = generate_json_api_handler(&endpoints);
        assert!(wasm.len() > 100);
    }
}
