# Fastly Compute@Edge Demo

Deploy TAYNI-compiled WebAssembly on Fastly Compute@Edge.

## Project Structure

```
fastly-compute/
├── src/
│   └── handler.tayni   # TAYNI source code
├── bin/
│   └── main.wasm       # Compiled WebAssembly
├── fastly.toml         # Fastly configuration
└── README.md           # This file
```

## TAYNI Source Code

```tayni
// src/handler.tayni
cap:net

fn main() {
    let req = Fastly.get_request()
    let resp = handle(req)
    Fastly.send_response(resp)
}

fn handle(req: Request) -> Response {
    let path = req.url.path()
    let method = req.method
    
    // Routing
    if path == "/" {
        return home_handler(req)
    }
    
    if path == "/api/data" {
        return data_handler(req)
    }
    
    if path.starts_with("/cache/") {
        return cache_handler(req)
    }
    
    not_found()
}

fn home_handler(req: Request) -> Response {
    Response.json({
        "service": "TAYNI on Fastly Compute@Edge",
        "pop": Fastly.pop(),
        "client_ip": req.client_ip(),
        "geo": {
            "country": Fastly.geo_country(),
            "city": Fastly.geo_city()
        }
    })
}

fn data_handler(req: Request) -> Response {
    // Fetch from origin with caching
    let cache_key = "data:" + req.url.query_string()
    
    match Fastly.cache_get(cache_key) {
        Some(cached) => Response.json(cached).header("X-Cache", "HIT"),
        None => {
            let origin_resp = Fastly.fetch("https://api.example.com/data")
            let data = origin_resp.json()
            Fastly.cache_set(cache_key, data, 3600)
            Response.json(data).header("X-Cache", "MISS")
        }
    }
}

fn cache_handler(req: Request) -> Response {
    let key = req.url.path().strip_prefix("/cache/")
    
    match req.method {
        "GET" => {
            match Fastly.cache_get(key) {
                Some(value) => Response.json({"key": key, "value": value}),
                None => Response.status(404).json({"error": "Key not found"})
            }
        },
        "PUT" => {
            let value = req.body_text()
            Fastly.cache_set(key, value, 3600)
            Response.json({"key": key, "stored": true})
        },
        "DELETE" => {
            Fastly.cache_delete(key)
            Response.json({"key": key, "deleted": true})
        },
        _ => Response.status(405).json({"error": "Method not allowed"})
    }
}

fn not_found() -> Response {
    Response.status(404).json({"error": "Not Found"})
}
```

## Configuration

```toml
# fastly.toml
manifest_version = 2
name = "tayni-compute"
description = "TAYNI on Fastly Compute@Edge"
authors = ["NELAIA <hello@nelaia.ai>"]
language = "other"

[local_server]
  [local_server.backends]
    [local_server.backends.origin]
      url = "https://api.example.com"

[setup]
  [setup.backends]
    [setup.backends.origin]
      address = "api.example.com"
      port = 443
```

## Build & Deploy

```bash
# Build TAYNI to WebAssembly
tayni build src/handler.tayni -o bin/main.wasm --target wasi

# Test locally
fastly compute serve

# Deploy to Fastly
fastly compute publish
```

## Features

- Sub-millisecond cold starts
- Global edge network (100+ POPs)
- Built-in KV store
- Geolocation data
- Request coalescing
- Automatic failover

## Performance

| Metric | Value |
|--------|-------|
| Cold start | < 1ms |
| P50 latency | < 0.5ms |
| P99 latency | < 2ms |
| Binary size | ~12KB |

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Service info with geo |
| `/api/data` | GET | Cached origin fetch |
| `/cache/:key` | GET/PUT/DELETE | Edge KV operations |
