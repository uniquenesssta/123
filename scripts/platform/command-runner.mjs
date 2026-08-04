import { spawnSync } from "node:child_process";

export function resolveCommand(command) {
  if (process.platform !== "win32") {
    return { command, args: [] };
  }

  if (command.endsWith(".cmd") || command.endsWith(".exe")) {
    return { command, args: [] };
  }

  return { command: `${command}.cmd`, args: [] };
}

export function runCommand(command, args, options = {}) {
  const resolved = resolveCommand(command);
  return spawnSync(resolved.command, [...resolved.args, ...args], {
    stdio: "inherit",
    windowsHide: true,
    ...options,
  });
}

export function assertCommandSuccess(result, label) {
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${label} 失败，退出码：${result.status ?? "unknown"}`);
  }
}
