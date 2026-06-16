//! TAYNI Compiler v0.23
//! Direct Native Emission (PE/ELF/Mach-O) - No External Dependencies
//! Supports: Windows PE, Linux ELF, macOS Mach-O, Capability System (SCN)
//! Phase 12: Product Ready - Direct emission as DEFAULT

mod ir;
mod parser;
mod emitter_pure;
mod binary;
mod native_emitter;
mod pe;
mod elf;
mod macho;
mod capabilities;

use parser::Parser;
use emitter_pure::{PureEmitter, TargetPlatform};
use std::env;
use std::fs;
use std::process::Command;

const VERSION: &str = "0.23.0";

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let quiet = args.contains(&"--quiet".to_string()) || args.contains(&"-q".to_string());
    
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
        println!("  tayni-c <file.tayni|file.nbin> [options]");
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
        println!("  --target=macos      macOS x64 (Intel)");
        println!("  --target=macos-arm64  macOS ARM64 (Apple Silicon)");
        println!();
        println!("VALIDATION OPTIONS:");
        println!("  --check             Syntax check only (no output)");
        println!("  --json              Output errors in JSON format");
        println!("  --quiet, -q         Suppress informational messages");
        println!("  --no-warn           Suppress warnings");
        println!();
        println!("INFO:");
        println!("  --version, -V       Show version");
        println!("  --help, -h          Show this help");
        println!();
        println!("SPECIAL COMMANDS:");
        println!("  --test-pe           Generate test PE executable");
        println!("  --tcp-server        Generate TCP server PE");
        println!("  --gui               Generate GUI PE");
        println!("  --tiny              Generate minimal PE (~1KB)");
        println!("  --ultra-tiny        Generate ultra-tiny PE (~1KB optimized)");
        println!("  --smallest          Generate smallest PE (~512 bytes)");
        println!();
        println!("EXAMPLES:");
        println!("  tayni-c hello.tayni                    # Compile to native (auto-detect)");
        println!("  tayni-c hello.tayni -o hello           # Specify output name");
        println!("  tayni-c hello.tayni --emit-macho-arm64 # macOS Apple Silicon");
        println!("  tayni-c hello.tayni --target=linux     # Cross-compile to Linux");
        println!("  tayni-c code.tayni --check             # Validate syntax only");
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
            eprintln!("usage: tayni-c <file.tayni|file.nbin> [output] [options]");
            eprintln!("Try 'tayni-c --help' for more information.");
        }
        return;
    }
    
    let input_file = &args[1];
    
    // Special command: generate test PE
    if input_file == "--test-pe" {
        let pe_data = pe::generate_hello_pe();
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
    
    // Special command: generate TCP server PE
    if input_file == "--tcp-server" {
        let port: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8080);
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello TAYNI!";
        let pe_data = pe::generate_tcp_server_pe(port, response);
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
    
    // Special command: generate GUI PE
    if input_file == "--gui" {
        let title = args.get(2).map(|s| s.as_str()).unwrap_or("NELAIA");
        let message = args.get(3).map(|s| s.as_str()).unwrap_or("Hello from TAYNI GUI!");
        let output = args.get(4).map(|s| s.as_str()).unwrap_or("gui.exe");
        let pe_data = pe::generate_gui_pe(title, message);
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
        let pe_data = pe::generate_tiny_pe(message);
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
        let pe_data = pe::generate_smallest_pe(message);
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
        let pe_data = pe::generate_ultra_tiny_pe(message);
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
    
    // Explicit format flags (override auto-detection)
    let force_pe = args.contains(&"--emit-pe".to_string());
    let force_elf = args.contains(&"--emit-elf".to_string());
    let force_macho = args.contains(&"--emit-macho".to_string());
    let force_macho_arm64 = args.contains(&"--emit-macho-arm64".to_string());
    
    // Parse target platform
    let target = if args.iter().any(|a| a == "--target=linux") {
        TargetPlatform::Linux
    } else if args.iter().any(|a| a == "--target=windows") {
        TargetPlatform::Windows
    } else if args.iter().any(|a| a == "--target=macos" || a == "--target=darwin") {
        TargetPlatform::MacOS
    } else if args.iter().any(|a| a == "--target=macos-arm64" || a == "--target=darwin-arm64") {
        TargetPlatform::MacOSArm64
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
        out.unwrap_or_else(|| input_file.replace(".tayni", "").replace(".nbin", ""))
    };
    
    // Read input file (text or binary)
    let graph = if input_file.ends_with(".nbin") {
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
    // DEFAULT: Direct native emission (no external dependencies)
    // Automatically selects PE (Windows), ELF (Linux), or Mach-O (macOS)
    // =========================================================================
    
    let emit_pe = force_pe || (!force_elf && !force_macho && !force_macho_arm64 && matches!(target, TargetPlatform::Windows));
    let emit_elf = force_elf || (!force_pe && !force_macho && !force_macho_arm64 && matches!(target, TargetPlatform::Linux));
    let emit_macho = force_macho || (!force_pe && !force_elf && !force_macho_arm64 && matches!(target, TargetPlatform::MacOS));
    let emit_macho_arm64 = force_macho_arm64 || (!force_pe && !force_elf && !force_macho && matches!(target, TargetPlatform::MacOSArm64));
    
    if emit_pe {
        let exe_data = pe::generate_pe_from_graph(&graph);
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
