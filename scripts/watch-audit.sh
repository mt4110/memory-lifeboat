#!/bin/sh
set -eu

store="${1:-$HOME/Library/Application Support/Memory Lifeboat}"
log="$store/audit/events.log"

mkdir -p "$(dirname "$log")"
touch "$log"
exec tail -n 0 -F "$log"
