mod ir;
mod parser;
mod emitter;

use parser::Parser;
use emitter::Emitter;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: nelaia-c <file.nts>");
        return;
    }

    let nts_file = &args[1];
    println!("[NELAIA-C] Ingiriendo Flujo de Tokens: {}", nts_file);

    let content = fs::read_to_string(nts_file).unwrap_or_else(|_| {
        println!("[NELAIA-C FATAL] Error al leer el archivo NTS.");
        std::process::exit(1);
    });

    // FASE 1: Análisis Léxico y Sintáctico (Zero-Syntax Ingest)
    let ast_result = Parser::parse(&content);
    
    match ast_result {
        Ok(graph) => {
            println!("[NELAIA-C] ✅ Fase 1 Completada: AST generado matemáticamente sin errores de sintaxis.");
            
            // FASE 2 & 3: Análisis Semántico y Generación (Backend LLVM IR)
            let output_exe = "nelaia_artifact.exe";
            match Emitter::emit_and_build(&graph, output_exe) {
                Ok(_) => {
                    println!("\n[EXITO TOTAL] NELAIA Compiler Toolchain Finalizado.");
                    println!("=> Binario autónomo compilado exitosamente: {}", output_exe);
                },
                Err(e) => {
                    println!("[NELAIA-C FATAL] Error en la generación de código: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Err(e) => {
            println!("[NELAIA-C FATAL] Rechazo Léxico: {}", e);
            std::process::exit(1);
        }
    }
}
