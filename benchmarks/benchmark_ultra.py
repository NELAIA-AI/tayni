"""
NELAIA AI Generation Benchmark - ULTRA Performance Test
Compares NELAIA Ultra (all optimizations) vs competitors
"""
import socket
import time
import threading
import os
import subprocess
import sys

REQUESTS = 1000
CONCURRENT = 20

def benchmark_server(port, name, requests=REQUESTS):
    results = {'success': 0, 'errors': 0, 'latencies': []}
    lock = threading.Lock()
    
    def make_request():
        try:
            start = time.perf_counter()
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.settimeout(2)
            s.connect(('127.0.0.1', port))
            s.send(b'GET / HTTP/1.1\r\nHost: localhost\r\n\r\n')
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
    
    for i in range(requests):
        t = threading.Thread(target=make_request)
        threads.append(t)
        t.start()
        if len(threads) >= CONCURRENT:
            for t in threads:
                t.join()
            threads = []
    
    for t in threads:
        t.join()
    
    total_time = time.perf_counter() - start_time
    
    avg_latency = sum(results['latencies']) / len(results['latencies']) if results['latencies'] else 0
    sorted_lat = sorted(results['latencies']) if results['latencies'] else [0]
    p50_idx = int(len(sorted_lat) * 0.50)
    p99_idx = int(len(sorted_lat) * 0.99)
    p50_latency = sorted_lat[min(p50_idx, len(sorted_lat)-1)]
    p99_latency = sorted_lat[min(p99_idx, len(sorted_lat)-1)]
    min_latency = min(sorted_lat) if sorted_lat else 0
    
    return {
        'name': name,
        'success': results['success'],
        'errors': results['errors'],
        'total_time': total_time,
        'req_per_sec': results['success'] / total_time if total_time > 0 else 0,
        'avg_latency_ms': avg_latency,
        'min_latency_ms': min_latency,
        'p50_latency_ms': p50_latency,
        'p99_latency_ms': p99_latency
    }

def file_size(filepath):
    try:
        return os.path.getsize(filepath)
    except:
        return 0

def count_chars(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            return len(f.read())
    except:
        return 0

def main():
    print("=" * 90)
    print("NELAIA ULTRA PERFORMANCE BENCHMARK")
    print("=" * 90)
    print(f"Test: {REQUESTS} requests, {CONCURRENT} concurrent connections")
    print()
    
    servers = [
        ('NELAIA Ultra', 8092, 'solar_nelaia_ultra.nela', 'solar_nelaia_ultra.exe'),
        ('NELAIA Opt', 8084, 'solar_nelaia_opt.nela', 'solar_nelaia_opt.exe'),
        ('NELAIA Fair', 8090, 'solar_nelaia_fair.nela', 'solar_nelaia_fair.exe'),
        ('Go 1.22', 8083, 'solar_go.go', 'solar_go.exe'),
        ('Rust 1.78', 8086, 'solar_rust.rs', 'solar_rust.exe'),
        ('C (Clang)', 8085, 'solar_c.c', 'solar_c.exe'),
    ]
    
    # Start all servers
    print("Starting servers...")
    procs = []
    
    for name, port, src, binary in servers:
        if binary:
            p = subprocess.Popen([binary], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            procs.append((name, p))
            print(f"  Started {name} on port {port}")
    
    time.sleep(2)
    
    print()
    print("=" * 90)
    print("PERFORMANCE RESULTS")
    print("=" * 90)
    print()
    print(f"{'Server':<15} {'Req/s':>10} {'Min ms':>10} {'P50 ms':>10} {'P99 ms':>10} {'Success':>10} {'Errors':>8}")
    print("-" * 90)
    
    results = []
    for name, port, src, binary in servers:
        try:
            r = benchmark_server(port, name)
            results.append(r)
            print(f"{name:<15} {r['req_per_sec']:>10.1f} {r['min_latency_ms']:>10.2f} {r['p50_latency_ms']:>10.2f} {r['p99_latency_ms']:>10.2f} {r['success']:>10} {r['errors']:>8}")
        except Exception as e:
            print(f"{name:<15} {'ERROR':>10} - {e}")
    
    # Kill servers
    for name, p in procs:
        p.terminate()
    
    print()
    print("=" * 90)
    print("EFFICIENCY ANALYSIS")
    print("=" * 90)
    print()
    
    # Find NELAIA Ultra for comparison
    ultra = next((r for r in results if 'Ultra' in r['name']), None)
    if ultra:
        print(f"{'Server':<15} {'vs Ultra':>12} {'Binary':>15} {'Tokens':>10}")
        print("-" * 55)
        for i, (name, port, src, binary) in enumerate(servers):
            if i < len(results):
                ratio = results[i]['req_per_sec'] / ultra['req_per_sec'] if ultra['req_per_sec'] > 0 else 0
                bin_size = file_size(binary) if binary else 0
                tokens = count_chars(src) // 4
                bin_str = f"{bin_size:,}" if bin_size > 0 else "N/A"
                print(f"{name:<15} {ratio:>11.2f}x {bin_str:>15} {tokens:>10}")
    
    print()
    print("=" * 90)
    print("NELAIA ULTRA OPTIMIZATIONS")
    print("=" * 90)
    print()
    print("Primitives used:")
    print("  - NDL: TCP_NODELAY (disable Nagle algorithm)")
    print("  - SBF: SO_SNDBUF/SO_RCVBUF = 64KB (large buffers)")
    print("  - LST: backlog = 4096 (high connection queue)")
    print()
    print("New primitives available:")
    print("  - QCK: TCP_QUICKACK (immediate ACK, Linux)")
    print("  - KAL: SO_KEEPALIVE (connection keep-alive)")
    print("  - EPL: epoll_create (Linux) / IOCP (Windows)")
    print("  - ECT: epoll_ctl (add/mod/del fd)")
    print("  - EWA: epoll_wait (wait for events)")
    print()
    print("=" * 90)

if __name__ == '__main__':
    main()
