param(
    [string]$Input = '{"a": [1, 2, 3]}'
)

Write-Host 'Building C reference binary...'
powershell -ExecutionPolicy Bypass -File .\build_c_reference.ps1

Write-Host 'Building differential binary...'
cargo build --release --bin differential --target-dir .\build-artifacts

Write-Host "Running differential check for: $Input"
.\build-artifacts\release\differential.exe $Input

Write-Host 'Running benchmark binary...'
cargo run --bin bench_main --target-dir .\build-artifacts
