//! Code emitter trait and unified backend interface
//! 
//! This module defines the common interface for all code generation backends.
//! Each target (Windows PE, Linux ELF, macOS Mach-O, WASM, etc.) implements
//! the CodeEmitter trait to generate native binaries.

use crate::ir::{Graph, Op};
use crate::target::{Target, Arch, Os, Accel};

/// Result type for code emission
pub type EmitResult<T> = Result<T, EmitError>;

/// Error during code emission
#[derive(Debug)]
pub struct EmitError {
    pub message: String,
    pub op: Option<String>,
    pub location: Option<String>,
}

impl EmitError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            op: None,
            location: None,
        }
    }
    
    pub fn with_op(mut self, op: impl Into<String>) -> Self {
        self.op = Some(op.into());
        self
    }
    
    pub fn unsupported_op(op: &str, target: &Target) -> Self {
        Self::new(format!("Operation '{}' not supported on {:?}/{:?}", 
            op, target.arch, target.os))
            .with_op(op)
    }
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(op) = &self.op {
            write!(f, " (op: {})", op)?;
        }
        if let Some(loc) = &self.location {
            write!(f, " at {}", loc)?;
        }
        Ok(())
    }
}

/// Trait for code generation backends
/// 
/// Each target platform implements this trait to generate native binaries.
/// The trait provides a unified interface for:
/// - Emitting individual operations
/// - Building complete binaries
/// - Handling platform-specific details
pub trait CodeEmitter {
    /// Get the target this emitter generates code for
    fn target(&self) -> &Target;
    
    /// Emit code for a complete graph
    fn emit_graph(&mut self, graph: &Graph) -> EmitResult<()>;
    
    /// Build the final binary
    fn build(&self) -> EmitResult<Vec<u8>>;
    
    /// Get the recommended file extension for this target
    fn extension(&self) -> &'static str {
        match self.target().native_format() {
            "pe" => "exe",
            "elf" => "",
            "macho" => "",
            "wasm" => "wasm",
            "ptx" => "ptx",
            "qir" => "ll",
            _ => "bin",
        }
    }
    
    /// Check if an operation is supported
    fn supports_op(&self, op: &Op) -> bool {
        use crate::target::{classify_op, check_op_available};
        let op_str = format!("{:?}", op);
        check_op_available(&op_str, self.target()).is_ok()
    }
}

/// Factory for creating code emitters
pub struct EmitterFactory;

impl EmitterFactory {
    /// Create an emitter for the given target
    pub fn create(target: Target) -> EmitResult<Box<dyn CodeEmitter>> {
        match (&target.arch, &target.os, &target.accel) {
            // CPU targets
            (Arch::X86_64, Os::Windows, Accel::None) => {
                Ok(Box::new(WindowsPeEmitter::new(target)))
            }
            (Arch::X86_64, Os::Linux, Accel::None) => {
                Ok(Box::new(LinuxElfEmitter::new(target)))
            }
            (Arch::Arm64, Os::Linux, Accel::None) => {
                Ok(Box::new(LinuxElfEmitter::new(target)))
            }
            (Arch::Arm64, Os::MacOS, Accel::None) => {
                Ok(Box::new(MacOsEmitter::new(target)))
            }
            (Arch::Wasm32, _, Accel::None) => {
                Ok(Box::new(WasmEmitter::new(target)))
            }
            // GPU targets
            (_, _, Accel::Gpu(_)) => {
                Ok(Box::new(GpuEmitter::new(target)))
            }
            // QPU targets
            (_, _, Accel::Qpu(_)) => {
                Ok(Box::new(QpuEmitter::new(target)))
            }
            _ => Err(EmitError::new(format!(
                "No emitter available for {:?}/{:?}/{:?}",
                target.arch, target.os, target.accel
            ))),
        }
    }
    
    /// Create emitter for current host platform
    pub fn host() -> EmitResult<Box<dyn CodeEmitter>> {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Self::create(Target::windows_x64());
        
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Self::create(Target::linux_x64());
        
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Self::create(Target::linux_arm64());
        
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Self::create(Target::macos_arm64());
        
        #[cfg(not(any(
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "aarch64"),
        )))]
        Err(EmitError::new("Unsupported host platform"))
    }
}

// ============================================================================
// Windows PE Emitter (x86-64)
// ============================================================================

pub struct WindowsPeEmitter {
    target: Target,
    graph: Option<Graph>,
}

impl WindowsPeEmitter {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            graph: None,
        }
    }
}

impl CodeEmitter for WindowsPeEmitter {
    fn target(&self) -> &Target {
        &self.target
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> EmitResult<()> {
        self.graph = Some(graph.clone());
        Ok(())
    }
    
    fn build(&self) -> EmitResult<Vec<u8>> {
        // Use existing pe_gen.rs implementation
        if let Some(ref graph) = self.graph {
            Ok(crate::pe_gen::generate_pe_from_graph(graph))
        } else {
            Err(EmitError::new("No graph to emit"))
        }
    }
}

// ============================================================================
// Linux ELF Emitter (x86-64, ARM64, RISC-V)
// ============================================================================

pub struct LinuxElfEmitter {
    target: Target,
    graph: Option<Graph>,
}

impl LinuxElfEmitter {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            graph: None,
        }
    }
}

impl CodeEmitter for LinuxElfEmitter {
    fn target(&self) -> &Target {
        &self.target
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> EmitResult<()> {
        self.graph = Some(graph.clone());
        Ok(())
    }
    
    fn build(&self) -> EmitResult<Vec<u8>> {
        if let Some(ref graph) = self.graph {
            match self.target.arch {
                Arch::X86_64 => Ok(crate::elf::generate_elf_from_graph(graph)),
                Arch::Arm64 => Ok(crate::elf_arm64::generate_elf_arm64_from_graph(graph)),
                Arch::RiscV64 => Ok(crate::riscv::generate_elf_riscv_from_graph(graph)),
                _ => Err(EmitError::new(format!(
                    "Architecture {:?} not supported for Linux ELF",
                    self.target.arch
                ))),
            }
        } else {
            Err(EmitError::new("No graph to emit"))
        }
    }
}

// ============================================================================
// macOS Mach-O Emitter (x86-64, ARM64)
// ============================================================================

pub struct MacOsEmitter {
    target: Target,
    graph: Option<Graph>,
}

impl MacOsEmitter {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            graph: None,
        }
    }
}

impl CodeEmitter for MacOsEmitter {
    fn target(&self) -> &Target {
        &self.target
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> EmitResult<()> {
        self.graph = Some(graph.clone());
        Ok(())
    }
    
    fn build(&self) -> EmitResult<Vec<u8>> {
        if let Some(ref graph) = self.graph {
            let arch = match self.target.arch {
                Arch::X86_64 => crate::macho::MacOSArch::X86_64,
                Arch::Arm64 => crate::macho::MacOSArch::ARM64,
                _ => return Err(EmitError::new("Unsupported architecture for Mach-O")),
            };
            Ok(crate::macho::generate_macho_from_graph(graph, arch))
        } else {
            Err(EmitError::new("No graph to emit"))
        }
    }
}

// ============================================================================
// WebAssembly Emitter
// ============================================================================

pub struct WasmEmitter {
    target: Target,
    graph: Option<Graph>,
}

impl WasmEmitter {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            graph: None,
        }
    }
}

impl CodeEmitter for WasmEmitter {
    fn target(&self) -> &Target {
        &self.target
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> EmitResult<()> {
        self.graph = Some(graph.clone());
        Ok(())
    }
    
    fn build(&self) -> EmitResult<Vec<u8>> {
        if let Some(ref graph) = self.graph {
            Ok(crate::wasm::generate_wasm_from_graph(graph))
        } else {
            Err(EmitError::new("No graph to emit"))
        }
    }
}

// ============================================================================
// GPU Emitter (CUDA PTX, AMD ROCm)
// ============================================================================

pub struct GpuEmitter {
    target: Target,
    graph: Option<Graph>,
}

impl GpuEmitter {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            graph: None,
        }
    }
}

impl CodeEmitter for GpuEmitter {
    fn target(&self) -> &Target {
        &self.target
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> EmitResult<()> {
        self.graph = Some(graph.clone());
        Ok(())
    }
    
    fn build(&self) -> EmitResult<Vec<u8>> {
        if let Some(ref graph) = self.graph {
            match &self.target.accel {
                Accel::Gpu(crate::target::GpuVendor::Nvidia) => {
                    match crate::gpu::PtxEmitter::emit(graph) {
                        Ok(code) => Ok(code.into_bytes()),
                        Err(e) => Err(EmitError::new(e)),
                    }
                }
                Accel::Gpu(crate::target::GpuVendor::Amd) => {
                    match crate::gpu::AmdGpuEmitter::emit(graph) {
                        Ok(code) => Ok(code.into_bytes()),
                        Err(e) => Err(EmitError::new(e)),
                    }
                }
                _ => Err(EmitError::new("Not a GPU target")),
            }
        } else {
            Err(EmitError::new("No graph to emit"))
        }
    }
    
    fn extension(&self) -> &'static str {
        match &self.target.accel {
            Accel::Gpu(crate::target::GpuVendor::Nvidia) => "ptx",
            Accel::Gpu(crate::target::GpuVendor::Amd) => "amdgpu.ll",
            _ => "gpu",
        }
    }
}

// ============================================================================
// QPU Emitter (Quantum IR)
// ============================================================================

pub struct QpuEmitter {
    target: Target,
    graph: Option<Graph>,
}

impl QpuEmitter {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            graph: None,
        }
    }
}

impl CodeEmitter for QpuEmitter {
    fn target(&self) -> &Target {
        &self.target
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> EmitResult<()> {
        self.graph = Some(graph.clone());
        Ok(())
    }
    
    fn build(&self) -> EmitResult<Vec<u8>> {
        if let Some(ref graph) = self.graph {
            match crate::qir::QirEmitter::emit(graph) {
                Ok(code) => Ok(code.into_bytes()),
                Err(e) => Err(EmitError::new(e)),
            }
        } else {
            Err(EmitError::new("No graph to emit"))
        }
    }
    
    fn extension(&self) -> &'static str {
        "qir"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_factory_windows() {
        let emitter = EmitterFactory::create(Target::windows_x64()).unwrap();
        assert_eq!(emitter.extension(), "exe");
    }
    
    #[test]
    fn test_factory_linux() {
        let emitter = EmitterFactory::create(Target::linux_x64()).unwrap();
        assert_eq!(emitter.extension(), "");
    }
    
    #[test]
    fn test_factory_wasm() {
        let emitter = EmitterFactory::create(Target::wasm()).unwrap();
        assert_eq!(emitter.extension(), "wasm");
    }
}
