use crate::ir::{NelaiaGraph, Opcode};
use std::fs;
use std::process::Command;

pub struct Emitter;

impl Emitter {
    pub fn emit_and_build(graph: &NelaiaGraph, output_name: &str) -> Result<(), String> {
        let mut llvm_ir = String::new();
        
        llvm_ir.push_str("; NELAIA AUTO-GENERATED LLVM IR (V8 OPTIMIZED)\n");
        llvm_ir.push_str("declare i32 @puts(ptr)\n");
        
        let mut strings_to_declare = Vec::new();
        let mut main_body = String::new();
        
        main_body.push_str("define i32 @main() {\n");
        main_body.push_str("entry:\n");

        for node in &graph.nodes {
            match node {
                Opcode::STR { ref_id, value } => {
                    let llvm_string = value.clone();
                    let len = llvm_string.len() + 1;
                    let global_name = format!("@str.{}", ref_id);
                    strings_to_declare.push(format!("{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n", global_name, len, llvm_string));
                },
                Opcode::OUT { buffer_ref } => {
                    let global_name = format!("@str.{}", buffer_ref);
                    main_body.push_str(&format!("  %call{} = call i32 @puts(ptr {})\n", buffer_ref, global_name));
                },
                Opcode::EXE { target: _ } => {
                    // Ignorado en generación
                },
                _ => {}
            }
        }
        
        main_body.push_str("  ret i32 0\n");
        main_body.push_str("}\n");

        for decl in strings_to_declare {
            llvm_ir.push_str(&decl);
        }
        llvm_ir.push_str("\n");
        llvm_ir.push_str(&main_body);

        let temp_file = "nelaia_temp_emit.ll";
        if let Err(e) = fs::write(temp_file, &llvm_ir) {
            return Err(format!("Error escribiendo archivo IR: {}", e));
        }

        println!("[NELAIA-C] Backend Emitter: Código LLVM IR purista generado (.ll).");
        println!("[NELAIA-C] Linker: Invocando ensamblador nativo puro con flag de Maxima Velocidad (-O3)...");

        let status = Command::new("clang")
            .arg(temp_file)
            .arg("-O3")  // OPTIMIZACION DE CPU NIVEL 3
            .arg("-w")   // SILENCIAR WARNINGS INNECESARIOS PARA LA IA
            .arg("-o")
            .arg(output_name)
            .status();

        match status {
            Ok(s) if s.success() => {
                let _ = fs::remove_file(temp_file);
                Ok(())
            },
            _ => Err("Fallo en la compilación nativa con clang.".to_string())
        }
    }
}
