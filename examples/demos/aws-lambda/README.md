# AWS Lambda Demo

Deploy TAYNI-compiled WebAssembly on AWS Lambda.

## Project Structure

```
aws-lambda/
├── src/
│   └── handler.tayni   # TAYNI source code
├── build/
│   └── bootstrap       # Lambda custom runtime
├── template.yaml       # SAM template
├── Makefile           # Build commands
└── README.md          # This file
```

## TAYNI Source Code

```tayni
// src/handler.tayni
cap:net

struct LambdaEvent {
    httpMethod: str,
    path: str,
    headers: Map<str, str>,
    queryStringParameters: Map<str, str>,
    body: str,
    isBase64Encoded: bool
}

struct LambdaResponse {
    statusCode: i32,
    headers: Map<str, str>,
    body: str,
    isBase64Encoded: bool
}

fn handler(event: LambdaEvent, context: LambdaContext) -> LambdaResponse {
    let path = event.path
    let method = event.httpMethod
    
    match (method, path) {
        ("GET", "/") => home_response(context),
        ("GET", "/health") => health_response(),
        ("GET", "/users") => list_users(),
        ("POST", "/users") => create_user(event.body),
        ("GET", p) if p.starts_with("/users/") => get_user(p),
        _ => not_found()
    }
}

fn home_response(ctx: LambdaContext) -> LambdaResponse {
    json_response(200, {
        "service": "TAYNI on AWS Lambda",
        "version": "1.0.0",
        "runtime": "WebAssembly",
        "function": ctx.function_name,
        "region": ENV.get("AWS_REGION"),
        "memory": ctx.memory_limit_mb,
        "remaining_time_ms": ctx.get_remaining_time_ms()
    })
}

fn health_response() -> LambdaResponse {
    json_response(200, {
        "status": "healthy",
        "timestamp": Time.now_iso()
    })
}

fn list_users() -> LambdaResponse {
    // In production, fetch from DynamoDB
    let users = [
        {"id": "1", "name": "Alice", "email": "alice@example.com"},
        {"id": "2", "name": "Bob", "email": "bob@example.com"}
    ]
    json_response(200, {"users": users})
}

fn create_user(body: str) -> LambdaResponse {
    let data = JSON.decode(body)
    let id = UUID.v4()
    
    // In production, save to DynamoDB
    let user = {
        "id": id,
        "name": data.name,
        "email": data.email,
        "created_at": Time.now_iso()
    }
    
    json_response(201, {"user": user})
}

fn get_user(path: str) -> LambdaResponse {
    let id = path.strip_prefix("/users/")
    
    // In production, fetch from DynamoDB
    if id == "1" {
        json_response(200, {
            "id": "1",
            "name": "Alice",
            "email": "alice@example.com"
        })
    } else {
        json_response(404, {"error": "User not found"})
    }
}

fn not_found() -> LambdaResponse {
    json_response(404, {"error": "Not Found"})
}

fn json_response(status: i32, data: any) -> LambdaResponse {
    LambdaResponse {
        statusCode: status,
        headers: {
            "Content-Type": "application/json",
            "X-Powered-By": "TAYNI"
        },
        body: JSON.encode(data),
        isBase64Encoded: false
    }
}
```

## SAM Template

```yaml
# template.yaml
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31
Description: TAYNI on AWS Lambda

Globals:
  Function:
    Timeout: 30
    MemorySize: 128

Resources:
  TayniFunction:
    Type: AWS::Serverless::Function
    Properties:
      FunctionName: tayni-handler
      Runtime: provided.al2
      Handler: bootstrap
      CodeUri: build/
      Architectures:
        - arm64
      Events:
        Api:
          Type: HttpApi
          Properties:
            Path: /{proxy+}
            Method: ANY
        Root:
          Type: HttpApi
          Properties:
            Path: /
            Method: ANY

Outputs:
  ApiEndpoint:
    Description: API Gateway endpoint URL
    Value: !Sub "https://${ServerlessHttpApi}.execute-api.${AWS::Region}.amazonaws.com/"
```

## Makefile

```makefile
.PHONY: build deploy clean

build:
	tayni build src/handler.tayni -o build/bootstrap --target wasi-lambda
	chmod +x build/bootstrap

deploy: build
	sam deploy --guided

local: build
	sam local start-api

clean:
	rm -rf build/ .aws-sam/
```

## Deployment

```bash
# Build
make build

# Deploy (first time)
make deploy

# Test locally
make local
curl http://localhost:3000/
```

## Performance

| Metric | Value |
|--------|-------|
| Cold start (ARM64) | ~50ms |
| Warm latency | < 5ms |
| Binary size | ~20KB |
| Memory usage | < 32MB |

## Cost Optimization

- Use ARM64 (Graviton2) for 20% cost savings
- 128MB memory is sufficient for most workloads
- Enable Provisioned Concurrency for latency-critical paths
