@echo off
setlocal

rem Delete old "dist" directory
rmdir /S /Q "dist"

rem Build 32-bit executable for maximum OS support
cargo build --release --target i686-pc-windows-msvc

rem Create "dist" directory
mkdir "dist"

rem Build Chocolatey package
choco pack chocolatey\package.nuspec --outputdirectory "dist"

endlocal
