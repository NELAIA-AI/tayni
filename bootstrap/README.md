# TAYNI Bootstrap

Este directorio contiene los binarios necesarios para compilar TAYNI desde cero.

## Gen15: Zero Rust Compiler

El bootstrap actual usa **Gen15**, el compilador Zero Rust que genera PE desde cero (sin templates).

## Archivos

| Archivo | Descripción | Tamaño |
|---------|-------------|--------|
| `tayni-bootstrap.exe` | Compilador Gen15 | ~8 KB |
| `compiler.exe` | Copia para self-replication | ~8 KB |

## Uso

```bash
# 1. Crear input.tyn con código TAYNI
echo .x: 42 > input.tyn

# 2. Compilar
tayni-bootstrap.exe

# 3. Ejecutar
out.exe
echo %ERRORLEVEL%  # Muestra: 42
```

## Self-Replication

```bash
# Si input.tyn empieza con "--", genera copia del compilador
echo -- TAYNI > input.tyn
compiler.exe
# out.exe == compiler.exe (bit-identical)
```

## Características

- **Zero Rust**: Genera PE byte-by-byte sin templates
- **Self-Replicating**: Puede generar copias de sí mismo
- **Minimal**: Ejecutables de 2,560 bytes
- **8 Syscalls**: ExitProcess, CreateFileA, ReadFile, WriteFile, CloseHandle, GetStdHandle, VirtualAlloc, VirtualFree

## Eficiencia

| Métrica | Gen15 | LLVM+Clang |
|---------|-------|------------|
| PE mínimo | 2,560 bytes | 3,584 bytes |
| Ratio | **1x** | 1.4x más grande |

## Verificación

```bash
# El bootstrap compiler debe poder compilar código simple
echo .x: 42 > input.tyn
tayni-bootstrap.exe
out.exe
# Exit code debe ser 42
```

## Próximos Pasos

Una vez que tengas el bootstrap funcionando:
1. Compila el compilador completo desde `src/tayni/`
2. Usa el compilador completo para desarrollo
3. El bootstrap solo se necesita una vez

## Plataformas

- **Windows x64**: ✅ Incluido
- **Linux x64**: 🔜 Próximamente
- **macOS**: 🔜 Próximamente

## Nota sobre Dependencias

El bootstrap compiler requiere:
- Windows 10/11 x64
- No requiere runtime adicional
- No requiere instalación

Los binarios son completamente autónomos.
