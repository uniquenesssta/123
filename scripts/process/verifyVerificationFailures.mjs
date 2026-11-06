import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

/** Exercise existing top-level runners. Only isolated child-process boundaries are faulted. */
export function verifyVerificationFailures(root, temporary) {
  const directory = path.join(temporary, "verification-failures");
  fs.mkdirSync(directory, { recursive: true });
  const preload = path.join(directory, "failure.cjs");
  fs.writeFileSync(preload, `const fs=require('node:fs'),cp=require('node:child_process'),path=require('node:path');
const {syncBuiltinESMExports}=require('node:module');
const mode=process.env.R17_FAILURE_MODE,trace=process.env.R17_FAILURE_TRACE;
fs.appendFileSync(trace,JSON.stringify({kind:'process',argv:process.argv})+'\\n');
if(mode==='all'&&process.argv[1]?.replaceAll('\\\\','/').endsWith('/scripts/verify-domain-root-exports.mjs'))process.exit(43);
const original=cp.spawnSync;
cp.spawnSync=function(command,args,options){
 const target=String(args?.[0]??'').replaceAll('\\\\','/');
 fs.appendFileSync(trace,JSON.stringify({kind:'spawn',command,target})+'\\n');
 if(target.endsWith('/scripts/architecture/verifyModuleBoundaries.mjs')){
  if(mode==='frontend-exit')return original(process.execPath,['-e','process.exit(37)'],options);
  if(mode==='frontend-missing')return original(path.join(process.env.R17_FAILURE_DIRECTORY,'absent-child.exe'),[],options);
 }
 return original.apply(this,arguments);
};syncBuiltinESMExports();\n`);
  const results = [];
  const check = (name, condition) => { assert.ok(condition, name); results.push({ name, passed: true }); };
  const environment = mode => ({
    ...process.env,
    NODE_OPTIONS: `${process.env.NODE_OPTIONS ?? ""} --require="${preload.replaceAll("\\", "/")}"`,
    R17_FAILURE_MODE: mode,
    R17_FAILURE_TRACE: path.join(directory, `${mode}.jsonl`),
    R17_FAILURE_DIRECTORY: directory,
    npm_config_cache: path.join(directory, "npm-cache"),
  });
  const run = (name, command, args, env = process.env) => {
    const result = spawnSync(command, args, { cwd: root, env, encoding: "utf8", windowsHide: true, timeout: 120_000 });
    fs.writeFileSync(path.join(directory, `${name}.log`), `${result.stdout ?? ""}\n${result.stderr ?? ""}`);
    if (result.error) throw result.error;
    return result;
  };
  for (const [mode, expected] of [["frontend-exit", 37], ["frontend-missing", 1]]) {
    const env = environment(mode);
    const result = run(mode, process.execPath, ["scripts/verify-frontend.mjs"], env);
    const trace = fs.readFileSync(env.R17_FAILURE_TRACE, "utf8").trim().split(/\r?\n/).map(JSON.parse);
    check(`${mode}: original frontend runner fails and never runs the next check`, result.status === expected && trace.filter(item => item.kind === "spawn").length === 1 && !result.stdout.includes("Vite build"));
  }
  const npmPath = process.env.npm_execpath ?? (() => {
    const located = spawnSync("where.exe", ["npm.cmd"], { encoding: "utf8", windowsHide: true });
    assert.equal(located.status, 0);
    return path.join(path.dirname(located.stdout.trim().split(/\r?\n/)[0]), "node_modules/npm/bin/npm-cli.js");
  })();
  const allEnv = environment("all");
  const all = run("verify-all", process.execPath, [npmPath, "run", "verify:all"], allEnv);
  const allTrace = fs.readFileSync(allEnv.R17_FAILURE_TRACE, "utf8").trim().split(/\r?\n/).map(JSON.parse);
  const entries = allTrace.filter(item => item.kind === "process").map(item => item.argv[1]?.replaceAll("\\", "/") ?? "");
  check("verify:all propagates architecture failure before frontend or Rust", all.status === 43 && entries.some(item => item.endsWith("/verify-domain-root-exports.mjs")) && !entries.some(item => item.endsWith("/verify-frontend.mjs")) && !all.stdout.includes("cargo fmt"));

  const contract = JSON.parse(fs.readFileSync(path.join(root, "architecture/command-contract.json"), "utf8"));
  const mismatch = path.join(directory, "wrong-count.json");
  fs.writeFileSync(mismatch, JSON.stringify({ ...contract, command_count: contract.command_count - 1 }));
  check("live three-party command scan remains valid", run("commands-live", process.execPath, ["scripts/verify-command-contract.mjs"]).status === 0);
  check("machine command contract rejects count drift", run("commands-count", process.execPath, ["scripts/verify_command_contract.mjs", "--contract", mismatch]).status !== 0);
  const stale = structuredClone(contract);
  stale.sources.frontend.api_file = "src/pages/apiWorkspace.ts";
  const stalePath = path.join(directory, "retired-owner.json");
  fs.writeFileSync(stalePath, JSON.stringify(stale));
  const missing = run("commands-owner", process.execPath, ["scripts/verify_command_contract.mjs", "--contract", stalePath]);
  check("retired source owner fails rather than skipping the guard", missing.status !== 0 && (missing.stdout + missing.stderr).includes("不存在"));

  const acceptance = path.join(directory, "acceptance");
  const bin = path.join(directory, "bin");
  fs.mkdirSync(acceptance); fs.mkdirSync(bin);
  fs.symlinkSync(root, path.join(acceptance, "项目源码"), "junction");
  fs.writeFileSync(path.join(bin, "npm.cmd"), '@echo off\r\nif "%~1"=="--version" (\r\n echo isolated-failure-fixture\r\n exit /b 0\r\n)\r\necho %*>>"%R17_FAILURE_TRACE%"\r\nexit /b 37\r\n');
  const psEnv = { ...process.env, PATH: bin + path.delimiter + process.env.PATH, R17_FAILURE_TRACE: path.join(directory, "windows-commands.txt") };
  const windows = run("windows-failure", "powershell.exe", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", path.join(root, "scripts/windows-acceptance.ps1"), "-Mode", "Automated", "-ProjectRoot", acceptance, "-LogDirectory", "logs"], psEnv);
  const calls = fs.readFileSync(psEnv.R17_FAILURE_TRACE, "utf8");
  const logName = fs.readdirSync(path.join(acceptance, "logs")).find(name => name.startsWith("windows-acceptance-") && name.endsWith(".txt"));
  const windowsLog = fs.readFileSync(path.join(acceptance, "logs", logName), "utf8");
  check("original Windows runner returns failure and stops before build or app launch", windows.status === 1 && windowsLog.includes("退出码 37") && calls.trim() === "run setup" && !windowsLog.includes("开始：Tauri Windows release"));
  fs.writeFileSync(path.join(directory, "results.json"), JSON.stringify({ scope: "runner failure propagation and actual command guards; not software/Full success", results }, null, 2));
  console.log(`R17-01 original verification failures: ${results.length} isolated checks PASS; ${directory}`);
  return results.length;
}
