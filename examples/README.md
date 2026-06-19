# TAYNI Examples

Examples demonstrating TAYNI v1.5 syntax and features.

## v1.5 Examples (20 files)

| # | File | Description | Capabilities |
|---|------|-------------|--------------|
| 01 | `01-hello-world.tayni` | Hello World | - |
| 02 | `02-variables-arithmetic.tayni` | Variables and math | - |
| 03 | `03-control-flow.tayni` | if/else, match, loops | - |
| 04 | `04-functions.tayni` | Function definitions | - |
| 05 | `05-structs.tayni` | Structs and methods | - |
| 06 | `06-arrays-maps.tayni` | Collections | - |
| 07 | `07-json.tayni` | JSON handling | - |
| 08 | `08-file-io.tayni` | File operations | `cap:fs` |
| 09 | `09-http-server.tayni` | HTTP server | `cap:net` |
| 10 | `10-error-handling.tayni` | Error handling | `cap:fs` |
| 11 | `11-http-client.tayni` | HTTP client | `cap:net` |
| 12 | `12-tcp-echo.tayni` | TCP echo server | `cap:net` |
| 13 | `13-environment.tayni` | Environment vars | `cap:env`, `cap:proc` |
| 14 | `14-cli-tool.tayni` | CLI application | `cap:proc`, `cap:fs`, `cap:env` |
| 15 | `15-strings.tayni` | String operations | - |
| 16 | `16-math.tayni` | Math functions | - |
| 17 | `17-time.tayni` | Time operations | `cap:time` |
| 18 | `18-lambdas.tayni` | Lambdas & HOF | - |
| 19 | `19-rest-api.tayni` | REST API server | `cap:net`, `cap:fs` |
| 20 | `20-wasm-export.tayni` | WebAssembly export | - |

## Running Examples

```bash
# Compile to Windows
tayni build examples/v1.5/01-hello-world.tayni -o hello.exe
./hello.exe

# Compile to Linux
tayni build examples/v1.5/01-hello-world.tayni --target elf -o hello
./hello

# Compile to WebAssembly
tayni build examples/v1.5/20-wasm-export.tayni --target wasm -o funcs.wasm
```

## Syntax Overview

### Basic Structure

```tayni
// Capabilities at file start
cap:net
cap:fs

// Functions
fn main() {
    PRT("Hello!")
}
```

### Variables

```tayni
let x = 10       // mutable
LET PI = 3.14    // immutable
let name: str = "TAYNI"  // typed
```

### Control Flow

```tayni
if condition {
    // ...
} else {
    // ...
}

match value {
    1 => PRT("one"),
    2 => PRT("two"),
    _ => PRT("other")
}

while condition {
    // ...
}

for item in collection {
    // ...
}
```

### Functions

```tayni
fn add(a: int, b: int) -> int {
    a + b
}

fn greet(name: str) {
    PRT("Hello, ${name}!")
}
```

### Structs

```tayni
struct Point {
    x: float,
    y: float
}

fn Point.distance(self, other: Point) -> float {
    // ...
}
```

### Error Handling

```tayni
fn risky() -> Result<int, str> {
    if error {
        return Err("failed")
    }
    Ok(42)
}

try {
    // ...
} catch err {
    PRTERR(err)
}
```

## Legacy Examples

The `v0.3/`, `v0.4/`, and root-level `.tyn` files use older syntax versions. See the v1.5 examples for current syntax.

---

*TAYNI v1.5 - AI-first programming language*
