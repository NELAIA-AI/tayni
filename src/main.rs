//! NELAIA Compiler v0.9
//! Pure Syscalls Only - No Embedded Programs

mod ir;
mod parser;
mod emitter_pure;

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
            eprintln!("nelaia-c 0.9 pure");
            eprintln!("usage: nelaia-c <file> [output] [options]");
        }
        return;
    }
    
    let nts_file = &args[1];
    let emit_only = args.contains(&"--emit-llvm".to_string());
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
    
    let output_name = if args.len() > 2 && !args[2].starts_with("--") {
        args[2].clone()
    } else {
        nts_file.replace(".nts", "")
    };
    
    // Read source file
    let content = match fs::read_to_string(nts_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("E:READ:{}", e);
            std::process::exit(1);
        }
    };
    
    // Phase 1: Parse
    let graph = match Parser::parse(&content) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("E:PARSE:{}", e);
            std::process::exit(1);
        }
    };
    
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
