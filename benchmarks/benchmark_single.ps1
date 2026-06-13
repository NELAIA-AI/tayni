# NELAIA Single Server Benchmark
# Run ONE server at a time for fair comparison
# Usage: .\benchmark_single.ps1 -Server nelaia|go|rust|c|python|node

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("nelaia", "go", "rust", "c", "python", "node")]
    [string]$Server,
    [int]$Requests = 3000,
    [int]$Concurrent = 100,
    [int]$Rounds = 5
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$Servers = @{
    "nelaia" = @{ Exe = "solar_nelaia_16w.exe"; Port = 8101; Color = "Magenta" }
    "go"     = @{ Exe = "solar_go.exe"; Port = 8083; Color = "Cyan" }
    "rust"   = @{ Exe = "solar_rust_opt.exe"; Port = 8087; Color = "Yellow" }
    "c"      = @{ Exe = "solar_c_opt.exe"; Port = 8089; Color = "Gray" }
    "python" = @{ Cmd = "python"; Args = "solar_python.py"; Port = 8081; Color = "Blue" }
    "node"   = @{ Cmd = "node"; Args = "solar_node.js"; Port = 8082; Color = "Green" }
}

$cfg = $Servers[$Server]
$port = $cfg.Port

Write-Host ""
Write-Host "========================================" -ForegroundColor $cfg.Color
Write-Host "  SINGLE SERVER BENCHMARK: $($Server.ToUpper())" -ForegroundColor $cfg.Color
Write-Host "========================================" -ForegroundColor $cfg.Color
Write-Host ""

# Check TIME_WAIT
$tw = (netstat -ano | Select-String "TIME_WAIT" | Measure-Object).Count
$portConns = (netstat -ano | Select-String ":$port " | Measure-Object).Count
Write-Host "System TIME_WAIT: $tw"
Write-Host "Port $port connections: $portConns"

if ($portConns -gt 100) {
    Write-Host "Port busy! Wait 2 minutes or use different port." -ForegroundColor Red
    exit 1
}

Write-Host ""

# Start server
$proc = $null
if ($cfg.Exe) {
    $exePath = Join-Path $ScriptDir $cfg.Exe
    if (-not (Test-Path $exePath)) {
        Write-Host "Binary not found: $exePath" -ForegroundColor Red
        exit 1
    }
    $proc = Start-Process -FilePath $exePath -PassThru -WindowStyle Hidden
    $size = (Get-Item $exePath).Length
} else {
    $proc = Start-Process -FilePath $cfg.Cmd -ArgumentList (Join-Path $ScriptDir $cfg.Args) -PassThru -WindowStyle Hidden
    $size = (Get-Item (Join-Path $ScriptDir $cfg.Args)).Length
}

Write-Host "Server started (PID: $($proc.Id))"
Write-Host "Binary size: $([math]::Round($size/1024, 1)) KB"
Write-Host ""
Start-Sleep 3

# Warmup
Write-Host "Warmup... " -NoNewline
$warmup = & "$ScriptDir\bench.exe" "127.0.0.1:$port" 1000 50 2>&1
$wmatch = $warmup | Select-String "RPS: (\d+)"
if ($wmatch) {
    Write-Host "$([int]$wmatch.Matches[0].Groups[1].Value) req/s (discarded)" -ForegroundColor Gray
}
Start-Sleep 2

# Benchmark rounds
$results = @()
for ($i = 1; $i -le $Rounds; $i++) {
    Write-Host "Round $i/$Rounds... " -NoNewline
    $output = & "$ScriptDir\bench.exe" "127.0.0.1:$port" $Requests $Concurrent 2>&1
    $match = $output | Select-String "RPS: (\d+)"
    if ($match) {
        $rps = [int]$match.Matches[0].Groups[1].Value
        $results += $rps
        Write-Host "$rps req/s" -ForegroundColor $cfg.Color
    } else {
        Write-Host "FAILED" -ForegroundColor Red
    }
    Start-Sleep 2
}

# Stop server
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue

# Results
Write-Host ""
Write-Host "========================================" -ForegroundColor $cfg.Color
$avg = [math]::Round(($results | Measure-Object -Average).Average)
$max = ($results | Measure-Object -Maximum).Maximum
$min = ($results | Measure-Object -Minimum).Minimum
Write-Host "  Average: $avg req/s"
Write-Host "  Max:     $max req/s"
Write-Host "  Min:     $min req/s"
Write-Host "  Size:    $([math]::Round($size/1024, 1)) KB"
Write-Host "========================================" -ForegroundColor $cfg.Color
