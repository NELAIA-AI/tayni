# NELAIA Examples

Ejemplos de programas NELAIA organizados por categoría.

## Estructura

```
examples/
├── basic/          # Ejemplos básicos (hello world, aritmética, etc.)
├── benchmarks/     # Programas de benchmark
├── fileio/         # Operaciones de archivo
├── gui/            # Interfaces gráficas (Win32)
├── networking/     # Servidores y clientes TCP/UDP
├── v0.3/           # Ejemplos para spec v0.3
└── v0.4/           # Ejemplos para spec v0.4
```

## Ejemplos Destacados

### basic/
- `hello.nts` - Hello World básico
- `fibonacci.nts` - Secuencia de Fibonacci
- `boot.nts` - Programa quine (se imprime a sí mismo)

### networking/
- `http_server.nts` - Servidor HTTP completo
- `webserver.nts` - Servidor web optimizado
- `tcp_client.nts` - Cliente TCP

### gui/
- `gui_final.nts` - Aplicación GUI completa

## Uso

```bash
# Compilar ejemplo
cargo run --release -- examples/basic/hello.nts -o hello

# Compilar a PE nativo
cargo run --release -- examples/basic/hello.nts -o hello --emit-pe

# Ejecutar
./hello.exe
```
