# NELAIA Complete Benchmark Suite v2.1
# With port cleanup and wait logic

param(
    [int]$Requests = 5000,
    [int]$Concurrent = 100,
    [int]$Rounds = 2,
    [switch]$AltPorts
)

$ErrorActionPreference = "SilentlyContinue"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Port configuration - use alternate ports if specified
if ($AltPorts) {
    $Ports = @{
        "Python" = 9081
        "Node.js" = 9082
        "Go" = 9083
        "Rust" = 9087
        "C" = 9089
        "NELAIA" = 9101
    }
    Write-Host "Using ALTERNATE ports (9xxx)" -ForegroundColor Yellow
} else {
    $Ports = @{
        "Python" = 8081
        "Node.js" = 8082
        "Go" = 8083
        "Rust" = 8087
        "C" = 8089
        "NELAIA" = 8101
    }
}

function Wait-ForCleanPort {
    param([int]$Port, [int]$MaxWait = 120)
    
    $waited = 0
    while ($waited -lt $MaxWait) {
        $count = (netstat -ano | Select-String ":$Port " | Measure-Object).Count
        if ($count -lt 50) {
            return $true
        }
        Write-Host "    Port $Port has $count connections, waiting..." -ForegroundColor Gray
        Start-Sleep -Seconds 10
        $waited += 10
    }
    Write-Host "    WARNING: Port $Port still busy after ${MaxWait}s" -ForegroundColor Red
    return $false
}

function Kill-AllServers {
    Write-Host "Killing all server processes..." -ForegroundColor Gray
    @("python", "node", "solar_rust", "solar_c", "solar_go", "solar_nelaia") | ForEach-Object {
        Get-Process -Name "$_*" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    }
    # Also kill by specific exe names
    @("solar_rust_opt", "solar_c_opt", "solar_go", "solar_nelaia_16w") | ForEach-Object {
        Get-Process -Name $_ -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "  NELAIA COMPLETE BENCHMARK SUITE v2.1" -ForegroundColor Cyan
Write-Host "  With automatic port cleanup" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Config: $Requests requests, $Concurrent concurrent, $Rounds rounds" -ForegroundColor Yellow
Write-Host ""

# Initial cleanup
Kill-AllServers
Write-Host ""

# Check all ports before starting
Write-Host "Checking port availability..." -ForegroundColor Yellow
$allClean = $true
foreach ($entry in $Ports.GetEnumerator()) {
    $count = (netstat -ano | Select-String ":$($entry.Value) " | Measure-Object).Count
    if ($count -gt 50) {
        Write-Host "  Port $($entry.Value) ($($entry.Key)): $count connections - BUSY" -ForegroundColor Red
        $allClean = $false
    } else {
        Write-Host "  Port $($entry.Value) ($($entry.Key)): $count connections - OK" -ForegroundColor Green
    }
}

if (-not $allClean) {
    Write-Host ""
    Write-Host "Some ports are busy. Waiting for cleanup (max 2 minutes)..." -ForegroundColor Yellow
    Start-Sleep -Seconds 30
    
    # Re-check
    foreach ($entry in $Ports.GetEnumerator()) {
        Wait-ForCleanPort -Port $entry.Value -MaxWait 90 | Out-Null
    }
}

Write-Host ""
Write-Host "Starting benchmarks..." -ForegroundColor Green
Write-Host ""

# Build bench if needed
if (-not (Test-Path "$ScriptDir\bench.exe")) {
    Write-Host "Building benchmark tool..." -ForegroundColor Yellow
    Push-Location $ScriptDir
    go build -o bench.exe bench.go 2>$null
    Pop-Location
}

$Results = @{}

# Function to run benchmark for a server
function Run-Benchmark {
    param(
        [string]$Name,
        [string]$StartCmd,
        [string]$StartArgs,
        [string]$ExePath,
        [int]$Port,
        [string]$Color,
        [string]$KillName
    )
    
    Write-Host ""
    Write-Host "--- Testing: $Name (port $Port) ---" -ForegroundColor $Color
    
    # Wait for clean port
    $portCount = (netstat -ano | Select-String ":$Port " | Measure-Object).Count
    if ($portCount -gt 20) {
        Write-Host "  Waiting for port to clear ($portCount connections)..." -ForegroundColor Gray
        Wait-ForCleanPort -Port $Port -MaxWait 60 | Out-Null
    }
    
    # Start server
    $proc = $null
    $size = 0
    if ($StartCmd) {
        $proc = Start-Process -FilePath $StartCmd -ArgumentList $StartArgs -PassThru -WindowStyle Hidden
        $size = (Get-Item $StartArgs -ErrorAction SilentlyContinue).Length
    } else {
        if (-not (Test-Path $ExePath)) {
            Write-Host "  SKIP: Binary not found" -ForegroundColor Red
            return $null
        }
        $proc = Start-Process -FilePath $ExePath -PassThru -WindowStyle Hidden
        $size = (Get-Item $ExePath).Length
    }
    
    Start-Sleep -Seconds 3
    
    # Warmup round (not counted)
    Write-Host "  Warmup... " -NoNewline
    $warmup = & "$ScriptDir\bench.exe" "127.0.0.1:$Port" 1000 50 2>&1
    $wmatch = $warmup | Select-String "RPS: (\d+)"
    if ($wmatch) {
        Write-Host "$([int]$wmatch.Matches[0].Groups[1].Value) req/s (discarded)" -ForegroundColor Gray
    } else {
        Write-Host "FAILED" -ForegroundColor Red
    }
    Start-Sleep -Seconds 2
    
    # Run benchmark rounds
    $rpsValues = @()
    for ($i = 1; $i -le $Rounds; $i++) {
        Write-Host "  Round $i/$Rounds... " -NoNewline
        $output = & "$ScriptDir\bench.exe" "127.0.0.1:$Port" $Requests $Concurrent 2>&1
        $match = $output | Select-String "RPS: (\d+)"
        if ($match) {
            $rps = [int]$match.Matches[0].Groups[1].Value
            $rpsValues += $rps
            Write-Host "$rps req/s" -ForegroundColor $Color
        } else {
            Write-Host "FAILED" -ForegroundColor Red
        }
        Start-Sleep -Seconds 2
    }
    
    # Stop server
    if ($proc) {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
    if ($KillName) {
        Get-Process -Name $KillName -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    }
    
    # Wait for port to start clearing
    Start-Sleep -Seconds 10
    
    if ($rpsValues.Count -gt 0) {
        $avg = [math]::Round(($rpsValues | Measure-Object -Average).Average)
        Write-Host "  Average: $avg req/s" -ForegroundColor $Color
        return @{ RPS = $avg; Size = $size }
    }
    return $null
}

# Run benchmarks in order (slowest to fastest expected)
$result = Run-Benchmark -Name "Python" -StartCmd "python" -StartArgs "$ScriptDir\solar_python.py" -Port 8081 -Color "Yellow" -KillName "python"
if ($result) { $Results["Python"] = $result }

$result = Run-Benchmark -Name "Node.js" -StartCmd "node" -StartArgs "$ScriptDir\solar_node.js" -Port 8082 -Color "Green" -KillName "node"
if ($result) { $Results["Node.js"] = $result }

$result = Run-Benchmark -Name "Rust" -ExePath "$ScriptDir\solar_rust_opt.exe" -Port 8087 -Color "DarkYellow" -KillName "solar_rust_opt"
if ($result) { $Results["Rust"] = $result }

$result = Run-Benchmark -Name "C" -ExePath "$ScriptDir\solar_c_opt.exe" -Port 8089 -Color "Gray" -KillName "solar_c_opt"
if ($result) { $Results["C"] = $result }

$result = Run-Benchmark -Name "Go" -ExePath "$ScriptDir\solar_go.exe" -Port 8083 -Color "Cyan" -KillName "solar_go"
if ($result) { $Results["Go"] = $result }

$result = Run-Benchmark -Name "NELAIA" -ExePath "$ScriptDir\solar_nelaia_16w.exe" -Port 8101 -Color "Magenta" -KillName "solar_nelaia_16w"
if ($result) { $Results["NELAIA"] = $result }

# Final cleanup
Kill-AllServers

# Display Results
Write-Host ""
Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "                    FINAL RESULTS" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host ""

if ($Results.Count -eq 0) {
    Write-Host "No results collected!" -ForegroundColor Red
    exit 1
}

$sorted = $Results.GetEnumerator() | Sort-Object { $_.Value.RPS }
$maxRPS = ($sorted | Select-Object -Last 1).Value.RPS
$nelaiaRPS = if ($Results["NELAIA"]) { $Results["NELAIA"].RPS } else { 0 }
$nelaiaSize = if ($Results["NELAIA"]) { $Results["NELAIA"].Size } else { 1 }

Write-Host "THROUGHPUT RANKING:" -ForegroundColor Yellow
Write-Host ""
$rank = 1
foreach ($item in $sorted) {
    $name = $item.Key.PadRight(12)
    $rps = $item.Value.RPS
    $barLen = [math]::Max(1, [math]::Round(($rps / $maxRPS) * 40))
    $bar = ([char]9608).ToString() * $barLen
    Write-Host "  #$rank $name $bar $rps req/s"
    $rank++
}

Write-Host ""
Write-Host "SIZE COMPARISON:" -ForegroundColor Yellow
Write-Host ""
foreach ($item in ($Results.GetEnumerator() | Sort-Object { $_.Value.Size })) {
    $name = $item.Key.PadRight(12)
    $sizeKB = [math]::Round($item.Value.Size / 1024, 1)
    $sizeStr = if ($sizeKB -ge 1024) { "$([math]::Round($sizeKB/1024, 1)) MB" } else { "$sizeKB KB" }
    Write-Host "  $name $sizeStr"
}

if ($nelaiaRPS -gt 0) {
    Write-Host ""
    Write-Host "NELAIA ADVANTAGE:" -ForegroundColor Green
    Write-Host ""
    foreach ($item in $sorted) {
        if ($item.Key -ne "NELAIA" -and $item.Value.RPS -gt 0) {
            $speedup = [math]::Round((($nelaiaRPS - $item.Value.RPS) / $item.Value.RPS) * 100)
            $sizeRatio = [math]::Round($item.Value.Size / $nelaiaSize, 1)
            $speedStr = if ($speedup -ge 0) { "+$speedup% faster" } else { "$speedup% slower" }
            $sizeStr = if ($sizeRatio -gt 1) { "${sizeRatio}x smaller" } else { "${sizeRatio}x size" }
            Write-Host "  vs $($item.Key): $speedStr, $sizeStr" -ForegroundColor Green
        }
    }
}

Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "                  BENCHMARK COMPLETE" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
