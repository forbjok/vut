<# Delete a file or directory and all its conents #>
function Remove-Dir([string] $Path) {
  if (Test-Path $Path) {
    Remove-Item $Path -Force -Recurse
  }
}

$DistDir = Join-Path $PSScriptRoot "dist"
$TargetDir = Join-Path $PSScriptRoot "target"

Write-Host "DistDir: $DistDir"
Write-Host "TargetDir: $TargetDir"

function Compress-Target([string] $ArchiveName, [string] $Target) {
  Push-Location (Join-Path $TargetDir "$Target\release")
  try {
    7z a -mx9 "$DistDir\$ArchiveName.zip" "vut.exe"
  } finally {
    Pop-Location
  }
}

# Delete old "dist" directory
Write-Host "-- CLEAN --"
Remove-Dir "$DistDir"

# Build executables
Write-Host "-- BUILD --"
cargo build --release --target i686-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

Write-Host "-- PACKAGE --"

# Create dist directory
New-Item "$DistDir" -ItemType Directory -Force

# Construct path to newly built Vut executable
$VutExe = Join-Path $TargetDir "x86_64-pc-windows-msvc\release\vut.exe"

# Get full version string
$Version = (& $VutExe get json | ConvertFrom-Json).FullVersion

# Create executable archives
Compress-Target "vut-$Version-windows-i686" "i686-pc-windows-msvc"
Compress-Target "vut-$Version-windows-x86_64" "x86_64-pc-windows-msvc"

# Build Chocolatey package
choco pack "chocolatey\package.nuspec" --outputdirectory "$DistDir"
