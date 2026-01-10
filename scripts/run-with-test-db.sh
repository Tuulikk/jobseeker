#!/usr/bin/env bash
set -euo pipefail

# run-with-test-db.sh
# Helper to launch Jobseeker against a test database so you can safely experiment.
# Usage:
#   ./scripts/run-with-test-db.sh                   # uses default test DB under ~/.local/share/jobseeker/jobseeker-test.db
#   ./scripts/run-with-test-db.sh --db /tmp/test.db -- [APP ARGS...]
#   JOBSEEKER_DB_PATH=/tmp/test.db ./scripts/run-with-test-db.sh
#
# Safety:
#  - The script refuses to run if you explicitly point to the production DB path.
#  - It creates parent directories if needed.
#  - It prefers a release binary if present, otherwise falls back to `cargo run --bin jobseeker`.
#
# Notes:
#  - Use this when experimenting or letting an agent test things so your real data is not affected.
#  - To permanently direct the app to a test DB, export JOBSEEKER_DB_PATH in your environment.

# Default paths
HOME_DIR="${HOME:-/home/$(whoami)}"
PROD_DB="${HOME_DIR}/.local/share/jobseeker/jobseeker.db"
DEFAULT_TEST_DB="${HOME_DIR}/.local/share/jobseeker/jobseeker-test.db"

# Parse args
DB_PATH="${JOBSEEKER_DB_PATH:-$DEFAULT_TEST_DB}"
EXTRA_ARGS=()

print_usage() {
  cat <<EOF
Usage: $(basename "$0") [-d|--db <test-db-path>] [--] [APP ARGS...]

Options:
  -d, --db <path>    Path to the test DB to use (overrides JOBSEEKER_DB_PATH)
  --                 Separator; anything after this is passed through to the app
  -h, --help         Show this help

Examples:
  $(basename "$0") --db /tmp/jobseeker-test.db -- --some-app-flag
  JOBSEEKER_DB_PATH=\$HOME/jobseeker-test.db $(basename "$0")
EOF
  exit 1
}

# Simple arg parsing
while [[ $# -gt 0 ]]; do
  case "$1" in
    -d|--db)
      if [[ -n "${2:-}" ]]; then
        DB_PATH="$2"
        shift 2
      else
        echo "Missing value for $1"
        print_usage
      fi
      ;;
    --)
      shift
      EXTRA_ARGS=("$@")
      break
      ;;
    -h|--help)
      print_usage
      ;;
    *)
      EXTRA_ARGS+=("$1")
      shift
      ;;
  esac
done

# Safety check: don't run against the production DB by mistake
if [[ "$(readlink -f "$DB_PATH")" == "$(readlink -f "$PROD_DB")" ]]; then
  echo "Refusing to run with production DB path: $DB_PATH"
  echo "Please use a dedicated test DB (e.g. --db $DEFAULT_TEST_DB) or set JOBSEEKER_DB_PATH."
  exit 2
fi

# Ensure parent directory exists
DB_DIR="$(dirname "$DB_PATH")"
mkdir -p "$DB_DIR"

# Ensure the DB file exists (touch, app can create data)
if [[ ! -e "$DB_PATH" ]]; then
  touch "$DB_PATH"
  echo "Created test DB path (empty) at: $DB_PATH"
fi

# Locate a release binary if present, else debug binary, else fallback to cargo run
CANDIDATES=(
  "./target/release/jobseeker"
  "./target/release/Jobseeker"
  "./target/debug/jobseeker"
  "./target/debug/Jobseeker"
)

BINARY=""
for c in "${CANDIDATES[@]}"; do
  if [[ -x "$c" ]]; then
    BINARY="$c"
    break
  fi
done

if [[ -z "$BINARY" ]]; then
  echo "Release/debug binary not found in target/; falling back to 'cargo run --bin jobseeker'."
  echo "This will rebuild the project if necessary."
  exec env JOBSEEKER_DB_PATH="$DB_PATH" cargo run --bin jobseeker -- "${EXTRA_ARGS[@]:-}"
else
  echo "Running Jobseeker with test DB: $DB_PATH"
  exec env JOBSEEKER_DB_PATH="$DB_PATH" "$BINARY" "${EXTRA_ARGS[@]:-}"
fi
