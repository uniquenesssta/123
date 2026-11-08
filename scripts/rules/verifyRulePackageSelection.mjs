import assert from "node:assert/strict";
import { runInNewContext } from "node:vm";
import { productionFixture } from "../design-refresh/productionFixture.mjs";

export function verifyRulePackageSelection(root) {
  const { script } = productionFixture(root, ["src/pages/prediction.ts"]);
  const context = { window: {}, console, Set };
  runInNewContext(script, context, { timeout: 10000 });
  const load = file => runInNewContext(`load(${JSON.stringify(file)})`, context);
  const fixture = load("scripts/design-refresh/fixtureData.ts");
  const prediction = load("src/pages/prediction.ts");
  const rules = load("src/features/rules/rulesView.ts");
  const research = load("src/features/prediction/research/researchView.ts");
  const row = (model, suffix, created, version, status = "active") => ({
    id: `${model}-${suffix}`, format_version: "football.rule-package.v1", package_key: `test.${model}`,
    display_name: `Selection regression ${model}`, version, model_id: model,
    model_version: "test", parameter_version: "test", competition_kind: "league",
    priority: 100, content_sha256: "0".repeat(64), status, created_at: created,
  });
  const packages = [
    row("p4", "older", "2026-01-01T00:00:00Z", "9.0.0"),
    row("p4", "newest", "2026-03-01T00:00:00Z", "3.0.0"),
    row("p4", "inactive", "2026-04-01T00:00:00Z", "99.0.0", "disabled"),
    row("p7", "older", "2026-01-01T00:00:00Z", "1.0.0"),
    row("p7", "newest", "2026-03-01T00:00:00Z", "2.0.0"),
  ];
  const select = (html, id) => {
    const match = html.match(new RegExp(`<select\\b[^>]*id="${id}"[^>]*>[\\s\\S]*?<\\/select>`));
    assert.ok(match, `production select exists: ${id}`);
    return match[0];
  };
  let checks = 0;
  for (const rows of [packages, [...packages].reverse()]) {
    const state = structuredClone(fixture.state); state.data.rule_packages = rows;
    const before = JSON.stringify(rows);
    const p4 = prediction.predictionPage(state, null, null, null, null, null, null, "T-1h", "p4", false, false, "formal", "");
    const p7 = prediction.predictionPage(state, null, null, null, null, null, null, "T-1h", "p7", false, false, "formal", "");
    for (const [label, html, id] of [
      ["formal prediction", p4, "explicit-rule-package-id"],
      ["rule bindings", rules.rulesPage(state, null, "routing"), "binding-rule-package-id"],
      ["P4 research planning", research.p4WorkbenchMarkup(true, rows, [fixture.match], null, null), "p4-plan-rule-package"],
    ]) {
      const control = select(html, id);
      assert.ok(control.includes('value="p4-newest"'), `${label} must expose the latest created active package`); checks++;
      assert.ok(!control.includes('value="p4-older"'), `${label} must not overwrite latest with older version`); checks++;
      assert.ok(!control.includes('value="p4-inactive"'), `${label} must retain inactive filtering`); checks++;
      assert.ok(!control.includes('value="p7-newest"'), `${label} must retain P4 selection scope`); checks++;
    }
    const p7Control = select(p7, "explicit-rule-package-id");
    assert.ok(p7Control.includes('value="p7-newest"') && !p7Control.includes('value="p7-older"'), "P7 latest version remains available"); checks++;
    assert.ok(!p7Control.includes('value="p4-newest"'), "P7 selection does not include P4"); checks++;
    assert.equal(JSON.stringify(rows), before, "production rendering must not reorder/mutate its catalog"); checks++;
  }
  console.log(`Rule package selection: ${checks} production-view checks passed`);
  return checks;
}
