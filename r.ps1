# Build (release) and run one CLI command, e.g.: .\r.ps1 render -o out\v2.mp4
# Set FFMPEG_DIR, LIBCLANG_PATH or CARGO_TARGET_DIR before running to override the defaults below.
if (-not $env:FFMPEG_DIR) { $env:FFMPEG_DIR = 'C:\ffmpeg-n9.0-latest-win64-gpl-shared-9.0' }
if (-not $env:LIBCLANG_PATH) { $env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin' }
if (-not $env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = Join-Path $PSScriptRoot 'target' }
$env:PATH = "$env:FFMPEG_DIR\bin;$env:PATH"
Set-Location $PSScriptRoot
cargo build --release --message-format=short 2>&1 | ForEach-Object { "$_" -replace '\x1b\[[0-9;]*m', '' } | Select-String -Pattern '^src.*(error|warning)|could not|^error'
if ($args.Count) { & (Join-Path $env:CARGO_TARGET_DIR 'release\clip-recreate.exe') @args 2>&1 | ForEach-Object { "$_" -replace '\x1b\[[0-9;]*m', '' } }
