// NELAIA Benchmark - Solar System Server - Rust Optimized
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::windows::io::AsRawSocket;
use std::thread;

const HTML: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 1094\r\nConnection: close\r\n\r\n<!DOCTYPE html><html><head><title>Solar</title><style>body{background:#000;margin:0;display:flex;justify-content:center;align-items:center;height:100vh}.c{position:relative;width:600px;height:600px}.sun{position:absolute;top:50%;left:50%;width:50px;height:50px;margin:-25px;background:#ff0;border-radius:50%;box-shadow:0 0 30px #ff0}.p{position:absolute;top:50%;left:50%;border-radius:50%}.m{width:6px;height:6px;background:#888;animation:r 2s linear infinite;--d:40px}.v{width:10px;height:10px;background:#da6;animation:r 3s linear infinite;--d:70px}.e{width:12px;height:12px;background:#48f;animation:r 4s linear infinite;--d:100px}.x{width:8px;height:8px;background:#f42;animation:r 6s linear infinite;--d:130px}.j{width:22px;height:22px;background:#db8;animation:r 10s linear infinite;--d:180px}@keyframes r{to{transform:rotate(1turn) translateX(var(--d))}}</style></head><body><div class=c><div class=sun></div><div class=\"p m\"></div><div class=\"p v\"></div><div class=\"p e\"></div><div class=\"p x\"></div><div class=\"p j\"></div></div></body></html>";

fn handle_client(mut stream: TcpStream) {
    let _ = stream.set_nodelay(true);
    let mut buf = [0u8; 512];
    let _ = stream.read(&mut buf);
    let _ = stream.write_all(HTML);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8087").unwrap();
    
    // Clone listener for multiple threads
    let listener = std::sync::Arc::new(listener);
    
    // Spawn 15 worker threads
    for _ in 0..15 {
        let listener = listener.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    handle_client(stream);
                }
            }
        });
    }
    
    // Main thread also accepts
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_client(stream);
        }
    }
}
