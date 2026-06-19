// Minimal HTTP server for benchmark comparison - Rust
// Compile with: rustc -O rust_http.rs -o rust_http.exe
use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:38082").unwrap();
    println!("Rust HTTP Server listening on port 38082...");
    
    if let Ok((mut stream, _)) = listener.accept() {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        
        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 26\r\n\r\n{\"benchmark\":\"rust\",\"ok\":1}";
        stream.write_all(response.as_bytes()).unwrap();
    }
}
