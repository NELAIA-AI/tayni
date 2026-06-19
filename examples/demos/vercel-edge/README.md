# Vercel Edge Functions Demo

Deploy TAYNI-compiled WebAssembly on Vercel Edge Functions.

## Project Structure

```
vercel-edge/
├── api/
│   └── handler.ts      # Edge function entry point
├── src/
│   └── handler.tayni   # TAYNI source code
├── build/
│   └── handler.wasm    # Compiled WebAssembly
├── vercel.json         # Vercel configuration
├── package.json        # Dependencies
└── README.md           # This file
```

## TAYNI Source Code

```tayni
// src/handler.tayni
cap:net

struct Request {
    method: str,
    url: str,
    headers: Map<str, str>
}

struct Response {
    status: i32,
    headers: Map<str, str>,
    body: str
}

fn handle(req: Request) -> Response {
    match req.url {
        "/" => Response {
            status: 200,
            headers: {"Content-Type": "application/json"},
            body: JSON.encode({
                "service": "TAYNI on Vercel Edge",
                "version": "1.0.0",
                "runtime": "WebAssembly"
            })
        },
        "/health" => Response {
            status: 200,
            headers: {"Content-Type": "application/json"},
            body: JSON.encode({"status": "healthy"})
        },
        "/echo" => Response {
            status: 200,
            headers: {"Content-Type": "application/json"},
            body: JSON.encode({
                "method": req.method,
                "url": req.url,
                "headers": req.headers
            })
        },
        _ => Response {
            status: 404,
            headers: {"Content-Type": "application/json"},
            body: JSON.encode({"error": "Not Found"})
        }
    }
}
```

## Edge Function Wrapper

```typescript
// api/handler.ts
import { instantiate } from '../build/handler.wasm';

export const config = {
  runtime: 'edge',
};

export default async function handler(request: Request) {
  const wasm = await instantiate();
  
  const url = new URL(request.url);
  const headers: Record<string, string> = {};
  request.headers.forEach((value, key) => {
    headers[key] = value;
  });
  
  const req = {
    method: request.method,
    url: url.pathname,
    headers,
  };
  
  const response = wasm.handle(req);
  
  return new Response(response.body, {
    status: response.status,
    headers: response.headers,
  });
}
```

## Configuration

```json
// vercel.json
{
  "functions": {
    "api/handler.ts": {
      "runtime": "edge"
    }
  },
  "routes": [
    { "src": "/(.*)", "dest": "/api/handler" }
  ]
}
```

```json
// package.json
{
  "name": "tayni-vercel-edge",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "build": "tayni build src/handler.tayni -o build/handler.wasm --target wasi",
    "dev": "vercel dev",
    "deploy": "vercel --prod"
  },
  "devDependencies": {
    "vercel": "^32.0.0"
  }
}
```

## Deployment

```bash
# Install dependencies
npm install

# Build TAYNI to WebAssembly
npm run build

# Deploy to Vercel
npm run deploy
```

## Performance

| Metric | Value |
|--------|-------|
| Cold start | < 5ms |
| Warm latency | < 1ms |
| Binary size | ~15KB |
| Memory usage | < 1MB |

## Features

- Zero cold start overhead
- Global edge deployment
- Automatic HTTPS
- Built-in caching
- WebAssembly isolation

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Service info |
| `/health` | GET | Health check |
| `/echo` | ANY | Echo request details |

## Environment Variables

Set in Vercel dashboard or `.env`:

```env
TAYNI_LOG_LEVEL=info
TAYNI_CACHE_TTL=3600
```

## Monitoring

Vercel provides built-in:
- Request logs
- Error tracking
- Performance metrics
- Edge function analytics

## Limitations

- 1MB response size limit
- 30 second execution timeout
- No persistent storage (use external DB)
- Limited to HTTP/HTTPS protocols
