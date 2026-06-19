# TAYNI Benchmark Runner
# Compares TAYNI compilation and execution with other languages

param(
    [string]$Category = "all"
)

$ErrorActionPreference = "Stop"

# Paths
$TayniCompiler = "..\..\archive\rust-bootstrap\target\release\tayni-c.exe"
$BenchDir = $PSScriptRoot

function Measure-Compilation {
    param([string]$Name, [string]$Source, [string]$Output, [string]$Compiler, [string]$Args)
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $Compiler $Args $Source -o $Output 2>$null
    $sw.Stop()
    
    $size = if (Test-Path $Output) { (Get-Item $Output).Length } else { 0 }
    
    return @{
        Name = $Name
        CompileTimeMs = $sw.ElapsedMilliseconds
        BinarySizeBytes = $size
    }
}

function Run-HelloBenchmark {
    Write-Host "=== Hello World Benchmark ===" -ForegroundColor Cyan
    
    # TAYNI
    $tayni = Measure-Compilation -Name "TAYNI" `
        -Source "$BenchDir\..\hello.tyn" `
        -Output "$BenchDir\hello_tayni.exe" `
        -Compiler $TayniCompiler `
        -Args ""
    
    Write-Host "TAYNI: $($tayni.CompileTimeMs)ms, $($tayni.BinarySizeBytes) bytes"
    
    # Cleanup
    Remove-Item "$BenchDir\hello_tayni.exe" -ErrorAction SilentlyContinue
    
    return $tayni
}

function Run-FibonacciBenchmark {
    Write-Host "=== Fibonacci Benchmark ===" -ForegroundColor Cyan
    
    # TAYNI
    $tayni = Measure-Compilation -Name "TAYNI" `
        -Source "$BenchDir\fibonacci.tyn" `
        -Output "$BenchDir\fib_tayni.exe" `
        -Compiler $TayniCompiler `
        -Args ""
    
    Write-Host "TAYNI: $($tayni.CompileTimeMs)ms, $($tayni.BinarySizeBytes) bytes"
    
    # Cleanup
    Remove-Item "$BenchDir\fib_tayni.exe" -ErrorAction SilentlyContinue
    
    return $tayni
}

# Main
Write-Host "TAYNI Benchmark Suite" -ForegroundColor Green
Write-Host "=====================" -ForegroundColor Green

$results = @{}

if ($Category -eq "all" -or $Category -eq "hello") {
    $results["hello"] = Run-HelloBenchmark
}

if ($Category -eq "all" -or $Category -eq "fibonacci") {
    $results["fibonacci"] = Run-FibonacciBenchmark
}

# Output JSON results
$results | ConvertTo-Json -Depth 3
