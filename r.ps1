# Build (release) and run one CLI command, e.g.: .\r.ps1 strip all -n 12
$env:FFMPEG_DIR = 'C:\ffmpeg-n9.0-latest-win64-gpl-shared-9.0'
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
$env:PATH = "$env:FFMPEG_DIR\bin;$env:PATH"
$env:CARGO_TARGET_DIR = 'B:\school\sql-video\target'
Set-Location $PSScriptRoot
cargo build --release --message-format=short 2>&1 | ForEach-Object { "$_" -replace '\x1b\[[0-9;]*m', '' } | Select-String -Pattern '^src.*(error|warning)|could not|^error'
if ($args.Count) { & B:\school\sql-video\target\release\clip-recreate.exe @args 2>&1 | ForEach-Object { "$_" -replace '\x1b\[[0-9;]*m', '' } }
