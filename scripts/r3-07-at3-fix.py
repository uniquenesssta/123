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
    'write(\n'
    '    "crates/application/src/use_cases/research/openai_gateway/tests.rs",\n'
    '    tests,\n'
    ')\n',
    "test helper imports",
)

GENERATOR.write_text(text, encoding="utf-8", newline="\n")
print("AT3 generator patched for moved paths and sibling visibility")
