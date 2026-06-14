import socket
import time
import sys

def benchmark(host, port, requests, name):
    print(f"\n=== {name} ===")
    
    # Warmup
    print("Warmup (100 requests)...")
    for _ in range(100):
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.connect((host, port))
            s.send(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            s.recv(4096)
            s.close()
        except:
            pass
    
    # Benchmark
    print(f"Benchmarking ({requests} requests)...")
    times = []
    errors = 0
    
    start = time.perf_counter()
    for i in range(requests):
        try:
            req_start = time.perf_counter()
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.settimeout(5)
            s.connect((host, port))
            s.send(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            s.recv(4096)
            s.close()
            times.append((time.perf_counter() - req_start) * 1000)
        except Exception as e:
            errors += 1
    
    total = time.perf_counter() - start
    successful = requests - errors
    rps = successful / total
    
    if times:
        times.sort()
        avg = sum(times) / len(times)
        p50 = times[int(len(times) * 0.5)]
        p99 = times[int(len(times) * 0.99)]
        min_t = min(times)
        max_t = max(times)
    else:
        avg = p50 = p99 = min_t = max_t = 0
    
    print(f"Results:")
    print(f"  Requests/sec: {rps:.2f}")
    print(f"  Total time:   {total:.2f}s")
    print(f"  Successful:   {successful} / {requests}")
    print(f"  Errors:       {errors}")
    print(f"  Latency avg:  {avg:.3f}ms")
    print(f"  Latency min:  {min_t:.3f}ms")
    print(f"  Latency max:  {max_t:.3f}ms")
    print(f"  Latency p50:  {p50:.3f}ms")
    print(f"  Latency p99:  {p99:.3f}ms")
    
    return {"name": name, "rps": rps, "avg": avg, "errors": errors}

if __name__ == "__main__":
    requests = 5000
    print("HTTP Server Benchmark - FAIR (all single-threaded blocking)")
    print("=" * 55)
    print(f"Requests per test: {requests}")
    
    results = []
    results.append(benchmark("127.0.0.1", 8080, requests, "NELAIA v0.9 (5KB)"))
    results.append(benchmark("127.0.0.1", 8081, requests, "Python 3 (socket)"))
    results.append(benchmark("127.0.0.1", 8082, requests, "Node.js (http)"))
    
    print("\n=== COMPARISON ===")
    print(f"{'Server':<30} {'Req/s':>10} {'Avg(ms)':>10} {'Errors':>8}")
    print("-" * 60)
    for r in sorted(results, key=lambda x: -x["rps"]):
        print(f"{r['name']:<30} {r['rps']:>10.2f} {r['avg']:>10.3f} {r['errors']:>8}")
