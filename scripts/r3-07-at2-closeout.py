from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

readme = ROOT / "README.md"
text = readme.read_text(encoding="utf-8")
old = "- Atomic Task 2 Windows hard gate run `31298524184` 只有在 Research 专项、Database/Prediction 兼容验证、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests 全部通过后才形成 clean 提交。"
new = "- Atomic Task 2 已正式关闭为 `DONE`。Windows hard gate run `31298524184` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean 提交 `1fb4c7b05573ef75eb48903eea25bc8b2072c9de`。该 HEAD 的 Public Platform CI run `31298887231` / Windows Automated job `93208343460` 已全部通过，validation evidence upload 为 SUCCESS；artifact `9034165649` 大小 `14273283` 字节，SHA-256 `761fa51e8f811d9fd85bf09b5ff798b9862613d7206185993dfff227cdaf160b`。AT2 施工 workflow / generator / fix script 均已清理；R3-07 继续为 `IN_PROGRESS`，下一 Atomic Task 进入 OpenAI Research Gateway execution。"
if text.count(old) != 1:
    raise RuntimeError("README AT2 closeout anchor mismatch")
readme.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

r3 = ROOT / "docs/modular-rewrite/R03-application-services/README.md"
text = r3.read_text(encoding="utf-8")
old = "- AT2 Windows hard gate run `31298524184` 负责 Research/Database/Prediction 专项、Ports、architecture、Application check/tests、workspace Clippy/tests；通过前 R3-07 保持 `IN_PROGRESS`。"
new = "- Atomic Task 2 已正式关闭为 `DONE`。Windows hard gate run `31298524184` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean 提交 `1fb4c7b05573ef75eb48903eea25bc8b2072c9de`。正式 Public Platform CI run `31298887231` / Windows Automated job `93208343460` 全部 SUCCESS，validation evidence upload 成功；artifact `9034165649` 大小 `14273283` 字节，SHA-256 `761fa51e8f811d9fd85bf09b5ff798b9862613d7206185993dfff227cdaf160b`。AT2 临时施工文件已清理；R3-07 保持 `IN_PROGRESS`，下一 Atomic Task 为 OpenAI Research Gateway execution。"
if text.count(old) != 1:
    raise RuntimeError("R03 AT2 closeout anchor mismatch")
r3.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

print("R3-07 AT2 closeout records updated")
