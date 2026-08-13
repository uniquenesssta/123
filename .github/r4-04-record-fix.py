from pathlib import Path

path = Path("docs/modular-rewrite/R04-persistence-foundation/R04-04-port-adapter-注册.md")
text = path.read_text(encoding="utf-8")
anchor = "## 验证\n\n"
insert = (
    "## 验证\n\n"
    "- 首次 implementation run `31686470233`：起点保护范围通过；实施 helper 在切换 Application adapters 时发现 `analytics.rs` 存在多行 `self\\n.transition_store()` 形式，生成器只覆盖单行形式后主动 fail-fast。最小门禁、阶段回归、文档与提交步骤均未执行，远端未产生任何生产源码提交。恢复 helper 改为同时规范单行/多行转发，并从基线重新生成全部 adapter 变更。\n\n"
)
if text.count(anchor) != 1:
    raise RuntimeError(f"R4-04 verification anchor expected once, found {text.count(anchor)}")
text = text.replace(anchor, insert, 1)
path.write_text(text, encoding="utf-8", newline="\n")
