//! Batch 2: More tests for TAYNI (targeting 400 total)

#[cfg(test)]
mod wasm_format_tests {
    use crate::target::format::wasm::*;
    
    #[test]
    fn test_wasm_magic() {
        assert_eq!(WASM_MAGIC, [0x00, 0x61, 0x73, 0x6D]);
    }
    
    #[test]
    fn test_wasm_version() {
        assert_eq!(WASM_VERSION, [0x01, 0x00, 0x00, 0x00]);
    }
    
    #[test]
    fn test_section_ids() {
        assert_eq!(SECTION_TYPE, 0x01);
        assert_eq!(SECTION_IMPORT, 0x02);
        assert_eq!(SECTION_FUNCTION, 0x03);
        assert_eq!(SECTION_MEMORY, 0x05);
        assert_eq!(SECTION_EXPORT, 0x07);
        assert_eq!(SECTION_CODE, 0x0A);
        assert_eq!(SECTION_DATA, 0x0B);
    }
    
    #[test]
    fn test_value_types() {
        assert_eq!(TYPE_I32, 0x7F);
        assert_eq!(TYPE_I64, 0x7E);
        assert_eq!(TYPE_F32, 0x7D);
        assert_eq!(TYPE_F64, 0x7C);
    }
    
    #[test]
    fn test_opcodes() {
        assert_eq!(OP_END, 0x0B);
        assert_eq!(OP_CALL, 0x10);
        assert_eq!(OP_LOCAL_GET, 0x20);
        assert_eq!(OP_LOCAL_SET, 0x21);
        assert_eq!(OP_I32_CONST, 0x41);
        assert_eq!(OP_I64_CONST, 0x42);
    }
    
    #[test]
    fn test_uleb128_zero() {
        assert_eq!(encode_uleb128(0), vec![0]);
    }
    
    #[test]
    fn test_uleb128_small() {
        assert_eq!(encode_uleb128(1), vec![1]);
        assert_eq!(encode_uleb128(127), vec![127]);
    }
    
    #[test]
    fn test_uleb128_medium() {
        assert_eq!(encode_uleb128(128), vec![0x80, 0x01]);
        assert_eq!(encode_uleb128(255), vec![0xFF, 0x01]);
    }
    
    #[test]
    fn test_uleb128_large() {
        assert_eq!(encode_uleb128(16384), vec![0x80, 0x80, 0x01]);
    }
    
    #[test]
    fn test_sleb128_zero() {
        assert_eq!(encode_sleb128(0), vec![0]);
    }
    
    #[test]
    fn test_sleb128_positive() {
        assert_eq!(encode_sleb128(1), vec![1]);
        assert_eq!(encode_sleb128(63), vec![63]);
    }
    
    #[test]
    fn test_sleb128_negative() {
        assert_eq!(encode_sleb128(-1), vec![0x7F]);
        assert_eq!(encode_sleb128(-64), vec![0x40]);
    }
    
    #[test]
    fn test_encode_string_empty() {
        let encoded = encode_string("");
        assert_eq!(encoded, vec![0]);
    }
    
    #[test]
    fn test_encode_string_short() {
        let encoded = encode_string("hi");
        assert_eq!(encoded, vec![2, b'h', b'i']);
    }
    
    #[test]
    fn test_encode_section() {
        let data = vec![1, 2, 3];
        let section = encode_section(0x01, &data);
        assert_eq!(section[0], 0x01);
        assert_eq!(section[1], 3);
    }
}

#[cfg(test)]
mod json_stress_tests {
    use crate::json::*;
    
    #[test]
    fn test_deeply_nested_object() {
        let json = r#"{"a":{"b":{"c":{"d":{"e":1}}}}}"#;
        assert!(parse(json).is_ok());
    }
    
    #[test]
    fn test_deeply_nested_array() {
        let json = "[[[[[[1]]]]]]";
        assert!(parse(json).is_ok());
    }
    
    #[test]
    fn test_large_array() {
        let json = format!("[{}]", (0..100).map(|i| i.to_string()).collect::<Vec<_>>().join(","));
        let result = parse(&json).unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 100);
        }
    }
    
    #[test]
    fn test_whitespace_variations() {
        let json = "  {  \"key\"  :  \"value\"  }  ";
        assert!(parse(json).is_ok());
    }
    
    #[test]
    fn test_newlines_in_json() {
        let json = "{\n\"key\"\n:\n\"value\"\n}";
        assert!(parse(json).is_ok());
    }
    
    #[test]
    fn test_tabs_in_json() {
        let json = "{\t\"key\"\t:\t\"value\"\t}";
        assert!(parse(json).is_ok());
    }
    
    #[test]
    fn test_empty_string_value() {
        let json = r#"{"key":""}"#;
        assert!(parse(json).is_ok());
    }
    
    #[test]
    fn test_number_zero() {
        assert!(parse("0").is_ok());
    }
    
    #[test]
    fn test_number_negative_zero() {
        assert!(parse("-0").is_ok());
    }
    
    #[test]
    fn test_number_with_exponent() {
        assert!(parse("1e10").is_ok());
        assert!(parse("1E10").is_ok());
        assert!(parse("1e+10").is_ok());
        assert!(parse("1e-10").is_ok());
    }
    
    #[test]
    fn test_boolean_true() {
        let result = parse("true").unwrap();
        assert!(matches!(result, JsonValue::Bool(true)));
    }
    
    #[test]
    fn test_boolean_false() {
        let result = parse("false").unwrap();
        assert!(matches!(result, JsonValue::Bool(false)));
    }
    
    #[test]
    fn test_null_value() {
        let result = parse("null").unwrap();
        assert!(matches!(result, JsonValue::Null));
    }
    
    #[test]
    fn test_string_with_backslash() {
        let json = r#""path\\to\\file""#;
        assert!(parse(json).is_ok());
    }
    
    #[test]
    fn test_string_with_quote() {
        let json = r#""say \"hello\"""#;
        assert!(parse(json).is_ok());
    }
    
    #[test]
    fn test_object_multiple_keys() {
        let json = r#"{"a":1,"b":2,"c":3,"d":4,"e":5}"#;
        let result = parse(json).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 5);
        }
    }
    
    #[test]
    fn test_mixed_types_in_array() {
        let json = r#"[1, "two", true, null, 3.14, {"nested": "obj"}, [1,2,3]]"#;
        let result = parse(json).unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 7);
        }
    }
}

#[cfg(test)]
mod arm64_instruction_tests {
    use crate::arm64::*;
    
    #[test]
    fn test_all_argument_registers() {
        assert_eq!(Reg::X0.0, 0);
        assert_eq!(Reg::X1.0, 1);
        assert_eq!(Reg::X2.0, 2);
        assert_eq!(Reg::X3.0, 3);
        assert_eq!(Reg::X4.0, 4);
        assert_eq!(Reg::X5.0, 5);
        assert_eq!(Reg::X6.0, 6);
        assert_eq!(Reg::X7.0, 7);
    }
    
    #[test]
    fn test_temp_registers() {
        assert_eq!(Reg::X9.0, 9);
        assert_eq!(Reg::X10.0, 10);
        assert_eq!(Reg::X15.0, 15);
    }
    
    #[test]
    fn test_callee_saved_registers() {
        assert_eq!(Reg::X19.0, 19);
        assert_eq!(Reg::X28.0, 28);
    }
    
    #[test]
    fn test_special_registers() {
        assert_eq!(Reg::X8.0, 8);   // syscall number
        assert_eq!(Reg::X16.0, 16); // IP0
        assert_eq!(Reg::X17.0, 17); // IP1
    }
    
    #[test]
    fn test_condition_codes() {
        assert_eq!(Cond::Eq as u8, 0b0000);
        assert_eq!(Cond::Ne as u8, 0b0001);
        assert_eq!(Cond::Lt as u8, 0b1011);
        assert_eq!(Cond::Ge as u8, 0b1010);
        assert_eq!(Cond::Al as u8, 0b1110);
    }
    
    #[test]
    fn test_shift_types() {
        assert_eq!(Shift::Lsl as u8, 0b00);
        assert_eq!(Shift::Lsr as u8, 0b01);
        assert_eq!(Shift::Asr as u8, 0b10);
        assert_eq!(Shift::Ror as u8, 0b11);
    }
    
    #[test]
    fn test_add_imm() {
        let mut enc = Arm64Encoder::new();
        enc.add_imm(Reg::X0, Reg::X1, 100);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_sub_imm() {
        let mut enc = Arm64Encoder::new();
        enc.sub_imm(Reg::X0, Reg::X1, 100);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_sdiv() {
        let mut enc = Arm64Encoder::new();
        enc.sdiv(Reg::X0, Reg::X1, Reg::X2);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_udiv() {
        let mut enc = Arm64Encoder::new();
        enc.udiv(Reg::X0, Reg::X1, Reg::X2);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_mov_reg() {
        let mut enc = Arm64Encoder::new();
        enc.mov_reg(Reg::X0, Reg::X1);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_cmp_reg() {
        let mut enc = Arm64Encoder::new();
        enc.cmp_reg(Reg::X0, Reg::X1);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_cmp_imm() {
        let mut enc = Arm64Encoder::new();
        enc.cmp_imm(Reg::X0, 42);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_ldr() {
        let mut enc = Arm64Encoder::new();
        enc.ldr(Reg::X0, Reg::SP, 0);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_str() {
        let mut enc = Arm64Encoder::new();
        enc.str(Reg::X0, Reg::SP, 8);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_ldrb() {
        let mut enc = Arm64Encoder::new();
        enc.ldrb(Reg::X0, Reg::X1, 0);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_strb() {
        let mut enc = Arm64Encoder::new();
        enc.strb(Reg::X0, Reg::X1, 1);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_stp_pre() {
        let mut enc = Arm64Encoder::new();
        enc.stp_pre(Reg::FP, Reg::LR, Reg::SP, -16);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_ldp_post() {
        let mut enc = Arm64Encoder::new();
        enc.ldp_post(Reg::FP, Reg::LR, Reg::SP, 16);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_br() {
        let mut enc = Arm64Encoder::new();
        enc.br(Reg::X0);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_blr() {
        let mut enc = Arm64Encoder::new();
        enc.blr(Reg::X0);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_svc() {
        let mut enc = Arm64Encoder::new();
        enc.svc(0);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_nop() {
        let mut enc = Arm64Encoder::new();
        enc.nop();
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_adr() {
        let mut enc = Arm64Encoder::new();
        enc.adr(Reg::X0, 0x100);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_adrp() {
        let mut enc = Arm64Encoder::new();
        enc.adrp(Reg::X0, 0x1000);
        assert_eq!(enc.code.len(), 4);
    }
}

#[cfg(test)]
mod dwarf_section_tests {
    use crate::dwarf::*;
    
    #[test]
    fn test_dwarf_version_4() {
        assert_eq!(DWARF_VERSION_4, 4);
    }
    
    #[test]
    fn test_dwarf_version_5() {
        assert_eq!(DWARF_VERSION_5, 5);
    }
    
    #[test]
    fn test_tag_compile_unit() {
        assert_eq!(DW_TAG_COMPILE_UNIT, 0x11);
    }
    
    #[test]
    fn test_tag_subprogram() {
        assert_eq!(DW_TAG_SUBPROGRAM, 0x2E);
    }
    
    #[test]
    fn test_tag_variable() {
        assert_eq!(DW_TAG_VARIABLE, 0x34);
    }
    
    #[test]
    fn test_tag_base_type() {
        assert_eq!(DW_TAG_BASE_TYPE, 0x24);
    }
    
    #[test]
    fn test_attr_name() {
        assert_eq!(DW_AT_NAME, 0x03);
    }
    
    #[test]
    fn test_attr_low_pc() {
        assert_eq!(DW_AT_LOW_PC, 0x11);
    }
    
    #[test]
    fn test_attr_high_pc() {
        assert_eq!(DW_AT_HIGH_PC, 0x12);
    }
    
    #[test]
    fn test_form_addr() {
        assert_eq!(DW_FORM_ADDR, 0x01);
    }
    
    #[test]
    fn test_form_string() {
        assert_eq!(DW_FORM_STRING, 0x08);
    }
    
    #[test]
    fn test_ate_signed() {
        assert_eq!(DW_ATE_SIGNED, 0x05);
    }
    
    #[test]
    fn test_ate_unsigned() {
        assert_eq!(DW_ATE_UNSIGNED, 0x07);
    }
    
    #[test]
    fn test_ate_float() {
        assert_eq!(DW_ATE_FLOAT, 0x04);
    }
    
    #[test]
    fn test_line_opcodes() {
        assert_eq!(DW_LNS_COPY, 0x01);
        assert_eq!(DW_LNS_ADVANCE_PC, 0x02);
        assert_eq!(DW_LNS_ADVANCE_LINE, 0x03);
    }
    
    #[test]
    fn test_extended_opcodes() {
        assert_eq!(DW_LNE_END_SEQUENCE, 0x01);
        assert_eq!(DW_LNE_SET_ADDRESS, 0x02);
    }
    
    #[test]
    fn test_base_type_u32() {
        let t = BaseType::u32();
        assert_eq!(t.name, "u32");
        assert_eq!(t.byte_size, 4);
        assert_eq!(t.encoding, DW_ATE_UNSIGNED);
    }
    
    #[test]
    fn test_base_type_u64() {
        let t = BaseType::u64();
        assert_eq!(t.name, "u64");
        assert_eq!(t.byte_size, 8);
    }
    
    #[test]
    fn test_base_type_f32() {
        let t = BaseType::f32();
        assert_eq!(t.byte_size, 4);
        assert_eq!(t.encoding, DW_ATE_FLOAT);
    }
    
    #[test]
    fn test_base_type_f64() {
        let t = BaseType::f64();
        assert_eq!(t.byte_size, 8);
    }
    
    #[test]
    fn test_generator_default_types() {
        let dwarf = DwarfGenerator::new();
        assert_eq!(dwarf.types.len(), 7);
    }
    
    #[test]
    fn test_generator_default_version() {
        let dwarf = DwarfGenerator::new();
        assert_eq!(dwarf.version, DWARF_VERSION_4);
    }
    
    #[test]
    fn test_generator_address_size() {
        let dwarf = DwarfGenerator::new();
        assert_eq!(dwarf.address_size, 8);
    }
    
    #[test]
    fn test_line_entry_creation() {
        let entry = LineEntry {
            address: 0x1000,
            file: 1,
            line: 10,
            column: 5,
            is_stmt: true,
            prologue_end: false,
            epilogue_begin: false,
        };
        assert_eq!(entry.address, 0x1000);
        assert_eq!(entry.line, 10);
    }
}

#[cfg(test)]
mod http_types_tests {
    use crate::wasi_http::*;
    
    #[test]
    fn test_method_get() {
        assert_eq!(Method::Get as u8, 0);
        assert_eq!(Method::Get.as_str(), "GET");
    }
    
    #[test]
    fn test_method_post() {
        assert_eq!(Method::Post as u8, 2);
        assert_eq!(Method::Post.as_str(), "POST");
    }
    
    #[test]
    fn test_method_put() {
        assert_eq!(Method::Put.as_str(), "PUT");
    }
    
    #[test]
    fn test_method_delete() {
        assert_eq!(Method::Delete.as_str(), "DELETE");
    }
    
    #[test]
    fn test_method_patch() {
        assert_eq!(Method::Patch.as_str(), "PATCH");
    }
    
    #[test]
    fn test_scheme_http() {
        assert_eq!(Scheme::Http as u8, 0);
    }
    
    #[test]
    fn test_scheme_https() {
        assert_eq!(Scheme::Https as u8, 1);
    }
    
    #[test]
    fn test_status_informational() {
        assert!(matches!(StatusClass::from_code(100), StatusClass::Informational));
        assert!(matches!(StatusClass::from_code(199), StatusClass::Informational));
    }
    
    #[test]
    fn test_status_success() {
        assert!(matches!(StatusClass::from_code(200), StatusClass::Success));
        assert!(matches!(StatusClass::from_code(299), StatusClass::Success));
    }
    
    #[test]
    fn test_status_redirection() {
        assert!(matches!(StatusClass::from_code(301), StatusClass::Redirection));
        assert!(matches!(StatusClass::from_code(302), StatusClass::Redirection));
    }
    
    #[test]
    fn test_status_client_error() {
        assert!(matches!(StatusClass::from_code(400), StatusClass::ClientError));
        assert!(matches!(StatusClass::from_code(404), StatusClass::ClientError));
    }
    
    #[test]
    fn test_status_server_error() {
        assert!(matches!(StatusClass::from_code(500), StatusClass::ServerError));
        assert!(matches!(StatusClass::from_code(503), StatusClass::ServerError));
    }
    
    #[test]
    fn test_header_new() {
        let h = Header::new("Content-Type", "application/json");
        assert_eq!(h.name, "Content-Type");
    }
    
    #[test]
    fn test_header_value_str() {
        let h = Header::new("X-Custom", "value");
        assert_eq!(h.value_str(), Some("value"));
    }
    
    #[test]
    fn test_headers_content_length() {
        let mut headers = Headers::new();
        headers.add("Content-Length", "100");
        assert_eq!(headers.content_length(), Some(100));
    }
    
    #[test]
    fn test_response_not_found() {
        let resp = OutgoingResponse::not_found();
        assert_eq!(resp.status, 404);
    }
    
    #[test]
    fn test_response_bad_request() {
        let resp = OutgoingResponse::bad_request();
        assert_eq!(resp.status, 400);
    }
    
    #[test]
    fn test_response_internal_error() {
        let resp = OutgoingResponse::internal_error();
        assert_eq!(resp.status, 500);
    }
    
    #[test]
    fn test_response_text() {
        let resp = OutgoingResponse::ok().text("hello");
        assert!(resp.body.is_some());
    }
    
    #[test]
    fn test_response_html() {
        let resp = OutgoingResponse::ok().html("<h1>Hi</h1>");
        assert_eq!(resp.headers.content_type(), Some("text/html; charset=utf-8"));
    }
    
    #[test]
    fn test_route_post() {
        let route = Route::post("/api").json("{}").build();
        assert_eq!(route.method, Some(Method::Post));
    }
    
    #[test]
    fn test_route_any() {
        let route = Route::any("/health").text("OK").build();
        assert_eq!(route.method, None);
    }
    
    #[test]
    fn test_route_status() {
        let route = Route::get("/").status(201).build();
        assert_eq!(route.response_status, 201);
    }
}
