import { normalizePath } from "./repository.mjs";

/** Resolve declared identities from their real public roots, including historical page owners. */
export function createFrontendFeatureIndex(features) {
  const owners = new Map();
  const roots = new Map();
  for (const [name, definition] of Object.entries(features)) {
    const owner = normalizePath(definition.owner ?? "");
    const publicEntry = definition.public_entry ? normalizePath(definition.public_entry) : null;
    if (!owner || owners.has(owner)) throw new Error(`Feature owner 重复或缺失：${name}/${owner}`);
    owners.set(owner, { name, publicEntry });
    if (!publicEntry) continue;
    const root = publicEntry.slice(0, publicEntry.lastIndexOf("/"));
    if (!root.startsWith("src/features/") || root === "src/features" || roots.has(root)) {
      throw new Error(`Feature 公开根重复或越界：${name}/${publicEntry}`);
    }
    roots.set(root, { name, publicEntry });
  }
  const ordered = [...roots].sort(([left], [right]) => right.length - left.length);
  const identify = file => {
    const normalized = normalizePath(file);
    const feature = owners.get(normalized) ?? ordered.find(([root]) => normalized.startsWith(`${root}/`))?.[1];
    return feature ? { name: feature.name, publicEntry: normalized === feature.publicEntry ? normalized : null } : null;
  };
  return {
    identify,
    isUnregistered(file) {
      const normalized = normalizePath(file);
      return normalized.startsWith("src/features/") && !identify(normalized);
    },
  };
}

export function inspectFrontendFeatureImport(index, importer, target) {
  const sourceFeature = index.identify(importer);
  const targetFeature = index.identify(target);
  return {
    sourceFeature,
    targetFeature,
    privateCrossFeature: Boolean(sourceFeature && targetFeature && sourceFeature.name !== targetFeature.name && !targetFeature.publicEntry),
  };
}
