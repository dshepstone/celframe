// Prints a table of every results file.
import { readFileSync, readdirSync } from "node:fs";
const fmt = (o) =>
  o ? [o.p50, o.p95, o.p99].map((v) => v.toFixed(1)).join(" / ") : "-";
for (const file of readdirSync("results")
  .filter((f) => f.endsWith(".json"))
  .sort()) {
  const r = JSON.parse(readFileSync(`results/${file}`));
  const info = r.info ?? {};
  const gpu =
    info.adapterName ||
    info.adapterInfo?.description ||
    info.adapterInfo?.vendor ||
    "?";
  console.log(
    `${file} | ${info.backend ?? "ERROR " + r.error} | ${gpu} | ipc p50 ${r.ipcRoundTripMs?.p50.toFixed(1)}`,
  );
  for (const run of r.runs ?? [])
    console.log(
      `  ${run.strokes}: gpu ${fmt(run.eventToGpuDoneMs)} | next frame ${fmt(run.eventToNextFrameMs)} | load ${Math.round(run.loadMs)}`,
    );
}
