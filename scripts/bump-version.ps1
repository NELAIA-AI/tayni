# TAYNI Version Bump Script
# Usage: .\scripts\bump-version.ps1 -Version "0.25.0"
#
# This script updates the version in all required files:
# - Cargo.toml
# - src/main.rs
# - README.md

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

# Validate version format
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Invalid version format. Use semantic versioning: X.Y.Z (e.g., 0.25.0)"
    exit 1
}

$rootDir = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not $rootDir) { $rootDir = "." }

Write-Host "Bumping TAYNI version to $Version" -ForegroundColor Cyan
Write-Host ""

# 1. Update Cargo.toml
$cargoPath = Join-Path $rootDir "Cargo.toml"
if (Test-Path $cargoPath) {
    $content = Get-Content $cargoPath -Raw
    $content = $content -replace 'version = "\d+\.\d+\.\d+"', "version = `"$Version`""
    Set-Content $cargoPath $content -NoNewline
    Write-Host "[OK] Cargo.toml" -ForegroundColor Green
} else {
    Write-Host "[SKIP] Cargo.toml not found" -ForegroundColor Yellow
}

# 2. Update src/main.rs
$mainPath = Join-Path $rootDir "src\main.rs"
if (Test-Path $mainPath) {
    $content = Get-Content $mainPath -Raw
    $content = $content -replace 'const VERSION: &str = "\d+\.\d+\.\d+"', "const VERSION: &str = `"$Version`""
    Set-Content $mainPath $content -NoNewline
    Write-Host "[OK] src/main.rs" -ForegroundColor Green
} else {
    Write-Host "[SKIP] src/main.rs not found" -ForegroundColor Yellow
}

# 3. Update README.md
$readmePath = Join-Path $rootDir "README.md"
if (Test-Path $readmePath) {
    $content = Get-Content $readmePath -Raw
    # Update "TAYNI Compiler vX.Y" pattern
    $majorMinor = $Version -replace '\.\d+$', ''
    $content = $content -replace 'TAYNI Compiler v\d+\.\d+', "TAYNI Compiler v$majorMinor"
    # Update "tayni-c X.Y.Z" pattern
    $content = $content -replace 'tayni-c \d+\.\d+\.\d+', "tayni-c $Version"
    Set-Content $readmePath $content -NoNewline
    Write-Host "[OK] README.md" -ForegroundColor Green
} else {
    Write-Host "[SKIP] README.md not found" -ForegroundColor Yellow
}

# 4. Update tests/compiler_tests.rs version check
$testsPath = Join-Path $rootDir "tests\compiler_tests.rs"
if (Test-Path $testsPath) {
    $content = Get-Content $testsPath -Raw
    $majorMinor = $Version -replace '\.\d+$', ''
    $content = $content -replace 'contains\("\d+\.\d+"\)', "contains(`"$majorMinor`")"
    Set-Content $testsPath $content -NoNewline
    Write-Host "[OK] tests/compiler_tests.rs" -ForegroundColor Green
} else {
    Write-Host "[SKIP] tests/compiler_tests.rs not found" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Version bumped to $Version" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. cargo build --release"
Write-Host "  2. git add -A"
Write-Host "  3. git commit -m 'Release v$Version'"
Write-Host "  4. git tag -a v$Version -m 'TAYNI v$Version'"
Write-Host "  5. git push origin main --tags"
