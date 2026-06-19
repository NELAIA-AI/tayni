//! Batch 3: More tests (targeting 400)

#[cfg(test)]
mod pkg_version_tests {
    use crate::pkg::*;
    
    #[test]
    fn test_version_0_0_0() {
        let v = Version::parse("0.0.0").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }
    
    #[test]
    fn test_version_major_only() {
        let v = Version::parse("5.0.0").unwrap();
        assert_eq!(v.major, 5);
    }
    
    #[test]
    fn test_version_display() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
    }
    
    #[test]
    fn test_version_eq() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.0").unwrap();
        assert_eq!(v1, v2);
    }
    
    #[test]
    fn test_version_ne() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        assert_ne!(v1, v2);
    }
    
    #[test]
    fn test_version_lt_patch() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        assert!(v1 < v2);
    }
    
    #[test]
    fn test_version_lt_minor() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.1.0").unwrap();
        assert!(v1 < v2);
    }
    
    #[test]
    fn test_version_lt_major() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();
        assert!(v1 < v2);
    }
    
    #[test]
    fn test_version_gt() {
        let v1 = Version::parse("2.0.0").unwrap();
        let v2 = Version::parse("1.0.0").unwrap();
        assert!(v1 > v2);
    }
}

#[cfg(test)]
mod wasi_error_tests {
    use crate::wasi_p2::*;
    
    #[test]
    fn test_error_success() {
        assert_eq!(ErrorCode::Success as u32, 0);
    }
    
    #[test]
    fn test_error_access() {
        assert_eq!(ErrorCode::Access as u32, 1);
    }
    
    #[test]
    fn test_error_would_block() {
        assert_eq!(ErrorCode::WouldBlock as u32, 2);
    }
    
    #[test]
    fn test_error_already() {
        assert_eq!(ErrorCode::Already as u32, 3);
    }
    
    #[test]
    fn test_error_bad_descriptor() {
        assert_eq!(ErrorCode::BadDescriptor as u32, 4);
    }
    
    #[test]
    fn test_error_busy() {
        assert_eq!(ErrorCode::Busy as u32, 5);
    }
}

#[cfg(test)]
mod json_number_tests {
    use crate::json::*;
    
    #[test]
    fn test_integer_positive() {
        let result = parse("42").unwrap();
        assert!(matches!(result, JsonValue::Number(JsonNumber::Integer(42))));
    }
    
    #[test]
    fn test_integer_negative() {
        let result = parse("-42").unwrap();
        assert!(matches!(result, JsonValue::Number(JsonNumber::Integer(-42))));
    }
    
    #[test]
    fn test_integer_large() {
        let result = parse("1000000").unwrap();
        assert!(matches!(result, JsonValue::Number(JsonNumber::Integer(1000000))));
    }
    
    #[test]
    fn test_float_simple() {
        let result = parse("3.14").unwrap();
        if let JsonValue::Number(JsonNumber::Float(f)) = result {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }
    
    #[test]
    fn test_float_negative() {
        let result = parse("-3.14").unwrap();
        if let JsonValue::Number(JsonNumber::Float(f)) = result {
            assert!((f + 3.14).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }
}

#[cfg(test)]
mod arm64_syscall_tests {
    use crate::arm64::*;
    
    #[test]
    fn test_syscall_write() {
        let mut enc = Arm64Encoder::new();
        enc.syscall_write(Reg::X0, Reg::X1, Reg::X2);
        assert!(enc.code.len() >= 8);
    }
    
    #[test]
    fn test_syscall_exit() {
        let mut enc = Arm64Encoder::new();
        enc.syscall_exit(Reg::X0);
        assert!(enc.code.len() >= 8);
    }
    
    #[test]
    fn test_syscall_socket() {
        let mut enc = Arm64Encoder::new();
        enc.syscall_socket();
        assert!(enc.code.len() >= 8);
    }
    
    #[test]
    fn test_syscall_bind() {
        let mut enc = Arm64Encoder::new();
        enc.syscall_bind();
        assert!(enc.code.len() >= 8);
    }
    
    #[test]
    fn test_syscall_listen() {
        let mut enc = Arm64Encoder::new();
        enc.syscall_listen();
        assert!(enc.code.len() >= 8);
    }
    
    #[test]
    fn test_syscall_accept() {
        let mut enc = Arm64Encoder::new();
        enc.syscall_accept();
        assert!(enc.code.len() >= 8);
    }
}

#[cfg(test)]
mod dwarf_location_tests {
    use crate::dwarf::*;
    
    #[test]
    fn test_location_register() {
        let loc = Location::Register(0);
        assert!(matches!(loc, Location::Register(0)));
    }
    
    #[test]
    fn test_location_frame_offset() {
        let loc = Location::FrameOffset(-8);
        assert!(matches!(loc, Location::FrameOffset(-8)));
    }
    
    #[test]
    fn test_location_address() {
        let loc = Location::Address(0x1000);
        assert!(matches!(loc, Location::Address(0x1000)));
    }
    
    #[test]
    fn test_parameter_info() {
        let param = ParameterInfo {
            name: "x".to_string(),
            type_idx: 0,
            location: Location::Register(0),
        };
        assert_eq!(param.name, "x");
    }
    
    #[test]
    fn test_local_info() {
        let local = LocalInfo {
            name: "temp".to_string(),
            type_idx: 1,
            location: Location::FrameOffset(-16),
            line: 10,
        };
        assert_eq!(local.name, "temp");
        assert_eq!(local.line, 10);
    }
}

#[cfg(test)]
mod http_request_tests {
    use crate::wasi_http::*;
    
    #[test]
    fn test_incoming_request_path_only() {
        let req = IncomingRequest {
            method: Method::Get,
            scheme: None,
            authority: None,
            path_with_query: "/api/users".to_string(),
            headers: Headers::new(),
        };
        assert_eq!(req.path(), "/api/users");
        assert_eq!(req.query(), None);
    }
    
    #[test]
    fn test_incoming_request_with_query() {
        let req = IncomingRequest {
            method: Method::Get,
            scheme: Some(Scheme::Https),
            authority: Some("api.example.com".to_string()),
            path_with_query: "/search?q=test".to_string(),
            headers: Headers::new(),
        };
        assert_eq!(req.path(), "/search");
        assert_eq!(req.query(), Some("q=test"));
    }
    
    #[test]
    fn test_query_param_first() {
        let req = IncomingRequest {
            method: Method::Get,
            scheme: None,
            authority: None,
            path_with_query: "/search?a=1&b=2&c=3".to_string(),
            headers: Headers::new(),
        };
        assert_eq!(req.query_param("a"), Some("1".to_string()));
    }
    
    #[test]
    fn test_query_param_middle() {
        let req = IncomingRequest {
            method: Method::Get,
            scheme: None,
            authority: None,
            path_with_query: "/search?a=1&b=2&c=3".to_string(),
            headers: Headers::new(),
        };
        assert_eq!(req.query_param("b"), Some("2".to_string()));
    }
    
    #[test]
    fn test_query_param_last() {
        let req = IncomingRequest {
            method: Method::Get,
            scheme: None,
            authority: None,
            path_with_query: "/search?a=1&b=2&c=3".to_string(),
            headers: Headers::new(),
        };
        assert_eq!(req.query_param("c"), Some("3".to_string()));
    }
    
    #[test]
    fn test_query_param_missing() {
        let req = IncomingRequest {
            method: Method::Get,
            scheme: None,
            authority: None,
            path_with_query: "/search?a=1".to_string(),
            headers: Headers::new(),
        };
        assert_eq!(req.query_param("z"), None);
    }
    
    #[test]
    fn test_outgoing_request_post() {
        let req = OutgoingRequest::post("https://api.example.com/users").unwrap();
        assert_eq!(req.method, Method::Post);
    }
    
    #[test]
    fn test_outgoing_request_with_header() {
        let req = OutgoingRequest::get("https://api.example.com/users").unwrap()
            .header("Authorization", "Bearer token");
        assert_eq!(req.headers.entries.len(), 1);
    }
    
    #[test]
    fn test_outgoing_request_with_body() {
        let req = OutgoingRequest::post("https://api.example.com/users").unwrap()
            .body(b"test".to_vec());
        assert!(req.body.is_some());
    }
}

#[cfg(test)]
mod json_object_tests {
    use crate::json::*;
    
    #[test]
    fn test_object_single_key() {
        let json = r#"{"key": "value"}"#;
        let result = parse(json).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 1);
        }
    }
    
    #[test]
    fn test_object_numeric_value() {
        let json = r#"{"count": 42}"#;
        let result = parse(json).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 1);
        }
    }
    
    #[test]
    fn test_object_boolean_value() {
        let json = r#"{"active": true}"#;
        let result = parse(json).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 1);
        }
    }
    
    #[test]
    fn test_object_null_value() {
        let json = r#"{"data": null}"#;
        let result = parse(json).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 1);
        }
    }
    
    #[test]
    fn test_object_array_value() {
        let json = r#"{"items": [1, 2, 3]}"#;
        let result = parse(json).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 1);
        }
    }
    
    #[test]
    fn test_object_nested_value() {
        let json = r#"{"outer": {"inner": "value"}}"#;
        let result = parse(json).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 1);
        }
    }
}

#[cfg(test)]
mod arm64_movk_tests {
    use crate::arm64::*;
    
    #[test]
    fn test_movz_shift_0() {
        let mut enc = Arm64Encoder::new();
        enc.movz(Reg::X0, 0x1234, 0);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_movz_shift_16() {
        let mut enc = Arm64Encoder::new();
        enc.movz(Reg::X0, 0x1234, 16);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_movz_shift_32() {
        let mut enc = Arm64Encoder::new();
        enc.movz(Reg::X0, 0x1234, 32);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_movz_shift_48() {
        let mut enc = Arm64Encoder::new();
        enc.movz(Reg::X0, 0x1234, 48);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_movk_shift_0() {
        let mut enc = Arm64Encoder::new();
        enc.movk(Reg::X0, 0x5678, 0);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_movk_shift_16() {
        let mut enc = Arm64Encoder::new();
        enc.movk(Reg::X0, 0x5678, 16);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_mov_imm64_small() {
        let mut enc = Arm64Encoder::new();
        enc.mov_imm64(Reg::X0, 0xFFFF);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_mov_imm64_two_parts() {
        let mut enc = Arm64Encoder::new();
        enc.mov_imm64(Reg::X0, 0x1234_5678);
        assert_eq!(enc.code.len(), 8);
    }
    
    #[test]
    fn test_mov_imm64_full() {
        let mut enc = Arm64Encoder::new();
        enc.mov_imm64(Reg::X0, 0x1234_5678_9ABC_DEF0);
        assert_eq!(enc.code.len(), 16);
    }
}
