import assert from "node:assert/strict";
import { parseCargoManifestContent } from "./cargo.mjs";

/** Keep dependency spelling from changing architecture policy. No production manifest mutation. */
export function verifyCargoDependencies() {
  const parse = source => parseCargoManifestContent("crates/domain/Cargo.toml", source).dependencies;
  let count = 0;
  const check = run => { run(); count++; };
  check(() => assert.equal(parse('[dependencies]\nserde = "1"')[0].packageName, "serde"));
  check(() => assert.deepEqual(parse('[dependencies]\napp = { package = "football-application", path = "../application" }').map(item => [item.key,item.packageName,item.path]), [["app","football-application","crates/application"]]));
  check(() => assert.deepEqual(parse('[dependencies.app]\npackage = "football-application"\npath = "../application"').map(item => [item.key,item.packageName,item.path]), [["app","football-application","crates/application"]]));
  check(() => assert.equal(parse('[target.\'cfg(windows)\'.dependencies.app]\npackage = "football-application"')[0].packageName, "football-application"));
  check(() => assert.equal(parse('[target."cfg(target_os = \\"windows\\")".build-dependencies.app]\npath = "../application"')[0].path, "crates/application"));
  check(() => assert.equal(parse('[dev-dependencies]\n"app".package = "football-application"\n"app".path = "../application"').length, 1));
  check(() => assert.equal(parse('[dependencies.\'app\']\nworkspace = true')[0].workspace, true));
  check(() => assert.equal(parse('[workspace.dependencies.app]\npackage = "football-application"')[0].packageName, "football-application"));
  check(() => assert.equal(parse('[dependencies]\napp.workspace = true')[0].workspace, true));
  check(() => assert.equal(parse('[dependencies]\napp = { features=["package = fake, path = wrong"], package="football-application", path="../application" }')[0].path, "crates/application"));
  check(() => assert.equal(parse('[dependencies.app]\n# package = "fake"\npackage = "football-application" # actual')[0].packageName, "football-application"));
  check(() => assert.equal(parse('[dependencies.app]\npath = \'../app#identity\'')[0].path, "crates/app#identity"));
  check(() => assert.equal(parse('[package.metadata.dependencies]\napp = "not-a-Cargo-dependency"').length, 0));
  check(() => assert.equal(parse('[dependencies]\napp="1"\n[target.\'cfg(windows)\'.dependencies.app]\nversion="1"').length, 2));
  check(() => assert.throws(() => parse('[dependencies]\napp = { path = "../application"'), /未闭合/));
  check(() => assert.throws(() => parse('[dependencies.app]\npath = 12'), /必须是字符串/));
  console.log(`R16-09 Cargo dependency spellings: ${count} row/table/alias/target/workspace checks PASS`);
  return count;
}
