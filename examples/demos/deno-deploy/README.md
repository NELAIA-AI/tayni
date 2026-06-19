# TAYNI on Deno Deploy

This demo shows how to run TAYNI-compiled WebAssembly on Deno Deploy.

## Prerequisites

- Deno installed (`curl -fsSL https://deno.land/install.sh | sh`)
- Deno Deploy account (https://deno.com/deploy)
- TAYNI compiler

## Project Structure

```
deno-deploy/
├── README.md
├── deno.json
├── main.ts           # Deno entry point
├── src/
│   └── handler.tayni # TAYNI source
└── build/
    └── handler.wasm  # Compiled Wasm
```

## TAYNI Source

```tayni
// handler.tayni - Deno Deploy handler
use http

// Export functions for Deno to call
@export
fn handle_request(method: str, path: str, body: str) -> str {
    if path == "/" {
        JSON.encode({
            "service": "TAYNI on Deno Deploy",
            "version": "1.0.0",
            "runtime": "Deno",
            "status": "ok"
        })
    } else if path == "/api/greet" {
        let data = JSON.parse(body)
        let name = data.name ?? "World"
        JSON.encode({
            "message": "Hello, " + name + "!",
            "timestamp": Time.now()
        })
    } else if path == "/api/compute" {
        let data = JSON.parse(body)
        let a = data.a ?? 0
        let b = data.b ?? 0
        let op = data.op ?? "add"
        
        let result = if op == "add" {
            a + b
        } else if op == "sub" {
            a - b
        } else if op == "mul" {
            a * b
        } else if op == "div" {
            a / b
        } else {
            0
        }
        
        JSON.encode({
            "a": a,
            "b": b,
            "op": op,
            "result": result
        })
    } else {
        JSON.encode({
            "error": "Not Found",
            "path": path
        })
    }
}
```

## Compilation

```bash
# Compile TAYNI to Wasm
tayni compile src/handler.tayni -o build/handler.wasm --target wasm

# Verify
deno run --allow-read verify.ts
```

## Deno Entry Point

```typescript
// main.ts
const wasmCode = await Deno.readFile("./build/handler.wasm");
const wasmModule = new WebAssembly.Module(wasmCode);

// Memory for string passing
const memory = new WebAssembly.Memory({ initial: 1 });
const encoder = new TextEncoder();
const decoder = new TextDecoder();

function writeString(str: string, offset: number): number {
  const bytes = encoder.encode(str);
  const view = new Uint8Array(memory.buffer);
  view.set(bytes, offset);
  view[offset + bytes.length] = 0; // null terminator
  return bytes.length;
}

function readString(offset: number, length: number): string {
  const view = new Uint8Array(memory.buffer);
  return decoder.decode(view.slice(offset, offset + length));
}

const wasmInstance = new WebAssembly.Instance(wasmModule, {
  env: {
    memory,
    print: (ptr: number, len: number) => {
      console.log(readString(ptr, len));
    }
  }
});

const exports = wasmInstance.exports as {
  handle_request: (method: number, path: number, body: number) => number;
};

Deno.serve(async (req: Request) => {
  const url = new URL(req.url);
  const method = req.method;
  const path = url.pathname;
  const body = req.method !== "GET" ? await req.text() : "";
  
  // Write strings to Wasm memory
  const methodOffset = 0;
  const pathOffset = 256;
  const bodyOffset = 512;
  const resultOffset = 4096;
  
  writeString(method, methodOffset);
  writeString(path, pathOffset);
  writeString(body, bodyOffset);
  
  // Call Wasm handler
  const resultLen = exports.handle_request(methodOffset, pathOffset, bodyOffset);
  const result = readString(resultOffset, resultLen);
  
  // Determine status code
  const parsed = JSON.parse(result);
  const status = parsed.error ? 404 : 200;
  
  return new Response(result, {
    status,
    headers: {
      "Content-Type": "application/json",
      "X-Powered-By": "TAYNI"
    }
  });
});
```

## Configuration

```json
// deno.json
{
  "tasks": {
    "build": "tayni compile src/handler.tayni -o build/handler.wasm --target wasm",
    "dev": "deno run --allow-net --allow-read --watch main.ts",
    "start": "deno run --allow-net --allow-read main.ts"
  },
  "deploy": {
    "project": "tayni-demo",
    "entrypoint": "main.ts"
  }
}
```

## Deployment

```bash
# Install deployctl
deno install -A --no-check -r -f https://deno.land/x/deploy/deployctl.ts

# Deploy
deployctl deploy --project=tayni-demo main.ts

# Or link to GitHub for automatic deployments
```

## Testing

```bash
# Local testing
deno task dev

# Test endpoints
curl http://localhost:8000/
# {"service":"TAYNI on Deno Deploy","version":"1.0.0","runtime":"Deno","status":"ok"}

curl -X POST -d '{"name":"TAYNI"}' http://localhost:8000/api/greet
# {"message":"Hello, TAYNI!","timestamp":1718780400}

curl -X POST -d '{"a":10,"b":5,"op":"mul"}' http://localhost:8000/api/compute
# {"a":10,"b":5,"op":"mul","result":50}
```

## Performance

Expected metrics on Deno Deploy:

| Metric | Value |
|--------|-------|
| Cold start | < 50ms |
| Warm response | < 5ms |
| Wasm size | ~8KB |
| Memory usage | < 5MB |

## Advantages of Deno Deploy

1. **Native Wasm support** - First-class WebAssembly
2. **Global edge network** - 35+ regions
3. **GitHub integration** - Auto-deploy on push
4. **Free tier** - 100K requests/day

## Limitations

1. **No filesystem** - Use Deno KV instead
2. **No raw sockets** - HTTP only
3. **50ms CPU limit** - Per request
4. **512MB memory** - Per isolate

## Next Steps

1. Add Deno KV integration for state
2. Implement streaming responses
3. Add WebSocket support
4. Create deployment GitHub Action

---

*Demo created: 2026-06-19*
*Status: Template ready, requires wasi-http implementation*
