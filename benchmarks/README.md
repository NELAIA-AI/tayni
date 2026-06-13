# NELAIA Benchmark Suite

## Para Correr los Benchmarks

### Opción 1: Script Visual (Recomendado)
```powershell
cd nelaia-core\benchmarks
.\benchmark_visual.ps1
```
Esto genera un **reporte HTML interactivo** con gráficos y lo abre automáticamente.

### Opción 2: Script de Consola
```powershell
cd nelaia-core\benchmarks
.\run_benchmark.ps1
```
Muestra resultados en la terminal con barras visuales.

### Parámetros Opcionales
```powershell
.\benchmark_visual.ps1 -Requests 10000 -Concurrent 200 -Rounds 5
```

## Requisitos

| Software | Versión | Para qué |
|----------|---------|----------|
| Go | 1.18+ | Benchmark tool y servidor Go |
| Rust | stable | Servidor Rust |
| Clang/LLVM | 14+ | Servidor C y compilación NELAIA |
| PowerShell | 5.1+ | Scripts de benchmark |

## Archivos Generados

- `benchmark_results.html` - Reporte visual interactivo
- `benchmark_YYYY-MM-DD_HH-mm-ss.txt` - Resultados en texto

## Compilar Binarios Manualmente

Si los binarios no existen:

```powershell
# Benchmark tool
go build -o bench.exe bench.go

# Go server
go build -o solar_go.exe solar_go.go

# Rust server
rustc -O -o solar_rust_opt.exe solar_rust_opt.rs

# C server
clang -O2 -o solar_c_opt.exe solar_c_opt.c -lws2_32

# NELAIA server
cd ..
cargo run -- benchmarks/solar_nelaia_16w.nts -o benchmarks/solar_nelaia_16w.ll
clang -O2 -o benchmarks/solar_nelaia_16w.exe benchmarks/solar_nelaia_16w.ll -lws2_32 -lkernel32
```

## Servidores Testeados

| Servidor | Descripción | Puerto |
|----------|-------------|--------|
| NELAIA v0.14 | 16 workers, TCP_NODELAY, syscalls puros | 8101 |
| Go HTTP | Librería estándar net/http | 8083 |
| Rust Optimized | 16 workers, TCP_NODELAY | 8087 |
| C Optimized | 16 workers, TCP_NODELAY | 8089 |

## Resultados Típicos

```
NELAIA v0.14:  ████████████████████████████████████████ 2,208 req/s  (7 KB)
Rust:          ███████████████████████████████          1,708 req/s  (158 KB)
Go:            ███████████████████████████              1,506 req/s  (8,196 KB)
C:             █████████████████████████                1,401 req/s  (110 KB)
```

## Troubleshooting

### Puerto en uso
```powershell
Get-Process -Name "solar_*" | Stop-Process -Force
```

### Resultados inconsistentes
- Cerrar otras aplicaciones
- Aumentar `-Rounds` para promedios más estables
- Ejecutar como Administrador

### Benchmark tool no encontrado
```powershell
go build -o bench.exe bench.go
```
