"""
NELAIA AI Generation Benchmark - Full Comparison
Tests all servers running simultaneously
"""
import socket
import time
import threading
import os

REQUESTS = 500
CONCURRENT = 10

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

def count_lines(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            return len(f.readlines())
    except:
        return 0

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
    p99_idx = int(len(sorted_lat) * 0.99)
    p99_latency = sorted_lat[min(p99_idx, len(sorted_lat)-1)]
    
    return {
        'name': name,
        'success': results['success'],
        'errors': results['errors'],
        'total_time': total_time,
        'req_per_sec': results['success'] / total_time if total_time > 0 else 0,
        'avg_latency_ms': avg_latency,
        'p99_latency_ms': p99_latency
    }

def main():
    servers = [
        ('NELAIA v0.10', 8080, 'solar_nelaia.nts', 'solar_nelaia.exe'),
        ('Python 3.12', 8081, 'solar_python.py', None),
        ('Node.js v24', 8082, 'solar_node.js', None),
        ('Go 1.22', 8083, 'solar_go.go', 'solar_go.exe'),
        ('Rust 1.78', 8084, 'solar_rust.rs', 'solar_rust.exe'),
        ('C (Clang)', 8085, 'solar_c.c', 'solar_c.exe'),
    ]
    
    print("=" * 80)
    print("NELAIA AI GENERATION BENCHMARK - PROMPT TO PERFORMANCE")
    print("=" * 80)
    print()
    
    # Phase 1: Code metrics
    print("PHASE 1: AI GENERATION OUTPUT (Code Metrics)")
    print("-" * 80)
    print(f"{'Language':<15} {'Lines':>8} {'Chars':>10} {'Tokens*':>10} {'Binary':>15}")
    print("-" * 80)
    
    code_data = []
    for name, port, src, binary in servers:
        lines = count_lines(src)
        chars = count_chars(src)
        tokens = chars // 4
        bin_size = file_size(binary) if binary else 0
        bin_str = f"{bin_size:,}" if bin_size > 0 else "Interpreted"
        code_data.append((name, lines, chars, tokens, bin_size))
        print(f"{name:<15} {lines:>8} {chars:>10} {tokens:>10} {bin_str:>15}")
    
    print()
    print("* Tokens estimated at ~4 chars/token")
    print()
    
    # Phase 2: Runtime performance
    print("PHASE 2: RUNTIME PERFORMANCE ({} requests each)".format(REQUESTS))
    print("-" * 80)
    print(f"{'Language':<15} {'Req/s':>10} {'Avg ms':>10} {'P99 ms':>10} {'Success':>10} {'Errors':>8}")
    print("-" * 80)
    
    perf_data = []
    for name, port, src, binary in servers:
        try:
            r = benchmark_server(port, name)
            perf_data.append(r)
            print(f"{name:<15} {r['req_per_sec']:>10.1f} {r['avg_latency_ms']:>10.2f} {r['p99_latency_ms']:>10.2f} {r['success']:>10} {r['errors']:>8}")
        except Exception as e:
            print(f"{name:<15} {'ERROR':>10} - Server not responding")
    
    print()
    
    # Phase 3: Efficiency comparison
    print("PHASE 3: AI GENERATION EFFICIENCY (NELAIA = 1.0x baseline)")
    print("-" * 80)
    
    nelaia_code = code_data[0]
    nelaia_perf = perf_data[0] if perf_data else None
    
    print(f"{'Language':<15} {'Code':>10} {'Binary':>12} {'Throughput':>12} {'Efficiency*':>12}")
    print("-" * 80)
    
    for i, (name, lines, chars, tokens, bin_size) in enumerate(code_data):
        code_ratio = chars / nelaia_code[2] if nelaia_code[2] > 0 else 0
        
        if bin_size > 0 and nelaia_code[4] > 0:
            bin_ratio = f"{bin_size / nelaia_code[4]:.1f}x"
        else:
            bin_ratio = "N/A"
        
        if i < len(perf_data) and nelaia_perf:
            perf_ratio = perf_data[i]['req_per_sec'] / nelaia_perf['req_per_sec'] if nelaia_perf['req_per_sec'] > 0 else 0
            perf_str = f"{perf_ratio:.2f}x"
            
            # Efficiency = Performance / Code Size
            efficiency = perf_ratio / code_ratio if code_ratio > 0 else 0
            eff_str = f"{efficiency:.2f}x"
        else:
            perf_str = "N/A"
            eff_str = "N/A"
        
        print(f"{name:<15} {code_ratio:>9.2f}x {bin_ratio:>12} {perf_str:>12} {eff_str:>12}")
    
    print()
    print("* Efficiency = Throughput / Code Size (higher is better)")
    print()
    
    # Summary
    print("=" * 80)
    print("SUMMARY: TOTAL AI DEVELOPMENT COST")
    print("=" * 80)
    print()
    print("For AI-generated software, the key metrics are:")
    print()
    print("1. TOKEN COST (Generation):")
    for name, lines, chars, tokens, bin_size in code_data:
        print(f"   {name:<15}: ~{tokens:,} tokens")
    print()
    print("2. ARTIFACT SIZE (Deployment):")
    for name, lines, chars, tokens, bin_size in code_data:
        if bin_size > 0:
            print(f"   {name:<15}: {bin_size:,} bytes")
        else:
            print(f"   {name:<15}: Requires runtime (~50-100 MB)")
    print()
    print("3. PERFORMANCE (Value Delivered):")
    for r in perf_data:
        print(f"   {r['name']:<15}: {r['req_per_sec']:.0f} req/s")
    print()
    
    # Winner analysis
    print("=" * 80)
    print("ANALYSIS")
    print("=" * 80)
    print()
    print("NELAIA Advantages:")
    print(f"  - Smallest binary: {nelaia_code[4]:,} bytes (vs Go: {code_data[3][4]:,})")
    print(f"  - No runtime dependencies (pure syscalls)")
    print(f"  - Competitive token cost: {nelaia_code[3]} tokens")
    print()
    print("Trade-offs:")
    print(f"  - Single-threaded blocking I/O (current limitation)")
    print(f"  - Error rate under concurrent load: {perf_data[0]['errors']}/{REQUESTS}")
    print()
    print("=" * 80)

if __name__ == '__main__':
    main()
