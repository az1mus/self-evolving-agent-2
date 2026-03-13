# Build and copy resources script

param(
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

Write-Host "Building project..." -ForegroundColor Cyan

# Build
if ($Profile -eq "release") {
    cargo build --release
} else {
    cargo build
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Determine target directory
if ($Profile -eq "release") {
    $TargetDir = "target\release"
} else {
    $TargetDir = "target\debug"
}

Write-Host "Copying resources to $TargetDir..." -ForegroundColor Cyan

# Remove existing resources directory
$TargetResources = Join-Path $TargetDir "resources"
if (Test-Path $TargetResources) {
    Remove-Item -Recurse -Force $TargetResources
    Write-Host "Removed existing resources directory" -ForegroundColor Yellow
}

# Copy resources directory
$SourceDir = "resources"
if (Test-Path $SourceDir) {
    Copy-Item -Recurse -Force $SourceDir $TargetResources
    Write-Host "Copied resources to $TargetResources" -ForegroundColor Green

    # List copied files
    Get-ChildItem -Recurse $TargetResources -File | ForEach-Object {
        $relativePath = $_.FullName.Substring((Get-Location).Path.Length + 1)
        Write-Host "  $relativePath" -ForegroundColor Gray
    }
} else {
    Write-Host "Resources directory not found: $SourceDir" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Build and copy completed!" -ForegroundColor Green
Write-Host "Executable: $TargetDir\self-evolving-agent.exe" -ForegroundColor Cyan