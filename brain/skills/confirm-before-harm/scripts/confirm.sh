#!/usr/bin/env bash
# confirm.sh — dry-run rehearsal + explicit-confirmation gate for
# irreversible or high-stakes commands. A fragile or destructive action
# must be shown before it runs, and must not run on a silent "yes".
#
# Usage:
#   confirm.sh --reason "what and why, in the owner's words" -- CMD...
#
#   Without --confirmed : prints every argument as "WOULD RUN", prints the
#       reason, and exits 2 (not approved). Nothing executes.
#   With --confirmed (an explicit, unambiguous go from the owner):
#       runs CMD... and returns its exit code.
#
# The gate is structural: the real command cannot run unless an explicit
# confirmation token is passed, so a rushed model step "just doing it"
# is impossible.

set -u

reason=""
confirmed=0
positional_args=()

while [ "$#" -gt 0 ]; do
    case "$1" in
        --reason)
            reason="${2:-}"
            shift 2
            ;;
        --confirmed)
            confirmed=1
            shift
            ;;
        --)
            shift
            break
            ;;
        *)
            positional_args+=("$1")
            shift
            ;;
    esac
done

# Any args consumed before "--" are treated as the command to run.
CMD=("${positional_args[@]}" "$@")

if [ "${#CMD[@]}" -eq 0 ]; then
    echo "confirm.sh: no command given to rehearse or run." >&2
    exit 3
fi

echo "--- DRY RUN / CONFIRMATION GATE ---"
echo "Reason: ${reason:-<none given>}"
echo "Command that would run:"
for arg in "${CMD[@]}"; do
    echo "    WOULD RUN: $arg"
done
echo "----------------------------------"

if [ "$confirmed" -ne 1 ]; then
    echo "NOT APPROVED — nothing was executed."
    echo "Show the owner exactly what will happen and the consequences,"
    echo "and get an explicit yes before retrying with --confirmed."
    exit 2
fi

echo "Executing the confirmed command..."
exec "${CMD[@]}"