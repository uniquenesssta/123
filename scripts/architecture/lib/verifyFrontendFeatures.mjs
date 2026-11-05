import assert from "node:assert/strict";
import { createFrontendFeatureIndex, inspectFrontendFeatureImport } from "./frontendFeatures.mjs";
import { parseJavaScriptImports, resolveRelativeImport } from "./imports.mjs";

/** In-memory negative fixtures exercise the same classifier used by the existing gate. */
export function verifyFrontendFeatures(features) {
  const index = createFrontendFeatureIndex(features);
  let count = 0;
  const check = run => { run(); count++; };
  const ai = "src/features/ai-workspace/aiWorkspaceActions.ts";
  const profile = "src/features/openai-profiles/ProfileOperations.ts";
  check(() => assert.equal(index.identify(ai)?.name, "api-workspace"));
  check(() => assert.equal(index.identify(profile)?.name, "openai"));
  check(() => assert.equal(index.identify(ai.replaceAll("/", "\\"))?.name, "api-workspace"));
  check(() => assert.equal(index.identify("src/pages/lineupPresets.ts")?.name, "lineup-presets"));
  check(() => assert.equal(index.identify("src/pages/lineups.ts")?.name, "lineups"));
  check(() => assert.equal(index.identify("src/features/prediction/research/researchFeature.ts")?.name, "prediction"));
  check(() => assert.equal(inspectFrontendFeatureImport(index, ai, profile).privateCrossFeature, true));
  check(() => assert.equal(inspectFrontendFeatureImport(index, profile, ai).privateCrossFeature, true));
  check(() => assert.equal(inspectFrontendFeatureImport(index, ai, "src/features/ai-workspace/aiWorkspaceFeature.ts").privateCrossFeature, false));
  check(() => assert.equal(inspectFrontendFeatureImport(index, ai, "src/features/openai-profiles/index.ts").privateCrossFeature, false));
  check(() => assert.equal(inspectFrontendFeatureImport(index, profile, "src/features/ai-workspace/index.ts").privateCrossFeature, false));
  check(() => assert.equal(index.identify("src/features/openai-profiles/index.ts")?.publicEntry, "src/features/openai-profiles/index.ts"));
  check(() => assert.equal(index.identify("src/features/openai-profiles/index.js")?.publicEntry, null));
  check(() => assert.equal(index.isUnregistered("src/features/unregistered/private.ts"), true));
  check(() => assert.equal(index.isUnregistered("src/features/ai-workspace-extra/private.ts"), true));
  check(() => assert.equal(index.isUnregistered("src/ui/primitives/button/index.ts"), false));
  check(() => assert.equal(index.identify("src/platform/diagnostics/index.ts"), null));
  for (const statement of ["import('../openai-profiles/ProfileOperations')", "export {ProfileOperations} from '../openai-profiles/ProfileOperations'"]) {
    check(() => {
      const targets = parseJavaScriptImports(statement).map(specifier => resolveRelativeImport(ai, specifier));
      assert.equal(targets.length, 1);
      assert.equal(inspectFrontendFeatureImport(index, ai, targets[0]).privateCrossFeature, true);
    });
  }
  check(() => assert.throws(() => createFrontendFeatureIndex({ a: { owner: "a.ts", public_entry: "src/features/shared/index.ts" }, b: { owner: "b.ts", public_entry: "src/features/shared/index.ts" } })));
  check(() => assert.throws(() => createFrontendFeatureIndex({ a: { owner: "same.ts" }, b: { owner: "same.ts" } })));
  check(() => assert.throws(() => createFrontendFeatureIndex({ a: { owner: "a.ts", public_entry: "src/index.ts" } })));
  check(() => assert.equal(createFrontendFeatureIndex({ sample: { owner: "src/features/sample/index.ts", public_entry: "src/features/sample/index.ts" } }).identify("src/features/sample/index.ts")?.publicEntry, "src/features/sample/index.ts"));
  for (const [specifier, expected] of [["./pages/lineups", "src/pages/lineups.ts"], ["./pages/prediction", "src/pages/prediction.ts"], ["./features/ai-workspace", "src/features/ai-workspace/index.ts"], ["./features/openai-profiles", "src/features/openai-profiles/index.ts"]]) {
    check(() => assert.equal(resolveRelativeImport("src/main.ts", specifier), expected));
  }
  check(() => assert.equal(inspectFrontendFeatureImport(index, ai, resolveRelativeImport(ai, "../openai-profiles")).privateCrossFeature, false));
  console.log(`R16-01 Feature identity: ${count} alias/public/private/dynamic/registration checks PASS`);
  return count;
}
