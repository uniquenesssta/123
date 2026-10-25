import { readFileSync } from "node:fs";
import { resolve } from "node:path";

/** Follow the registered implementation owners without changing the frozen command/DTO contract. */
export function readTransportSources(root) {
  const contract = JSON.parse(readFileSync(resolve(root, "architecture/module-boundaries.json"), "utf8"));
  const transport = contract.frontend.transport;
  const entries = transport.source_entries ?? [transport.owner];
  if (!entries.includes(transport.owner) || new Set(entries).size !== entries.length) throw new Error("Tauri transport owner/source entries are inconsistent");
  for (const file of entries) {
    if (file !== "src/api/client.ts" && !/^src\/platform\/tauri\/[A-Za-z]+\.ts$/.test(file)) throw new Error(`Invalid transport source: ${file}`);
  }
  const diagnosticOwner = contract.frontend.platform?.diagnostics?.owner;
  if (diagnosticOwner !== "src/platform/diagnostics/logSummary.ts") throw new Error("Missing platform diagnostics owner");
  return [...entries, diagnosticOwner].map(file => ({ file, text: readFileSync(resolve(root, file), "utf8").replaceAll("\r\n", "\n") }));
}
