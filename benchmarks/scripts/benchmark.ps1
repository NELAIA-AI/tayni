$requests = 1000
$warmup = 100

function Benchmark-Server {
    param($url, $name)
    
    Write-Host "`n=== $name ===" -ForegroundColor Cyan
    
    # Warmup
    Write-Host "Warmup ($warmup requests)..."
    for ($i = 0; $i -lt $warmup; $i++) {
        try { Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 5 | Out-Null } catch {}
    }
    
    # Benchmark
    Write-Host "Benchmarking ($requests requests)..."
    $times = @()
    $errors = 0
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    
    for ($i = 0; $i -lt $requests; $i++) {
        $reqSw = [System.Diagnostics.Stopwatch]::StartNew()
        try {
            $response = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 5
            $reqSw.Stop()
            $times += $reqSw.Elapsed.TotalMilliseconds
        } catch {
            $errors++
        }
    }
    
    $sw.Stop()
    $totalSec = $sw.Elapsed.TotalSeconds
    $successful = $requests - $errors
    $rps = [math]::Round($successful / $totalSec, 2)
    
    if ($times.Count -gt 0) {
        $avgMs = [math]::Round(($times | Measure-Object -Average).Average, 3)
        $minMs = [math]::Round(($times | Measure-Object -Minimum).Minimum, 3)
        $maxMs = [math]::Round(($times | Measure-Object -Maximum).Maximum, 3)
        $sorted = $times | Sort-Object
        $p50 = [math]::Round($sorted[[int]($sorted.Count * 0.5)], 3)
        $p99 = [math]::Round($sorted[[int]($sorted.Count * 0.99)], 3)
    }
    
    Write-Host "Results:" -ForegroundColor Green
    Write-Host "  Requests/sec: $rps"
    Write-Host "  Total time:   $([math]::Round($totalSec, 2))s"
    Write-Host "  Successful:   $successful / $requests"
    Write-Host "  Errors:       $errors"
    Write-Host "  Latency avg:  ${avgMs}ms"
    Write-Host "  Latency min:  ${minMs}ms"
    Write-Host "  Latency max:  ${maxMs}ms"
    Write-Host "  Latency p50:  ${p50}ms"
    Write-Host "  Latency p99:  ${p99}ms"
    
    return @{
        Name = $name
        RPS = $rps
        AvgMs = $avgMs
        Errors = $errors
    }
}

Write-Host "HTTP Server Benchmark" -ForegroundColor Yellow
Write-Host "=====================" -ForegroundColor Yellow
Write-Host "Requests per test: $requests"
Write-Host "Warmup requests: $warmup"

$results = @()

# Test NELAIA
$results += Benchmark-Server "http://127.0.0.1:8080/" "NELAIA v0.9 (5KB binary)"

# Test Python
$results += Benchmark-Server "http://127.0.0.1:8081/" "Python 3 (socket)"

Write-Host "`n=== COMPARISON ===" -ForegroundColor Yellow
Write-Host ("{0,-30} {1,10} {2,10}" -f "Server", "Req/s", "Avg(ms)")
Write-Host ("-" * 52)
foreach ($r in $results | Sort-Object -Property RPS -Descending) {
    Write-Host ("{0,-30} {1,10} {2,10}" -f $r.Name, $r.RPS, $r.AvgMs)
}
