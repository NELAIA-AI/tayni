//! PE Import Table Generation
//! Import Address Table (IAT) and Import Directory

/// Import function entry
#[derive(Clone)]
pub struct ImportFunction {
    pub name: &'static str,
    pub hint: u16,
}

/// Import DLL entry
pub struct ImportDll {
    pub name: &'static str,
    pub functions: Vec<ImportFunction>,
}

/// Standard kernel32.dll imports
pub fn kernel32_imports() -> ImportDll {
    ImportDll {
        name: "kernel32.dll",
        functions: vec![
            ImportFunction { name: "GetStdHandle", hint: 0 },
            ImportFunction { name: "WriteFile", hint: 1 },
            ImportFunction { name: "ExitProcess", hint: 2 },
            ImportFunction { name: "GetSystemTimeAsFileTime", hint: 3 },
            ImportFunction { name: "Sleep", hint: 4 },
            ImportFunction { name: "CreateThread", hint: 5 },
            ImportFunction { name: "WaitForSingleObject", hint: 6 },
            ImportFunction { name: "CreateMutexA", hint: 7 },
            ImportFunction { name: "ReleaseMutex", hint: 8 },
            ImportFunction { name: "CreateSemaphoreA", hint: 9 },
            ImportFunction { name: "ReleaseSemaphore", hint: 10 },
            ImportFunction { name: "GetProcessHeap", hint: 11 },
            ImportFunction { name: "HeapAlloc", hint: 12 },
            ImportFunction { name: "GetEnvironmentVariableA", hint: 13 },
            ImportFunction { name: "GetCommandLineA", hint: 14 },
            ImportFunction { name: "CreateFileA", hint: 15 },
            ImportFunction { name: "ReadFile", hint: 16 },
            ImportFunction { name: "CloseHandle", hint: 17 },
        ],
    }
}

/// Standard ws2_32.dll imports for networking
pub fn ws2_32_imports() -> ImportDll {
    ImportDll {
        name: "ws2_32.dll",
        functions: vec![
            ImportFunction { name: "WSAStartup", hint: 0 },
            ImportFunction { name: "socket", hint: 1 },
            ImportFunction { name: "bind", hint: 2 },
            ImportFunction { name: "listen", hint: 3 },
            ImportFunction { name: "accept", hint: 4 },
            ImportFunction { name: "send", hint: 5 },
            ImportFunction { name: "recv", hint: 6 },
            ImportFunction { name: "closesocket", hint: 7 },
            ImportFunction { name: "connect", hint: 8 },
            ImportFunction { name: "setsockopt", hint: 9 },
            ImportFunction { name: "ioctlsocket", hint: 10 },
            ImportFunction { name: "select", hint: 11 },
        ],
    }
}

/// user32.dll imports for GUI
pub fn user32_imports() -> ImportDll {
    ImportDll {
        name: "user32.dll",
        functions: vec![
            ImportFunction { name: "MessageBoxA", hint: 0 },
            ImportFunction { name: "CreateWindowExA", hint: 1 },
            ImportFunction { name: "ShowWindow", hint: 2 },
            ImportFunction { name: "GetMessageA", hint: 3 },
            ImportFunction { name: "TranslateMessage", hint: 4 },
            ImportFunction { name: "DispatchMessageA", hint: 5 },
            ImportFunction { name: "DefWindowProcA", hint: 6 },
            ImportFunction { name: "RegisterClassExA", hint: 7 },
            ImportFunction { name: "PostQuitMessage", hint: 8 },
            ImportFunction { name: "GetWindowTextA", hint: 9 },
            ImportFunction { name: "SetWindowTextA", hint: 10 },
        ],
    }
}

/// bcrypt.dll imports for cryptography (Windows CNG)
pub fn bcrypt_imports() -> ImportDll {
    ImportDll {
        name: "bcrypt.dll",
        functions: vec![
            ImportFunction { name: "BCryptOpenAlgorithmProvider", hint: 0 },
            ImportFunction { name: "BCryptCloseAlgorithmProvider", hint: 1 },
            ImportFunction { name: "BCryptGenerateSymmetricKey", hint: 2 },
            ImportFunction { name: "BCryptDestroyKey", hint: 3 },
            ImportFunction { name: "BCryptEncrypt", hint: 4 },
            ImportFunction { name: "BCryptDecrypt", hint: 5 },
            ImportFunction { name: "BCryptSetProperty", hint: 6 },
            ImportFunction { name: "BCryptGetProperty", hint: 7 },
            ImportFunction { name: "BCryptGenRandom", hint: 8 },
            ImportFunction { name: "BCryptHash", hint: 9 },
            ImportFunction { name: "BCryptCreateHash", hint: 10 },
            ImportFunction { name: "BCryptHashData", hint: 11 },
            ImportFunction { name: "BCryptFinishHash", hint: 12 },
            ImportFunction { name: "BCryptDestroyHash", hint: 13 },
        ],
    }
}

/// Build import directory and IAT
pub struct ImportBuilder {
    pub dlls: Vec<ImportDll>,
    pub iat_rva: u32,
    pub import_dir_rva: u32,
}

impl ImportBuilder {
    pub fn new(iat_rva: u32, import_dir_rva: u32) -> Self {
        Self {
            dlls: Vec::new(),
            iat_rva,
            import_dir_rva,
        }
    }
    
    pub fn add_dll(&mut self, dll: ImportDll) {
        self.dlls.push(dll);
    }
    
    /// Calculate total size needed for import section
    pub fn calculate_size(&self) -> usize {
        let mut size = 0;
        
        // Import directory entries (20 bytes each + null terminator)
        size += (self.dlls.len() + 1) * 20;
        
        // For each DLL: IAT, ILT, hint/name entries, DLL name
        for dll in &self.dlls {
            // IAT entries (8 bytes each + null terminator)
            size += (dll.functions.len() + 1) * 8;
            // ILT entries (same as IAT)
            size += (dll.functions.len() + 1) * 8;
            // Hint/Name entries (2 byte hint + name + null + padding)
            for func in &dll.functions {
                size += 2 + func.name.len() + 1;
                if (2 + func.name.len() + 1) % 2 != 0 {
                    size += 1; // Padding
                }
            }
            // DLL name
            size += dll.name.len() + 1;
        }
        
        size
    }
    
    /// Get IAT offset for a specific function
    pub fn get_iat_offset(&self, dll_name: &str, func_name: &str) -> Option<u32> {
        let mut offset = 0u32;
        
        for dll in &self.dlls {
            if dll.name == dll_name {
                for (i, func) in dll.functions.iter().enumerate() {
                    if func.name == func_name {
                        return Some(self.iat_rva + offset + (i as u32 * 8));
                    }
                }
            }
            offset += ((dll.functions.len() + 1) * 8) as u32;
        }
        
        None
    }
}

// BCrypt algorithm identifiers (wide strings)
pub const BCRYPT_AES_ALGORITHM: &[u8] = b"A\0E\0S\0\0\0";
pub const BCRYPT_SHA256_ALGORITHM: &[u8] = b"S\0H\0A\02\05\06\0\0\0";
pub const BCRYPT_SHA1_ALGORITHM: &[u8] = b"S\0H\0A\01\0\0\0";
pub const BCRYPT_RSA_ALGORITHM: &[u8] = b"R\0S\0A\0\0\0";

// BCrypt chaining modes
pub const BCRYPT_CHAIN_MODE_GCM: &[u8] = b"C\0h\0a\0i\0n\0i\0n\0g\0M\0o\0d\0e\0G\0C\0M\0\0\0";
pub const BCRYPT_CHAIN_MODE_CBC: &[u8] = b"C\0h\0a\0i\0n\0i\0n\0g\0M\0o\0d\0e\0C\0B\0C\0\0\0";

// BCrypt property names
pub const BCRYPT_CHAINING_MODE: &[u8] = b"C\0h\0a\0i\0n\0i\0n\0g\0M\0o\0d\0e\0\0\0";
pub const BCRYPT_AUTH_TAG_LENGTH: &[u8] = b"A\0u\0t\0h\0T\0a\0g\0L\0e\0n\0g\0t\0h\0\0\0";

// BCrypt flags
pub const BCRYPT_BLOCK_PADDING: u32 = 0x00000001;
pub const BCRYPT_PAD_NONE: u32 = 0x00000001;
pub const BCRYPT_PAD_PKCS1: u32 = 0x00000002;
pub const BCRYPT_PAD_OAEP: u32 = 0x00000004;

/// secur32.dll imports for SChannel TLS
pub fn secur32_imports() -> ImportDll {
    ImportDll {
        name: "secur32.dll",
        functions: vec![
            ImportFunction { name: "AcquireCredentialsHandleA", hint: 0 },
            ImportFunction { name: "FreeCredentialsHandle", hint: 1 },
            ImportFunction { name: "InitializeSecurityContextA", hint: 2 },
            ImportFunction { name: "AcceptSecurityContext", hint: 3 },
            ImportFunction { name: "DeleteSecurityContext", hint: 4 },
            ImportFunction { name: "QueryContextAttributesA", hint: 5 },
            ImportFunction { name: "EncryptMessage", hint: 6 },
            ImportFunction { name: "DecryptMessage", hint: 7 },
            ImportFunction { name: "FreeContextBuffer", hint: 8 },
        ],
    }
}

// SChannel constants
pub const UNISP_NAME: &[u8] = b"Microsoft Unified Security Protocol Provider\0";
pub const SECPKG_CRED_OUTBOUND: u32 = 0x00000002;
pub const SECPKG_CRED_INBOUND: u32 = 0x00000001;
pub const ISC_REQ_SEQUENCE_DETECT: u32 = 0x00000008;
pub const ISC_REQ_REPLAY_DETECT: u32 = 0x00000004;
pub const ISC_REQ_CONFIDENTIALITY: u32 = 0x00000010;
pub const ISC_REQ_ALLOCATE_MEMORY: u32 = 0x00000100;
pub const ISC_REQ_STREAM: u32 = 0x00008000;
pub const ISC_REQ_MANUAL_CRED_VALIDATION: u32 = 0x00080000;
pub const SEC_E_OK: i32 = 0;
pub const SEC_I_CONTINUE_NEEDED: i32 = 0x00090312;
pub const SEC_E_INCOMPLETE_MESSAGE: i32 = -2146893032; // 0x80090318

