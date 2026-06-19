# TAYNI Benchmarks

Comparative benchmarks between TAYNI and other languages.

## Benchmark Categories

### 1. Compilation Speed
Measures time to compile a simple program.

### 2. Binary Size
Measures output executable size.

### 3. Startup Time
Measures time from execution to first output.

### 4. Arithmetic Performance
Measures integer/float computation speed.

### 5. Memory Operations
Measures allocation and copy performance.

## Running Benchmarks

```bash
# Run all benchmarks
./run_benchmarks.ps1

# Run specific benchmark
./run_benchmarks.ps1 -Category compilation
```

## Results Format

Results are output in JSON format:
```json
{
  "benchmark": "compilation_speed",
  "tayni": { "time_ms": 45, "size_bytes": 2048 },
  "c": { "time_ms": 120, "size_bytes": 8192 },
  "rust": { "time_ms": 850, "size_bytes": 245760 },
  "go": { "time_ms": 320, "size_bytes": 1900544 }
}
```

## Expected Results

| Metric | TAYNI | C | Rust | Go |
|--------|-------|---|------|-----|
| Compile Time | ~50ms | ~100ms | ~1s | ~300ms |
| Binary Size | ~2KB | ~8KB | ~250KB | ~2MB |
| Startup Time | <1ms | <1ms | <1ms | ~5ms |

## Benchmark Programs

### hello_world
Simple "Hello, World!" program.

### fibonacci
Calculate first 20 Fibonacci numbers.

### http_server
Start HTTP server, handle 1000 requests.

### json_parse
Parse 1MB JSON file.
