#!/usr/bin/env bash
# Each prototype gets a limited time, and a screenshot of what it actually drew.
set -uo pipefail
os="$1"
mkdir -p results
exe=target/release/spike-app
shot() {
  if [ "$os" = "Linux" ]; then import -window root "results/$1.png" || true
  else screencapture -x "results/$1.png" || true; fi
}
scripted() {
  local name="$1" mode="$2" api="$3"
  echo "--- scripted $name"
  SPIKE_MODE="$mode" SPIKE_API="$api" SPIKE_OUT="results/$os-$name.json" timeout --kill-after=10 300 "$exe"
  echo "$name exit $?" | tee -a results/exit-codes.txt
}
visual() {
  local name="$1" mode="$2" api="$3"
  echo "--- visual $name"
  SPIKE_MODE="$mode" SPIKE_API="$api" timeout --kill-after=5 45 "$exe" &
  sleep 30
  shot "$os-$name"
  wait
  echo "$name visual exit $?" | tee -a results/exit-codes.txt
}
scripted webview-webgpu webview webgpu
scripted webview-webgl2 webview webgl2
scripted native native webgpu
visual webview-webgpu webview webgpu
visual native native webgpu
