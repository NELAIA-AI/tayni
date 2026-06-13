"""
NELAIA Multi-Process Benchmark
Runs multiple NELAIA instances to simulate multi-threading
"""
import socket
import time
import threading
import subprocess
import random
import os

REQUESTS = 1000
CONCURRENT = 50

def benchmark_multi(ports, n=REQUESTS, concurrent=CONCURRENT):
    results = {'success': 0, 'errors': 0, 'latencies': []}
    lock = threading.Lock()
    
    def req():
        try:
            port = random.choice(ports)
            start = time.perf_counter()
            s = socket.socket()
            s.settimeout(2)
            s.connect(('127.0.0.1', port))
            s.send(b'GET / HTTP/1.1\r\nHost: x\r\n\r\n')
            data = s.recv(4096)
            s.close()
            latency = (time.perf_counter() - start) * 1000
            with lock:
                if data and b'200 OK' in data:
                    results['success'] += 1
                    results['latencies'].append(latency)
                else:
                    results['errors'] += 1
        except:
            with lock:
                results['errors'] += 1
    
    threads = []
    start_time = time.perf_counter()
    
    for i in range(n):
        t = threading.Thread(target=req)
        threads.append(t)
        t.start()
        if len(threads) >= concurrent:
            for t in threads:
                t.join()
            threads = []
    
    for t in threads:
        t.join()
    
    elapsed = time.perf_counter() - start_time
    
    avg_lat = sum(results['latencies']) / len(results['latencies']) if results['latencies'] else 0
    sorted_lat = sorted(results['latencies']) if results['latencies'] else [0]
    p99_idx = min(int(len(sorted_lat) * 0.99), len(sorted_lat)-1)
    
    return {
        'success': results['success'],
        'errors': results['errors'],
        'req_per_sec': results['success'] / elapsed if elapsed > 0 else 0,
        'avg_latency': avg_lat,
        'p99_latency': sorted_lat[p99_idx] if sorted_lat else 0
    }

def main():
    print("=" * 80)
    print("NELAIA MULTI-PROCESS vs GO BENCHMARK")
    print(f"Test: {REQUESTS} requests, {CONCURRENT} concurrent")
    print("=" * 80)
    print()
    
    # We need to create NELAIA servers on different ports
    # Port calculation: high_byte * 256 + low_byte
    # 8092 = 31*256 + 156
    # 8093 = 31*256 + 157
    # etc.
    
    # For now, test with single instances
    procs = []
    
    # Start Go (single instance, but uses goroutines internally)
    procs.append(('Go', subprocess.Popen(['solar_go.exe'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)))
    
    # Start NELAIA (single instance)
    procs.append(('NELAIA x1', subprocess.Popen(['solar_nelaia_ultra.exe'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)))
    
    time.sleep(1)
    
    print("Testing single instances...")
    print()
    print(f"{'Server':<20} {'Req/s':>10} {'Avg ms':>10} {'P99 ms':>10} {'Success':>10} {'Errors':>8}")
    print("-" * 75)
    
    # Test Go
    r = benchmark_multi([8083])
    print(f"{'Go (goroutines)':<20} {r['req_per_sec']:>10.0f} {r['avg_latency']:>10.2f} {r['p99_latency']:>10.2f} {r['success']:>10} {r['errors']:>8}")
    
    # Test NELAIA single
    r = benchmark_multi([8092])
    print(f"{'NELAIA x1':<20} {r['req_per_sec']:>10.0f} {r['avg_latency']:>10.2f} {r['p99_latency']:>10.2f} {r['success']:>10} {r['errors']:>8}")
    
    # Cleanup
    for name, p in procs:
        p.terminate()
    
    print()
    print("=" * 80)
    print("CONCLUSION")
    print("=" * 80)
    print()
    print("Go uses goroutines (M:N threading) internally - one process handles all.")
    print("NELAIA is single-threaded - needs multiple processes for parallelism.")
    print()
    print("To match Go's concurrency model, NELAIA needs:")
    print("  1. Thread primitives (THR/JON) - IMPLEMENTED")
    print("  2. Function/subroutine support - NEEDED")
    print("  3. Shared state between threads - NEEDED")
    print()
    print("Alternative: Run N NELAIA instances behind a load balancer (like nginx)")
    print()

if __name__ == '__main__':
    main()
