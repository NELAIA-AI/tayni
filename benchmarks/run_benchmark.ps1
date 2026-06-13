# NELAIA Benchmark Suite
# Run this script to benchmark all servers and generate visual results
# Requirements: Go 1.18+, Rust, Clang/LLVM, NELAIA compiler

param(
    [int]$Requests = 5000,
    [int]$Concurrent = 100,
    [int]$Rounds = 3
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  NELAIA BENCHMARK SUITE v1.0" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Configuration:" -ForegroundColor Yellow
Write-Host "  Requests per test: $Requests"
Write-Host "  Concurrent connections: $Concurrent"
Write-Host "  Rounds: $Rounds"
Write-Host ""

# Kill any existing servers
Write-Host "Cleaning up existing processes..." -ForegroundColor Gray
@("solar_nelaia", "solar_go", "solar_rust", "solar_c") | ForEach-Object {
    Get-Process -Name "$_*" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Seconds 2

# Check if benchmark tool exists
if (-not (Test-Path "$ScriptDir\bench.exe")) {
    Write-Host "Building benchmark tool..." -ForegroundColor Yellow
    Push-Location $ScriptDir
    go build -o bench.exe bench.go
    Pop-Location
}

# Results storage
$Results = @{}
$Servers = @(
    @{Name="NELAIA v0.14"; Exe="solar_nelaia_16w.exe"; Port=8101; Color="Green"},
    @{Name="Go HTTP"; Exe="solar_go.exe"; Port=8083; Color="Blue"},
    @{Name="Rust Optimized"; Exe="solar_rust_opt.exe"; Port=8087; Color="Red"},
    @{Name="C Optimized"; Exe="solar_c_opt.exe"; Port=8089; Color="Yellow"}
)

# Run benchmarks
foreach ($server in $Servers) {
    $name = $server.Name
    $exe = "$ScriptDir\$($server.Exe)"
    $port = $server.Port
    $color = $server.Color
    
    if (-not (Test-Path $exe)) {
        Write-Host "SKIP: $name - binary not found ($exe)" -ForegroundColor Red
        continue
    }
    
    Write-Host ""
    Write-Host "Testing: $name" -ForegroundColor $color
    Write-Host "  Binary: $($server.Exe)"
    Write-Host "  Port: $port"
    
    $rpsValues = @()
    
    for ($round = 1; $round -le $Rounds; $round++) {
        Write-Host "  Round $round/$Rounds... " -NoNewline
        
        # Start server
        $proc = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
        Start-Sleep -Seconds 2
        
        # Run benchmark
        $output = & "$ScriptDir\bench.exe" "127.0.0.1:$port" $Requests $Concurrent 2>&1
        $rpsMatch = $output | Select-String "RPS: (\d+)"
        
        if ($rpsMatch) {
            $rps = [int]$rpsMatch.Matches[0].Groups[1].Value
            $rpsValues += $rps
            Write-Host "$rps req/s" -ForegroundColor $color
        } else {
            Write-Host "ERROR" -ForegroundColor Red
        }
        
        # Stop server
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }
    
    if ($rpsValues.Count -gt 0) {
        $avg = [math]::Round(($rpsValues | Measure-Object -Average).Average)
        $size = (Get-Item $exe).Length
        $Results[$name] = @{
            RPS = $avg
            Size = $size
            SizeKB = [math]::Round($size / 1024, 1)
        }
        Write-Host "  Average: $avg req/s" -ForegroundColor $color
    }
}

# Display Results
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  FINAL RESULTS" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Sort by RPS
$sorted = $Results.GetEnumerator() | Sort-Object { $_.Value.RPS } -Descending

# Find max RPS for scaling
$maxRPS = ($sorted | Select-Object -First 1).Value.RPS

Write-Host "THROUGHPUT (requests/second):" -ForegroundColor Yellow
Write-Host ""
foreach ($item in $sorted) {
    $name = $item.Key.PadRight(20)
    $rps = $item.Value.RPS
    $barLen = [math]::Round(($rps / $maxRPS) * 40)
    $bar = "█" * $barLen
    $color = switch -Wildcard ($item.Key) {
        "*NELAIA*" { "Green" }
        "*Go*" { "Blue" }
        "*Rust*" { "Red" }
        "*C *" { "Yellow" }
        default { "White" }
    }
    Write-Host "  $name " -NoNewline
    Write-Host $bar -ForegroundColor $color -NoNewline
    Write-Host " $rps req/s"
}

Write-Host ""
Write-Host "BINARY SIZE:" -ForegroundColor Yellow
Write-Host ""
foreach ($item in $sorted) {
    $name = $item.Key.PadRight(20)
    $sizeKB = $item.Value.SizeKB
    $sizeStr = if ($sizeKB -ge 1024) { "$([math]::Round($sizeKB/1024, 1)) MB" } else { "$sizeKB KB" }
    Write-Host "  $name $sizeStr"
}

# Calculate comparisons
$nelaiaRPS = $Results["NELAIA v0.14"].RPS
$nelaiaSize = $Results["NELAIA v0.14"].Size

Write-Host ""
Write-Host "NELAIA ADVANTAGE:" -ForegroundColor Green
Write-Host ""
foreach ($item in $sorted) {
    if ($item.Key -ne "NELAIA v0.14") {
        $speedup = [math]::Round((($nelaiaRPS - $item.Value.RPS) / $item.Value.RPS) * 100)
        $sizeRatio = [math]::Round($item.Value.Size / $nelaiaSize)
        Write-Host "  vs $($item.Key): " -NoNewline
        if ($speedup -gt 0) {
            Write-Host "+$speedup% faster" -ForegroundColor Green -NoNewline
        } else {
            Write-Host "$speedup% slower" -ForegroundColor Red -NoNewline
        }
        Write-Host ", ${sizeRatio}x smaller"
    }
}

# Save results to file
$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$reportFile = "$ScriptDir\benchmark_$timestamp.txt"

@"
NELAIA Benchmark Results
========================
Date: $(Get-Date)
Requests: $Requests
Concurrent: $Concurrent
Rounds: $Rounds

RESULTS:
"@ | Out-File $reportFile

foreach ($item in $sorted) {
    "$($item.Key): $($item.Value.RPS) req/s, $($item.Value.SizeKB) KB" | Out-File $reportFile -Append
}

Write-Host ""
Write-Host "Results saved to: $reportFile" -ForegroundColor Gray
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  BENCHMARK COMPLETE" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
