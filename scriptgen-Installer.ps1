# PowerShell Installer for scriptgen
$ProgressPreference = 'SilentlyContinue'
$url = "https://github.com/Pjdur/scriptgen/releases/download/v0.1.0/scriptgen.exe"
$installDir = "$env:USERPROFILE\scriptgen"
$tempFile = "$env:TEMP\scriptgen_installer_temp"

# Determine file extension from URL
$extension = [System.IO.Path]::GetExtension($url)
$destFile = $tempFile + $extension

Write-Host "--- Starting Installation for scriptgen ---" -ForegroundColor Cyan

if (!(Test-Path $installDir)) {
    Write-Host "[1/4] Creating directory: $installDir"
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

Write-Host "[2/4] Downloading binaries..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $url -OutFile $destFile -ErrorAction Stop
} catch {
    Write-Host "ERROR: Failed to download." -ForegroundColor Red
    exit
}

if ($extension -eq ".zip") {
    Write-Host "[3/4] Extracting ZIP..." -ForegroundColor Yellow
    Expand-Archive -Path $destFile -DestinationPath $installDir -Force
    Remove-Item -Path $destFile
} else {
    Write-Host "[3/4] Moving binary..." -ForegroundColor Yellow
    Move-Item -Path $destFile -Destination "$installDir\scriptgen$extension" -Force
}

# 4. Add to PATH (Permanent for User)
Write-Host "[4/4] Adding $installDir to User PATH..." -ForegroundColor Yellow
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -split ";" -notcontains $installDir) {
    $newPath = "$currentPath;$installDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "PATH updated successfully. You will have to either restart your terminal
    or open a new one to be able to use scriptgen" -ForegroundColor Gray
} else {
    Write-Host "Directory already in PATH." -ForegroundColor Gray
}

Write-Host "Done!" -ForegroundColor Green
Pause
