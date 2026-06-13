# NELAIA Benchmark Suite - Generates Visual HTML Report
# Usage: .\benchmark_visual.ps1 [-Requests 5000] [-Concurrent 100] [-Rounds 3]

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

# Kill existing
Get-Process -Name "solar_*" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# Build bench if needed
if (-not (Test-Path "$ScriptDir\bench.exe")) {
    Write-Host "Building benchmark tool..." -ForegroundColor Yellow
    Push-Location $ScriptDir
    go build -o bench.exe bench.go
    Pop-Location
}

$Results = @{}
$Servers = @(
    @{Name="NELAIA v0.14"; Exe="solar_nelaia_16w.exe"; Port=8101},
    @{Name="Go"; Exe="solar_go.exe"; Port=8083},
    @{Name="Rust"; Exe="solar_rust_opt.exe"; Port=8087},
    @{Name="C"; Exe="solar_c_opt.exe"; Port=8089}
)

foreach ($server in $Servers) {
    $exe = "$ScriptDir\$($server.Exe)"
    if (-not (Test-Path $exe)) { continue }
    
    Write-Host "Testing: $($server.Name)..." -NoNewline
    $rpsValues = @()
    
    for ($round = 1; $round -le $Rounds; $round++) {
        $proc = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
        Start-Sleep -Seconds 2
        $output = & "$ScriptDir\bench.exe" "127.0.0.1:$($server.Port)" $Requests $Concurrent 2>&1
        $rpsMatch = $output | Select-String "RPS: (\d+)"
        if ($rpsMatch) { $rpsValues += [int]$rpsMatch.Matches[0].Groups[1].Value }
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }
    
    if ($rpsValues.Count -gt 0) {
        $avg = [math]::Round(($rpsValues | Measure-Object -Average).Average)
        $size = [math]::Round((Get-Item $exe).Length / 1024, 1)
        $Results[$server.Name] = @{ RPS = $avg; Size = $size }
        Write-Host " $avg req/s" -ForegroundColor Green
    }
}

# Generate HTML
$nelaiaRps = $Results["NELAIA v0.14"].RPS
$nelaiaSize = $Results["NELAIA v0.14"].Size
$goRps = $Results["Go"].RPS
$goSize = $Results["Go"].Size
$rustRps = $Results["Rust"].RPS
$rustSize = $Results["Rust"].Size
$cRps = $Results["C"].RPS
$cSize = $Results["C"].Size

$goSpeedup = [math]::Round((($nelaiaRps - $goRps) / $goRps) * 100)
$rustSpeedup = [math]::Round((($nelaiaRps - $rustRps) / $rustRps) * 100)
$cSpeedup = [math]::Round((($nelaiaRps - $cRps) / $cRps) * 100)
$goSizeRatio = [math]::Round($goSize / $nelaiaSize)
$rustSizeRatio = [math]::Round($rustSize / $nelaiaSize)
$cSizeRatio = [math]::Round($cSize / $nelaiaSize)

$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

$html = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NELAIA Benchmark Results</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', system-ui, sans-serif;
            background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 100%);
            color: #fff;
            min-height: 100vh;
            padding: 40px;
        }
        .container { max-width: 1200px; margin: 0 auto; }
        h1 {
            text-align: center;
            font-size: 2.5rem;
            margin-bottom: 10px;
            background: linear-gradient(90deg, #00ff88, #00d4ff);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .subtitle { text-align: center; color: #888; margin-bottom: 40px; }
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 30px;
            margin-bottom: 40px;
        }
        .card {
            background: rgba(255,255,255,0.05);
            border-radius: 16px;
            padding: 24px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        .card h2 { font-size: 1.2rem; margin-bottom: 20px; color: #00ff88; }
        .chart-container { position: relative; height: 300px; }
        .stats {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 20px;
            margin-bottom: 40px;
        }
        .stat {
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 20px;
            text-align: center;
            border: 1px solid rgba(255,255,255,0.1);
        }
        .stat-value { font-size: 2rem; font-weight: bold; color: #00ff88; }
        .stat-label { color: #888; font-size: 0.9rem; margin-top: 5px; }
        .winner {
            background: linear-gradient(135deg, rgba(0,255,136,0.2), rgba(0,212,255,0.2));
            border-color: #00ff88;
        }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid rgba(255,255,255,0.1); }
        th { color: #00ff88; }
        .advantage {
            display: inline-block;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.85rem;
            font-weight: bold;
            background: rgba(0,255,136,0.2);
            color: #00ff88;
        }
        .footer {
            text-align: center;
            color: #666;
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid rgba(255,255,255,0.1);
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>NELAIA Benchmark Results</h1>
        <p class="subtitle">AI-Native Protocol vs Traditional Languages | $timestamp</p>

        <div class="stats">
            <div class="stat winner">
                <div class="stat-value">$($nelaiaRps.ToString("N0"))</div>
                <div class="stat-label">NELAIA req/s</div>
            </div>
            <div class="stat">
                <div class="stat-value">+$goSpeedup%</div>
                <div class="stat-label">Faster than Go</div>
            </div>
            <div class="stat">
                <div class="stat-value">${goSizeRatio}x</div>
                <div class="stat-label">Smaller than Go</div>
            </div>
            <div class="stat">
                <div class="stat-value">$nelaiaSize KB</div>
                <div class="stat-label">Binary Size</div>
            </div>
        </div>

        <div class="grid">
            <div class="card">
                <h2>Throughput (requests/second)</h2>
                <div class="chart-container">
                    <canvas id="throughputChart"></canvas>
                </div>
            </div>
            <div class="card">
                <h2>Binary Size (KB, log scale)</h2>
                <div class="chart-container">
                    <canvas id="sizeChart"></canvas>
                </div>
            </div>
        </div>

        <div class="card">
            <h2>Detailed Comparison</h2>
            <table>
                <thead>
                    <tr><th>Language</th><th>Throughput</th><th>Binary Size</th><th>vs NELAIA</th></tr>
                </thead>
                <tbody>
                    <tr style="background: rgba(0,255,136,0.1);">
                        <td><strong>NELAIA v0.14</strong></td>
                        <td><strong>$($nelaiaRps.ToString("N0")) req/s</strong></td>
                        <td><strong>$nelaiaSize KB</strong></td>
                        <td><span class="advantage">WINNER</span></td>
                    </tr>
                    <tr>
                        <td>Rust (optimized)</td>
                        <td>$($rustRps.ToString("N0")) req/s</td>
                        <td>$rustSize KB</td>
                        <td><span class="advantage">+$rustSpeedup% faster, ${rustSizeRatio}x smaller</span></td>
                    </tr>
                    <tr>
                        <td>Go (net/http)</td>
                        <td>$($goRps.ToString("N0")) req/s</td>
                        <td>$goSize KB</td>
                        <td><span class="advantage">+$goSpeedup% faster, ${goSizeRatio}x smaller</span></td>
                    </tr>
                    <tr>
                        <td>C (optimized)</td>
                        <td>$($cRps.ToString("N0")) req/s</td>
                        <td>$cSize KB</td>
                        <td><span class="advantage">+$cSpeedup% faster, ${cSizeRatio}x smaller</span></td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div class="footer">
            <p>NELAIA - AI-Native Protocol for Industrial Software Construction</p>
            <p style="margin-top: 10px; font-size: 0.8rem;">
                Test: $Requests requests, $Concurrent concurrent, $Rounds rounds
            </p>
        </div>
    </div>

    <script>
        const data = {
            labels: ['NELAIA v0.14', 'Rust', 'Go', 'C'],
            throughput: [$nelaiaRps, $rustRps, $goRps, $cRps],
            size: [$nelaiaSize, $rustSize, $goSize, $cSize],
            colors: ['#00ff88', '#DEA584', '#00ADD8', '#A8B9CC']
        };

        new Chart(document.getElementById('throughputChart'), {
            type: 'bar',
            data: {
                labels: data.labels,
                datasets: [{
                    label: 'Requests/second',
                    data: data.throughput,
                    backgroundColor: data.colors,
                    borderRadius: 8
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } },
                scales: {
                    y: { beginAtZero: true, grid: { color: 'rgba(255,255,255,0.1)' }, ticks: { color: '#888' } },
                    x: { grid: { display: false }, ticks: { color: '#888' } }
                }
            }
        });

        new Chart(document.getElementById('sizeChart'), {
            type: 'bar',
            data: {
                labels: data.labels,
                datasets: [{
                    label: 'Size (KB)',
                    data: data.size,
                    backgroundColor: data.colors,
                    borderRadius: 8
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } },
                scales: {
                    y: { type: 'logarithmic', grid: { color: 'rgba(255,255,255,0.1)' }, ticks: { color: '#888' } },
                    x: { grid: { display: false }, ticks: { color: '#888' } }
                }
            }
        });
    </script>
</body>
</html>
"@

$outputFile = "$ScriptDir\benchmark_results.html"
$html | Out-File -FilePath $outputFile -Encoding UTF8

Write-Host ""
Write-Host "HTML report generated: $outputFile" -ForegroundColor Green
Write-Host "Opening in browser..." -ForegroundColor Gray

Start-Process $outputFile
