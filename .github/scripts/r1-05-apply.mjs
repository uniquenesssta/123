import { execFileSync } from "node:child_process";
import { rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";

const verifiedPayloadCommit = "cdfefb98c617f0362aa5deee3fcb5ff06faa8cab";
const wrapper = execFileSync(
  "git",
  ["show", `${verifiedPayloadCommit}:.github/scripts/r1-05-apply.mjs`],
  { encoding: "utf8" },
);
const match = wrapper.match(/Buffer\.from\("([A-Za-z0-9+/=]+)", "base64"\)/);
if (!match) {
  throw new Error("Unable to locate the verified R1-05 payload");
}

let source = gunzipSync(Buffer.from(match[1], "base64")).toString("utf8");

function replaceOnce(search, replacement) {
  const index = source.indexOf(search);
  if (index < 0) {
    throw new Error(`R1-05 payload anchor missing: ${search.slice(0, 80)}`);
  }
  source = `${source.slice(0, index)}${replacement}${source.slice(index + search.length)}`;
}

replaceOnce(
  '  "database.rs",\n  "openai_research.rs",',
  '  "database.rs",\n  "fact_pipeline.rs",\n  "openai_research.rs",',
);
replaceOnce(
  'replaceAllRequired(`${applicationRoot}/database.rs`, "PostgresStore", "PersistenceStore");\n\nreplaceOnce(\n  `${applicationRoot}/openai_research.rs`,',
  `replaceAllRequired(\`${'${applicationRoot}'}/database.rs\`, "PostgresStore", "PersistenceStore");

replaceOnce(
  \`${'${applicationRoot}'}/fact_pipeline.rs\`,
  "use super::{ApplicationError, ApplicationResult, ApplicationService};",
  "use super::{ApplicationError, ApplicationResult, ApplicationService};\\nuse crate::PersistenceStore;",
);
replaceAllRequired(
  \`${'${applicationRoot}'}/fact_pipeline.rs\`,
  "football_persistence_postgres::PostgresStore",
  "PersistenceStore",
);

replaceOnce(
  \`${'${applicationRoot}'}/openai_research.rs\`,`,
);
const escapedTick = "\\`";
replaceOnce(
  `- ${escapedTick}crates/application/src/database.rs${escapedTick}\n- ${escapedTick}crates/application/src/openai_research.rs${escapedTick}`,
  `- ${escapedTick}crates/application/src/database.rs${escapedTick}\n- ${escapedTick}crates/application/src/fact_pipeline.rs${escapedTick}\n- ${escapedTick}crates/application/src/openai_research.rs${escapedTick}`,
);


const temporaryScript = join(tmpdir(), `r1-05-patched-${process.pid}.mjs`);
try {
  writeFileSync(temporaryScript, source);
  await import(pathToFileURL(temporaryScript).href);
} finally {
  rmSync(temporaryScript, { force: true });
}
