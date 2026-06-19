//! HTTP Client for TAYNI
//! 
//! A minimal, zero-dependency HTTP/1.1 client implementation.
//! Designed for AI-generated code with simple, predictable API.

use std::collections::HashMap;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::TcpStream;
use std::time::Duration;

// ============================================================================
// HTTP Types
// ============================================================================

/// HTTP Method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
        }
    }
}

/// HTTP Request
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout: Option<Duration>,
}

impl Request {
    pub fn new(method: Method, url: &str) -> Result<Self, HttpError> {
        let url = Url::parse(url)?;
        Ok(Request {
            method,
            url,
            headers: HashMap::new(),
            body: None,
            timeout: Some(Duration::from_secs(30)),
        })
    }
    
    pub fn get(url: &str) -> Result<Self, HttpError> {
        Self::new(Method::Get, url)
    }
    
    pub fn post(url: &str) -> Result<Self, HttpError> {
        Self::new(Method::Post, url)
    }
    
    pub fn put(url: &str) -> Result<Self, HttpError> {
        Self::new(Method::Put, url)
    }
    
    pub fn delete(url: &str) -> Result<Self, HttpError> {
        Self::new(Method::Delete, url)
    }
    
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn body(mut self, data: Vec<u8>) -> Self {
        self.body = Some(data);
        self
    }
    
    pub fn json(self, data: &str) -> Self {
        self.header("Content-Type", "application/json")
            .body(data.as_bytes().to_vec())
    }
    
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
    
    /// Send the request and get response
    pub fn send(self) -> Result<Response, HttpError> {
        send_request(self)
    }
}

/// HTTP Response
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    /// Check if status is 2xx
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
    
    /// Check if status is 4xx
    pub fn is_client_error(&self) -> bool {
        self.status >= 400 && self.status < 500
    }
    
    /// Check if status is 5xx
    pub fn is_server_error(&self) -> bool {
        self.status >= 500 && self.status < 600
    }
    
    /// Get body as UTF-8 string
    pub fn text(&self) -> Result<String, HttpError> {
        String::from_utf8(self.body.clone())
            .map_err(|_| HttpError::InvalidUtf8)
    }
    
    /// Get header value (case-insensitive)
    pub fn header(&self, key: &str) -> Option<&str> {
        let key_lower = key.to_lowercase();
        self.headers.iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v.as_str())
    }
    
    /// Get Content-Type header
    pub fn content_type(&self) -> Option<&str> {
        self.header("Content-Type")
    }
    
    /// Get Content-Length header
    pub fn content_length(&self) -> Option<usize> {
        self.header("Content-Length")
            .and_then(|v| v.parse().ok())
    }
}

/// Parsed URL
#[derive(Debug, Clone)]
pub struct Url {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
}

impl Url {
    pub fn parse(url: &str) -> Result<Self, HttpError> {
        // Parse scheme
        let (scheme, rest) = if url.starts_with("https://") {
            ("https", &url[8..])
        } else if url.starts_with("http://") {
            ("http", &url[7..])
        } else {
            return Err(HttpError::InvalidUrl("Missing scheme".to_string()));
        };
        
        // Parse host and path
        let (host_port, path_query) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };
        
        // Parse port
        let (host, port) = match host_port.find(':') {
            Some(idx) => {
                let port = host_port[idx+1..].parse()
                    .map_err(|_| HttpError::InvalidUrl("Invalid port".to_string()))?;
                (&host_port[..idx], port)
            }
            None => {
                let default_port = if scheme == "https" { 443 } else { 80 };
                (host_port, default_port)
            }
        };
        
        // Parse query
        let (path, query) = match path_query.find('?') {
            Some(idx) => (&path_query[..idx], Some(path_query[idx+1..].to_string())),
            None => (path_query, None),
        };
        
        Ok(Url {
            scheme: scheme.to_string(),
            host: host.to_string(),
            port,
            path: path.to_string(),
            query,
        })
    }
    
    pub fn full_path(&self) -> String {
        match &self.query {
            Some(q) => format!("{}?{}", self.path, q),
            None => self.path.clone(),
        }
    }
}

/// HTTP Error
#[derive(Debug, Clone)]
pub enum HttpError {
    InvalidUrl(String),
    ConnectionFailed(String),
    Timeout,
    InvalidResponse(String),
    InvalidUtf8,
    TlsNotSupported,
    IoError(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            HttpError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            HttpError::Timeout => write!(f, "Request timeout"),
            HttpError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            HttpError::InvalidUtf8 => write!(f, "Response body is not valid UTF-8"),
            HttpError::TlsNotSupported => write!(f, "HTTPS not supported (use HTTP)"),
            HttpError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for HttpError {}

// ============================================================================
// HTTP Client Implementation
// ============================================================================

/// Send an HTTP request
fn send_request(req: Request) -> Result<Response, HttpError> {
    // Check for HTTPS (not supported without TLS)
    if req.url.scheme == "https" {
        return Err(HttpError::TlsNotSupported);
    }
    
    // Connect to server
    let addr = format!("{}:{}", req.url.host, req.url.port);
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| HttpError::ConnectionFailed(e.to_string()))?;
    
    // Set timeout
    if let Some(timeout) = req.timeout {
        stream.set_read_timeout(Some(timeout))
            .map_err(|e| HttpError::IoError(e.to_string()))?;
        stream.set_write_timeout(Some(timeout))
            .map_err(|e| HttpError::IoError(e.to_string()))?;
    }
    
    // Build request
    let mut request_str = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\n",
        req.method.as_str(),
        req.url.full_path(),
        req.url.host
    );
    
    // Add headers
    for (key, value) in &req.headers {
        request_str.push_str(&format!("{}: {}\r\n", key, value));
    }
    
    // Add Content-Length if body present
    if let Some(ref body) = req.body {
        if !req.headers.contains_key("Content-Length") {
            request_str.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
    }
    
    // Add Connection header
    if !req.headers.contains_key("Connection") {
        request_str.push_str("Connection: close\r\n");
    }
    
    // End headers
    request_str.push_str("\r\n");
    
    // Send request
    stream.write_all(request_str.as_bytes())
        .map_err(|e| HttpError::IoError(e.to_string()))?;
    
    // Send body if present
    if let Some(body) = req.body {
        stream.write_all(&body)
            .map_err(|e| HttpError::IoError(e.to_string()))?;
    }
    
    stream.flush()
        .map_err(|e| HttpError::IoError(e.to_string()))?;
    
    // Read response
    let mut reader = BufReader::new(stream);
    
    // Parse status line
    let mut status_line = String::new();
    reader.read_line(&mut status_line)
        .map_err(|e| HttpError::IoError(e.to_string()))?;
    
    let (status, status_text) = parse_status_line(&status_line)?;
    
    // Parse headers
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)
            .map_err(|e| HttpError::IoError(e.to_string()))?;
        
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        
        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim().to_string();
            let value = line[idx+1..].trim().to_string();
            headers.insert(key, value);
        }
    }
    
    // Read body
    let body = read_body(&mut reader, &headers)?;
    
    Ok(Response {
        status,
        status_text,
        headers,
        body,
    })
}

fn parse_status_line(line: &str) -> Result<(u16, String), HttpError> {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(HttpError::InvalidResponse("Invalid status line".to_string()));
    }
    
    let status = parts[1].parse()
        .map_err(|_| HttpError::InvalidResponse("Invalid status code".to_string()))?;
    
    let status_text = if parts.len() > 2 {
        parts[2].trim().to_string()
    } else {
        String::new()
    };
    
    Ok((status, status_text))
}

fn read_body<R: BufRead>(reader: &mut R, headers: &HashMap<String, String>) -> Result<Vec<u8>, HttpError> {
    // Check for Content-Length
    if let Some(len_str) = headers.get("Content-Length").or_else(|| headers.get("content-length")) {
        let len: usize = len_str.parse()
            .map_err(|_| HttpError::InvalidResponse("Invalid Content-Length".to_string()))?;
        
        let mut body = vec![0u8; len];
        reader.read_exact(&mut body)
            .map_err(|e| HttpError::IoError(e.to_string()))?;
        
        return Ok(body);
    }
    
    // Check for Transfer-Encoding: chunked
    let transfer_encoding = headers.get("Transfer-Encoding")
        .or_else(|| headers.get("transfer-encoding"));
    
    if transfer_encoding.map(|s| s.contains("chunked")).unwrap_or(false) {
        return read_chunked_body(reader);
    }
    
    // Read until connection closes
    let mut body = Vec::new();
    reader.read_to_end(&mut body)
        .map_err(|e| HttpError::IoError(e.to_string()))?;
    
    Ok(body)
}

fn read_chunked_body<R: BufRead>(reader: &mut R) -> Result<Vec<u8>, HttpError> {
    let mut body = Vec::new();
    
    loop {
        // Read chunk size line
        let mut size_line = String::new();
        reader.read_line(&mut size_line)
            .map_err(|e| HttpError::IoError(e.to_string()))?;
        
        let size = usize::from_str_radix(size_line.trim(), 16)
            .map_err(|_| HttpError::InvalidResponse("Invalid chunk size".to_string()))?;
        
        if size == 0 {
            // Read trailing CRLF
            let mut _trailer = String::new();
            let _ = reader.read_line(&mut _trailer);
            break;
        }
        
        // Read chunk data
        let mut chunk = vec![0u8; size];
        reader.read_exact(&mut chunk)
            .map_err(|e| HttpError::IoError(e.to_string()))?;
        body.extend(chunk);
        
        // Read trailing CRLF
        let mut _crlf = [0u8; 2];
        let _ = reader.read_exact(&mut _crlf);
    }
    
    Ok(body)
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Simple GET request
pub fn get(url: &str) -> Result<Response, HttpError> {
    Request::get(url)?.send()
}

/// Simple POST request with body
pub fn post(url: &str, body: &[u8]) -> Result<Response, HttpError> {
    Request::post(url)?
        .body(body.to_vec())
        .send()
}

/// POST JSON data
pub fn post_json(url: &str, json: &str) -> Result<Response, HttpError> {
    Request::post(url)?
        .json(json)
        .send()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_url_parse_simple() {
        let url = Url::parse("http://example.com/path").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 80);
        assert_eq!(url.path, "/path");
        assert!(url.query.is_none());
    }
    
    #[test]
    fn test_url_parse_with_port() {
        let url = Url::parse("http://localhost:8080/api").unwrap();
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, 8080);
        assert_eq!(url.path, "/api");
    }
    
    #[test]
    fn test_url_parse_with_query() {
        let url = Url::parse("http://api.example.com/search?q=test&limit=10").unwrap();
        assert_eq!(url.path, "/search");
        assert_eq!(url.query, Some("q=test&limit=10".to_string()));
    }
    
    #[test]
    fn test_url_parse_https() {
        let url = Url::parse("https://secure.example.com/").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.port, 443);
    }
    
    #[test]
    fn test_request_builder() {
        let req = Request::get("http://example.com/api")
            .unwrap()
            .header("Accept", "application/json")
            .header("User-Agent", "TAYNI/1.0")
            .timeout(Duration::from_secs(10));
        
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.headers.get("Accept"), Some(&"application/json".to_string()));
        assert_eq!(req.timeout, Some(Duration::from_secs(10)));
    }
    
    #[test]
    fn test_request_with_json() {
        let req = Request::post("http://api.example.com/data")
            .unwrap()
            .json(r#"{"key": "value"}"#);
        
        assert_eq!(req.headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert!(req.body.is_some());
    }
    
    #[test]
    fn test_response_helpers() {
        let response = Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
            body: b"Hello".to_vec(),
        };
        
        assert!(response.is_success());
        assert!(!response.is_client_error());
        assert!(!response.is_server_error());
        assert_eq!(response.content_type(), Some("application/json"));
        assert_eq!(response.text().unwrap(), "Hello");
    }
    
    #[test]
    fn test_status_line_parse() {
        let (status, text) = parse_status_line("HTTP/1.1 200 OK\r\n").unwrap();
        assert_eq!(status, 200);
        assert_eq!(text, "OK");
        
        let (status, text) = parse_status_line("HTTP/1.1 404 Not Found\r\n").unwrap();
        assert_eq!(status, 404);
        assert_eq!(text, "Not Found");
    }
    
    #[test]
    fn test_https_not_supported() {
        let result = Request::get("https://example.com").unwrap().send();
        assert!(matches!(result, Err(HttpError::TlsNotSupported)));
    }
}
