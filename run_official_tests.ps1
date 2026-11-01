# Run from the Rust project directory (the folder containing Cargo.toml).
# Checks the official input files against the professor's expected pass/fail outcomes.
$ErrorActionPreference = "Continue"
if (-not (Test-Path "Cargo.toml")) { throw "Run this in the Cargo.toml project folder." }
if (-not (Test-Path "test\Test1.txt")) { throw "Copy the supplied test folder beside Cargo.toml first." }

Write-Host "Building release executable..."
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "Release build failed." }
$exe = Join-Path (Get-Location).Path "target\release\lolcompiler.exe"
$passed = 0
$failed = 0

for ($n = 1; $n -le 10; $n++) {
    $inputFile = if ($n -eq 1) { "test\Test1.txt" } else { "test\Test$n.lol" }
    $htmlFile = "test\Test$n.html"
    if (Test-Path $htmlFile) { Remove-Item $htmlFile -Force }

    # Invoke the .exe directly to avoid Cargo's normal exit-code-one summary.
    # The compiler may attempt to launch a browser for successful inputs.
    $output = & $exe $inputFile 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    $shouldSucceed = $n -in @(2,3,6,8,9,10)
    $exists = Test-Path $htmlFile
    $ok = if ($shouldSucceed) { $exitCode -eq 0 -and $exists } else { $exitCode -ne 0 -and -not $exists }

    # Additional checks of expected HTML for important semantic test cases.
    if ($ok -and $shouldSucceed) {
        $html = Get-Content $htmlFile -Raw
        if ($n -eq 3) { $ok = $html.Contains('<b>') -and $html.Contains('</b>') }
        if ($n -eq 6) { $ok = $html.Contains('42') }
        if ($n -eq 8 -or $n -eq 9) {
            $ok = $html.Contains('42') -and $html.Contains('happiness')
        }
        if ($n -eq 10) { $ok = $html.Contains('<a href="https://www.youtube.com/watch?v=aboZctrHfK8">') }
    }
    $result = if ($ok) { 'PASS' } else { 'FAIL' }
    Write-Host ("Test {0,2}: {1} | exit={2} | HTML={3}" -f $n, $result, $exitCode, $exists)
    if (-not $ok) { Write-Host ($output.Trim()) }
    if ($ok) { $passed++ } else { $failed++ }
}
Write-Host "`nOfficial test validation: $passed passed, $failed failed."
if ($failed -gt 0) { exit 1 }
