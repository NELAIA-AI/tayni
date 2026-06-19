# TAYNI on Cloudflare Workers

This demo shows how to run TAYNI-compiled WebAssembly on Cloudflare Workers.

## Prerequisites

- Cloudflare account
- Wrangler CLI (`npm install -g wrangler`)
- TAYNI compiler

## Project Structure

```
cloudflare-worker/
├── README.md
├── wrangler.toml
├── src/
│   ├── index.js      # Worker entry point
│   └── handler.tayni # TAYNI source
└── build/
    └── handler.wasm  # Compiled Wasm
```

## TAYNI Source

```tayni
// handler.tayni - Cloudflare Worker handler
use http

fn handle(request: Request) -> Response {
    let path = request.path()
    
    if path == "/" {
        Response.json({
            "service": "TAYNI on Cloudflare Workers",
            "version": "1.0.0",
            "status": "ok"
        })
    } else if path == "/hello" {
        let name = request.query("name") ?? "World"
        Response.text("Hello, " + name + "!")
    } else if path == "/echo" {
        let body = request.body()
        Response.json({
            "method": request.method(),
            "path": path,
            "body": body
        })
    } else {
        Response.status(404).json({
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

# Verify the output
wasm-tools validate build/handler.wasm
```

## Worker Entry Point

```javascript
// src/index.js
import handler from '../build/handler.wasm';

export default {
  async fetch(request, env, ctx) {
    // Instantiate the Wasm module
    const instance = await WebAssembly.instantiate(handler, {
      env: {
        // Provide imports for TAYNI runtime
        print: (ptr, len) => {
          // Handle print calls
        }
      }
    });
    
    // Call the handler
    const response = instance.exports.handle(request);
    return response;
  }
};
```

## Wrangler Configuration

```toml
# wrangler.toml
name = "tayni-demo"
main = "src/index.js"
compatibility_date = "2024-01-01"

[build]
command = "tayni compile src/handler.tayni -o build/handler.wasm --target wasm"

[[rules]]
type = "CompiledWasm"
globs = ["**/*.wasm"]
```

## Deployment

```bash
# Login to Cloudflare
wrangler login

# Deploy
wrangler deploy

# Test locally
wrangler dev
```

## Testing

```bash
# Test root endpoint
curl https://tayni-demo.your-subdomain.workers.dev/
# {"service":"TAYNI on Cloudflare Workers","version":"1.0.0","status":"ok"}

# Test hello endpoint
curl https://tayni-demo.your-subdomain.workers.dev/hello?name=TAYNI
# Hello, TAYNI!

# Test echo endpoint
curl -X POST -d '{"test":true}' https://tayni-demo.your-subdomain.workers.dev/echo
# {"method":"POST","path":"/echo","body":"{\"test\":true}"}
```

## Performance

Expected metrics on Cloudflare Workers:

| Metric | Value |
|--------|-------|
| Cold start | < 5ms |
| Warm response | < 1ms |
| Wasm size | ~8KB |
| Memory usage | < 1MB |

## Limitations

Current limitations when running TAYNI on Cloudflare Workers:

1. **No filesystem access** - Workers are stateless
2. **No raw sockets** - Use fetch() API instead
3. **Execution time limit** - 50ms CPU time (free), 30s (paid)
4. **Memory limit** - 128MB

## Next Steps

1. Implement wasi-http bindings for native fetch
2. Add KV storage integration
3. Add D1 database integration
4. Add R2 storage integration

---

*Demo created: 2026-06-19*
*Status: Template ready, requires wasi-http implementation*
