#!/usr/bin/env bash
# Runs the scripted test for each prototype and writes results/<os>-<name>-<run>.json.
set -uo pipefail
cd "$(dirname "$0")"
os="${1:?usage: run.sh <os-name>}"
repeats="${2:-3}"
mkdir -p results
exe=target/release/spike-app
run() {
  local name="$1" mode="$2" api="$3" power="$4" count="$5"
  for i in $(seq 1 "$count"); do
    echo "--- $name $i"
    SPIKE_MODE="$mode" SPIKE_API="$api" SPIKE_POWER="$power" SPIKE_OUT="results/$os-$name-$i.json"       timeout --kill-after=10 240 "$exe"
    echo "exit $?"
  done
}
run webview-webgpu webview webgpu high "$repeats"
run native-high native webgpu high "$repeats"
run native-low native webgpu low "$repeats"
run webview-webgl2 webview webgl2 high 1
