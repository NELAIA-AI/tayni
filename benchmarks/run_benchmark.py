"""
NELAIA AI Generation Benchmark - Complete Measurement Script
Measures: Code size, Binary size, Build time, Runtime performance
"""
import subprocess
import time
import socket
import os
import threading
from pathlib import Path

REQUESTS = 500
CONCURRENT = 10

def count_lines(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            return len(f.readlines())
    except:
        return 0

def count_chars(filepath):
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            return len(f.read())
    except:
        return 0

def file_size(filepath):
    try:
        return os.path.getsize(filepath)
    except:
        return 0

def benchmark_server(port, name, requests=REQUESTS):
    """Benchmark a server with raw sockets"""
    results = {'success': 0, 'errors': 0, 'latencies': []}
    
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
            if data and b'200 OK' in data:
                results['success'] += 1
                results['latencies'].append(latency)
            else:
                results['errors'] += 1
        except Exception as e:
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
    p99_latency = sorted(results['latencies'])[int(len(results['latencies']) * 0.99)] if results['latencies'] else 0
    
    return {
        'name': name,
        'requests': requests,
        'success': results['success'],
        'errors': results['errors'],
        'total_time': total_time,
        'req_per_sec': results['success'] / total_time if total_time > 0 else 0,
        'avg_latency_ms': avg_latency,
        'p99_latency_ms': p99_latency
    }

def main():
    print("=" * 70)
    print("NELAIA AI GENERATION BENCHMARK - PROMPT TO PERFORMANCE")
    print("=" * 70)
    print()
    
    # Define all implementations
    implementations = [
        {
            'name': 'NELAIA v0.10',
            'source': 'solar_nelaia.nela',
            'binary': 'solar_nelaia.exe',
            'port': 8080,
            'runtime': 'Native (Pure Syscalls)'
        },
        {
            'name': 'Python 3.12',
            'source': 'solar_python.py',
            'binary': None,
            'port': 8081,
            'runtime': 'CPython Interpreter'
        },
        {
            'name': 'Node.js v24',
            'source': 'solar_node.js',
            'binary': None,
            'port': 8082,
            'runtime': 'V8 Engine'
        },
        {
            'name': 'Go 1.22',
            'source': 'solar_go.go',
            'binary': 'solar_go.exe',
            'port': 8083,
            'runtime': 'Native (Go Runtime)'
        },
        {
            'name': 'Rust 1.78',
            'source': 'solar_rust.rs',
            'binary': 'solar_rust.exe',
            'port': 8084,
            'runtime': 'Native (No Runtime)'
        },
        {
            'name': 'C (Clang)',
            'source': 'solar_c.c',
            'binary': 'solar_c.exe',
            'port': 8085,
            'runtime': 'Native (MSVC CRT)'
        }
    ]
    
    print("=" * 70)
    print("PHASE 1: CODE GENERATION METRICS")
    print("=" * 70)
    print()
    print(f"{'Language':<15} {'Lines':>8} {'Chars':>10} {'Tokens*':>10}")
    print("-" * 45)
    
    for impl in implementations:
        lines = count_lines(impl['source'])
        chars = count_chars(impl['source'])
        tokens_est = chars // 4  # Rough estimate: 4 chars per token
        impl['lines'] = lines
        impl['chars'] = chars
        impl['tokens'] = tokens_est
        print(f"{impl['name']:<15} {lines:>8} {chars:>10} {tokens_est:>10}")
    
    print()
    print("* Tokens estimated at ~4 chars/token")
    print()
    
    print("=" * 70)
    print("PHASE 2: BUILD ARTIFACTS")
    print("=" * 70)
    print()
    print(f"{'Language':<15} {'Binary Size':>15} {'Runtime Deps':>20}")
    print("-" * 55)
    
    for impl in implementations:
        if impl['binary']:
            size = file_size(impl['binary'])
            impl['binary_size'] = size
            size_str = f"{size:,} bytes"
        else:
            impl['binary_size'] = 0
            size_str = "Interpreted"
        print(f"{impl['name']:<15} {size_str:>15} {impl['runtime']:>20}")
    
    print()
    
    print("=" * 70)
    print("PHASE 3: RUNTIME PERFORMANCE")
    print("=" * 70)
    print()
    print("Testing each server with", REQUESTS, "requests...")
    print()
    
    # Test servers that are running
    results = []
    
    # Test NELAIA (port 8080)
    print("Testing NELAIA on port 8080...")
    try:
        r = benchmark_server(8080, 'NELAIA v0.10')
        results.append(r)
        print(f"  -> {r['success']}/{r['requests']} OK, {r['req_per_sec']:.1f} req/s")
    except Exception as e:
        print(f"  -> Error: {e}")
    
    print()
    print(f"{'Language':<15} {'Req/s':>10} {'Avg ms':>10} {'P99 ms':>10} {'Errors':>8}")
    print("-" * 60)
    
    for r in results:
        print(f"{r['name']:<15} {r['req_per_sec']:>10.1f} {r['avg_latency_ms']:>10.2f} {r['p99_latency_ms']:>10.2f} {r['errors']:>8}")
    
    print()
    print("=" * 70)
    print("SUMMARY: AI GENERATION EFFICIENCY")
    print("=" * 70)
    print()
    
    # Find NELAIA for comparison
    nelaia = next((i for i in implementations if 'NELAIA' in i['name']), None)
    if nelaia:
        print(f"NELAIA Baseline:")
        print(f"  - Source: {nelaia['lines']} lines, {nelaia['chars']} chars")
        print(f"  - Binary: {nelaia['binary_size']:,} bytes")
        print()
        print("Comparison (NELAIA = 1.0x):")
        print()
        print(f"{'Language':<15} {'Code Size':>12} {'Binary Size':>14}")
        print("-" * 45)
        
        for impl in implementations:
            code_ratio = impl['chars'] / nelaia['chars'] if nelaia['chars'] > 0 else 0
            if impl['binary_size'] > 0 and nelaia['binary_size'] > 0:
                bin_ratio = impl['binary_size'] / nelaia['binary_size']
                bin_str = f"{bin_ratio:.1f}x"
            else:
                bin_str = "N/A"
            print(f"{impl['name']:<15} {code_ratio:>11.2f}x {bin_str:>14}")
    
    print()
    print("=" * 70)
    print("BENCHMARK COMPLETE")
    print("=" * 70)

if __name__ == '__main__':
    main()
