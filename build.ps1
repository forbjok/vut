<# Delete a file or directory and all its conents #>
function Clean-Item([string] $Path) {
  if (Test-Path $Path) {
    Remove-Item $Path -Force -Recurse
  }
}

# Delete old "dist" directory
Write-Information "-- CLEAN --"
Clean-Item "dist"

# Build executables
Write-Information "-- BUILD --"
cargo build --release --target i686-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

Write-Information "-- PACKAGE --"

# Create dist directory
New-Item "dist" -ItemType Directory -Force

# Create executable archives
7z a -mx9 "dist\bin-windows-x86.7z" "target/i686-pc-windows-msvc/release/vut.exe"
7z a -mx9 "dist\bin-windows-x86_64.7z" "target/x86_64-pc-windows-msvc/release/vut.exe"

# Build Chocolatey package
choco pack chocolatey\package.nuspec --outputdirectory "dist"
