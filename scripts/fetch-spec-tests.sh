#!/usr/bin/env bash
# Fetch the official WebAssembly spec test suite (testsuite/*.wast)
set -euo pipefail

DEST="crates/flame-spec-tests/fixtures/spec"
REPO="https://github.com/WebAssembly/testsuite.git"

echo "Fetching spec tests into $DEST ..."
if [ -d "$DEST/.git" ]; then
    git -C "$DEST" pull --ff-only
else
    git clone --depth=1 "$REPO" "$DEST"
fi
echo "Done. Run 'cargo run -p flame-spec-tests -- --report' to execute."
