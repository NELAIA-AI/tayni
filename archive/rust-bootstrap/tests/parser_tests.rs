//! Parser Unit Tests
//! Tests for TAYNI v1.0 and v1.5 syntax parsing

#[cfg(test)]
mod tests {
    use tayni_c::parser::{Parser, SyntaxVersion};
    use tayni_c::ir::{Node, Op, Value, Arg, Capability};

    // ============================================
    // Version Detection Tests
    // ============================================

    #[test]
    fn test_detect_v1_0_literal() {
        let code = ".x: 42\n.y: 10";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_0);
    }

    #[test]
    fn test_detect_v1_0_runtime() {
        let code = "@.x: ADD .a .b";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_0);
    }

    #[test]
    fn test_detect_v1_0_use() {
        let code = "USE http\n.x: 42";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_0);
    }

    #[test]
    fn test_detect_v1_5_assignment() {
        let code = "x = 42\ny = add(x, 10)";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_5);
    }

    #[test]
    fn test_detect_v1_5_compound() {
        let code = "x = 10\nx += 5";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_5);
    }

    #[test]
    fn test_detect_v1_5_for_loop() {
        let code = "for i in 0..10\n  print(i)";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_5);
    }

    #[test]
    fn test_detect_v1_5_function() {
        let code = "fn main()\n  x = 42";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_5);
    }

    #[test]
    fn test_detect_v1_5_guard() {
        let code = "guard x > 0 else :error";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_5);
    }

    #[test]
    fn test_detect_v1_5_use_lowercase() {
        let code = "use http, json\nx = 42";
        assert_eq!(Parser::detect_version(code), SyntaxVersion::V1_5);
    }

    // ============================================
    // v1.0 Parsing Tests - Literals
    // ============================================

    #[test]
    fn test_parse_v1_0_integer_literal() {
        let code = ".x: 42";
        let graph = Parser::parse(code).expect("Parse failed");
        assert_eq!(graph.nodes.len(), 1);
        
        if let Node::Literal { id, value, runtime } = &graph.nodes[0] {
            assert_eq!(id, "x");
            assert!(!runtime);
            if let Value::Int(n) = value {
                assert_eq!(*n, 42);
            } else {
                panic!("Expected Int value");
            }
        } else {
            panic!("Expected Literal node");
        }
    }

    #[test]
    fn test_parse_v1_0_negative_literal() {
        let code = ".x: -42";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Literal { value, .. } = &graph.nodes[0] {
            if let Value::Int(n) = value {
                assert_eq!(*n, -42);
            } else {
                panic!("Expected Int value");
            }
        } else {
            panic!("Expected Literal node");
        }
    }

    #[test]
    fn test_parse_v1_0_string_literal() {
        let code = ".msg: \"Hello World\"";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Literal { id, value, .. } = &graph.nodes[0] {
            assert_eq!(id, "msg");
            if let Value::String(s) = value {
                assert_eq!(s, "Hello World");
            } else {
                panic!("Expected String value");
            }
        } else {
            panic!("Expected Literal node");
        }
    }

    #[test]
    fn test_parse_v1_0_string_escape_sequences() {
        let code = ".msg: \"Hello\\nWorld\\t!\"";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Literal { value, .. } = &graph.nodes[0] {
            if let Value::String(s) = value {
                assert_eq!(s, "Hello\nWorld\t!");
            } else {
                panic!("Expected String value");
            }
        } else {
            panic!("Expected Literal node");
        }
    }

    #[test]
    fn test_parse_v1_0_float_literal() {
        let code = ".pi: 3.14159";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Literal { value, .. } = &graph.nodes[0] {
            if let Value::Float(f) = value {
                assert!((*f - 3.14159).abs() < 0.00001);
            } else {
                panic!("Expected Float value");
            }
        } else {
            panic!("Expected Literal node");
        }
    }

    // ============================================
    // v1.0 Parsing Tests - Operations
    // ============================================

    #[test]
    fn test_parse_v1_0_add_operation() {
        let code = ".a: 10\n.b: 20\n@.sum: ADD .a .b";
        let graph = Parser::parse(code).expect("Parse failed");
        assert_eq!(graph.nodes.len(), 3);
        
        if let Node::Operation { id, op, args, runtime } = &graph.nodes[2] {
            assert_eq!(id, "sum");
            assert_eq!(*op, Op::Add);
            assert!(*runtime);
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected Operation node");
        }
    }

    #[test]
    fn test_parse_v1_0_all_arithmetic_ops() {
        let ops = vec![
            ("ADD", Op::Add),
            ("SUB", Op::Sub),
            ("MUL", Op::Mul),
            ("DIV", Op::Div),
            ("MOD", Op::Mod),
        ];
        
        for (op_str, expected_op) in ops {
            let code = format!(".a: 10\n.b: 5\n@.r: {} .a .b", op_str);
            let graph = Parser::parse(&code).expect(&format!("Parse failed for {}", op_str));
            
            if let Node::Operation { op, .. } = &graph.nodes[2] {
                assert_eq!(*op, expected_op, "Op mismatch for {}", op_str);
            } else {
                panic!("Expected Operation node for {}", op_str);
            }
        }
    }

    #[test]
    fn test_parse_v1_0_comparison_ops() {
        let ops = vec![
            ("EQ", Op::Eq),
            ("NE", Op::Ne),
            ("LT", Op::Lt),
            ("GT", Op::Gt),
            ("LE", Op::Le),
            ("GE", Op::Ge),
        ];
        
        for (op_str, expected_op) in ops {
            let code = format!(".a: 10\n.b: 5\n@.r: {} .a .b", op_str);
            let graph = Parser::parse(&code).expect(&format!("Parse failed for {}", op_str));
            
            if let Node::Operation { op, .. } = &graph.nodes[2] {
                assert_eq!(*op, expected_op, "Op mismatch for {}", op_str);
            } else {
                panic!("Expected Operation node for {}", op_str);
            }
        }
    }

    #[test]
    fn test_parse_v1_0_memory_ops() {
        let code = ".buf: ALC 1024\n@.byte: GET .buf 0\n@.write: PUT .buf 0 65";
        let graph = Parser::parse(code).expect("Parse failed");
        assert_eq!(graph.nodes.len(), 3);
        
        if let Node::Operation { op, .. } = &graph.nodes[1] {
            assert_eq!(*op, Op::Get);
        }
        if let Node::Operation { op, .. } = &graph.nodes[2] {
            assert_eq!(*op, Op::Put);
        }
    }

    #[test]
    fn test_parse_v1_0_file_io_ops() {
        let code = ".path: \"test.txt\"\n@.f: FOP .path 0\n.buf: ALC 1024\n@.n: FRD .f .buf 1024\n@.c: FCL .f";
        let graph = Parser::parse(code).expect("Parse failed");
        
        let ops: Vec<Op> = graph.nodes.iter()
            .filter_map(|n| if let Node::Operation { op, .. } = n { Some(op.clone()) } else { None })
            .collect();
        
        assert!(ops.contains(&Op::Fop));
        assert!(ops.contains(&Op::Frd));
        assert!(ops.contains(&Op::Fcl));
    }

    #[test]
    fn test_parse_v1_0_network_ops() {
        let code = "@.sock: TCP\n@.bind: BND .sock .addr\n@.listen: LST .sock 10";
        let graph = Parser::parse(code).expect("Parse failed");
        
        let ops: Vec<Op> = graph.nodes.iter()
            .filter_map(|n| if let Node::Operation { op, .. } = n { Some(op.clone()) } else { None })
            .collect();
        
        assert!(ops.contains(&Op::Tcp));
        assert!(ops.contains(&Op::Bnd));
        assert!(ops.contains(&Op::Lst));
    }

    // ============================================
    // v1.0 Parsing Tests - Control Flow
    // ============================================

    #[test]
    fn test_parse_v1_0_label() {
        let code = ":loop\n.x: 42";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Label(name) = &graph.nodes[0] {
            assert_eq!(name, "loop");
        } else {
            panic!("Expected Label node");
        }
    }

    #[test]
    fn test_parse_v1_0_jmp() {
        let code = ":start\n.x: 42\n@.j: JMP :start";
        let graph = Parser::parse(code).expect("Parse failed");
        
        let has_jmp = graph.nodes.iter().any(|n| {
            if let Node::Operation { op, .. } = n {
                *op == Op::Jmp
            } else {
                false
            }
        });
        assert!(has_jmp);
    }

    #[test]
    fn test_parse_v1_0_jz() {
        let code = ".cond: 0\n@.j: JZ .cond :end\n:end";
        let graph = Parser::parse(code).expect("Parse failed");
        
        let has_jz = graph.nodes.iter().any(|n| {
            if let Node::Operation { op, .. } = n {
                *op == Op::Jz
            } else {
                false
            }
        });
        assert!(has_jz);
    }

    // ============================================
    // v1.0 Parsing Tests - USE Directive
    // ============================================

    #[test]
    fn test_parse_v1_0_use_single() {
        let code = "USE http\n.x: 42";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Use { module } = &graph.nodes[0] {
            assert_eq!(module, "http");
        } else {
            panic!("Expected Use node");
        }
    }

    #[test]
    fn test_parse_v1_0_use_multiple() {
        let code = "USE http\nUSE json\nUSE sql\n.x: 42";
        let graph = Parser::parse(code).expect("Parse failed");
        
        let uses = graph.get_uses();
        assert_eq!(uses.len(), 3);
        assert!(uses.contains(&"http".to_string()));
        assert!(uses.contains(&"json".to_string()));
        assert!(uses.contains(&"sql".to_string()));
    }

    #[test]
    fn test_parse_v1_0_use_case_insensitive() {
        let code = "USE HTTP\n.x: 42";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Use { module } = &graph.nodes[0] {
            assert_eq!(module, "http"); // Should be lowercased
        }
    }

    // ============================================
    // v1.0 Parsing Tests - REQUIRES
    // ============================================

    #[test]
    fn test_parse_v1_0_requires_brace_syntax() {
        let code = ".caps: REQUIRES { http, json, sql }\n.x: 42";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Requires { capabilities, .. } = &graph.nodes[0] {
            assert_eq!(capabilities.len(), 3);
        } else {
            panic!("Expected Requires node");
        }
    }

    #[test]
    fn test_parse_v1_0_requires_space_syntax() {
        let code = ".caps: REQ http json\n.x: 42";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Requires { capabilities, .. } = &graph.nodes[0] {
            assert_eq!(capabilities.len(), 2);
        } else {
            panic!("Expected Requires node");
        }
    }

    // ============================================
    // v1.0 Parsing Tests - Functions
    // ============================================

    #[test]
    fn test_parse_v1_0_function_def() {
        let code = ".add: FUN .a .b\n@.sum: ADD .a .b\n!FUN";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Function { id, params, body } = &graph.nodes[0] {
            assert_eq!(id, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected Function node");
        }
    }

    // ============================================
    // v1.0 Parsing Tests - Comments
    // ============================================

    #[test]
    fn test_parse_v1_0_comment_line() {
        let code = "-- This is a comment\n.x: 42";
        let graph = Parser::parse(code).expect("Parse failed");
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_parse_v1_0_comment_inline() {
        let code = ".x: 42 -- inline comment";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Literal { value, .. } = &graph.nodes[0] {
            if let Value::Int(n) = value {
                assert_eq!(*n, 42);
            }
        }
    }

    #[test]
    fn test_parse_v1_0_comment_in_string() {
        let code = ".msg: \"Hello -- not a comment\"";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Literal { value, .. } = &graph.nodes[0] {
            if let Value::String(s) = value {
                assert_eq!(s, "Hello -- not a comment");
            }
        }
    }

    // ============================================
    // v1.0 Parsing Tests - Error Cases
    // ============================================

    #[test]
    fn test_parse_v1_0_error_invalid_syntax() {
        let code = "invalid syntax here";
        let result = Parser::parse(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_v1_0_error_unclosed_function() {
        let code = ".f: FUN .x\n@.y: ADD .x 1";
        let result = Parser::parse(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_v1_0_error_use_empty() {
        let code = "USE \n.x: 42";
        let result = Parser::parse(code);
        assert!(result.is_err());
    }

    // ============================================
    // v1.0 Parsing Tests - Complex Programs
    // ============================================

    #[test]
    fn test_parse_v1_0_fibonacci() {
        let code = r#"
.n: 10
.a: 0
.b: 1
:loop
@.temp: .b
@.b: ADD .a .b
@.a: .temp
@.n: SUB .n 1
@.done: EQ .n 0
@.j: JZ .done :loop
"#;
        let graph = Parser::parse(code).expect("Parse failed");
        assert!(graph.nodes.len() > 5);
    }

    #[test]
    fn test_parse_v1_0_http_server() {
        let code = r#"
USE http
.caps: REQUIRES { http }
@.server: HTTP.LISTEN 8080
:accept_loop
@.req: HTTP.ACCEPT .server
@.path: HTTP.PATH .req
@.resp: HTTP.RESPOND .req 200 "Hello"
@.j: JMP :accept_loop
"#;
        let graph = Parser::parse(code).expect("Parse failed");
        let uses = graph.get_uses();
        assert!(uses.contains(&"http".to_string()));
    }

    // ============================================
    // Stdlib Operations Tests
    // ============================================

    #[test]
    fn test_parse_stdlib_tier0_ops() {
        let ops = vec![
            "@.ts: TIME.NOW",
            "@.sleep: TIME.SLEEP 1000",
            "@.log: LOG .msg .len",
            "@.route: ROUTE .path .handler",
            "@.env: GETENV .name .buf",
        ];
        
        for op_code in ops {
            let code = format!(".msg: \"test\"\n.len: 4\n.name: \"PATH\"\n.buf: ALC 256\n.path: \"/\"\n.handler: 0\n{}", op_code);
            let result = Parser::parse(&code);
            assert!(result.is_ok(), "Failed to parse: {}", op_code);
        }
    }

    #[test]
    fn test_parse_stdlib_tier1_ops() {
        let ops = vec![
            "@.joined: PATH.JOIN .dst .a .b",
            "@.dir: PATH.DIR .dst .src",
            "@.base: PATH.BASE .dst .src",
            "@.ext: PATH.EXT .dst .src",
            "@.md5: HASH.MD5 .dst .src .len",
            "@.sha: HASH.SHA256 .dst .src .len",
            "@.year: TIME.YEAR .ts",
            "@.month: TIME.MONTH .ts",
        ];
        
        for op_code in ops {
            let code = format!(".dst: ALC 256\n.src: \"test\"\n.a: \"a\"\n.b: \"b\"\n.len: 4\n.ts: 0\n{}", op_code);
            let result = Parser::parse(&code);
            assert!(result.is_ok(), "Failed to parse: {}", op_code);
        }
    }

    #[test]
    fn test_parse_stdlib_tier2_ops() {
        let ops = vec![
            "@.csv: CSV.PARSE .str .len",
            "@.yaml: YAML.PARSE .str .len",
            "@.xml: XML.PARSE .str .len",
            "@.enc: AES.ENCRYPT .key .data .len .out",
            "@.dec: AES.DECRYPT .key .data .len .out",
            "@.ws: WS.CONNECT .url .len",
            "@.tls: TLS.CONNECT .host .port",
            "@.pg: PG.CONNECT .connstr",
            "@.redis: REDIS.CONNECT .host .port",
        ];
        
        for op_code in ops {
            let code = format!(".str: \"test\"\n.len: 4\n.key: ALC 32\n.data: ALC 256\n.out: ALC 256\n.url: \"ws://\"\n.host: \"localhost\"\n.port: 5432\n.connstr: \"host=localhost\"\n{}", op_code);
            let result = Parser::parse(&code);
            assert!(result.is_ok(), "Failed to parse: {}", op_code);
        }
    }

    // ============================================
    // Threading Operations Tests
    // ============================================

    #[test]
    fn test_parse_threading_ops() {
        let ops = vec![
            "@.t: THR .func .arg",
            "@.j: JON .t",
            "@.m: MTX",
            "@.l: LCK .m",
            "@.u: ULK .m",
            "@.ch: CHN 10",
            "@.s: CHN.SND .ch .val",
            "@.r: CHN.RCV .ch",
            "@.c: CHN.CLS .ch",
        ];
        
        for op_code in ops {
            let code = format!(".func: 0\n.arg: 0\n.t: 0\n.m: 0\n.ch: 0\n.val: 42\n{}", op_code);
            let result = Parser::parse(&code);
            assert!(result.is_ok(), "Failed to parse: {}", op_code);
        }
    }

    #[test]
    fn test_parse_atomic_ops() {
        let ops = vec![
            "@.v: ATM.LD .ptr",
            "@.s: ATM.ST .ptr .val",
            "@.x: ATM.XCHG .ptr .new",
            "@.c: ATM.CAS .ptr .exp .des",
            "@.a: ATM.ADD .ptr .val",
            "@.sub: ATM.SUB .ptr .val",
            "@.f: FNC",
        ];
        
        for op_code in ops {
            let code = format!(".ptr: ALC 8\n.val: 42\n.new: 100\n.exp: 42\n.des: 100\n{}", op_code);
            let result = Parser::parse(&code);
            assert!(result.is_ok(), "Failed to parse: {}", op_code);
        }
    }

    // ============================================
    // GUI Operations Tests
    // ============================================

    #[test]
    fn test_parse_gui_ops() {
        let ops = vec![
            "@.w: GUI.WIN .title .tlen 800 600",
            "@.s: GUI.SHOW .w",
            "@.h: GUI.HIDE .w",
            "@.e: GUI.EVENT",
            "@.r: GUI.RUN",
            "@.l: GUI.LABEL .w .text .tlen 10 10 100 20",
            "@.t: GUI.TEXTBOX .w 10 40 200 25",
            "@.b: GUI.BUTTON .w .text .tlen 10 70 100 30",
            "@.g: GUI.GETVAL .w .buf 256",
            "@.sv: GUI.SETVAL .w .text .tlen",
            "@.m: GUI.MSGBOX .title .tlen .text .tlen",
        ];
        
        for op_code in ops {
            let code = format!(".title: \"Window\"\n.tlen: 6\n.text: \"Hello\"\n.w: 0\n.buf: ALC 256\n{}", op_code);
            let result = Parser::parse(&code);
            assert!(result.is_ok(), "Failed to parse: {}", op_code);
        }
    }

    // ============================================
    // JSON Operations Tests
    // ============================================

    #[test]
    fn test_parse_json_ops() {
        let code = r#"
.input: "{\"key\":\"value\"}"
@.obj: JSON.PARSE .input
@.val: JSON.GET .obj .key
@.set: JSON.SET .obj .key .newval
@.out: JSON.ENCODE .obj
"#;
        let graph = Parser::parse(code).expect("Parse failed");
        
        let ops: Vec<Op> = graph.nodes.iter()
            .filter_map(|n| if let Node::Operation { op, .. } = n { Some(op.clone()) } else { None })
            .collect();
        
        assert!(ops.contains(&Op::JsonParse));
        assert!(ops.contains(&Op::JsonGet));
        assert!(ops.contains(&Op::JsonSet));
        assert!(ops.contains(&Op::JsonEncode));
    }

    // ============================================
    // Capability System Tests
    // ============================================

    #[test]
    fn test_parse_capability_http_server() {
        let code = ".caps: REQUIRES { http:server }";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Requires { capabilities, .. } = &graph.nodes[0] {
            assert!(capabilities.contains(&Capability::HttpServer));
        }
    }

    #[test]
    fn test_parse_capability_http_client() {
        let code = ".caps: REQUIRES { http:client }";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Requires { capabilities, .. } = &graph.nodes[0] {
            assert!(capabilities.contains(&Capability::HttpClient));
        }
    }

    #[test]
    fn test_parse_capability_filesystem() {
        let code = ".caps: REQUIRES { filesystem }";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Requires { capabilities, .. } = &graph.nodes[0] {
            assert!(capabilities.contains(&Capability::FileSystem));
        }
    }

    #[test]
    fn test_parse_capability_threading() {
        let code = ".caps: REQUIRES { threading }";
        let graph = Parser::parse(code).expect("Parse failed");
        
        if let Node::Requires { capabilities, .. } = &graph.nodes[0] {
            assert!(capabilities.contains(&Capability::Threading));
        }
    }

    // ============================================
    // Redis Operations Tests
    // ============================================

    #[test]
    fn test_parse_redis_connect() {
        let code = r#"
.host: "127.0.0.1"
.port: 6379
@.redis: REDIS.CONNECT .host .port
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse REDIS.CONNECT");
    }

    #[test]
    fn test_parse_redis_set() {
        let code = r#"
.redis: 1
.key: "mykey"
.val: "myvalue"
@.ok: REDIS.SET .redis .key 5 .val 7
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse REDIS.SET");
    }

    #[test]
    fn test_parse_redis_get() {
        let code = r#"
.redis: 1
.key: "mykey"
.buf: ALC 256
@.len: REDIS.GET .redis .buf .key 5
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse REDIS.GET");
    }

    #[test]
    fn test_parse_redis_del() {
        let code = r#"
.redis: 1
.key: "mykey"
@.deleted: REDIS.DEL .redis .key 5
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse REDIS.DEL");
    }

    #[test]
    fn test_parse_redis_close() {
        let code = r#"
.redis: 1
@.closed: REDIS.CLOSE .redis
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse REDIS.CLOSE");
    }

    // ============================================
    // Hash Operations Tests
    // ============================================

    #[test]
    fn test_parse_hash_md5() {
        let code = r#"
.data: "Hello, World!"
.hash: ALC 33
@.len: HASH.MD5 .hash .data 13
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse HASH.MD5");
    }

    #[test]
    fn test_parse_hash_sha256() {
        let code = r#"
.data: "Hello, World!"
.hash: ALC 65
@.len: HASH.SHA256 .hash .data 13
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse HASH.SHA256");
    }

    // ============================================
    // TLS Operations Tests
    // ============================================

    #[test]
    fn test_parse_tls_connect() {
        let code = r#"
.host: "example.com"
.port: 443
@.tls: TLS.CONNECT .host .port
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse TLS.CONNECT");
    }

    #[test]
    fn test_parse_tls_send_recv() {
        let code = r#"
.tls: 1
.data: "GET / HTTP/1.1\r\n"
.buf: ALC 4096
@.sent: TLS.SEND .tls .data 16
@.recv: TLS.RECV .tls .buf 4096
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse TLS.SEND/RECV");
    }

    #[test]
    fn test_parse_tls_close() {
        let code = r#"
.tls: 1
@.closed: TLS.CLOSE .tls
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse TLS.CLOSE");
    }

    // ============================================
    // WebSocket Operations Tests
    // ============================================

    #[test]
    fn test_parse_websocket_connect() {
        let code = r#"
.url: "ws://localhost:8080/chat"
@.ws: WS.CONNECT .url 25
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse WS.CONNECT");
    }

    #[test]
    fn test_parse_websocket_send_recv() {
        let code = r#"
.ws: 1
.msg: "Hello"
.frame: ALC 256
.recv: ALC 256
@.frame_len: WS.SEND .frame .ws .msg 5
@.recv_len: WS.RECV .recv .ws .frame .frame_len
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse WS.SEND/RECV");
    }

    #[test]
    fn test_parse_websocket_close() {
        let code = r#"
.ws: 1
@.closed: WS.CLOSE .ws
"#;
        let result = Parser::parse(code);
        assert!(result.is_ok(), "Failed to parse WS.CLOSE");
    }
}
