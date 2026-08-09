from pathlib import Path

GENERATOR = Path(__file__).resolve().with_name("r3-07-at3.py")
text = GENERATOR.read_text(encoding="utf-8")


def replace_once(old: str, new: str, label: str) -> None:
    global text
    if text.count(old) != 1:
        raise RuntimeError(f"AT3 fix anchor mismatch: {label}")
    text = text.replace(old, new, 1)


replace_once(
    'artifacts_functions = artifacts_functions.replace("../../../src-tauri", "../../../../../../src-tauri")\n',
    'artifacts_functions = artifacts_functions.replace("../../../src-tauri", "../../../../../../src-tauri")\n'
    'artifacts_functions = artifacts_functions.replace("../../../schemas", "../../../../../../schemas")\n'
    'artifacts_functions = artifacts_functions.replace("fn built_in_research_schema(", "pub(super) fn built_in_research_schema(")\n'
    'artifacts_functions = artifacts_functions.replace("fn built_in_research_prompt(", "pub(super) fn built_in_research_prompt(")\n',
    "artifact paths and visibility",
)

replace_once(
    'gateway_functions = gateway_functions.replace("../../../src-tauri", "../../../../../../src-tauri")\n',
    'gateway_functions = gateway_functions.replace("../../../src-tauri", "../../../../../../src-tauri")\n'
    'gateway_functions = gateway_functions.replace("fn built_in_gateway(", "pub(super) fn built_in_gateway(")\n'
    'gateway_functions = gateway_functions.replace("fn built_in_gateway_config(", "pub(super) fn built_in_gateway_config(")\n',
    "gateway visibility",
)

replace_once(
    'reference_functions = "\\n\\n".join([\n    fn("citation_drafts"),\n    fn("source_drafts"),\n    fn("verified_reference_domain"),\n    fn("optional_u32"),\n])\n',
    'reference_functions = "\\n\\n".join([\n    fn("citation_drafts"),\n    fn("source_drafts"),\n    fn("verified_reference_domain"),\n    fn("optional_u32"),\n])\n'
    'reference_functions = reference_functions.replace("fn citation_drafts(", "pub(super) fn citation_drafts(")\n'
    'reference_functions = reference_functions.replace("fn source_drafts(", "pub(super) fn source_drafts(")\n',
    "reference visibility",
)

replace_once(
    'validate = fn("validate_command")\n',
    'validate = fn("validate_command")\n'
    'validate = validate.replace("fn validate_command(", "pub(super) fn validate_command(")\n',
    "validation visibility",
)

replace_once(
    "    '''use super::*;\n\npub(crate) async fn execute(\n",
    "    '''use super::{\n    artifacts::{built_in_research_prompt, built_in_research_schema},\n    attempt_audit::PortAttemptSink,\n    gateway::built_in_gateway,\n    references::{citation_drafts, source_drafts},\n    validation::validate_command,\n    *,\n};\n\npub(crate) async fn execute(\n",
    "execution sibling imports",
)

replace_once(
    'use artifacts::*;\nuse attempt_audit::*;\nuse gateway::*;\nuse references::*;\npub use types::OpenAiResearchCommand;\nuse validation::*;\n',
    'pub use types::OpenAiResearchCommand;\n',
    "remove parent glob imports",
)

replace_once(
    'write(\n    "crates/application/src/use_cases/research/openai_gateway/tests.rs",\n    test_body(),\n)\n',
    'tests = test_body()\n'
    'tests = tests.replace(\n'
    '    "use super::*;",\n'
    '    "use super::*;\\nuse super::artifacts::{built_in_research_prompt, built_in_research_schema};\\nuse super::gateway::built_in_gateway_config;",\n'
    '    1,\n'
    ')\n'
    'if tests.count("../../../src-tauri") != 1:\n'
    '    raise RuntimeError("OpenAI gateway test resource path anchor mismatch")\n'
    'tests = tests.replace("../../../src-tauri", "../../../../../../src-tauri")\n'
    'write(\n'
    '    "crates/application/src/use_cases/research/openai_gateway/tests.rs",\n'
    '    tests,\n'
    ')\n',
    "test helper imports and resource path",
)

replace_once(
    'service_path = ROOT / "crates/application/src/services/research/service.rs"\n'
    'text = service_path.read_text(encoding="utf-8")\n'
    'text = text.replace(\n',
    'service_path = ROOT / "crates/application/src/services/research/service.rs"\n'
    'text = service_path.read_text(encoding="utf-8")\n'
    'obsolete_fact_pipeline_bridge = \'\'\'    pub(crate) async fn register_fact_pipeline_artifacts(\n'
    '        &self,\n'
    '        port: &dyn ResearchArtifactPort,\n'
    '    ) -> ApplicationResult<()> {\n'
    '        fact_pipeline::register_fact_pipeline_artifacts(port).await\n'
    '    }\n\n'
    '\'\'\'\n'
    'if text.count(obsolete_fact_pipeline_bridge) != 1:\n'
    '    raise RuntimeError("obsolete ResearchService Fact Pipeline bridge anchor mismatch")\n'
    'text = text.replace(obsolete_fact_pipeline_bridge, "", 1)\n'
    'text = text.replace(\n',
    "remove obsolete ResearchService bridge",
)

replace_once(
    'database_path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\\n")\n\n'
    '# Root export moves to the new owner; legacy owner is removed.\n',
    'database_path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\\n")\n\n'
    'database_service_path = ROOT / "crates/application/src/services/database/service.rs"\n'
    'text = database_service_path.read_text(encoding="utf-8")\n'
    'obsolete_transition_store = \'\'\'    pub(crate) fn transition_store(&self) -> PersistenceStore {\n'
    '        self.session.transition_store()\n'
    '    }\n\n'
    '\'\'\'\n'
    'if text.count(obsolete_transition_store) != 1:\n'
    '    raise RuntimeError("obsolete PreparedDatabaseConnection transition_store anchor mismatch")\n'
    'database_service_path.write_text(\n'
    '    text.replace(obsolete_transition_store, "", 1), encoding="utf-8", newline="\\n"\n'
    ')\n\n'
    '# Root export moves to the new owner; legacy owner is removed.\n',
    "remove obsolete database transition bridge",
)

replace_once(
    '数据库 Schema、迁移、生产依赖、research-gateway transport 与 P4 worker / 人工冲突写入行为未改变。AT3 仍处于 `IN_PROGRESS`',
    '迁移后不再使用的 `PreparedDatabaseConnection::transition_store` 与 `ResearchService::register_fact_pipeline_artifacts` 过渡桥接已移除；数据库 Schema、迁移、生产依赖、research-gateway transport 与 P4 worker / 人工冲突写入行为未改变。AT3 仍处于 `IN_PROGRESS`',
    "root README AT3 cleanup record",
)

replace_once(
    'P4 orchestration worker 与 manual conflict mutation 保持后续 Atomic Task 边界。AT3 当前 `IN_PROGRESS`',
    '迁移后无调用者的 Database transition-store 与 ResearchService Fact Pipeline 转发桥接已移除；P4 orchestration worker 与 manual conflict mutation 保持后续 Atomic Task 边界。AT3 当前 `IN_PROGRESS`',
    "R03 README AT3 cleanup record",
)

GENERATOR.write_text(text, encoding="utf-8", newline="\n")
print("AT3 generator patched for moved paths, sibling visibility, test resources, and obsolete bridges")
