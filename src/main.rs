//! NELAIA Compiler v0.22
//! Pure Syscalls Only - Self-Hosting Ready
//! Supports: Windows PE, Linux ELF, Capability System (SCN)
//! Phase 8-11: Contracts, Negotiation, Testing, Cache, SEN Ecosystem (IA-first)

mod ir;
mod parser;
mod emitter_pure;
mod binary;
mod native_emitter;
mod pe;
mod elf;
mod capabilities;

use parser::Parser;
use emitter_pure::{PureEmitter, TargetPlatform};
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let quiet = args.contains(&"--quiet".to_string()) || args.contains(&"-q".to_string());
    
    if args.len() < 2 {
        if !quiet {
            eprintln!("nelaia-c 0.18");
            eprintln!("usage: nelaia-c <file.nts|file.nbin> [output] [options]");
            eprintln!("options:");
            eprintln!("  --emit-llvm    Output LLVM IR only");
            eprintln!("  --emit-bin     Output binary format (.nbin)");
            eprintln!("  --emit-pe      Output Windows PE executable");
            eprintln!("  --emit-elf     Output Linux ELF executable");
            eprintln!("  --target=linux/windows");
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
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello NELAIA!";
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
        let message = args.get(3).map(|s| s.as_str()).unwrap_or("Hello from NELAIA GUI!");
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
        let message = args.get(2).map(|s| s.as_str()).unwrap_or("Hello from NELAIA!\n");
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
    
    let emit_only = args.contains(&"--emit-llvm".to_string());
    let emit_bin = args.contains(&"--emit-bin".to_string());
    let emit_pe = args.contains(&"--emit-pe".to_string());
    let emit_elf = args.contains(&"--emit-elf".to_string());
    let no_warn = args.contains(&"--no-warn".to_string());
    
    // Parse target platform
    let target = if args.iter().any(|a| a == "--target=linux") {
        TargetPlatform::Linux
    } else if args.iter().any(|a| a == "--target=windows") {
        TargetPlatform::Windows
    } else {
        #[cfg(target_os = "windows")]
        { TargetPlatform::Windows }
        #[cfg(not(target_os = "windows"))]
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
        out.unwrap_or_else(|| input_file.replace(".nts", "").replace(".nbin", ""))
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
    
    // Emit native executable if requested
    let emit_native = args.contains(&"--emit-native".to_string());
    if emit_native {
        let exe_data = native_emitter::emit_native_exe(&graph);
        let exe_file = format!("{}.exe", output_name);
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:NATIVE:{}:{} bytes", exe_file, exe_data.len());
        }
        return;
    }
    
    // Emit PE executable directly (no clang)
    if emit_pe {
        let exe_data = pe::generate_pe_from_graph(&graph);
        let exe_file = format!("{}.exe", output_name);
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:PE:{}:{} bytes", exe_file, exe_data.len());
        }
        return;
    }
    
    // Emit ELF executable directly (no clang)
    if emit_elf {
        let exe_data = elf::generate_elf_from_graph(&graph);
        let exe_file = output_name.clone();
        if let Err(e) = fs::write(&exe_file, &exe_data) {
            eprintln!("E:WRITE:{}", e);
            std::process::exit(1);
        }
        if !quiet {
            eprintln!("OK:ELF:{}:{} bytes", exe_file, exe_data.len());
        }
        return;
    }
    
    // Phase 1.5: Analyze graph
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
    
    // Phase 2: Emit LLVM IR
    let llvm_ir = match PureEmitter::emit(&graph, target) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("E:EMIT:{}", e);
            std::process::exit(1);
        }
    };
    
    // Write LLVM IR
    let ll_file = format!("{}.ll", output_name);
    if let Err(e) = fs::write(&ll_file, &llvm_ir) {
        eprintln!("E:WRITE:{}", e);
        std::process::exit(1);
    }
    
    if emit_only {
        return;
    }
    
    // Phase 3: Compile with clang
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
            let subsystem = if args.contains(&"--gui".to_string()) {
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
    };
    
    match compile_result {
        Ok(output) if output.status.success() => {
            // Success - silent
        }
        Ok(output) => {
            eprintln!("E:CLANG:{}", output.status);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("E:CLANG:{}", e);
            std::process::exit(1);
        }
    }
}
