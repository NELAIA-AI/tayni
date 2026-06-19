//! TAYNI Compiler v0.25
//! Direct Native Emission (PE/ELF/Mach-O) - No External Dependencies
//! Supports: Windows PE, Linux ELF, macOS Mach-O, Capability System (SCN)
//! Phase 12.6a: USE directive and module resolution

mod ir;
mod parser;
mod emitter_pure;
mod binary;
mod native_emitter;
mod pe;  // Modular PE constants, headers, imports
mod pe_gen;  // PE generator
mod elf;
mod elf_arm64;
mod macho;
mod capabilities;
mod modules;
mod wasm;
mod riscv;
mod interface;
mod intent;
mod qir;
mod gpu;
mod target;
mod codegen;

use parser::Parser;
use emitter_pure::{PureEmitter, TargetPlatform};
use std::env;
use std::fs;
use std::process::Command;

const VERSION: &str = "0.25.0";

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let quiet = args.contains(&"--quiet".to_string()) || args.contains(&"-q".to_string());
    let use_new_backend = args.contains(&"--new-backend".to_string());
    
    // Handle --version
    if args.contains(&"--version".to_string()) || args.contains(&"-V".to_string()) {
        println!("tayni-c {}", VERSION);
        return;
    }
    
    // Handle --help
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("tayni-c {} - TAYNI Compiler", VERSION);
        println!();
        println!("USAGE:");
        println!("  tayni-c <file.tyn|file.nbin> [options]");
        println!();
        println!("OUTPUT OPTIONS:");
        println!("  -o <file>           Output file name");
        println!("  --emit-pe           Force Windows PE output (default on Windows)");
        println!("  --emit-elf          Force Linux ELF output (default on Linux)");
        println!("  --emit-macho        Force macOS Mach-O x64 output");
        println!("  --emit-macho-arm64  Force macOS Mach-O ARM64 output (Apple Silicon)");
        println!("  --emit-bin          Output binary format (.nbin)");
        println!("  --emit-llvm         Output LLVM IR only (.ll file)");
        println!("  --use-clang         Use LLVM+Clang flow (requires clang installed)");
        println!();
    println!("TARGET PLATFORMS:");
    println!("  --target=windows    Windows x64");
    println!("  --target=linux      Linux x64");
    println!("  --target=linux-arm64  Linux ARM64");
    println!("  --target=macos      macOS x64 (Intel)");
    println!("  --target=macos-arm64  macOS ARM64 (Apple Silicon)");
    println!("  --target=wasm       WebAssembly");
    println!("  --target=riscv64    RISC-V 64-bit");
    println!();
    println!("QUANTUM TARGETS:");
    println!("  --target=qir        QIR (native quantum executable)");
    println!("                      Runs on: Azure Quantum, IonQ, Quantinuum");
    println!();
    println!("QIR TRANSLATIONS (export from QIR, not native targets):");
    println!("  --export=qasm       Translate QIR to OpenQASM 3.0 (IBM)");
    println!("  --export=cirq       Translate QIR to Cirq Python (Google)");
    println!("  --export=quil       Translate QIR to Quil (Rigetti)");
    println!();
    println!("GPU TARGETS:");
    println!("  --target=ptx        PTX (native NVIDIA CUDA executable)");
    println!("  --target=amdgpu     AMDGPU IR (native AMD ROCm executable)");
    println!();
    println!("GPU TRANSLATIONS (export from PTX/AMDGPU, not native targets):");
    println!("  --export=opencl     Translate to OpenCL C (cross-platform)");
    println!("  --export=spirv      Translate to SPIR-V (Vulkan/OpenCL)");
    println!("  --export=wgsl       Translate to WGSL (WebGPU)");
    println!("  --export=metal      Translate to Metal (Apple)");
        println!();
        println!("VALIDATION OPTIONS:");
        println!("  --check             Syntax check only (no output)");
        println!("  --json              Output errors in JSON format");
        println!("  --quiet, -q         Suppress informational messages");
        println!("  --no-warn           Suppress warnings");
        println!("  --tree-shake        Remove unused code (dead code elimination)");
        println!();
        println!("INFO:");
        println!("  --version, -V       Show version");
        println!("  --help, -h          Show this help");
        println!();
        println!("EXPERIMENTAL:");
        println!("  --new-backend       Use unified CodeEmitter backend (experimental)");
        println!();
        println!("SPECIAL COMMANDS:");
        println!("  --test-pe           Generate test PE executable");
        println!("  --tcp-server        Generate TCP server PE");
        println!("  --http-server       Generate HTTP server PE (with recv)");
        println!("  --http-get          Generate HTTP GET client PE");
        println!("  --gui               Generate GUI PE");
        println!("  --tiny              Generate minimal PE (~1KB)");
        println!("  --ultra-tiny        Generate ultra-tiny PE (~1KB optimized)");
        println!("  --smallest          Generate smallest PE (~512 bytes)");
        println!();
        println!("EXAMPLES:");
        println!("  tayni-c hello.tyn                    # Compile to native (auto-detect)");
        println!("  tayni-c hello.tyn -o hello           # Specify output name");
        println!("  tayni-c hello.tyn --emit-macho-arm64 # macOS Apple Silicon");
        println!("  tayni-c hello.tyn --target=linux     # Cross-compile to Linux");
        println!("  tayni-c code.tyn --check             # Validate syntax only");
        println!();
        println!("SUPPORTED PLATFORMS:");
        println!("  Windows x64, Linux x64, macOS x64 (Intel), macOS ARM64 (Apple Silicon)");
        println!();
        println!("NOTE: Default compilation produces native executables directly,");
        println!("      with NO external dependencies (no clang/gcc required).");
        return;
    }
    
    if args.len() < 2 {
        if !quiet {
            eprintln!("tayni-c {}", VERSION);
            eprintln!("usage: tayni-c <file.tyn|file.nbin> [output] [options]");
            eprintln!("Try 'tayni-c --help' for more information.");
        }
        return;
    }
    
    let input_file = &args[1];
    
    // Special command: generate test PE
    if input_file == "--test-pe" {
        let pe_data = pe_gen::generate_hello_pe();
        let output = if args.len() > 2 { &args[2] } else { "test_pe.exe" };
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated {} ({} bytes)", output, pe_data.len());
        }
        return;
    }
    
    // Special command: generate PE template for TAYNI self-hosting
    if input_file == "--pe-template" {
        let pe_data = pe_gen::generate_pe_template();
        let output = if args.len() > 2 { &args[2] } else { "pe-template.bin" };
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated PE template {} ({} bytes)", output, pe_data.len());
        }
        return;
    }
    
    // Special command: generate TCP server PE
    if input_file == "--tcp-server" {
        let port: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8080);
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello TAYNI!";
        let pe_data = pe_gen::generate_tcp_server_pe(port, response);
        let output = args.get(3).map(|s| s.as_str()).unwrap_or("tcp_server.exe");
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated TCP server {} on port {} ({} bytes)", output, port, pe_data.len());
        }
        return;
    }
    
    // Special command: generate HTTP server PE
    if input_file == "--http-server" {
        let port: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8080);
        let output = args.get(3).map(|s| s.as_str()).unwrap_or("http_server.exe");
        let pe_data = pe_gen::generate_http_server_pe(port, &[]);
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated HTTP server {} on port {} ({} bytes)", output, port, pe_data.len());
        }
        return;
    }
    
    // Special command: generate HTTP client PE
    if input_file == "--http-get" {
        // Usage: --http-get <host> <port> <path> [output]
        let host = args.get(2).map(|s| s.as_str()).unwrap_or("127.0.0.1");
        let port: u16 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(80);
        let path = args.get(4).map(|s| s.as_str()).unwrap_or("/");
        let output = args.get(5).map(|s| s.as_str()).unwrap_or("http_get.exe");
        let pe_data = pe_gen::generate_http_get_pe(host, port, path);
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated HTTP GET client {} -> {}:{}{} ({} bytes)", output, host, port, path, pe_data.len());
        }
        return;
    }
    
    // Special command: generate GUI PE
    if input_file == "--gui" {
        let title = args.get(2).map(|s| s.as_str()).unwrap_or("NELAIA");
        let message = args.get(3).map(|s| s.as_str()).unwrap_or("Hello from TAYNI GUI!");
        let output = args.get(4).map(|s| s.as_str()).unwrap_or("gui.exe");
        let pe_data = pe_gen::generate_gui_pe(title, message);
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated GUI {} ({} bytes)", output, pe_data.len());
        }
        return;
    }
    
    // Special command: generate tiny PE (1KB)
    if input_file == "--tiny" {
        let message = args.get(2).map(|s| s.as_str()).unwrap_or("Hello from TAYNI!\n");
        let output = args.get(3).map(|s| s.as_str()).unwrap_or("tiny.exe");
        let pe_data = pe_gen::generate_tiny_pe(message);
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated tiny PE {} ({} bytes)", output, pe_data.len());
        }
        return;
    }
    
    // Special command: generate smallest PE (~512 bytes)
    if input_file == "--smallest" {
        let message = args.get(2).map(|s| s.as_str()).unwrap_or("Hello\n");
        let output = args.get(3).map(|s| s.as_str()).unwrap_or("smallest.exe");
        let pe_data = pe_gen::generate_smallest_pe(message);
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated smallest PE {} ({} bytes)", output, pe_data.len());
        }
        return;
    }
    
    // Special command: generate ultra-tiny PE (~1KB optimized)
    if input_file == "--ultra-tiny" {
        let message = args.get(2).map(|s| s.as_str()).unwrap_or("Hello\n");
        let output = args.get(3).map(|s| s.as_str()).unwrap_or("ultra_tiny.exe");
        let pe_data = pe_gen::generate_ultra_tiny_pe(message);
        if let Err(e) = fs::write(output, &pe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("Generated ultra-tiny PE {} ({} bytes)", output, pe_data.len());
        }
        return;
    }
    
    let emit_llvm = args.contains(&"--emit-llvm".to_string());
    let emit_bin = args.contains(&"--emit-bin".to_string());
    let use_clang = args.contains(&"--use-clang".to_string());
    let no_warn = args.contains(&"--no-warn".to_string());
    let check_only = args.contains(&"--check".to_string());
    let json_output = args.contains(&"--json".to_string());
    let tree_shake = args.contains(&"--tree-shake".to_string());
    
    // Explicit format flags (override auto-detection)
    let force_pe = args.contains(&"--emit-pe".to_string());
    let force_elf = args.contains(&"--emit-elf".to_string());
    let force_macho = args.contains(&"--emit-macho".to_string());
    let force_macho_arm64 = args.contains(&"--emit-macho-arm64".to_string());
    
    // Parse target platform
    let target = if args.iter().any(|a| a == "--target=linux") {
        TargetPlatform::Linux
    } else if args.iter().any(|a| a == "--target=linux-arm64") {
        TargetPlatform::LinuxArm64
    } else if args.iter().any(|a| a == "--target=windows") {
        TargetPlatform::Windows
    } else if args.iter().any(|a| a == "--target=macos" || a == "--target=darwin") {
        TargetPlatform::MacOS
    } else if args.iter().any(|a| a == "--target=macos-arm64" || a == "--target=darwin-arm64") {
        TargetPlatform::MacOSArm64
    } else if args.iter().any(|a| a == "--target=wasm" || a == "--target=wasm32") {
        TargetPlatform::Wasm
    } else if args.iter().any(|a| a == "--target=riscv64" || a == "--target=riscv") {
        TargetPlatform::RiscV64
    } else if args.iter().any(|a| a == "--target=qir" || a == "--target=quantum") {
        TargetPlatform::Qir
    } else if args.iter().any(|a| a == "--target=ptx" || a == "--target=cuda" || a == "--target=nvidia") {
        TargetPlatform::Ptx
    } else if args.iter().any(|a| a == "--target=amdgpu" || a == "--target=rocm" || a == "--target=amd") {
        TargetPlatform::AmdGpu
    } else {
        #[cfg(target_os = "windows")]
        { TargetPlatform::Windows }
        #[cfg(target_os = "macos")]
        { 
            #[cfg(target_arch = "aarch64")]
            { TargetPlatform::MacOSArm64 }
            #[cfg(not(target_arch = "aarch64"))]
            { TargetPlatform::MacOS }
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        { TargetPlatform::Linux }
    };
    
    let output_name = {
        // Find -o flag or use positional argument
        let mut out = None;
        let mut i = 2;
        while i < args.len() {
            if args[i] == "-o" && i + 1 < args.len() {
                out = Some(args[i + 1].clone());
                break;
            } else if !args[i].starts_with("-") && out.is_none() {
                out = Some(args[i].clone());
            }
            i += 1;
        }
        out.unwrap_or_else(|| input_file.replace(".tyn", "").replace(".nbin", ""))
    };
    
    // Read input file (text or binary)
    let mut graph = if input_file.ends_with(".nbin") {
        // Load binary format
        let data = match fs::read(input_file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("E:READ:{}", e);
                std::process::exit(1);
            }
        };
        
        if !binary::is_binary(&data) {
            eprintln!("E:FORMAT:Not a valid .nbin file");
            std::process::exit(1);
        }
        
        match binary::deserialize(&data) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("E:BINARY:{}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Parse text format
        let content = match fs::read_to_string(input_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("E:READ:{}", e);
                std::process::exit(1);
            }
        };
        
        match Parser::parse(&content) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("E:PARSE:{}", e);
                std::process::exit(1);
            }
        }
    };
    
    // Resolve USE directives
    let source_dir = std::path::Path::new(input_file)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    
    let uses = graph.get_uses();
    if !uses.is_empty() {
        let mut resolver = modules::ModuleResolver::with_default_path();
        
        for module_name in &uses {
            match resolver.resolve(module_name, source_dir) {
                Ok(resolved) => {
                    if !quiet {
                        eprintln!("USE:{} from {:?} (tier {:?})", 
                            resolved.name, resolved.path, resolved.tier);
                    }
                    
                    // Parse the module and merge its nodes
                    match Parser::parse(&resolved.content) {
                        Ok(module_graph) => {
                            // Add module nodes to main graph (prepend)
                            for node in module_graph.nodes {
                                // Skip USE nodes from modules (don't re-resolve)
                                if !matches!(node, ir::Node::Use { .. }) {
                                    graph.nodes.insert(0, node);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("E:MODULE:{}:{}", module_name, e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("E:USE:{}", e);
                    std::process::exit(1);
                }
            }
        }
        
        // Update node_map after merging
        graph.rebuild_node_map();
    }
    
    // Apply tree-shaking if requested
    if tree_shake {
        let removed = graph.tree_shake_aggressive();
        if !quiet && removed > 0 {
            eprintln!("TREE-SHAKE: Removed {} unused nodes", removed);
        }
    }
    
    // Check only mode - validate syntax and exit
    if check_only {
        // Analyze graph for errors
        let analysis = graph.analyze();
        
        if !analysis.cycles.is_empty() {
            if json_output {
                eprintln!("{{\"error\":\"cycle\",\"details\":\"{}\"}}", 
                    analysis.cycles.iter().map(|c| c.join(">")).collect::<Vec<_>>().join(","));
            } else {
                eprintln!("E:CYCLE:{}", analysis.cycles.iter().map(|c| c.join(">")).collect::<Vec<_>>().join(","));
            }
            std::process::exit(1);
        }
        
        if !analysis.undefined_refs.is_empty() {
            if json_output {
                eprintln!("{{\"error\":\"undefined\",\"details\":\"{}\"}}", analysis.undefined_refs.join(","));
            } else {
                eprintln!("E:UNDEF:{}", analysis.undefined_refs.join(","));
            }
            std::process::exit(1);
        }
        
        if json_output {
            println!("{{\"status\":\"ok\",\"nodes\":{}}}", graph.nodes.len());
        } else if !quiet {
            println!("OK:CHECK:{} nodes", graph.nodes.len());
        }
        return;
    }
    
    // Emit binary format if requested
    if emit_bin {
        let bin_data = binary::serialize(&graph);
        let bin_file = format!("{}.nbin", output_name);
        if let Err(e) = fs::write(&bin_file, &bin_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:BIN:{}:{} bytes", bin_file, bin_data.len());
        }
        return;
    }
    
    // Analyze graph for errors
    let analysis = graph.analyze();
    
    // Report cycles (errors)
    if !analysis.cycles.is_empty() {
        eprintln!("E:CYCLE:{}", analysis.cycles.iter().map(|c| c.join(">")).collect::<Vec<_>>().join(","));
        std::process::exit(1);
    }
    
    // Report undefined references (errors)
    if !analysis.undefined_refs.is_empty() {
        eprintln!("E:UNDEF:{}", analysis.undefined_refs.join(","));
        std::process::exit(1);
    }
    
    // Report dead nodes (warnings) - only if not quiet and not suppressed
    if !quiet && !no_warn && !analysis.dead_nodes.is_empty() {
        eprintln!("W:DEAD:{}", analysis.dead_nodes.join(","));
    }
    
    // =========================================================================
    // EMISSION STRATEGY (v0.22+):
    // DEFAULT: Direct native emission (PE/ELF) - NO external dependencies
    // OPTIONAL: --emit-llvm for LLVM IR output, --use-clang for legacy flow
    // =========================================================================
    
    // Option 1: LLVM IR only (requires clang to compile separately)
    if emit_llvm && !use_clang {
        let llvm_ir = match PureEmitter::emit(&graph, target) {
            Ok(ir) => ir,
            Err(e) => {
                eprintln!("E:EMIT:{}", e);
                std::process::exit(1);
            }
        };
        let ll_file = format!("{}.ll", output_name);
        if let Err(e) = fs::write(&ll_file, &llvm_ir) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:LLVM:{}:{} bytes", ll_file, llvm_ir.len());
            eprintln!("Note: Use 'clang {} -o {}' to compile", ll_file, output_name);
        }
        return;
    }
    
    // Option 2: Legacy LLVM+Clang flow (explicit opt-in)
    if use_clang {
        let llvm_ir = match PureEmitter::emit(&graph, target) {
            Ok(ir) => ir,
            Err(e) => {
                eprintln!("E:EMIT:{}", e);
                std::process::exit(1);
            }
        };
        let ll_file = format!("{}.ll", output_name);
        if let Err(e) = fs::write(&ll_file, &llvm_ir) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        
        // Compile with clang
        let compile_result = match target {
            TargetPlatform::Linux => {
                Command::new("clang")
                    .args(&[
                        "-nostdlib",
                        "-static",
                        "-O2",
                        "-o", &output_name,
                        &ll_file,
                    ])
                    .output()
            }
            TargetPlatform::LinuxArm64 => {
                Command::new("clang")
                    .args(&[
                        "-target", "aarch64-unknown-linux-gnu",
                        "-nostdlib",
                        "-static",
                        "-O2",
                        "-o", &output_name,
                        &ll_file,
                    ])
                    .output()
            }
            TargetPlatform::Windows => {
                let exe_name = format!("{}.exe", output_name);
                let subsystem = if args.iter().any(|a| a == "--subsystem=windows") {
                    "/SUBSYSTEM:WINDOWS"
                } else {
                    "/SUBSYSTEM:CONSOLE"
                };
                Command::new("clang")
                    .args(&[
                        "-O2",
                        "-Wl,/ENTRY:mainCRTStartup",
                        &format!("-Wl,{}", subsystem),
                        "-lkernel32",
                        "-lws2_32",
                        "-luser32",
                        "-lgdi32",
                        "-o", &exe_name,
                        &ll_file,
                    ])
                    .output()
            }
            TargetPlatform::MacOS | TargetPlatform::MacOSArm64 => {
                Command::new("clang")
                    .args(&[
                        "-O2",
                        "-o", &output_name,
                        &ll_file,
                    ])
                    .output()
            }
            TargetPlatform::Wasm => {
                Command::new("clang")
                    .args(&[
                        "-target", "wasm32-unknown-unknown",
                        "-nostdlib",
                        "-O2",
                        "-o", &format!("{}.wasm", output_name),
                        &ll_file,
                    ])
                    .output()
            }
            TargetPlatform::RiscV64 => {
                Command::new("clang")
                    .args(&[
                        "-target", "riscv64-unknown-linux-gnu",
                        "-nostdlib",
                        "-static",
                        "-O2",
                        "-o", &output_name,
                        &ll_file,
                    ])
                    .output()
            }
            TargetPlatform::Qir => {
                // QIR doesn't use clang - it's already in QIR format
                // Just copy the .ll file to .qir
                Command::new("cmd")
                    .args(&["/C", "copy", &ll_file, &format!("{}.qir", output_name)])
                    .output()
            }
            TargetPlatform::Ptx => {
                // PTX uses NVCC or clang with CUDA target
                Command::new("clang")
                    .args(&[
                        "-target", "nvptx64-nvidia-cuda",
                        "-S",
                        "-O2",
                        "-o", &format!("{}.ptx", output_name),
                        &ll_file,
                    ])
                    .output()
            }
            TargetPlatform::AmdGpu => {
                // AMDGPU uses clang with amdgcn target
                Command::new("clang")
                    .args(&[
                        "-target", "amdgcn-amd-amdhsa",
                        "-S",
                        "-O2",
                        "-o", &format!("{}.amdgpu.s", output_name),
                        &ll_file,
                    ])
                    .output()
            }
        };
        
        match compile_result {
            Ok(output) if output.status.success() => {
                if !quiet {
                    eprintln!("OK:CLANG:{}", output_name);
                }
            }
            Ok(output) => {
                eprintln!("E:CLANG:{}", output.status);
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("E:CLANG:{}. Install clang or use default direct emission.", e);
                std::process::exit(1);
            }
        }
        return;
    }
    
    // =========================================================================
    // NEW UNIFIED BACKEND (experimental)
    // Uses the CodeEmitter trait for all targets
    // =========================================================================
    
    if use_new_backend {
        use codegen::{EmitterFactory, CodeEmitter};
        use target::Target;
        
        // Convert TargetPlatform to Target
        let unified_target = match target {
            TargetPlatform::Windows => Target::windows_x64(),
            TargetPlatform::Linux => Target::linux_x64(),
            TargetPlatform::LinuxArm64 => Target::linux_arm64(),
            TargetPlatform::MacOS => Target::macos_arm64(), // Default to ARM64 for new backend
            TargetPlatform::MacOSArm64 => Target::macos_arm64(),
            TargetPlatform::Wasm => Target::wasm(),
            TargetPlatform::Ptx => Target::cuda(),
            TargetPlatform::AmdGpu => Target::rocm(),
            TargetPlatform::Qir => Target::qpu_azure(),
            _ => Target::windows_x64(), // Fallback
        };
        
        let mut emitter = match EmitterFactory::create(unified_target.clone()) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("E:EMITTER:{}", e);
                std::process::exit(1);
            }
        };
        
        if let Err(e) = emitter.emit_graph(&graph) {
            eprintln!("E:EMIT:{}", e);
            std::process::exit(1);
        }
        
        let binary = match emitter.build() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("E:BUILD:{}", e);
                std::process::exit(1);
            }
        };
        
        let ext = emitter.extension();
        let out_file = if ext.is_empty() {
            output_name.clone()
        } else {
            format!("{}.{}", output_name, ext)
        };
        
        if let Err(e) = fs::write(&out_file, &binary) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        
        if !quiet {
            eprintln!("OK:{}:{}:{} bytes (unified backend)", 
                unified_target.native_format().to_uppercase(),
                out_file, 
                binary.len());
        }
        return;
    }
    
    // =========================================================================
    // DEFAULT: Direct native emission (no external dependencies)
    // Automatically selects PE (Windows), ELF (Linux), Mach-O (macOS), WASM, RISC-V
    // =========================================================================
    
    let emit_pe = force_pe || (!force_elf && !force_macho && !force_macho_arm64 && matches!(target, TargetPlatform::Windows));
    let emit_elf = force_elf || (!force_pe && !force_macho && !force_macho_arm64 && matches!(target, TargetPlatform::Linux));
    let emit_elf_arm64 = matches!(target, TargetPlatform::LinuxArm64);
    let emit_macho = force_macho || (!force_pe && !force_elf && !force_macho_arm64 && matches!(target, TargetPlatform::MacOS));
    let emit_macho_arm64 = force_macho_arm64 || (!force_pe && !force_elf && !force_macho && matches!(target, TargetPlatform::MacOSArm64));
    let emit_wasm = matches!(target, TargetPlatform::Wasm);
    let emit_riscv = matches!(target, TargetPlatform::RiscV64);
    let emit_qir = matches!(target, TargetPlatform::Qir);
    
    // QIR translation exports (not native targets)
    let export_qasm = args.iter().any(|a| a == "--export=qasm");
    let export_cirq = args.iter().any(|a| a == "--export=cirq");
    let export_quil = args.iter().any(|a| a == "--export=quil");
    
    if emit_qir {
        // Generate native QIR (the only quantum executable format)
        let qir_code = match qir::QirEmitter::emit(&graph) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("E:QIR:{}", e);
                std::process::exit(1);
            }
        };
        
        // Count qubits for translations (simple heuristic)
        let num_qubits = qir_code.matches("%q").count().max(2);
        
        // Write native QIR
        let qir_file = format!("{}.qir", output_name);
        if let Err(e) = fs::write(&qir_file, &qir_code) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:QIR:{}:{} bytes (native quantum executable)", qir_file, qir_code.len());
            eprintln!("   Targets: Azure Quantum, IonQ, Quantinuum");
        }
        
        // Generate translations if requested
        if export_qasm {
            let qasm = qir::QirTranslator::to_qasm(&qir_code, num_qubits);
            let qasm_file = format!("{}.qasm", output_name);
            if let Err(e) = fs::write(&qasm_file, &qasm) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:QASM:{}:{} bytes (IBM OpenQASM 3.0 translation)", qasm_file, qasm.len());
            }
        }
        
        if export_cirq {
            let cirq = qir::QirTranslator::to_cirq(&qir_code, num_qubits);
            let cirq_file = format!("{}.py", output_name);
            if let Err(e) = fs::write(&cirq_file, &cirq) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:CIRQ:{}:{} bytes (Google Cirq Python translation)", cirq_file, cirq.len());
            }
        }
        
        if export_quil {
            let quil = qir::QirTranslator::to_quil(&qir_code, num_qubits);
            let quil_file = format!("{}.quil", output_name);
            if let Err(e) = fs::write(&quil_file, &quil) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:QUIL:{}:{} bytes (Rigetti Quil translation)", quil_file, quil.len());
            }
        }
        
        return;
    }
    
    // GPU translation exports
    let export_opencl = args.iter().any(|a| a == "--export=opencl");
    let export_spirv = args.iter().any(|a| a == "--export=spirv");
    let export_wgsl = args.iter().any(|a| a == "--export=wgsl");
    let export_metal = args.iter().any(|a| a == "--export=metal");
    
    let emit_ptx = matches!(target, TargetPlatform::Ptx);
    let emit_amdgpu = matches!(target, TargetPlatform::AmdGpu);
    
    if emit_ptx {
        // Generate native PTX (NVIDIA CUDA)
        let ptx_code = match gpu::PtxEmitter::emit(&graph) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("E:PTX:{}", e);
                std::process::exit(1);
            }
        };
        
        // Write native PTX
        let ptx_file = format!("{}.ptx", output_name);
        if let Err(e) = fs::write(&ptx_file, &ptx_code) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:PTX:{}:{} bytes (native NVIDIA CUDA executable)", ptx_file, ptx_code.len());
        }
        
        // Generate translations if requested
        if export_opencl {
            let opencl = gpu::GpuTranslator::to_opencl(256);
            let cl_file = format!("{}.cl", output_name);
            if let Err(e) = fs::write(&cl_file, &opencl) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:OPENCL:{}:{} bytes (cross-platform translation)", cl_file, opencl.len());
            }
        }
        
        if export_spirv {
            let spirv = gpu::GpuTranslator::to_spirv();
            let spirv_file = format!("{}.spvasm", output_name);
            if let Err(e) = fs::write(&spirv_file, &spirv) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:SPIRV:{}:{} bytes (Vulkan/OpenCL translation)", spirv_file, spirv.len());
            }
        }
        
        if export_wgsl {
            let wgsl = gpu::GpuTranslator::to_wgsl();
            let wgsl_file = format!("{}.wgsl", output_name);
            if let Err(e) = fs::write(&wgsl_file, &wgsl) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:WGSL:{}:{} bytes (WebGPU translation)", wgsl_file, wgsl.len());
            }
        }
        
        if export_metal {
            let metal = gpu::GpuTranslator::to_metal();
            let metal_file = format!("{}.metal", output_name);
            if let Err(e) = fs::write(&metal_file, &metal) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:METAL:{}:{} bytes (Apple Metal translation)", metal_file, metal.len());
            }
        }
        
        return;
    }
    
    if emit_amdgpu {
        // Generate native AMDGPU IR (AMD ROCm)
        let amdgpu_code = match gpu::AmdGpuEmitter::emit(&graph) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("E:AMDGPU:{}", e);
                std::process::exit(1);
            }
        };
        
        // Write native AMDGPU IR
        let amdgpu_file = format!("{}.amdgpu.ll", output_name);
        if let Err(e) = fs::write(&amdgpu_file, &amdgpu_code) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:AMDGPU:{}:{} bytes (native AMD ROCm executable)", amdgpu_file, amdgpu_code.len());
        }
        
        // Generate translations if requested (same as PTX)
        if export_opencl {
            let opencl = gpu::GpuTranslator::to_opencl(256);
            let cl_file = format!("{}.cl", output_name);
            if let Err(e) = fs::write(&cl_file, &opencl) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:OPENCL:{}:{} bytes (cross-platform translation)", cl_file, opencl.len());
            }
        }
        
        if export_spirv {
            let spirv = gpu::GpuTranslator::to_spirv();
            let spirv_file = format!("{}.spvasm", output_name);
            if let Err(e) = fs::write(&spirv_file, &spirv) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:SPIRV:{}:{} bytes (Vulkan/OpenCL translation)", spirv_file, spirv.len());
            }
        }
        
        if export_wgsl {
            let wgsl = gpu::GpuTranslator::to_wgsl();
            let wgsl_file = format!("{}.wgsl", output_name);
            if let Err(e) = fs::write(&wgsl_file, &wgsl) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:WGSL:{}:{} bytes (WebGPU translation)", wgsl_file, wgsl.len());
            }
        }
        
        if export_metal {
            let metal = gpu::GpuTranslator::to_metal();
            let metal_file = format!("{}.metal", output_name);
            if let Err(e) = fs::write(&metal_file, &metal) {
                eprintln!("E:WRITE:{}", e);
            } else if !quiet {
                eprintln!("OK:METAL:{}:{} bytes (Apple Metal translation)", metal_file, metal.len());
            }
        }
        
        return;
    }
    
    if emit_wasm {
        let wasm_data = wasm::generate_wasm_from_graph(&graph);
        let wasm_file = format!("{}.wasm", output_name);
        if let Err(e) = fs::write(&wasm_file, &wasm_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:WASM:{}:{} bytes", wasm_file, wasm_data.len());
        }
        return;
    }
    
    if emit_riscv {
        let exe_data = riscv::generate_elf_riscv_from_graph(&graph);
        let exe_file = output_name.clone();
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:ELF-RISCV64:{}:{} bytes", exe_file, exe_data.len());
        }
        return;
    }
    
    if emit_elf_arm64 {
        let exe_data = elf_arm64::generate_elf_arm64_from_graph(&graph);
        let exe_file = output_name.clone();
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:ELF-ARM64:{}:{} bytes", exe_file, exe_data.len());
        }
        return;
    }
    
    if emit_pe {
        let exe_data = pe_gen::generate_pe_from_graph(&graph);
        let exe_file = format!("{}.exe", output_name);
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:PE:{}:{} bytes (direct emission, no clang)", exe_file, exe_data.len());
        }
        return;
    }
    
    if emit_macho {
        let exe_data = macho::generate_macho_from_graph(&graph, macho::MacOSArch::X86_64);
        let exe_file = output_name.clone();
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            // Extract payload size from header bytes 8-9 (stored during generation)
            let payload_size = (exe_data[8] as usize) | ((exe_data[9] as usize) << 8);
            if payload_size > 0 && payload_size < exe_data.len() {
                eprintln!("OK:MACHO:{}:{} bytes ({} payload, x86-64)", exe_file, exe_data.len(), payload_size);
            } else {
                eprintln!("OK:MACHO:{}:{} bytes (x86-64, direct emission)", exe_file, exe_data.len());
            }
        }
        return;
    }
    
    if emit_macho_arm64 {
        let exe_data = macho::generate_macho_from_graph(&graph, macho::MacOSArch::ARM64);
        let exe_file = output_name.clone();
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            // Extract payload size from header bytes 8-9 (stored during generation)
            let payload_size = (exe_data[8] as usize) | ((exe_data[9] as usize) << 8);
            if payload_size > 0 && payload_size < exe_data.len() {
                eprintln!("OK:MACHO:{}:{} bytes ({} payload, ARM64)", exe_file, exe_data.len(), payload_size);
            } else {
                eprintln!("OK:MACHO:{}:{} bytes (ARM64 Apple Silicon, direct emission)", exe_file, exe_data.len());
            }
        }
        return;
    }
    
    if emit_elf {
        let exe_data = elf::generate_elf_from_graph(&graph);
        let exe_file = output_name.clone();
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:ELF:{}:{} bytes (direct emission, no clang)", exe_file, exe_data.len());
        }
    }
}
