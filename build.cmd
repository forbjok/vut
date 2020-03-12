@echo off
setlocal

rem Delete old "dist" directory
rmdir /S /Q "dist"

rem Build executables
cargo build --release --target i686-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

rem Create "dist" directory
mkdir "dist"

rem Create executable archives
7z a -mx9 "dist\bin-windows-x86.7z" "target/i686-pc-windows-msvc/release/vut.exe"
7z a -mx9 "dist\bin-windows-x86_64.7z" "target/x86_64-pc-windows-msvc/release/vut.exe"

rem Build Chocolatey package
choco pack chocolatey\package.nuspec --outputdirectory "dist"

endlocal
