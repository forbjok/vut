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
  Push-Location Join-Path $TargetDir "$Target\release"
  try {
    7z a -mx9 "$DistDir\$ArchiveName.7z" "vut.exe"
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

# Create executable archives
Compress-Target "bin-windows-x86" "i686-pc-windows-msvc"
Compress-Target "bin-windows-x86_64" "x86_64-pc-windows-msvc"

# Build Chocolatey package
choco pack "chocolatey\package.nuspec" --outputdirectory "$DistDir"
