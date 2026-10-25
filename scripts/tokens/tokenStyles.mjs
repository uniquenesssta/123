import { readFileSync } from "node:fs";
import { resolve, dirname, relative } from "node:path";

/** Resolve the production CSS import graph for existing static and browser fixtures. */
export function readStylesheet(root, file, stack = []) {
  const target = resolve(root, file);
  const location = relative(root, target).replaceAll("\\", "/");
  if (!location.startsWith("src/") || location.includes("../") || !location.endsWith(".css")) throw new Error(`Invalid stylesheet: ${file}`);
  if (stack.includes(target)) throw new Error(`Cyclic stylesheet import: ${file}`);
  return readFileSync(target, "utf8").replaceAll("\r\n", "\n").replace(/@import\s+"([^"\n]+)"\s*;/g, (_match, specifier) => {
    const imported = relative(root, resolve(dirname(target), specifier));
    return readStylesheet(root, imported, [...stack, target]);
  });
}
