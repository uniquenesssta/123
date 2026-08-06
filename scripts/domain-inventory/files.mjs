import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

const EXCLUDED_DIRECTORIES = new Set([
  ".git",
  ".cargo-target",
  "node_modules",
  "dist",
  "target",
]);

export function listFiles(root, start, predicate) {
  const output = [];
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (entry.isDirectory() && EXCLUDED_DIRECTORIES.has(entry.name)) continue;
      const absolute = join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(absolute);
      } else if (predicate(absolute)) {
        output.push(relative(root, absolute).replaceAll("\\", "/"));
      }
    }
  };
  visit(join(root, start));
  return output.sort();
}

export function readText(root, relativePath) {
  return readFileSync(join(root, relativePath), "utf8").replaceAll("\r\n", "\n");
}
