from pathlib import Path

GENERATOR = Path(__file__).resolve().with_name("r3-07-at2.py")
text = GENERATOR.read_text(encoding="utf-8")


def replace_once(old: str, new: str, label: str) -> None:
    global text
    if text.count(old) != 1:
        raise RuntimeError(f"AT2 generator anchor mismatch for {label}: {text.count(old)}")
    text = text.replace(old, new, 1)


replace_once(
    'check(openai.includes("self.research.register_fact_pipeline_artifacts(session).await?"), "OpenAI artifact 初始化未通过 ResearchService 注册 Fact Pipeline 来源策略");',
    'check(openai.replaceAll(" ", "").replaceAll(String.fromCharCode(10), "").replaceAll(String.fromCharCode(13), "").replaceAll(String.fromCharCode(9), "").includes("self.research.register_fact_pipeline_artifacts(session).await?"), "OpenAI artifact 初始化未通过 ResearchService 注册 Fact Pipeline 来源策略");',
    "rustfmt-insensitive Research verifier",
)

replace_once(
    'write("crates/application/src/use_cases/research/fact_pipeline/types.rs", internal_structs)',
    r'write("crates/application/src/use_cases/research/fact_pipeline/types.rs", "use super::*;\n\n" + internal_structs)',
    "types module parent imports",
)

replace_once(
    'process_body = re.sub(r"\\bstore\\b", "port", process_body)\nprocess_body = textwrap.indent(textwrap.dedent(process_body).strip("\\n"), "    ")',
    'process_body = re.sub(r"\\bstore\\b", "port", process_body)\nif process_body.count("&port,") != 2:\n    raise RuntimeError(f"expected exactly two coordinator &port references, found {process_body.count(\'&port,\')}")\nprocess_body = process_body.replace("&port,", "port,")\nprocess_body = textwrap.indent(textwrap.dedent(process_body).strip("\\n"), "    ")',
    "coordinator port references",
)

GENERATOR.write_text(text, encoding="utf-8", newline="\n")
print("AT2 generator fixed: verifier formatting, types imports, coordinator port references")
