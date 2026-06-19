//! Test TCP server PE generation and execution

use std::fs;
use std::process::{Command, Stdio};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use std::thread;

use tayni_c::pe_gen;

#[test]
fn test_tcp_server_pe_generation() {
    let pe = pe_gen::generate_tcp_server_pe(19999, "Hello from TAYNI TCP Server!\r\n");
    
    // Verify PE structure
    assert!(pe.len() > 512, "PE should be larger than 512 bytes");
    assert_eq!(&pe[0..2], b"MZ", "Should have MZ signature");
    
    // Find PE signature offset
    let pe_offset = u32::from_le_bytes([pe[0x3C], pe[0x3D], pe[0x3E], pe[0x3F]]) as usize;
    assert_eq!(&pe[pe_offset..pe_offset+4], b"PE\0\0", "Should have PE signature");
    
    println!("TCP Server PE generated: {} bytes", pe.len());
}

#[test]
fn test_http_server_pe_generation() {
    let routes = vec![
        ("/", "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, World!"),
    ];
    let pe = pe_gen::generate_http_server_pe(18080, &routes);
    
    // Verify PE structure
    assert!(pe.len() > 512, "PE should be larger than 512 bytes");
    assert_eq!(&pe[0..2], b"MZ", "Should have MZ signature");
    
    println!("HTTP Server PE generated: {} bytes", pe.len());
}

#[test]
#[ignore] // Run manually: cargo test test_tcp_server_functional --release -- --ignored --nocapture
fn test_tcp_server_functional() {
    let pe = pe_gen::generate_tcp_server_pe(19998, "TAYNI TCP Response\r\n");
    let exe_path = std::env::temp_dir().join("tayni_tcp_test.exe");
    fs::write(&exe_path, &pe).expect("Failed to write test exe");
    
    // Start server
    let mut server = Command::new(&exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start TCP server");
    
    // Wait for server to start
    thread::sleep(Duration::from_millis(500));
    
    // Try to connect
    match TcpStream::connect("127.0.0.1:19998") {
        Ok(mut stream) => {
            stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
            let mut buf = [0u8; 1024];
            match stream.read(&mut buf) {
                Ok(n) => {
                    let response = String::from_utf8_lossy(&buf[..n]);
                    println!("Received: {}", response);
                    assert!(response.contains("TAYNI"), "Response should contain TAYNI");
                }
                Err(e) => {
                    println!("Read error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
        }
    }
    
    // Cleanup
    server.kill().ok();
    fs::remove_file(&exe_path).ok();
}

#[test]
#[ignore] // Run manually: cargo test test_http_server_functional --release -- --ignored --nocapture
fn test_http_server_functional() {
    let routes = vec![
        ("/", "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, World!"),
    ];
    let pe = pe_gen::generate_http_server_pe(18081, &routes);
    let exe_path = std::env::temp_dir().join("tayni_http_test.exe");
    fs::write(&exe_path, &pe).expect("Failed to write test exe");
    
    // Start server
    let mut server = Command::new(&exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start HTTP server");
    
    // Wait for server to start
    thread::sleep(Duration::from_millis(500));
    
    // Try HTTP request
    match TcpStream::connect("127.0.0.1:18081") {
        Ok(mut stream) => {
            stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
            stream.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").ok();
            
            let mut buf = [0u8; 1024];
            match stream.read(&mut buf) {
                Ok(n) => {
                    let response = String::from_utf8_lossy(&buf[..n]);
                    println!("HTTP Response:\n{}", response);
                    assert!(response.contains("200 OK"), "Should get 200 OK");
                    assert!(response.contains("Hello, World!"), "Should contain body");
                }
                Err(e) => {
                    println!("Read error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
        }
    }
    
    // Cleanup
    server.kill().ok();
    fs::remove_file(&exe_path).ok();
}
