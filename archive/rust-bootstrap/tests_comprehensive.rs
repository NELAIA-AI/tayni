//! Additional tests for TAYNI modules

#[cfg(test)]
mod json_extended_tests {
    use crate::json::*;
    
    #[test]
    fn test_parse_empty_object() {
        let result = parse("{}").unwrap();
        assert!(matches!(result, JsonValue::Object(o) if o.is_empty()));
    }
    
    #[test]
    fn test_parse_empty_array() {
        let result = parse("[]").unwrap();
        assert!(matches!(result, JsonValue::Array(a) if a.is_empty()));
    }
    
    #[test]
    fn test_parse_nested_objects() {
        let json = r#"{"a": {"b": {"c": 1}}}"#;
        let result = parse(json).unwrap();
        assert!(matches!(result, JsonValue::Object(_)));
    }
    
    #[test]
    fn test_parse_mixed_array() {
        let json = r#"[1, "two", true, null, 3.14]"#;
        let result = parse(json).unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 5);
        }
    }
    
    #[test]
    fn test_encode_roundtrip() {
        let original = r#"{"name":"test","values":[1,2,3]}"#;
        let parsed = parse(original).unwrap();
        let encoded = encode(&parsed);
        let reparsed = parse(&encoded).unwrap();
        assert_eq!(encode(&parsed), encode(&reparsed));
    }
    
    #[test]
    fn test_parse_error_invalid() {
        assert!(parse("{invalid}").is_err());
    }
    
    #[test]
    fn test_parse_error_unclosed() {
        assert!(parse("[1, 2, 3").is_err());
    }
}

#[cfg(test)]
mod pkg_extended_tests {
    use crate::pkg::*;
    
    #[test]
    fn test_version_parsing() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }
    
    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        assert!(v1 < v2);
    }
}

#[cfg(test)]
mod wasi_http_extended_tests {
    use crate::wasi_http::*;
    
    #[test]
    fn test_method_roundtrip() {
        for i in 0..9u8 {
            if let Some(method) = Method::from_u8(i) {
                assert!(!method.as_str().is_empty());
            }
        }
    }
    
    #[test]
    fn test_status_class() {
        assert!(matches!(StatusClass::from_code(200), StatusClass::Success));
        assert!(matches!(StatusClass::from_code(404), StatusClass::ClientError));
    }
    
    #[test]
    fn test_headers_case_insensitive() {
        let mut headers = Headers::new();
        headers.add("Content-Type", "application/json");
        assert!(headers.get("content-type").is_some());
    }
    
    #[test]
    fn test_query_parsing() {
        let req = IncomingRequest {
            method: Method::Get,
            scheme: Some(Scheme::Https),
            authority: Some("example.com".to_string()),
            path_with_query: "/search?q=test&page=1".to_string(),
            headers: Headers::new(),
        };
        assert_eq!(req.path(), "/search");
        assert_eq!(req.query_param("q"), Some("test".to_string()));
    }
    
    #[test]
    fn test_response_builder() {
        let resp = OutgoingResponse::ok().json(r#"{"ok":true}"#);
        assert_eq!(resp.status, 200);
    }
    
    #[test]
    fn test_request_url_parsing() {
        let req = OutgoingRequest::get("https://api.example.com/users").unwrap();
        assert_eq!(req.scheme, Scheme::Https);
        assert_eq!(req.authority, "api.example.com");
    }
}

#[cfg(test)]
mod arm64_extended_tests {
    use crate::arm64::*;
    
    #[test]
    fn test_registers() {
        assert_eq!(Reg::X0.0, 0);
        assert_eq!(Reg::LR.0, 30);
        assert_eq!(Reg::SP.0, 31);
    }
    
    #[test]
    fn test_arithmetic() {
        let mut enc = Arm64Encoder::new();
        enc.add_reg(Reg::X0, Reg::X1, Reg::X2);
        enc.sub_reg(Reg::X0, Reg::X1, Reg::X2);
        enc.mul_reg(Reg::X0, Reg::X1, Reg::X2);
        assert_eq!(enc.code.len(), 12);
    }
    
    #[test]
    fn test_logical() {
        let mut enc = Arm64Encoder::new();
        enc.and_reg(Reg::X0, Reg::X1, Reg::X2);
        enc.orr_reg(Reg::X0, Reg::X1, Reg::X2);
        enc.eor_reg(Reg::X0, Reg::X1, Reg::X2);
        assert_eq!(enc.code.len(), 12);
    }
    
    #[test]
    fn test_branches() {
        let mut enc = Arm64Encoder::new();
        enc.b(0x100);
        enc.bl(0x200);
        enc.ret();
        assert_eq!(enc.code.len(), 12);
    }
    
    #[test]
    fn test_conditionals() {
        let mut enc = Arm64Encoder::new();
        enc.b_cond(Cond::Eq, 8);
        enc.cbz(Reg::X0, 8);
        enc.cbnz(Reg::X0, 8);
        assert_eq!(enc.code.len(), 12);
    }
}

#[cfg(test)]
mod dwarf_extended_tests {
    use crate::dwarf::*;
    
    #[test]
    fn test_source_file() {
        let file = SourceFile::new("/home/user/src/main.tayni");
        assert_eq!(file.name, "main.tayni");
    }
    
    #[test]
    fn test_base_types() {
        assert_eq!(BaseType::i32().byte_size, 4);
        assert_eq!(BaseType::i64().byte_size, 8);
        assert_eq!(BaseType::bool().byte_size, 1);
    }
    
    #[test]
    fn test_generator_files() {
        let mut dwarf = DwarfGenerator::new();
        let idx = dwarf.add_file("test.tayni");
        assert_eq!(idx, 1);
    }
    
    #[test]
    fn test_generator_functions() {
        let mut dwarf = DwarfGenerator::new();
        dwarf.add_function(FunctionInfo {
            name: "main".to_string(),
            low_pc: 0x1000,
            high_pc: 0x1100,
            file: 1,
            line: 1,
            is_external: true,
            parameters: vec![],
            locals: vec![],
        });
        assert_eq!(dwarf.functions.len(), 1);
    }
    
    #[test]
    fn test_debug_info() {
        let mut dwarf = DwarfGenerator::new();
        dwarf.add_file("test.tayni");
        let info = dwarf.generate_debug_info();
        assert!(info.len() > 10);
    }
    
    #[test]
    fn test_debug_abbrev() {
        let dwarf = DwarfGenerator::new();
        let abbrev = dwarf.generate_debug_abbrev();
        assert_eq!(abbrev[0], 1);
    }
}

#[cfg(test)]
mod wasi_p2_extended_tests {
    use crate::wasi_p2::*;
    
    #[test]
    fn test_error_codes() {
        assert_eq!(ErrorCode::Success as u32, 0);
    }
}
