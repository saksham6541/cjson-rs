$src = Join-Path $PSScriptRoot 'original/cJSON'
$bin = Join-Path $PSScriptRoot 'target/cjson_reference.exe'
$source = Join-Path $PSScriptRoot 'c_reference_main.c'
$include = $src
$command = "gcc -O2 -I$include $source $($src)\cJSON.c -o $bin"
Write-Host $command
Invoke-Expression $command
