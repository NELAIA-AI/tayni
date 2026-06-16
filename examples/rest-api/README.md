# TAYNI REST API Example

This example demonstrates a complete REST API server and client using TAYNI's standard library.

## Modules Used

| Module | Purpose |
|--------|---------|
| `http` | HTTP server/client |
| `json` | JSON parsing/encoding |
| `router` | URL routing |
| `cors` | CORS handling |
| `log` | Logging |

## Files

- `server.tyn` - REST API server with CRUD endpoints
- `client.tyn` - HTTP client that queries the server

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/users` | List all users |
| GET | `/api/users/:id` | Get user by ID |
| POST | `/api/users` | Create new user |
| PUT | `/api/users/:id` | Update user |
| DELETE | `/api/users/:id` | Delete user |

## Usage

### 1. Compile

```bash
# Compile server
tayni-c server.tyn -o server

# Compile client
tayni-c client.tyn -o client
```

### 2. Run Server

```bash
./server
# Output: Starting REST API server on port 8080
#         Server listening...
```

### 3. Run Client (in another terminal)

```bash
./client
# Output:
# Health Check: {"status":"ok","version":"1.0"}
# Users List: [{"id":1,"name":"Alice"},{"id":2,"name":"Bob"},{"id":3,"name":"Charlie"}]
# User 1: {"id":1,"name":"Alice","email":"alice@example.com"}
# User 2: {"id":2,"name":"Bob","email":"bob@example.com"}
# User 99 (404): {"error":"User not found"}
# Create User: {"id":4,"name":"New User","created":true}
# Parsed name: Alice
# Client finished.
```

### 4. Test with curl

```bash
# Health check
curl http://localhost:8080/api/health
# {"status":"ok","version":"1.0"}

# List users
curl http://localhost:8080/api/users
# [{"id":1,"name":"Alice"},{"id":2,"name":"Bob"},{"id":3,"name":"Charlie"}]

# Get specific user
curl http://localhost:8080/api/users/1
# {"id":1,"name":"Alice","email":"alice@example.com"}

# Create user
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Diana","email":"diana@example.com"}'
# {"id":4,"name":"New User","created":true}

# Non-existent user
curl http://localhost:8080/api/users/99
# {"error":"User not found"}
```

## Response Format

All responses are JSON:

```json
// Success
{
  "id": 1,
  "name": "Alice",
  "email": "alice@example.com"
}

// Error
{
  "error": "User not found"
}

// Health
{
  "status": "ok",
  "version": "1.0"
}
```

## CORS Support

The server includes CORS headers for cross-origin requests:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
```

## AI-Native Design

This example demonstrates TAYNI's AI-native approach:

1. **No loops** - Request handling uses conditional selection (`IFZ`)
2. **Declarative** - Routes defined as data, not control flow
3. **Minimal syntax** - Entire server in ~150 lines
4. **Zero dependencies** - Direct syscalls, no runtime
