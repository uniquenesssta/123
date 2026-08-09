from pathlib import Path

GENERATOR = Path(__file__).resolve().with_name("r3-07-at2.py")
text = GENERATOR.read_text(encoding="utf-8")
old = 'check(openai.includes("self.research.register_fact_pipeline_artifacts(session).await?"), "OpenAI artifact 初始化未通过 ResearchService 注册 Fact Pipeline 来源策略");'
new = 'check(/self\\.research\\s*\\.register_fact_pipeline_artifacts\\(session\\)\\s*\\.await\\?/.test(openai), "OpenAI artifact 初始化未通过 ResearchService 注册 Fact Pipeline 来源策略");'
if text.count(old) != 1:
    raise RuntimeError("AT2 Research verifier template anchor mismatch")
GENERATOR.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")
print("AT2 Research verifier template made rustfmt-insensitive")
