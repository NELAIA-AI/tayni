//! Integration Tests
//! End-to-end tests for compiling and running TAYNI programs

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::fs;
    use std::path::Path;

    fn compiler_path() -> String {
        if cfg!(windows) {
            "target/release/tayni-c.exe".to_string()
        } else {
            "target/release/tayni-c".to_string()
        }
    }

    fn write_test_file(content: &str, name: &str) -> String {
        let path = format!("target/test_{}.tyn", name);
        fs::write(&path, content).expect("Failed to write test file");
        path
    }

    fn compile_and_check(code: &str, name: &str) -> bool {
        let path = write_test_file(code, name);
        let output = Command::new(compiler_path())
            .args(&[&path, "--check"])
            .output();
        
        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    // ============================================
    // Arithmetic Integration Tests
    // ============================================

    #[test]
    fn test_integration_add() {
        let code = ".a: 10\n.b: 20\n@.sum: ADD .a .b\n!";
        assert!(compile_and_check(code, "int_add"));
    }

    #[test]
    fn test_integration_chained_arithmetic() {
        let code = r#"
.a: 5
.b: 3
@.sum: ADD .a .b
@.prod: MUL .sum 2
@.result: SUB .prod 1
!
"#;
        assert!(compile_and_check(code, "int_chain"));
    }

    // ============================================
    // Control Flow Integration Tests
    // ============================================

    #[test]
    fn test_integration_conditional() {
        let code = r#"
.x: 0
.y: 42
@.result: IFZ .x .y 0
!
"#;
        assert!(compile_and_check(code, "int_cond"));
    }

    #[test]
    #[ignore] // Requires compiled binary
    fn test_integration_loop() {
        let code = r#"
.i: 0
.n: 10
:loop
@.i: ADD .i 1
@.done: GE .i .n
@.j: JZ :loop .done
!
"#;
        assert!(compile_and_check(code, "int_loop"));
    }

    // ============================================
    // String Integration Tests
    // ============================================

    #[test]
    fn test_integration_string_length() {
        let code = r#"
.msg: "Hello World"
@.len: SLN .msg
!
"#;
        assert!(compile_and_check(code, "int_sln"));
    }

    #[test]
    fn test_integration_string_concat() {
        let code = r#"
.a: "Hello "
.b: "World"
.dst: ALC 256
@.len: CAT .dst .a .b
!
"#;
        assert!(compile_and_check(code, "int_cat"));
    }

    // ============================================
    // Memory Integration Tests
    // ============================================

    #[test]
    fn test_integration_memory_ops() {
        let code = r#"
.buf: ALC 1024
@.w: PUT .buf 0 65
@.r: GET .buf 0
!
"#;
        assert!(compile_and_check(code, "int_mem"));
    }

    // ============================================
    // File I/O Integration Tests
    // ============================================

    #[test]
    fn test_integration_file_write() {
        let code = r#"
.path: "test_output.txt"
.msg: "Hello File"
@.f: FOP .path 1
@.w: FWR .f .msg 10
@.c: FCL .f
!
"#;
        assert!(compile_and_check(code, "int_fwrite"));
    }

    // ============================================
    // Network Integration Tests
    // ============================================

    #[test]
    fn test_integration_tcp_socket() {
        let code = r#"
@.sock: TCP
!
"#;
        assert!(compile_and_check(code, "int_tcp"));
    }

    // ============================================
    // JSON Integration Tests
    // ============================================

    #[test]
    fn test_integration_json_encode() {
        let code = r#"
.key: "name"
.val: "test"
.buf: ALC 256
@.len: JSON.ENCODE .buf .key .val
!
"#;
        assert!(compile_and_check(code, "int_json"));
    }

    // ============================================
    // Threading Integration Tests
    // ============================================

    #[test]
    fn test_integration_mutex() {
        let code = r#"
@.m: MTX
@.l: LCK .m
@.u: ULK .m
!
"#;
        assert!(compile_and_check(code, "int_mutex"));
    }

    #[test]
    fn test_integration_channel() {
        let code = r#"
@.ch: CHN 10
@.s: CHN.SND .ch 42
@.r: CHN.RCV .ch
@.c: CHN.CLS .ch
!
"#;
        assert!(compile_and_check(code, "int_chan"));
    }

    // ============================================
    // Capability Integration Tests
    // ============================================

    #[test]
    fn test_integration_requires() {
        let code = r#"
.caps: REQUIRES { http, json }
.x: 42
!
"#;
        assert!(compile_and_check(code, "int_caps"));
    }
}
