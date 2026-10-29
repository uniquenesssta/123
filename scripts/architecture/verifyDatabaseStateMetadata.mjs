import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const contractPath = path.join(root, "architecture/state-ownership.json");
const verifierUrl = pathToFileURL(path.join(root, "scripts/verify-database-service.mjs")).href;
const original = fs.readFileSync(contractPath, "utf8");
const contract = JSON.parse(original);
assert.match(contract.last_updated_task, /^R\d+-\d+$/, "manifest edit retains a named task without freezing its value");
assert.equal(contract.states.find(state => state.id === "application.active-database").migration_task, "R3-02");

// Exercise the real verifier in isolated child processes. Only the in-memory metadata
// input varies: no repository files, production I/O or normal gate results are changed.
const cases = [
  { name: "current later-stage manifest", mutation: "", status: 0 },
  { name: "future manifest edit", mutation: 'contract.last_updated_task = "R12-15";', status: 0 },
  { name: "missing migration source", mutation: 'delete state.migration_task;', status: 1, error: "R3-02 归属迁移来源" },
  { name: "wrong migration source", mutation: 'state.migration_task = "R3-01";', status: 1, error: "R3-02 归属迁移来源" },
  { name: "owner rollback", mutation: 'state.owner = "crates/application/src/service/application_service.rs::ApplicationService.database";', status: 1, error: "状态所有者未切换到 DatabaseService.session" },
  { name: "missing database state", mutation: 'contract.states = contract.states.filter(row => row !== state);', status: 1, error: "状态所有者未切换到 DatabaseService.session" },
];
for (const test of cases) {
  const source = `import fs from "node:fs";
import path from "node:path";
const actualRead = fs.readFileSync;
fs.readFileSync = function(file, ...args) {
  const value = actualRead.call(this, file, ...args);
  if (typeof file !== "string" || path.resolve(file) !== ${JSON.stringify(contractPath)}) return value;
  const contract = JSON.parse(value);
  const state = contract.states.find(row => row.id === "application.active-database");
  ${test.mutation}
  return JSON.stringify(contract);
};
await import(${JSON.stringify(verifierUrl)});`;
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", source], { cwd: root, encoding: "utf8", windowsHide: true, timeout: 15000 });
  assert.ifError(result.error);
  assert.equal(result.status, test.status, test.name + "\n" + result.stdout + result.stderr);
  if (test.error) assert.ok(result.stderr.includes(test.error), test.name + ": expected ownership failure\n" + result.stderr);
  console.log("Database state metadata: " + test.name + " PASS");
}
assert.equal(fs.readFileSync(contractPath, "utf8"), original, "negative fixtures must not mutate the repository contract");
console.log("Database state metadata: 2 positive / 4 negative cases PASS; migration provenance and current edit remain independent.");
