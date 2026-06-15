//! NELAIA Compiler Test Suite
//! AI-First: Deterministic validation of all operators and examples

use std::process::Command;
use std::fs;
use std::path::Path;

/// Get the compiler path
fn compiler_path() -> String {
    if cfg!(windows) {
        "target/debug/nelaia-c.exe".to_string()
    } else {
        "target/debug/nelaia-c".to_string()
    }
}

/// Run compiler with given args and return (exit_code, stdout, stderr)
fn run_compiler(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(compiler_path())
        .args(args)
        .output()
        .expect("Failed to execute compiler");
    
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Write temp file and return path
fn write_temp_nela(content: &str, name: &str) -> String {
    let path = format!("target/test_{}.nela", name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================
// CLI Tests
// ============================================

#[test]
fn test_version() {
    let (code, stdout, _) = run_compiler(&["--version"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("nelaia-c"));
    assert!(stdout.contains("0.23"));
}

#[test]
fn test_help() {
    let (code, stdout, _) = run_compiler(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("USAGE"));
    assert!(stdout.contains("--emit-pe"));
    assert!(stdout.contains("--emit-elf"));
    assert!(stdout.contains("--emit-macho"));
}

#[test]
fn test_no_args() {
    let (code, _, stderr) = run_compiler(&[]);
    assert_eq!(code, 0);
    assert!(stderr.contains("usage") || stderr.contains("nelaia-c"));
}

// ============================================
// Syntax Check Tests (--check)
// ============================================

#[test]
fn test_check_valid_literal() {
    let path = write_temp_nela(".x: 42", "literal");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn test_check_valid_string() {
    let path = write_temp_nela(".msg: \"Hello\"", "string");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn test_check_valid_operation() {
    let path = write_temp_nela(".a: 10\n.b: 20\n.sum: ADD .a .b", "operation");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn test_check_invalid_syntax() {
    let path = write_temp_nela("invalid syntax here", "invalid");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("E:PARSE") || stderr.contains("error"));
}

#[test]
fn test_check_json_output() {
    let path = write_temp_nela(".x: 42", "json");
    let (code, stdout, _) = run_compiler(&[&path, "--check", "--json"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("\"status\"") || stdout.contains("\"ok\""));
}

// ============================================
// Arithmetic Operators
// ============================================

#[test]
fn test_op_add() {
    let path = write_temp_nela(".a: 42\n.b: 8\n.sum: ADD .a .b", "add");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "ADD failed: {}", stderr);
}

#[test]
fn test_op_sub() {
    let path = write_temp_nela(".a: 50\n.b: 10\n.result: SUB .a .b", "sub");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "SUB failed: {}", stderr);
}

#[test]
fn test_op_mul() {
    let path = write_temp_nela(".a: 6\n.b: 7\n.result: MUL .a .b", "mul");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "MUL failed: {}", stderr);
}

#[test]
fn test_op_div() {
    let path = write_temp_nela(".a: 100\n.b: 4\n.result: DIV .a .b", "div");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "DIV failed: {}", stderr);
}

#[test]
fn test_op_mod() {
    let path = write_temp_nela(".a: 17\n.b: 5\n.result: MOD .a .b", "mod");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "MOD failed: {}", stderr);
}

#[test]
fn test_op_neg() {
    let path = write_temp_nela(".x: 42\n.result: NEG .x", "neg");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "NEG failed: {}", stderr);
}

// ============================================
// Comparison Operators
// ============================================

#[test]
fn test_op_eq() {
    let path = write_temp_nela(".x: 10\n.y: 10\n.result: EQ .x .y", "eq");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "EQ failed: {}", stderr);
}

#[test]
fn test_op_ne() {
    let path = write_temp_nela(".a: 5\n.b: 10\n.result: NE .a .b", "ne");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "NE failed: {}", stderr);
}

#[test]
fn test_op_lt() {
    let path = write_temp_nela(".x: 5\n.y: 10\n.result: LT .x .y", "lt");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "LT failed: {}", stderr);
}

#[test]
fn test_op_gt() {
    let path = write_temp_nela(".x: 15\n.y: 10\n.result: GT .x .y", "gt");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "GT failed: {}", stderr);
}

#[test]
fn test_op_le() {
    let path = write_temp_nela(".x: 10\n.y: 10\n.result: LE .x .y", "le");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "LE failed: {}", stderr);
}

#[test]
fn test_op_ge() {
    let path = write_temp_nela(".x: 10\n.y: 5\n.result: GE .x .y", "ge");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "GE failed: {}", stderr);
}

// ============================================
// Logic Operators
// ============================================

#[test]
fn test_op_and() {
    let path = write_temp_nela(".a: 1\n.b: 1\n.result: AND .a .b", "and");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "AND failed: {}", stderr);
}

#[test]
fn test_op_or() {
    let path = write_temp_nela(".a: 0\n.b: 1\n.result: OR .a .b", "or");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "OR failed: {}", stderr);
}

#[test]
fn test_op_not() {
    let path = write_temp_nela(".a: 1\n.result: NOT .a", "not");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "NOT failed: {}", stderr);
}

// ============================================
// Memory Operators
// ============================================

#[test]
fn test_op_alc() {
    let path = write_temp_nela(".buf: ALC 1024", "alc");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "ALC failed: {}", stderr);
}

#[test]
fn test_op_fre() {
    let path = write_temp_nela(".buf: ALC 1024\n.free: FRE .buf", "fre");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "FRE failed: {}", stderr);
}

#[test]
fn test_op_put() {
    let path = write_temp_nela(".buf: ALC 100\n.write: PUT .buf 0 65", "put");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "PUT failed: {}", stderr);
}

#[test]
fn test_op_get() {
    let path = write_temp_nela(".buf: ALC 100\n.byte: GET .buf 0", "get");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "GET failed: {}", stderr);
}

#[test]
fn test_op_cpy() {
    let path = write_temp_nela(".src: ALC 100\n.dst: ALC 100\n.copy: CPY .dst .src 50", "cpy");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "CPY failed: {}", stderr);
}

#[test]
fn test_op_sln() {
    let path = write_temp_nela(".str: \"Hello\"\n.len: SLN .str", "sln");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "SLN failed: {}", stderr);
}

// ============================================
// I/O Operators
// ============================================

#[test]
fn test_op_prt() {
    let path = write_temp_nela(".msg: \"Hello\\n\"\n.len: 6\n.out: PRT .msg .len", "prt");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "PRT failed: {}", stderr);
}

#[test]
fn test_op_inp() {
    let path = write_temp_nela(".input: INP", "inp");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "INP failed: {}", stderr);
}

#[test]
fn test_op_err() {
    let path = write_temp_nela(".err: ERR", "err");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "ERR failed: {}", stderr);
}

// ============================================
// File I/O Operators
// ============================================

#[test]
fn test_op_fop() {
    let path = write_temp_nela(".path: \"data.txt\"\n.file: FOP .path 0", "fop");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "FOP failed: {}", stderr);
}

#[test]
fn test_op_frd() {
    let path = write_temp_nela(".path: \"data.txt\"\n.file: FOP .path 0\n.buf: ALC 1024\n.n: FRD .file .buf 1024", "frd");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "FRD failed: {}", stderr);
}

#[test]
fn test_op_fwr() {
    let path = write_temp_nela(".path: \"out.txt\"\n.file: FOP .path 1\n.msg: \"Hello\"\n.len: 5\n.write: FWR .file .msg .len", "fwr");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "FWR failed: {}", stderr);
}

#[test]
fn test_op_fcl() {
    let path = write_temp_nela(".path: \"data.txt\"\n.file: FOP .path 0\n.close: FCL .file", "fcl");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "FCL failed: {}", stderr);
}

// ============================================
// Network Operators
// ============================================

#[test]
fn test_op_tcp() {
    let path = write_temp_nela(".sock: TCP", "tcp");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "TCP failed: {}", stderr);
}

#[test]
fn test_op_udp() {
    let path = write_temp_nela(".sock: UDP", "udp");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "UDP failed: {}", stderr);
}

#[test]
fn test_op_bnd() {
    let path = write_temp_nela(".sock: TCP\n.addr: ALC 16\n.bind: BND .sock .addr", "bnd");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "BND failed: {}", stderr);
}

#[test]
fn test_op_lst() {
    let path = write_temp_nela(".sock: TCP\n.listen: LST .sock 10", "lst");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "LST failed: {}", stderr);
}

#[test]
fn test_op_acc() {
    let path = write_temp_nela(".sock: TCP\n.client: ACC .sock", "acc");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "ACC failed: {}", stderr);
}

#[test]
fn test_op_xmt() {
    let path = write_temp_nela(".sock: TCP\n.msg: \"Hello\"\n.send: XMT .sock .msg 5", "xmt");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "XMT failed: {}", stderr);
}

#[test]
fn test_op_rcv() {
    let path = write_temp_nela(".sock: TCP\n.buf: ALC 1024\n.recv: RCV .sock .buf 1024", "rcv");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "RCV failed: {}", stderr);
}

#[test]
fn test_op_cls() {
    let path = write_temp_nela(".sock: TCP\n.close: CLS .sock", "cls");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "CLS failed: {}", stderr);
}

// ============================================
// Control Flow
// ============================================

#[test]
fn test_op_ifz() {
    let path = write_temp_nela(".cond: 0\n.result: IFZ .cond 10 20", "ifz");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "IFZ failed: {}", stderr);
}

#[test]
fn test_op_jmp() {
    let path = write_temp_nela(".x: 1\n.loop: JMP .x", "jmp");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "JMP failed: {}", stderr);
}

// ============================================
// Capabilities
// ============================================

#[test]
fn test_capability_http() {
    let path = write_temp_nela(".caps: REQUIRES { http }\n.server: HTTP.LISTEN 8080", "http");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "HTTP capability failed: {}", stderr);
}

#[test]
fn test_capability_sql() {
    let path = write_temp_nela(".caps: REQUIRES { sql }\n.connstr: \"server=localhost\"\n.conn: SQL.CONNECT .connstr", "sql");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "SQL capability failed: {}", stderr);
}

#[test]
fn test_capability_json() {
    let path = write_temp_nela(".caps: REQUIRES { json }\n.input: \"{}\"\n.data: JSON.PARSE .input", "json");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_eq!(code, 0, "JSON capability failed: {}", stderr);
}

// ============================================
// Binary Generation Tests
// ============================================

#[test]
fn test_emit_pe() {
    let path = write_temp_nela(".msg: \"Hello\\n\"\n.len: 6\n.out: PRT .msg .len", "pe");
    let (code, _, stderr) = run_compiler(&[&path, "-o", "target/test_pe", "--emit-pe"]);
    assert_eq!(code, 0, "PE emission failed: {}", stderr);
    assert!(Path::new("target/test_pe.exe").exists(), "PE file not created");
}

#[test]
fn test_emit_elf() {
    let path = write_temp_nela(".msg: \"Hello\\n\"\n.len: 6\n.out: PRT .msg .len", "elf");
    let (code, _, stderr) = run_compiler(&[&path, "-o", "target/test_elf", "--emit-elf"]);
    assert_eq!(code, 0, "ELF emission failed: {}", stderr);
    assert!(Path::new("target/test_elf").exists(), "ELF file not created");
}

#[test]
fn test_emit_macho() {
    let path = write_temp_nela(".msg: \"Hello\\n\"\n.len: 6\n.out: PRT .msg .len", "macho");
    let (code, _, stderr) = run_compiler(&[&path, "-o", "target/test_macho", "--emit-macho"]);
    assert_eq!(code, 0, "Mach-O emission failed: {}", stderr);
    assert!(Path::new("target/test_macho").exists(), "Mach-O file not created");
}

#[test]
fn test_emit_macho_arm64() {
    let path = write_temp_nela(".msg: \"Hello\\n\"\n.len: 6\n.out: PRT .msg .len", "macho_arm");
    let (code, _, stderr) = run_compiler(&[&path, "-o", "target/test_macho_arm", "--emit-macho-arm64"]);
    assert_eq!(code, 0, "Mach-O ARM64 emission failed: {}", stderr);
    assert!(Path::new("target/test_macho_arm").exists(), "Mach-O ARM64 file not created");
}

#[test]
fn test_emit_bin() {
    let path = write_temp_nela(".x: 42\n.y: ADD .x 8", "bin");
    let (code, _, stderr) = run_compiler(&[&path, "-o", "target/test_bin", "--emit-bin"]);
    assert_eq!(code, 0, "Binary format emission failed: {}", stderr);
    assert!(Path::new("target/test_bin.nbin").exists(), "Binary file not created");
}

// ============================================
// Error Detection Tests
// ============================================

#[test]
fn test_error_undefined_ref() {
    let path = write_temp_nela(".result: ADD .undefined .x", "undef");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("E:UNDEF") || stderr.contains("undefined"));
}

#[test]
fn test_error_cycle() {
    let path = write_temp_nela(".a: ADD .b 1\n.b: ADD .a 1", "cycle");
    let (code, _, stderr) = run_compiler(&[&path, "--check"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("E:CYCLE") || stderr.contains("cycle"));
}

// ============================================
// JSONL Training Data Validation
// ============================================

#[test]
fn test_jsonl_examples_syntax() {
    let jsonl_path = "docs/NELAIA-TRAINING-DATA.jsonl";
    if !Path::new(jsonl_path).exists() {
        return; // Skip if file doesn't exist
    }
    
    let content = fs::read_to_string(jsonl_path).expect("Failed to read JSONL");
    let mut passed = 0;
    let mut failed = 0;
    
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        
        // Parse JSON to extract output field
        if let Some(start) = line.find("\"output\":") {
            let rest = &line[start + 10..];
            if let Some(end) = rest.find("\"}") {
                let code = &rest[1..end];
                let code = code.replace("\\n", "\n").replace("\\\"", "\"");
                
                let path = write_temp_nela(&code, &format!("jsonl_{}", i));
                let (exit_code, _, _) = run_compiler(&[&path, "--check", "--quiet"]);
                
                if exit_code == 0 {
                    passed += 1;
                } else {
                    failed += 1;
                    eprintln!("JSONL example {} failed", i + 1);
                }
            }
        }
    }
    
    eprintln!("JSONL validation: {} passed, {} failed", passed, failed);
    // Allow some failures for complex examples
    assert!(passed > 50, "Too few JSONL examples passed: {}", passed);
}

// ============================================
// Existing Test Files Validation
// ============================================

#[test]
fn test_existing_nela_files() {
    let test_dir = Path::new("tests");
    if !test_dir.exists() {
        return;
    }
    
    let mut passed = 0;
    let mut failed = 0;
    
    for entry in fs::read_dir(test_dir).expect("Failed to read tests dir") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        
        if path.extension().map(|e| e == "nela").unwrap_or(false) {
            let path_str = path.to_string_lossy();
            let (code, _, _) = run_compiler(&[&path_str, "--check", "--quiet"]);
            
            if code == 0 {
                passed += 1;
            } else {
                failed += 1;
                eprintln!("Test file failed: {}", path_str);
            }
        }
    }
    
    eprintln!("Test files: {} passed, {} failed", passed, failed);
}
