import { realpathSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

function canonicalPath(value) {
  const absolute = resolve(value);
  let canonical;
  try {
    canonical = typeof realpathSync.native === "function"
      ? realpathSync.native(absolute)
      : realpathSync(absolute);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
    canonical = absolute;
  }
  return process.platform === "win32" ? canonical.toLowerCase() : canonical;
}

export function isDirectExecution(moduleUrl, argvEntry = process.argv[1]) {
  if (!argvEntry) return false;
  return canonicalPath(argvEntry) === canonicalPath(fileURLToPath(moduleUrl));
}
