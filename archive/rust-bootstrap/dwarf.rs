//! DWARF Debug Information Generator for TAYNI
//!
//! Generates DWARF debugging information for ELF binaries.
//! Enables source-level debugging with GDB, LLDB, etc.

// ============================================================================
// DWARF Constants
// ============================================================================

// DWARF versions
pub const DWARF_VERSION_4: u16 = 4;
pub const DWARF_VERSION_5: u16 = 5;

// Unit types (DWARF 5)
pub const DW_UT_COMPILE: u8 = 0x01;

// Tags
pub const DW_TAG_COMPILE_UNIT: u16 = 0x11;
pub const DW_TAG_SUBPROGRAM: u16 = 0x2E;
pub const DW_TAG_VARIABLE: u16 = 0x34;
pub const DW_TAG_FORMAL_PARAMETER: u16 = 0x05;
pub const DW_TAG_BASE_TYPE: u16 = 0x24;
pub const DW_TAG_POINTER_TYPE: u16 = 0x0F;
pub const DW_TAG_ARRAY_TYPE: u16 = 0x01;
pub const DW_TAG_STRUCTURE_TYPE: u16 = 0x13;
pub const DW_TAG_MEMBER: u16 = 0x0D;
pub const DW_TAG_LEXICAL_BLOCK: u16 = 0x0B;

// Attributes
pub const DW_AT_NAME: u16 = 0x03;
pub const DW_AT_STMT_LIST: u16 = 0x10;
pub const DW_AT_LOW_PC: u16 = 0x11;
pub const DW_AT_HIGH_PC: u16 = 0x12;
pub const DW_AT_LANGUAGE: u16 = 0x13;
pub const DW_AT_COMP_DIR: u16 = 0x1B;
pub const DW_AT_PRODUCER: u16 = 0x25;
pub const DW_AT_DECL_FILE: u16 = 0x3A;
pub const DW_AT_DECL_LINE: u16 = 0x3B;
pub const DW_AT_TYPE: u16 = 0x49;
pub const DW_AT_EXTERNAL: u16 = 0x3F;
pub const DW_AT_LOCATION: u16 = 0x02;
pub const DW_AT_BYTE_SIZE: u16 = 0x0B;
pub const DW_AT_ENCODING: u16 = 0x3E;

// Forms
pub const DW_FORM_ADDR: u8 = 0x01;
pub const DW_FORM_DATA1: u8 = 0x0B;
pub const DW_FORM_DATA2: u8 = 0x05;
pub const DW_FORM_DATA4: u8 = 0x06;
pub const DW_FORM_DATA8: u8 = 0x07;
pub const DW_FORM_STRING: u8 = 0x08;
pub const DW_FORM_STRP: u8 = 0x0E;
pub const DW_FORM_FLAG: u8 = 0x0C;
pub const DW_FORM_FLAG_PRESENT: u8 = 0x19;
pub const DW_FORM_REF4: u8 = 0x13;
pub const DW_FORM_SEC_OFFSET: u8 = 0x17;
pub const DW_FORM_EXPRLOC: u8 = 0x18;

// Languages
pub const DW_LANG_C: u16 = 0x0002;
pub const DW_LANG_RUST: u16 = 0x001C;

// Base type encodings
pub const DW_ATE_SIGNED: u8 = 0x05;
pub const DW_ATE_UNSIGNED: u8 = 0x07;
pub const DW_ATE_FLOAT: u8 = 0x04;
pub const DW_ATE_BOOLEAN: u8 = 0x02;
pub const DW_ATE_UTF: u8 = 0x10;

// Location expressions
pub const DW_OP_ADDR: u8 = 0x03;
pub const DW_OP_FBREG: u8 = 0x91;
pub const DW_OP_REG0: u8 = 0x50;
pub const DW_OP_BREG0: u8 = 0x70;

// Line number opcodes
pub const DW_LNS_COPY: u8 = 0x01;
pub const DW_LNS_ADVANCE_PC: u8 = 0x02;
pub const DW_LNS_ADVANCE_LINE: u8 = 0x03;
pub const DW_LNS_SET_FILE: u8 = 0x04;
pub const DW_LNS_SET_COLUMN: u8 = 0x05;
pub const DW_LNS_NEGATE_STMT: u8 = 0x06;
pub const DW_LNS_SET_PROLOGUE_END: u8 = 0x0A;
pub const DW_LNS_SET_EPILOGUE_BEGIN: u8 = 0x0B;

// Extended opcodes
pub const DW_LNE_END_SEQUENCE: u8 = 0x01;
pub const DW_LNE_SET_ADDRESS: u8 = 0x02;

// ============================================================================
// DWARF Data Structures
// ============================================================================

/// Source file information
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: String,
    pub directory: String,
    pub name: String,
}

impl SourceFile {
    pub fn new(path: &str) -> Self {
        let (dir, name) = match path.rfind('/') {
            Some(idx) => (&path[..idx], &path[idx + 1..]),
            None => (".", path),
        };
        SourceFile {
            path: path.to_string(),
            directory: dir.to_string(),
            name: name.to_string(),
        }
    }
}

/// Line number entry
#[derive(Debug, Clone)]
pub struct LineEntry {
    pub address: u64,
    pub file: u32,
    pub line: u32,
    pub column: u32,
    pub is_stmt: bool,
    pub prologue_end: bool,
    pub epilogue_begin: bool,
}

/// Function/subprogram info
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub low_pc: u64,
    pub high_pc: u64,
    pub file: u32,
    pub line: u32,
    pub is_external: bool,
    pub parameters: Vec<ParameterInfo>,
    pub locals: Vec<LocalInfo>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub type_idx: u32,
    pub location: Location,
}

/// Local variable
#[derive(Debug, Clone)]
pub struct LocalInfo {
    pub name: String,
    pub type_idx: u32,
    pub location: Location,
    pub line: u32,
}

/// Variable location
#[derive(Debug, Clone)]
pub enum Location {
    Register(u8),
    FrameOffset(i32),
    Address(u64),
}

/// Base type info
#[derive(Debug, Clone)]
pub struct BaseType {
    pub name: String,
    pub byte_size: u8,
    pub encoding: u8,
}

impl BaseType {
    pub fn i32() -> Self {
        BaseType { name: "i32".to_string(), byte_size: 4, encoding: DW_ATE_SIGNED }
    }
    pub fn i64() -> Self {
        BaseType { name: "i64".to_string(), byte_size: 8, encoding: DW_ATE_SIGNED }
    }
    pub fn u32() -> Self {
        BaseType { name: "u32".to_string(), byte_size: 4, encoding: DW_ATE_UNSIGNED }
    }
    pub fn u64() -> Self {
        BaseType { name: "u64".to_string(), byte_size: 8, encoding: DW_ATE_UNSIGNED }
    }
    pub fn f32() -> Self {
        BaseType { name: "f32".to_string(), byte_size: 4, encoding: DW_ATE_FLOAT }
    }
    pub fn f64() -> Self {
        BaseType { name: "f64".to_string(), byte_size: 8, encoding: DW_ATE_FLOAT }
    }
    pub fn bool() -> Self {
        BaseType { name: "bool".to_string(), byte_size: 1, encoding: DW_ATE_BOOLEAN }
    }
}

// ============================================================================
// DWARF Generator
// ============================================================================

/// DWARF debug info generator
pub struct DwarfGenerator {
    pub version: u16,
    pub address_size: u8,
    pub files: Vec<SourceFile>,
    pub functions: Vec<FunctionInfo>,
    pub types: Vec<BaseType>,
    pub lines: Vec<LineEntry>,
    pub producer: String,
    pub comp_dir: String,
    pub low_pc: u64,
    pub high_pc: u64,
}

impl DwarfGenerator {
    pub fn new() -> Self {
        DwarfGenerator {
            version: DWARF_VERSION_4,
            address_size: 8,
            files: Vec::new(),
            functions: Vec::new(),
            types: vec![
                BaseType::i32(),
                BaseType::i64(),
                BaseType::u32(),
                BaseType::u64(),
                BaseType::f32(),
                BaseType::f64(),
                BaseType::bool(),
            ],
            lines: Vec::new(),
            producer: "TAYNI Compiler 0.1.0".to_string(),
            comp_dir: ".".to_string(),
            low_pc: 0,
            high_pc: 0,
        }
    }
    
    pub fn add_file(&mut self, path: &str) -> u32 {
        let idx = self.files.len() as u32;
        self.files.push(SourceFile::new(path));
        idx + 1 // DWARF file indices are 1-based
    }
    
    pub fn add_function(&mut self, func: FunctionInfo) {
        if self.low_pc == 0 || func.low_pc < self.low_pc {
            self.low_pc = func.low_pc;
        }
        if func.high_pc > self.high_pc {
            self.high_pc = func.high_pc;
        }
        self.functions.push(func);
    }
    
    pub fn add_line(&mut self, entry: LineEntry) {
        self.lines.push(entry);
    }
    
    /// Generate .debug_info section
    pub fn generate_debug_info(&self) -> Vec<u8> {
        let mut info = Vec::new();
        
        // Compile unit header (will fix length later)
        let header_start = info.len();
        info.extend(&0u32.to_le_bytes()); // unit_length placeholder
        info.extend(&self.version.to_le_bytes());
        info.extend(&0u32.to_le_bytes()); // debug_abbrev_offset
        info.push(self.address_size);
        
        let content_start = info.len();
        
        // DIE: Compile unit (abbrev 1)
        info.push(1); // abbrev code
        
        // DW_AT_producer (string)
        info.extend(self.producer.as_bytes());
        info.push(0);
        
        // DW_AT_language (data2)
        info.extend(&DW_LANG_RUST.to_le_bytes());
        
        // DW_AT_name (string)
        if !self.files.is_empty() {
            info.extend(self.files[0].name.as_bytes());
        } else {
            info.extend(b"main.tayni");
        }
        info.push(0);
        
        // DW_AT_comp_dir (string)
        info.extend(self.comp_dir.as_bytes());
        info.push(0);
        
        // DW_AT_low_pc (addr)
        info.extend(&self.low_pc.to_le_bytes());
        
        // DW_AT_high_pc (data8 - offset from low_pc)
        info.extend(&(self.high_pc - self.low_pc).to_le_bytes());
        
        // DW_AT_stmt_list (sec_offset)
        info.extend(&0u32.to_le_bytes());
        
        // Base types (abbrev 2)
        for (idx, ty) in self.types.iter().enumerate() {
            info.push(2); // abbrev code for base_type
            info.extend(ty.name.as_bytes());
            info.push(0);
            info.push(ty.byte_size);
            info.push(ty.encoding);
        }
        
        // Functions (abbrev 3)
        for func in &self.functions {
            info.push(3); // abbrev code for subprogram
            
            // DW_AT_name
            info.extend(func.name.as_bytes());
            info.push(0);
            
            // DW_AT_low_pc
            info.extend(&func.low_pc.to_le_bytes());
            
            // DW_AT_high_pc
            info.extend(&(func.high_pc - func.low_pc).to_le_bytes());
            
            // DW_AT_decl_file
            info.push(func.file as u8);
            
            // DW_AT_decl_line
            info.extend(&(func.line as u16).to_le_bytes());
            
            // DW_AT_external
            info.push(if func.is_external { 1 } else { 0 });
        }
        
        // End of children
        info.push(0);
        
        // Fix unit length
        let unit_length = (info.len() - content_start + 7) as u32;
        info[header_start..header_start + 4].copy_from_slice(&unit_length.to_le_bytes());
        
        info
    }
    
    /// Generate .debug_abbrev section
    pub fn generate_debug_abbrev(&self) -> Vec<u8> {
        let mut abbrev = Vec::new();
        
        // Abbrev 1: Compile unit
        abbrev.push(1); // abbrev code
        abbrev.extend(&encode_uleb128(DW_TAG_COMPILE_UNIT as u64));
        abbrev.push(1); // has children
        
        abbrev.extend(&encode_uleb128(DW_AT_PRODUCER as u64));
        abbrev.push(DW_FORM_STRING);
        abbrev.extend(&encode_uleb128(DW_AT_LANGUAGE as u64));
        abbrev.push(DW_FORM_DATA2);
        abbrev.extend(&encode_uleb128(DW_AT_NAME as u64));
        abbrev.push(DW_FORM_STRING);
        abbrev.extend(&encode_uleb128(DW_AT_COMP_DIR as u64));
        abbrev.push(DW_FORM_STRING);
        abbrev.extend(&encode_uleb128(DW_AT_LOW_PC as u64));
        abbrev.push(DW_FORM_ADDR);
        abbrev.extend(&encode_uleb128(DW_AT_HIGH_PC as u64));
        abbrev.push(DW_FORM_DATA8);
        abbrev.extend(&encode_uleb128(DW_AT_STMT_LIST as u64));
        abbrev.push(DW_FORM_SEC_OFFSET);
        abbrev.push(0); abbrev.push(0); // end attrs
        
        // Abbrev 2: Base type
        abbrev.push(2);
        abbrev.extend(&encode_uleb128(DW_TAG_BASE_TYPE as u64));
        abbrev.push(0); // no children
        
        abbrev.extend(&encode_uleb128(DW_AT_NAME as u64));
        abbrev.push(DW_FORM_STRING);
        abbrev.extend(&encode_uleb128(DW_AT_BYTE_SIZE as u64));
        abbrev.push(DW_FORM_DATA1);
        abbrev.extend(&encode_uleb128(DW_AT_ENCODING as u64));
        abbrev.push(DW_FORM_DATA1);
        abbrev.push(0); abbrev.push(0);
        
        // Abbrev 3: Subprogram
        abbrev.push(3);
        abbrev.extend(&encode_uleb128(DW_TAG_SUBPROGRAM as u64));
        abbrev.push(0); // no children (simplified)
        
        abbrev.extend(&encode_uleb128(DW_AT_NAME as u64));
        abbrev.push(DW_FORM_STRING);
        abbrev.extend(&encode_uleb128(DW_AT_LOW_PC as u64));
        abbrev.push(DW_FORM_ADDR);
        abbrev.extend(&encode_uleb128(DW_AT_HIGH_PC as u64));
        abbrev.push(DW_FORM_DATA8);
        abbrev.extend(&encode_uleb128(DW_AT_DECL_FILE as u64));
        abbrev.push(DW_FORM_DATA1);
        abbrev.extend(&encode_uleb128(DW_AT_DECL_LINE as u64));
        abbrev.push(DW_FORM_DATA2);
        abbrev.extend(&encode_uleb128(DW_AT_EXTERNAL as u64));
        abbrev.push(DW_FORM_FLAG);
        abbrev.push(0); abbrev.push(0);
        
        // End of abbrevs
        abbrev.push(0);
        
        abbrev
    }
    
    /// Generate .debug_line section
    pub fn generate_debug_line(&self) -> Vec<u8> {
        let mut line = Vec::new();
        
        // Header
        let header_start = line.len();
        line.extend(&0u32.to_le_bytes()); // unit_length placeholder
        line.extend(&self.version.to_le_bytes());
        
        let header_length_pos = line.len();
        line.extend(&0u32.to_le_bytes()); // header_length placeholder
        
        let header_content_start = line.len();
        
        // Standard parameters
        line.push(1);  // minimum_instruction_length
        line.push(1);  // maximum_operations_per_instruction
        line.push(1);  // default_is_stmt
        line.push((-5i8) as u8); // line_base
        line.push(14); // line_range
        line.push(13); // opcode_base
        
        // Standard opcode lengths
        line.extend(&[0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1]);
        
        // Include directories (empty for now)
        line.push(0);
        
        // File names
        for file in &self.files {
            line.extend(file.name.as_bytes());
            line.push(0);
            line.push(0); // directory index
            line.push(0); // modification time
            line.push(0); // file size
        }
        line.push(0); // end of files
        
        // Fix header length
        let header_length = (line.len() - header_content_start) as u32;
        line[header_length_pos..header_length_pos + 4].copy_from_slice(&header_length.to_le_bytes());
        
        // Line number program
        if !self.lines.is_empty() {
            // Set address
            line.push(0); // extended opcode
            line.push(9); // length
            line.push(DW_LNE_SET_ADDRESS);
            line.extend(&self.lines[0].address.to_le_bytes());
            
            let mut prev_line = 1u32;
            let mut prev_addr = self.lines[0].address;
            
            for entry in &self.lines {
                let line_delta = entry.line as i32 - prev_line as i32;
                let addr_delta = entry.address - prev_addr;
                
                if entry.prologue_end {
                    line.push(DW_LNS_SET_PROLOGUE_END);
                }
                
                // Advance line
                if line_delta != 0 {
                    line.push(DW_LNS_ADVANCE_LINE);
                    line.extend(&encode_sleb128(line_delta as i64));
                }
                
                // Advance PC
                if addr_delta > 0 {
                    line.push(DW_LNS_ADVANCE_PC);
                    line.extend(&encode_uleb128(addr_delta));
                }
                
                // Copy row
                line.push(DW_LNS_COPY);
                
                prev_line = entry.line;
                prev_addr = entry.address;
            }
            
            // End sequence
            line.push(0);
            line.push(1);
            line.push(DW_LNE_END_SEQUENCE);
        }
        
        // Fix unit length
        let unit_length = (line.len() - header_start - 4) as u32;
        line[header_start..header_start + 4].copy_from_slice(&unit_length.to_le_bytes());
        
        line
    }
}

// ============================================================================
// LEB128 Encoding
// ============================================================================

fn encode_uleb128(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if value == 0 {
            break;
        }
    }
    result
}

fn encode_sleb128(mut value: i64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut more = true;
    while more {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0) {
            more = false;
        } else {
            byte |= 0x80;
        }
        result.push(byte);
    }
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_source_file() {
        let file = SourceFile::new("src/main.tayni");
        assert_eq!(file.directory, "src");
        assert_eq!(file.name, "main.tayni");
    }
    
    #[test]
    fn test_base_types() {
        let i32_type = BaseType::i32();
        assert_eq!(i32_type.byte_size, 4);
        assert_eq!(i32_type.encoding, DW_ATE_SIGNED);
    }
    
    #[test]
    fn test_uleb128() {
        assert_eq!(encode_uleb128(0), vec![0]);
        assert_eq!(encode_uleb128(127), vec![127]);
        assert_eq!(encode_uleb128(128), vec![0x80, 0x01]);
        assert_eq!(encode_uleb128(624485), vec![0xE5, 0x8E, 0x26]);
    }
    
    #[test]
    fn test_sleb128() {
        assert_eq!(encode_sleb128(0), vec![0]);
        assert_eq!(encode_sleb128(-1), vec![0x7F]);
        assert_eq!(encode_sleb128(-5), vec![0x7B]);
    }
    
    #[test]
    fn test_dwarf_generator() {
        let mut dwarf = DwarfGenerator::new();
        
        let file_idx = dwarf.add_file("main.tayni");
        assert_eq!(file_idx, 1);
        
        dwarf.add_function(FunctionInfo {
            name: "main".to_string(),
            low_pc: 0x400078,
            high_pc: 0x4000A0,
            file: file_idx,
            line: 1,
            is_external: true,
            parameters: vec![],
            locals: vec![],
        });
        
        dwarf.add_line(LineEntry {
            address: 0x400078,
            file: file_idx,
            line: 1,
            column: 1,
            is_stmt: true,
            prologue_end: true,
            epilogue_begin: false,
        });
        
        let info = dwarf.generate_debug_info();
        assert!(info.len() > 20);
        
        let abbrev = dwarf.generate_debug_abbrev();
        assert!(abbrev.len() > 10);
        
        let line = dwarf.generate_debug_line();
        assert!(line.len() > 20);
    }
}
