# Fetch the official WebAssembly spec test suite (PowerShell version)
$Dest = "crates\flame-spec-tests\fixtures\spec"
$Repo = "https://github.com/WebAssembly/testsuite.git"

Write-Host "Fetching spec tests into $Dest ..."
if (Test-Path "$Dest\.git") {
    git -C $Dest pull --ff-only
} else {
    git clone --depth 1 $Repo $Dest
}
Write-Host "Done. Run 'cargo run -p flame-spec-tests -- --report' to execute."
