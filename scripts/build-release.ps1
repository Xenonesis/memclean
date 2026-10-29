$ErrorActionPreference = "Stop"

Write-Host "Building MemClean in Release Mode..." -ForegroundColor Cyan
Set-Location -Path "$PSScriptRoot\..\memclean"
cargo build --release

$targetDir = "$PSScriptRoot\..\dist"
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir | Out-Null
}

# Copy portable executable
Copy-Item "target\release\memclean.exe" -Destination "$targetDir\memclean.exe" -Force
Write-Host "Portable binary ready at dist\memclean.exe" -ForegroundColor Green

# Compile installer if ISCC (Inno Setup) is present
$isccPath = (Get-Command iscc.exe -ErrorAction SilentlyContinue).Source
if (-not $isccPath) {
    $candidates = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
    )
    foreach ($cand in $candidates) {
        if (Test-Path $cand) {
            $isccPath = $cand
            break
        }
    }
}

if ($isccPath) {
    Write-Host "Compiling Installer with Inno Setup ($isccPath)..." -ForegroundColor Cyan
    & $isccPath "$PSScriptRoot\..\installer\setup.iss"
    Write-Host "Installer generated at dist\MemClean_Setup.exe" -ForegroundColor Green
} else {
    Write-Host "Inno Setup compiler (ISCC.exe) not found. Portable binary compiled successfully." -ForegroundColor Yellow
}
