#!/usr/bin/env bash
set -euo pipefail

# run_jobseeker.sh
# Enkel startscript för Jobseeker (för användning av .desktop-ikon).
# Den försöker köra en byggd binär i projektets target/ (release först om vald),
# och om binären saknas försöker den bygga med cargo.
#
# Usage:
#   ./run_jobseeker.sh [--release] [--build] [--help] [-- <args>]
#
# Options:
#   --release   Preferera release-build (vid byggning)
#   --build     Kör cargo build innan start (respekterar --release)
#   --help      Visa denna hjälptext
#
# Extra argument (efter '--') skickas vidare till Jobseeker-binarien.
#
# OBS: Efter att filen sparats i repositoryt, gör den körbar:
#   chmod +x run_jobseeker.sh
#
# (Det här skriptet är enkelt och avsett för utvecklingsanvändning.)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
BIN_RELEASE="$SCRIPT_DIR/target/release/Jobseeker"
BIN_DEBUG="$SCRIPT_DIR/target/debug/Jobseeker"

print_usage() {
    cat <<EOF
Usage: $(basename "$0") [--release] [--build] [--help] [-- <args>]

Starts Jobseeker from the project directory.
If no built binary is found, the script will attempt to build it using cargo.

Options:
  --release   Prefer release binary (build with --release if building)
  --build     Force building before running
  --help      Show this message

Any arguments after '--' are forwarded to the Jobseeker binary.
EOF
}

# Parse options
BUILD=0
PREFER_RELEASE=0
EXTRA_ARGS=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --release) PREFER_RELEASE=1; shift ;;
        --build) BUILD=1; shift ;;
        --help) print_usage; exit 0 ;;
        --) shift; EXTRA_ARGS+=("$@"); break ;;
        *) EXTRA_ARGS+=("$1"); shift ;;
    esac
done

run_binary() {
    local bin="$1"
    if [[ -x "$bin" ]]; then
        echo "Starting Jobseeker -> $bin" >&2
        exec "$bin" "${EXTRA_ARGS[@]}"
        return 0
    fi
    return 1
}

# 1) If an installed system binary exists and user didn't request build, prefer that
if command -v Jobseeker >/dev/null 2>&1 && [[ "$BUILD" -eq 0 ]]; then
    echo "Found 'Jobseeker' in PATH, running installed binary." >&2
    exec "$(command -v Jobseeker)" "${EXTRA_ARGS[@]}"
fi

# 2) If user asked to build, run cargo build
if [[ "$BUILD" -eq 1 ]]; then
    echo "Building Jobseeker (this may take a while)..." >&2
    if [[ "$PREFER_RELEASE" -eq 1 ]]; then
        (cd "$SCRIPT_DIR" && cargo build --release)
    else
        (cd "$SCRIPT_DIR" && cargo build)
    fi
fi

# 3) Try to run release or debug based on preference
if [[ "$PREFER_RELEASE" -eq 1 ]]; then
    run_binary "$BIN_RELEASE" || run_binary "$BIN_DEBUG"
else
    run_binary "$BIN_DEBUG" || run_binary "$BIN_RELEASE"
fi

# 4) If still not found, attempt an automatic debug build (if cargo available)
echo "No built binary found. Attempting cargo build (debug)..." >&2
if command -v cargo >/dev/null 2>&1; then
    (cd "$SCRIPT_DIR" && cargo build) || { echo "cargo build failed" >&2; exit 1; }
    run_binary "$BIN_DEBUG" || { echo "Built but can't find binary at $BIN_DEBUG" >&2; exit 1; }
fi

echo "Unable to start Jobseeker - binary not found and cargo build failed or cargo not installed." >&2
exit 1
