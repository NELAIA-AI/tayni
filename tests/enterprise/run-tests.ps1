#!/usr/bin/env pwsh
# TAYNI-C Enterprise Test Suite
# Automated testing framework for tayni-c compiler
# Version: 1.0.0

param(
    [switch]$Verbose,
    [switch]$StopOnFail,
    [string]$Filter = "*",
    [switch]$GenerateReport
)

$ErrorActionPreference = "Continue"
$script:TestResults = @()
$script:PassCount = 0
$script:FailCount = 0
$script:SkipCount = 0

$TAYNI_C = "C:\work\nelaia\products\tayni-core\archive\rust-bootstrap\target\debug\tayni-c.exe"
$TEST_DIR = "C:\work\nelaia\products\tayni-core\tests\enterprise"
$TEMP_DIR = "$TEST_DIR\temp"

function Write-TestHeader {
    Write-Host ""
    Write-Host "================================================================" -ForegroundColor Cyan
    Write-Host "           TAYNI-C Enterprise Test Suite v1.0                   " -ForegroundColor Cyan
    Write-Host "                   $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')                       " -ForegroundColor Cyan
    Write-Host "================================================================" -ForegroundColor Cyan
    Write-Host ""
}

function Write-TestFooter {
    $total = $script:PassCount + $script:FailCount + $script:SkipCount
    $passRate = if ($total -gt 0) { [math]::Round(($script:PassCount / $total) * 100, 1) } else { 0 }
    
    Write-Host ""
    Write-Host "================================================================" -ForegroundColor Cyan
    Write-Host "                      TEST SUMMARY                              " -ForegroundColor Cyan
    Write-Host "================================================================" -ForegroundColor Cyan
    Write-Host "  Total:   $total" -ForegroundColor White
    Write-Host "  Passed:  $($script:PassCount)  [$passRate%]" -ForegroundColor Green
    if ($script:FailCount -gt 0) {
        Write-Host "  Failed:  $($script:FailCount)" -ForegroundColor Red
    } else {
        Write-Host "  Failed:  0" -ForegroundColor White
    }
    Write-Host "  Skipped: $($script:SkipCount)" -ForegroundColor Yellow
    Write-Host "================================================================" -ForegroundColor Cyan
    
    if ($script:FailCount -gt 0) {
        Write-Host ""
        Write-Host "FAILED TESTS:" -ForegroundColor Red
        $script:TestResults | Where-Object { $_.Status -eq "FAIL" } | ForEach-Object {
            Write-Host "  - $($_.Name): $($_.Error)" -ForegroundColor Red
        }
    }
}

function Test-Compile {
    param(
        [string]$Name,
        [string]$Source,
        [string]$ExpectedOutput,
        [string]$Category,
        [switch]$ShouldFail,
        [int]$TimeoutSeconds = 10
    )
    
    if ($Name -notlike $Filter) {
        return
    }
    
    $testFile = "$TEMP_DIR\$Name.tyn"
    $exeFile = "$TEMP_DIR\$Name.exe"
    
    # Write source
    $Source | Out-File -FilePath $testFile -Encoding ASCII -NoNewline
    
    # Compile
    Push-Location $TEMP_DIR
    $compileResult = & $TAYNI_C $testFile -o "$Name.exe" 2>&1
    $compileExitCode = $LASTEXITCODE
    Pop-Location
    
    if ($ShouldFail) {
        if ($compileExitCode -ne 0) {
            Write-TestResult -Name $Name -Category $Category -Status "PASS" -Message "Correctly failed to compile"
            return
        } else {
            Write-TestResult -Name $Name -Category $Category -Status "FAIL" -Error "Should have failed but compiled successfully"
            return
        }
    }
    
    if ($compileExitCode -ne 0) {
        Write-TestResult -Name $Name -Category $Category -Status "FAIL" -Error "Compilation failed: $compileResult"
        return
    }
    
    # Run
    $actualExe = "$TEMP_DIR\$Name.exe.exe"
    if (-not (Test-Path $actualExe)) {
        Write-TestResult -Name $Name -Category $Category -Status "FAIL" -Error "Executable not found: $actualExe"
        return
    }
    
    try {
        Push-Location $TEMP_DIR
        $output = & $actualExe 2>&1 | Out-String
        $runExitCode = $LASTEXITCODE
        Pop-Location
        
        $outputStr = $output
        $expectedStr = $ExpectedOutput.Trim()
        
        # Extract first number from output (handles trailing nulls and garbage)
        $cleanOutput = ""
        $foundDigit = $false
        foreach ($char in $outputStr.ToCharArray()) {
            $code = [int][char]$char
            if ($code -ge 48 -and $code -le 57) {  # 0-9
                $cleanOutput += $char
                $foundDigit = $true
            } elseif ($code -eq 45 -and -not $foundDigit) {  # - at start
                $cleanOutput += $char
            } elseif ($foundDigit) {
                break  # Stop at first non-digit after we have digits
            }
        }
        
        if ($cleanOutput -eq $expectedStr) {
            Write-TestResult -Name $Name -Category $Category -Status "PASS" -Message "Output: $cleanOutput"
        } else {
            Write-TestResult -Name $Name -Category $Category -Status "FAIL" -Error "Expected '$expectedStr', got '$cleanOutput'"
        }
    } catch {
        Pop-Location
        Write-TestResult -Name $Name -Category $Category -Status "FAIL" -Error "Runtime error: $_"
    }
}

function Write-TestResult {
    param(
        [string]$Name,
        [string]$Category,
        [string]$Status,
        [string]$Message = "",
        [string]$Error = ""
    )
    
    $icon = switch ($Status) {
        "PASS" { "[OK]"; $script:PassCount++ }
        "FAIL" { "[X]"; $script:FailCount++ }
        "SKIP" { "[--]"; $script:SkipCount++ }
    }
    
    $color = switch ($Status) {
        "PASS" { "Green" }
        "FAIL" { "Red" }
        "SKIP" { "Yellow" }
    }
    
    $script:TestResults += [PSCustomObject]@{
        Name = $Name
        Category = $Category
        Status = $Status
        Message = $Message
        Error = $Error
        Timestamp = Get-Date
    }
    
    Write-Host "  $icon " -NoNewline -ForegroundColor $color
    Write-Host "$Category/$Name" -NoNewline -ForegroundColor White
    if ($Verbose -and $Message) {
        Write-Host " - $Message" -ForegroundColor DarkGray
    } elseif ($Error) {
        Write-Host " - $Error" -ForegroundColor Red
    } else {
        Write-Host ""
    }
    
    if ($StopOnFail -and $Status -eq "FAIL") {
        Write-Host "Stopping on first failure." -ForegroundColor Red
        exit 1
    }
}

function Run-CategoryTests {
    param([string]$Category, [scriptblock]$Tests)
    
    Write-Host ""
    Write-Host "--- $Category ---" -ForegroundColor Yellow
    
    & $Tests
}

# ============================================================================
# MAIN
# ============================================================================

Write-TestHeader

# Create temp directory (clean it first to avoid stale files)
if (Test-Path $TEMP_DIR) {
    Remove-Item "$TEMP_DIR\*" -Force -ErrorAction SilentlyContinue
} else {
    New-Item -ItemType Directory -Path $TEMP_DIR -Force | Out-Null
}

# ============================================================================
# CORE ARITHMETIC TESTS
# ============================================================================
Run-CategoryTests "Core/Arithmetic" {
    
    Test-Compile -Name "add_simple" -Category "Core/Arithmetic" -ExpectedOutput "15" -Source @'
.out: ALC 32
@.a: PUT .out 0 10
@.b: PUT .out 1 5
@.va: GET .out 0
@.vb: GET .out 1
@.sum: ADD @.va @.vb
@.str: ITS @.sum .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "sub_simple" -Category "Core/Arithmetic" -ExpectedOutput "5" -Source @'
.out: ALC 32
@.a: PUT .out 0 10
@.b: PUT .out 1 5
@.va: GET .out 0
@.vb: GET .out 1
@.diff: SUB @.va @.vb
@.str: ITS @.diff .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "mul_simple" -Category "Core/Arithmetic" -ExpectedOutput "50" -Source @'
.out: ALC 32
@.a: PUT .out 0 10
@.b: PUT .out 1 5
@.va: GET .out 0
@.vb: GET .out 1
@.prod: MUL @.va @.vb
@.str: ITS @.prod .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "div_simple" -Category "Core/Arithmetic" -ExpectedOutput "2" -Source @'
.out: ALC 32
@.a: PUT .out 0 10
@.b: PUT .out 1 5
@.va: GET .out 0
@.vb: GET .out 1
@.quot: DIV @.va @.vb
@.str: ITS @.quot .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "mod_simple" -Category "Core/Arithmetic" -ExpectedOutput "1" -Source @'
.out: ALC 32
@.a: PUT .out 0 10
@.b: PUT .out 1 3
@.va: GET .out 0
@.vb: GET .out 1
@.rem: MOD @.va @.vb
@.str: ITS @.rem .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "chained_arith" -Category "Core/Arithmetic" -ExpectedOutput "25" -Source @'
.out: ALC 32
@.a: PUT .out 0 10
@.b: PUT .out 1 5
@.c: PUT .out 2 2
@.va: GET .out 0
@.vb: GET .out 1
@.vc: GET .out 2
@.sum: ADD @.va @.vb
@.prod: MUL @.sum @.vc
@.final: SUB @.prod 5
@.str: ITS @.final .out
.prt: PRT .out @.str
!
'@
}

# ============================================================================
# CONTROL FLOW TESTS
# ============================================================================
Run-CategoryTests "Core/ControlFlow" {
    
    Test-Compile -Name "ifz_zero" -Category "Core/ControlFlow" -ExpectedOutput "1" -Source @'
.out: ALC 32
@.val: PUT .out 0 0
@.v: GET .out 0
@.cmp: IFZ @.v 1 0
@.str: ITS @.cmp .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "ifz_nonzero" -Category "Core/ControlFlow" -ExpectedOutput "0" -Source @'
.out: ALC 32
@.val: PUT .out 0 5
@.v: GET .out 0
@.cmp: IFZ @.v 1 0
@.str: ITS @.cmp .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "loop_count" -Category "Core/ControlFlow" -ExpectedOutput "5" -Source @'
.out: ALC 32
@.cnt: PUT .out 0 0
@.max: PUT .out 1 5
:loop
@.c: GET .out 0
@.m: GET .out 1
@.diff: SUB @.m @.c
@.jz: JZ @.diff :end
@.inc: ADD @.c 1
@.upd: PUT .out 0 @.inc
@.jmp: JMP :loop
:end
@.final: GET .out 0
@.str: ITS @.final .out
.prt: PRT .out @.str
!
'@
}

# ============================================================================
# MEMORY TESTS
# ============================================================================
Run-CategoryTests "Memory" {
    
    Test-Compile -Name "put_get_byte" -Category "Memory" -ExpectedOutput "65" -Source @'
.out: ALC 32
@.put: PUT .out 0 65
@.get: GET .out 0
@.str: ITS @.get .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "multiple_buffers" -Category "Memory" -ExpectedOutput "30" -Source @'
.buf1: ALC 32
.buf2: ALC 32
.out: ALC 32
@.p1: PUT .buf1 0 10
@.p2: PUT .buf2 0 20
@.v1: GET .buf1 0
@.v2: GET .buf2 0
@.sum: ADD @.v1 @.v2
@.str: ITS @.sum .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "sequential_bytes" -Category "Memory" -ExpectedOutput "6" -Source @'
.buf: ALC 32
.out: ALC 32
@.p0: PUT .buf 0 1
@.p1: PUT .buf 1 2
@.p2: PUT .buf 2 3
@.v0: GET .buf 0
@.v1: GET .buf 1
@.v2: GET .buf 2
@.s1: ADD @.v0 @.v1
@.sum: ADD @.s1 @.v2
@.str: ITS @.sum .out
.prt: PRT .out @.str
!
'@
}

# ============================================================================
# VECTOR TESTS
# ============================================================================
Run-CategoryTests "Vector" {
    
    Test-Compile -Name "vec_create_empty" -Category "Vector" -ExpectedOutput "0" -Source @'
.data: ALC 64
.out: ALC 64
@.vec: VEC 16
@.len: VLN .vec
@.str: ITS @.len .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "vec_push_one" -Category "Vector" -ExpectedOutput "1" -Source @'
.data: ALC 64
.out: ALC 64
@.vec: VEC 16
@.p1: VPH .vec 42
@.len: VLN .vec
@.str: ITS @.len .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "vec_push_multi" -Category "Vector" -ExpectedOutput "4" -Source @'
.data: ALC 64
.out: ALC 64
@.vec: VEC 16
@.p1: VPH .vec 10
@.p2: VPH .vec 20
@.p3: VPH .vec 30
@.p4: VPH .vec 40
@.len: VLN .vec
@.str: ITS @.len .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "vec_get" -Category "Vector" -ExpectedOutput "20" -Source @'
.data: ALC 64
.out: ALC 64
@.vec: VEC 16
@.p1: VPH .vec 10
@.p2: VPH .vec 20
@.p3: VPH .vec 30
@.val: VGT .vec 1
@.str: ITS @.val .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "vec_set" -Category "Vector" -ExpectedOutput "99" -Source @'
.data: ALC 64
.out: ALC 64
@.vec: VEC 16
@.p1: VPH .vec 10
@.p2: VPH .vec 20
@.p3: VPH .vec 30
@.set: VST .vec 1 99
@.val: VGT .vec 1
@.str: ITS @.val .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "vec_capacity" -Category "Vector" -ExpectedOutput "16" -Source @'
.data: ALC 64
.out: ALC 64
@.vec: VEC 16
@.cap: VCP .vec
@.str: ITS @.cap .out
.prt: PRT .out @.str
!
'@
}

# ============================================================================
# NETWORKING TESTS
# ============================================================================
Run-CategoryTests "Networking" {
    
    # Use random high ports to avoid conflicts
    $port1 = Get-Random -Minimum 40000 -Maximum 50000
    $port2 = $port1 + 1
    
    Test-Compile -Name "tcp_bind" -Category "Networking" -ExpectedOutput "0" -Source @"
.out: ALC 32
@.sock: TCP
@.bnd: BND @.sock $port1
@.str: ITS @.bnd .out
.prt: PRT .out @.str
!
"@

    Test-Compile -Name "tcp_listen" -Category "Networking" -ExpectedOutput "0" -Source @"
.out: ALC 32
@.sock: TCP
@.bnd: BND @.sock $port2
@.lst: LST @.sock 5
@.str: ITS @.lst .out
.prt: PRT .out @.str
!
"@
}

# ============================================================================
# INTEGRATION TESTS
# ============================================================================
Run-CategoryTests "Integration" {
    
    Test-Compile -Name "fibonacci_9" -Category "Integration" -ExpectedOutput "55" -Source @'
.out: ALC 64
@.n: PUT .out 0 9
@.a: PUT .out 8 0
@.b: PUT .out 16 1
@.i: PUT .out 24 0
:loop
@.idx: GET .out 24
@.max: GET .out 0
@.diff: SUB @.max @.idx
@.jz: JZ @.diff :end
@.va: GET .out 8
@.vb: GET .out 16
@.sum: ADD @.va @.vb
@.ua: PUT .out 8 @.vb
@.ub: PUT .out 16 @.sum
@.ii: GET .out 24
@.inc: ADD @.ii 1
@.ui: PUT .out 24 @.inc
@.jmp: JMP :loop
:end
@.result: GET .out 16
@.str: ITS @.result .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "factorial_5" -Category "Integration" -ExpectedOutput "120" -Source @'
.out: ALC 64
@.n: PUT .out 0 6
@.result: PUT .out 8 1
@.i: PUT .out 16 1
:loop
@.idx: GET .out 16
@.max: GET .out 0
@.diff: SUB @.max @.idx
@.jz: JZ @.diff :end
@.r: GET .out 8
@.prod: MUL @.r @.idx
@.ur: PUT .out 8 @.prod
@.inc: ADD @.idx 1
@.ui: PUT .out 16 @.inc
@.jmp: JMP :loop
:end
@.final: GET .out 8
@.str: ITS @.final .out
.prt: PRT .out @.str
!
'@

    Test-Compile -Name "sum_1_to_10" -Category "Integration" -ExpectedOutput "55" -Source @'
.out: ALC 64
@.sum: PUT .out 0 0
@.i: PUT .out 8 1
:loop
@.idx: GET .out 8
@.cmp: SUB @.idx 11
@.jz: JZ @.cmp :end
@.s: GET .out 0
@.add: ADD @.s @.idx
@.us: PUT .out 0 @.add
@.inc: ADD @.idx 1
@.ui: PUT .out 8 @.inc
@.jmp: JMP :loop
:end
@.final: GET .out 0
@.str: ITS @.final .out
.prt: PRT .out @.str
!
'@
}

# ============================================================================
# FINISH
# ============================================================================

Write-TestFooter

# Generate report if requested
if ($GenerateReport) {
    $reportPath = "$TEST_DIR\test-report-$(Get-Date -Format 'yyyyMMdd-HHmmss').json"
    $report = @{
        Timestamp = Get-Date -Format "o"
        Summary = @{
            Total = $script:PassCount + $script:FailCount + $script:SkipCount
            Passed = $script:PassCount
            Failed = $script:FailCount
            Skipped = $script:SkipCount
            PassRate = if (($script:PassCount + $script:FailCount) -gt 0) { 
                [math]::Round(($script:PassCount / ($script:PassCount + $script:FailCount)) * 100, 2) 
            } else { 0 }
        }
        Tests = $script:TestResults
    }
    $report | ConvertTo-Json -Depth 10 | Out-File $reportPath -Encoding UTF8
    Write-Host ""
    Write-Host "Report saved to: $reportPath" -ForegroundColor Cyan
}

# Exit with appropriate code
if ($script:FailCount -gt 0) {
    exit 1
} else {
    exit 0
}
