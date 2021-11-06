$packageName = 'vut'
$version = '0.1.2'
$url = "https://github.com/forbjok/vut/releases/download/v$version/vut-$version-windows-i686.zip"
$url64 = "https://github.com/forbjok/vut/releases/download/v$version/vut-$version-windows-x86_64.zip"
$unzipLocation = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"

$packageArgs = @{
  packageName = $packageName
  url = $url
  url64bit = $url64
  unzipLocation = $unzipLocation
}

Install-ChocolateyZipPackage @packageArgs
