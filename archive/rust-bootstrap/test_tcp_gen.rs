use std::fs;
mod pe_gen;
fn main() {
    let pe = pe_gen::generate_tcp_server_pe(9999, \"Hello from TAYNI TCP!\");
    fs::write(\"test_tcp.exe\", pe).unwrap();
    println!(\"Generated test_tcp.exe\");
}
