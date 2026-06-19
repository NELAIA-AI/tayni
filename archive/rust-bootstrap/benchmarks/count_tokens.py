#!/usr/bin/env python3
"""
Token Counter for Cross-Language Benchmark
Uses tiktoken (OpenAI's tokenizer) to count tokens for each HTTP server implementation
"""

import os
import sys

try:
    import tiktoken
except ImportError:
    print("Installing tiktoken...")
    os.system(f"{sys.executable} -m pip install tiktoken -q")
    import tiktoken

def count_tokens(text, model="gpt-4"):
    """Count tokens using tiktoken"""
    try:
        encoding = tiktoken.encoding_for_model(model)
    except KeyError:
        encoding = tiktoken.get_encoding("cl100k_base")
    return len(encoding.encode(text))

def main():
    benchmark_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Define all source files
    sources = {
        "TAYNI": """SEN http
@http:8080 -> srv
srv.ACC -> cli
cli.SND "HTTP/1.1 200 OK\\r\\nContent-Type: application/json\\r\\nContent-Length: 27\\r\\n\\r\\n{\\"benchmark\\":\\"tayni\\",\\"ok\\":1}"
cli.CLS
srv.CLS""",
        
        "Python": open(os.path.join(benchmark_dir, "python_http.py")).read(),
        "Node.js": open(os.path.join(benchmark_dir, "nodejs_http.js")).read(),
        "Rust": open(os.path.join(benchmark_dir, "rust_http.rs")).read(),
        "Go": open(os.path.join(benchmark_dir, "go_http.go")).read(),
        "C": open(os.path.join(benchmark_dir, "c_http.c")).read(),
        "Zig": open(os.path.join(benchmark_dir, "zig_http.zig")).read(),
    }
    
    print("=" * 60)
    print("       TOKEN COUNT COMPARISON (GPT-4 Tokenizer)")
    print("=" * 60)
    print()
    print(f"{'Language':<12} {'Tokens':>8} {'Lines':>8} {'Chars':>8} {'Tok/Line':>10}")
    print("-" * 60)
    
    results = []
    for lang, code in sources.items():
        tokens = count_tokens(code)
        lines = len(code.strip().split('\n'))
        chars = len(code)
        tok_per_line = tokens / lines if lines > 0 else 0
        results.append((lang, tokens, lines, chars, tok_per_line))
    
    # Sort by tokens
    results.sort(key=lambda x: x[1])
    
    tayni_tokens = next(r[1] for r in results if r[0] == "TAYNI")
    
    for lang, tokens, lines, chars, tok_per_line in results:
        ratio = tokens / tayni_tokens if tayni_tokens > 0 else 0
        marker = " <-- MOST EFFICIENT" if lang == "TAYNI" else f" ({ratio:.1f}x more tokens)"
        print(f"{lang:<12} {tokens:>8} {lines:>8} {chars:>8} {tok_per_line:>10.1f}{marker}")
    
    print()
    print("=" * 60)
    print("                    TOKEN SAVINGS")
    print("=" * 60)
    print()
    
    for lang, tokens, _, _, _ in results:
        if lang != "TAYNI":
            savings = tokens - tayni_tokens
            pct = (savings / tokens) * 100 if tokens > 0 else 0
            print(f"  TAYNI saves {savings} tokens ({pct:.0f}%) vs {lang}")
    
    print()
    print("=" * 60)
    print("              AI COST IMPLICATIONS")
    print("=" * 60)
    print()
    
    # GPT-4 pricing (approximate)
    cost_per_1k_input = 0.03  # $0.03 per 1K input tokens
    cost_per_1k_output = 0.06  # $0.06 per 1K output tokens
    
    print("If an AI generates this code 1000 times:")
    print()
    for lang, tokens, _, _, _ in results:
        cost = (tokens / 1000) * cost_per_1k_output * 1000
        print(f"  {lang:<12} ${cost:>8.2f}")
    
    tayni_cost = (tayni_tokens / 1000) * cost_per_1k_output * 1000
    max_cost = max((t / 1000) * cost_per_1k_output * 1000 for _, t, _, _, _ in results)
    savings = max_cost - tayni_cost
    print()
    print(f"  TAYNI saves up to ${savings:.2f} per 1000 generations")

if __name__ == "__main__":
    main()
