import { join, resolve } from "node:path";

export function resolveNodeDependencyLayout(root) {
  const sourceRoot = resolve(root);
  const dependencyRoot = resolve(sourceRoot, "..");
  const nodeModulesRoot = join(dependencyRoot, "node_modules");
  const expectedNodeModulesRoot = resolve(sourceRoot, "..", "node_modules");

  if (resolve(nodeModulesRoot) !== expectedNodeModulesRoot) {
    throw new Error(`Node 依赖目录必须位于源码根目录上一级：${expectedNodeModulesRoot}`);
  }

  return {
    sourceRoot,
    dependencyRoot,
    nodeModulesRoot,
    markerPath: join(nodeModulesRoot, ".football-deps-lock"),
    legacyRootNodeModules: join(sourceRoot, "node_modules"),
  };
}
