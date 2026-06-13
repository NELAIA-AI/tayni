# NELAIA Complete Benchmark Suite v2.0
# Tests all languages from slowest to fastest, NELAIA last

param(
    [int]$Requests = 5000,
    [int]$Concurrent = 100,
    [int]$Rounds = 2
)

$ErrorActionPreference = "SilentlyContinue"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "  NELAIA COMPLETE BENCHMARK SUITE v2.0" -ForegroundColor Cyan
Write-Host "  Python -> Node.js -> Rust -> C -> Go -> NELAIA" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Config: $Requests requests, $Concurrent concurrent, $Rounds rounds" -ForegroundColor Yellow
Write-Host ""

# Kill existing
@("python", "node", "solar_rust", "solar_c", "solar_go", "solar_nelaia") | ForEach-Object {
    Get-Process -Name "$_*" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Seconds 3

# Build bench if needed
if (-not (Test-Path "$ScriptDir\bench.exe")) {
    Write-Host "Building benchmark tool..." -ForegroundColor Yellow
    Push-Location $ScriptDir
    go build -o bench.exe bench.go 2>$null
    Pop-Location
}

$Results = @{}

# 1. Python
Write-Host ""
Write-Host "--- Testing: Python (port 8081) ---" -ForegroundColor Yellow
$proc = Start-Process -FilePath "python" -ArgumentList "$ScriptDir\solar_python.py" -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 3
$rpsValues = @()
for ($i = 1; $i -le $Rounds; $i++) {
    Write-Host "  Round $i... " -NoNewline
    $output = & "$ScriptDir\bench.exe" "127.0.0.1:8081" $Requests $Concurrent 2>&1
    $match = $output | Select-String "RPS: (\d+)"
    if ($match) { 
        $rps = [int]$match.Matches[0].Groups[1].Value
        $rpsValues += $rps
        Write-Host "$rps req/s" -ForegroundColor Yellow
    }
    Start-Sleep -Seconds 1
}
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Get-Process -Name "python" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
if ($rpsValues.Count -gt 0) {
    $Results["Python"] = @{ RPS = [math]::Round(($rpsValues | Measure-Object -Average).Average); Size = (Get-Item "$ScriptDir\solar_python.py").Length }
    Write-Host "  Average: $($Results['Python'].RPS) req/s" -ForegroundColor Yellow
}
Start-Sleep -Seconds 2

# 2. Node.js
Write-Host ""
Write-Host "--- Testing: Node.js (port 8082) ---" -ForegroundColor Green
$proc = Start-Process -FilePath "node" -ArgumentList "$ScriptDir\solar_node.js" -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 3
$rpsValues = @()
for ($i = 1; $i -le $Rounds; $i++) {
    Write-Host "  Round $i... " -NoNewline
    $output = & "$ScriptDir\bench.exe" "127.0.0.1:8082" $Requests $Concurrent 2>&1
    $match = $output | Select-String "RPS: (\d+)"
    if ($match) { 
        $rps = [int]$match.Matches[0].Groups[1].Value
        $rpsValues += $rps
        Write-Host "$rps req/s" -ForegroundColor Green
    }
    Start-Sleep -Seconds 1
}
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Get-Process -Name "node" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
if ($rpsValues.Count -gt 0) {
    $Results["Node.js"] = @{ RPS = [math]::Round(($rpsValues | Measure-Object -Average).Average); Size = (Get-Item "$ScriptDir\solar_node.js").Length }
    Write-Host "  Average: $($Results['Node.js'].RPS) req/s" -ForegroundColor Green
}
Start-Sleep -Seconds 2

# 3. Rust
Write-Host ""
Write-Host "--- Testing: Rust (port 8087) ---" -ForegroundColor DarkYellow
if (Test-Path "$ScriptDir\solar_rust_opt.exe") {
    $proc = Start-Process -FilePath "$ScriptDir\solar_rust_opt.exe" -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds 2
    $rpsValues = @()
    for ($i = 1; $i -le $Rounds; $i++) {
        Write-Host "  Round $i... " -NoNewline
        $output = & "$ScriptDir\bench.exe" "127.0.0.1:8087" $Requests $Concurrent 2>&1
        $match = $output | Select-String "RPS: (\d+)"
        if ($match) { 
            $rps = [int]$match.Matches[0].Groups[1].Value
            $rpsValues += $rps
            Write-Host "$rps req/s" -ForegroundColor DarkYellow
        }
        Start-Sleep -Seconds 1
    }
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    if ($rpsValues.Count -gt 0) {
        $Results["Rust"] = @{ RPS = [math]::Round(($rpsValues | Measure-Object -Average).Average); Size = (Get-Item "$ScriptDir\solar_rust_opt.exe").Length }
        Write-Host "  Average: $($Results['Rust'].RPS) req/s" -ForegroundColor DarkYellow
    }
} else { Write-Host "  SKIP: Binary not found" -ForegroundColor Red }
Start-Sleep -Seconds 2

# 4. C
Write-Host ""
Write-Host "--- Testing: C (port 8089) ---" -ForegroundColor Gray
if (Test-Path "$ScriptDir\solar_c_opt.exe") {
    $proc = Start-Process -FilePath "$ScriptDir\solar_c_opt.exe" -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds 2
    $rpsValues = @()
    for ($i = 1; $i -le $Rounds; $i++) {
        Write-Host "  Round $i... " -NoNewline
        $output = & "$ScriptDir\bench.exe" "127.0.0.1:8089" $Requests $Concurrent 2>&1
        $match = $output | Select-String "RPS: (\d+)"
        if ($match) { 
            $rps = [int]$match.Matches[0].Groups[1].Value
            $rpsValues += $rps
            Write-Host "$rps req/s" -ForegroundColor Gray
        }
        Start-Sleep -Seconds 1
    }
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    if ($rpsValues.Count -gt 0) {
        $Results["C"] = @{ RPS = [math]::Round(($rpsValues | Measure-Object -Average).Average); Size = (Get-Item "$ScriptDir\solar_c_opt.exe").Length }
        Write-Host "  Average: $($Results['C'].RPS) req/s" -ForegroundColor Gray
    }
} else { Write-Host "  SKIP: Binary not found" -ForegroundColor Red }
Start-Sleep -Seconds 2

# 5. Go
Write-Host ""
Write-Host "--- Testing: Go (port 8083) ---" -ForegroundColor Cyan
if (Test-Path "$ScriptDir\solar_go.exe") {
    $proc = Start-Process -FilePath "$ScriptDir\solar_go.exe" -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds 2
    $rpsValues = @()
    for ($i = 1; $i -le $Rounds; $i++) {
        Write-Host "  Round $i... " -NoNewline
        $output = & "$ScriptDir\bench.exe" "127.0.0.1:8083" $Requests $Concurrent 2>&1
        $match = $output | Select-String "RPS: (\d+)"
        if ($match) { 
            $rps = [int]$match.Matches[0].Groups[1].Value
            $rpsValues += $rps
            Write-Host "$rps req/s" -ForegroundColor Cyan
        }
        Start-Sleep -Seconds 1
    }
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    if ($rpsValues.Count -gt 0) {
        $Results["Go"] = @{ RPS = [math]::Round(($rpsValues | Measure-Object -Average).Average); Size = (Get-Item "$ScriptDir\solar_go.exe").Length }
        Write-Host "  Average: $($Results['Go'].RPS) req/s" -ForegroundColor Cyan
    }
} else { Write-Host "  SKIP: Binary not found" -ForegroundColor Red }
Start-Sleep -Seconds 2

# 6. NELAIA (last - the champion)
Write-Host ""
Write-Host "--- Testing: NELAIA v0.15 (port 8101) ---" -ForegroundColor Magenta
if (Test-Path "$ScriptDir\solar_nelaia_16w.exe") {
    $proc = Start-Process -FilePath "$ScriptDir\solar_nelaia_16w.exe" -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds 2
    $rpsValues = @()
    for ($i = 1; $i -le $Rounds; $i++) {
        Write-Host "  Round $i... " -NoNewline
        $output = & "$ScriptDir\bench.exe" "127.0.0.1:8101" $Requests $Concurrent 2>&1
        $match = $output | Select-String "RPS: (\d+)"
        if ($match) { 
            $rps = [int]$match.Matches[0].Groups[1].Value
            $rpsValues += $rps
            Write-Host "$rps req/s" -ForegroundColor Magenta
        }
        Start-Sleep -Seconds 1
    }
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    if ($rpsValues.Count -gt 0) {
        $Results["NELAIA"] = @{ RPS = [math]::Round(($rpsValues | Measure-Object -Average).Average); Size = (Get-Item "$ScriptDir\solar_nelaia_16w.exe").Length }
        Write-Host "  Average: $($Results['NELAIA'].RPS) req/s" -ForegroundColor Magenta
    }
} else { Write-Host "  SKIP: Binary not found" -ForegroundColor Red }

# Final Results
Write-Host ""
Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "                    FINAL RESULTS" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host ""

$sorted = $Results.GetEnumerator() | Sort-Object { $_.Value.RPS }
$maxRPS = ($sorted | Select-Object -Last 1).Value.RPS
$nelaiaRPS = $Results["NELAIA"].RPS
$nelaiaSize = $Results["NELAIA"].Size

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

Write-Host ""
Write-Host "NELAIA ADVANTAGE:" -ForegroundColor Green
Write-Host ""
foreach ($item in $sorted) {
    if ($item.Key -ne "NELAIA" -and $nelaiaRPS -gt 0 -and $item.Value.RPS -gt 0) {
        $speedup = [math]::Round((($nelaiaRPS - $item.Value.RPS) / $item.Value.RPS) * 100)
        $sizeRatio = [math]::Round($item.Value.Size / $nelaiaSize, 1)
        Write-Host "  vs $($item.Key): +$speedup% faster, ${sizeRatio}x smaller" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "                  BENCHMARK COMPLETE" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan
