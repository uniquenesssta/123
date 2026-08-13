from pathlib import Path

path = Path("docs/modular-rewrite/R04-persistence-foundation/R04-04-port-adapter-注册.md")
text = path.read_text(encoding="utf-8")
anchor = "## 验证\n\n"
insert = (
    "## 验证\n\n"
    "- 首次 implementation run `31686470233`：起点保护范围通过；实施 helper 在切换 Application adapters 时发现 `analytics.rs` 存在多行 `self\\n.transition_store()` 形式，生成器只覆盖单行形式后主动 fail-fast。最小门禁、阶段回归、文档与提交步骤均未执行，远端未产生任何生产源码提交。恢复 helper 改为同时规范单行/多行转发，并从基线重新生成全部 adapter 变更。\n"
    "- 第二次 implementation run `31686662626`：起点保护范围与规范化 adapter 生成已完成，`write_verifier_and_package()` 也实际成功；临时 runner 却因控制流保护错误，把“无需 fallback”误判为异常并主动 fail-fast。最小门禁、阶段回归、文档与提交步骤仍未执行，远端仍无生产源码提交。恢复只移除该错误 guard，成功路径直接继续，fallback 仍仅捕获唯一已知 package anchor mismatch，其他异常继续硬失败。\n"
    "- 第三次 implementation run `31686951586`：Apply 已成功并首次进入真正最小门禁；`verify:persistence-adapters` 先行失败，原因是 verifier 用单行字符串匹配 `register_adapters(options).await.map_err(...)`，而 rustfmt 将相同调用链格式化为多行。该 run 在专项静态门禁即 fail-fast，R4-01/R4-02/R4-03 后续门禁、Rust compile/tests、stage regression、文档与提交均未执行。恢复仅将此断言改为格式无关正则，不修改生产注册实现、不降低语义检查。\n\n"
)
if text.count(anchor) != 1:
    raise RuntimeError(f"R4-04 verification anchor expected once, found {text.count(anchor)}")
text = text.replace(anchor, insert, 1)
path.write_text(text, encoding="utf-8", newline="\n")
