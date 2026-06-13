# AI Generation Benchmark - Solar System Visualization

## Objective
Measure the complete AI-to-execution pipeline across languages.

## Task Specification
Create a web server that serves an animated Solar System visualization:
- Sun at center (yellow)
- 8 planets orbiting at different speeds
- Planets: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune
- Each planet has correct relative size and orbital period
- Pure HTML/CSS/JS animation (no external libraries)
- Server responds on port 8080

## Metrics Measured

### 1. Generation Phase
- **Output tokens**: Characters/tokens in generated code
- **Code lines**: Lines of source code
- **Generation time**: Time to produce the code

### 2. Build Phase
- **Compile time**: Time to build executable
- **Binary size**: Size of final artifact
- **Dependencies**: External packages required

### 3. Runtime Phase
- **Startup time**: Time from launch to first request served
- **Memory usage**: RAM consumed
- **Requests/sec**: Throughput under load
- **Latency p50/p99**: Response times

### 4. Total Cost
- **Total tokens**: Input + Output tokens
- **Total time**: Generation + Build + Startup
- **Total footprint**: Binary + Runtime + Dependencies

## Languages Tested
1. NELAIA v0.10
2. Python 3.12
3. Node.js v24
4. Go 1.22
5. Rust 1.78

## Benchmark Protocol
1. Same prompt given to AI for each language
2. Measure generation output
3. Build/compile the artifact
4. Run performance test (1000 requests)
5. Record all metrics
6. Compare results
