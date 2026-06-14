# NELAIA Self-Hosted Compilers

Compiladores NELAIA escritos en NELAIA.

## Archivos Principales

| Archivo | Descripción |
|---------|-------------|
| `nelaia-c.nts` | **Compilador principal auto-hospedado** - Compila .nts a .nts |
| `compiler_v32.nts` | Parser/compiler funcional para programas de 2 líneas |
| `boot_compiler.nts` | Bootstrap compiler - compila `boot.nts` |
| `parser_v1.nts` | Parser básico con FSM |
| `pe_emitter_simple.nts` | Ejemplo de emisión PE |

## Uso

```bash
# Compilar nelaia-c.nts a ejecutable nativo
cargo run --release -- src/nelaia/nelaia-c.nts -o nelaia-c --emit-pe

# Ejecutar el compilador auto-hospedado
./nelaia-c.exe < input.nts
```

## archive/

Contiene versiones anteriores de desarrollo (v02-v31, debug, módulos).
Mantenido como referencia histórica del proceso de bootstrap.
