from pathlib import Path
import subprocess

BASE = "f7bc0101a4c443e1fc6e95d2e604a7df7e073d5e"
ROOT = Path(".")

stage_readme = ROOT / "docs/modular-rewrite/R04-persistence-foundation/README.md"
node_record = ROOT / "docs/modular-rewrite/R04-persistence-foundation/R04-04-port-adapter-注册.md"
stage_completion = ROOT / "docs/modular-rewrite/R04-persistence-foundation/R04-stage-completion.md"
r5_readme = ROOT / "docs/modular-rewrite/R05-competition-routing-persistence/README.md"
root_readme = ROOT / "README.md"

# 1) Remove the accidental quoted-newline literals from the R4 stage index.
text = stage_readme.read_text(encoding="utf-8")
count = text.count("\n'\n'- ")
if count != 5:
    raise SystemExit(f"unexpected malformed R4 stage README marker count: {count}")
text = text.replace("\n'\n'- ", "\n\n- ")
if "\n'\n'" in text or "\n'- " in text:
    raise SystemExit("malformed quote marker remains in R4 stage README")
closeout_line = (
    "- 最终 closeout HEAD `f7bc0101a4c443e1fc6e95d2e604a7df7e073d5e` 的 Public Platform CI "
    "run `31730498450` / job `94549484609`：`SUCCESS`；artifact `9193694750`，"
    "13,894,301 bytes，SHA-256 `231cf7729f383c655f70cc9fa35cc9588c14770ba9b5dc09b2e26b527ccdd824`。"
)
anchor = "- R5-01 `Competitions Repository` 为唯一 `READY`；R5-02~R5-06 `BLOCKED`。本收口未包含任何 R5 生产源码修改。"
if anchor not in text:
    raise SystemExit("R4 stage README R5 anchor missing")
if "31730498450" not in text:
    text = text.replace(anchor, closeout_line + "\n\n" + anchor)
stage_readme.write_text(text.rstrip() + "\n", encoding="utf-8")

# 2) Correct the stale VERIFYING sentence and record the final closeout CI in stage completion.
text = stage_completion.read_text(encoding="utf-8")
stale = "- 本阶段 `README.md` 已将 R4-04 标记 `DONE`、阶段标记 `VERIFYING`，并链接本记录。"
fixed = "- 本阶段 `README.md` 已将 R4-04 与 R4 阶段均标记 `DONE`，并链接本记录。"
if stale not in text:
    raise SystemExit("stale stage completion sentence missing")
text = text.replace(stale, fixed, 1)
ci_anchor = "- R4-04 squash merge `b97587c9d20165018f80040dc2a2c098dbbec177` 后 canonical stage Public Platform CI `31724131556` / job `94528167665`：SUCCESS；artifact `9191415520`，SHA-256 `3ccb37d8eab17c0e589354c10c3423579397629f9f76481b267acd0749db38cf`。"
ci_line = "- formal closeout HEAD `f7bc0101a4c443e1fc6e95d2e604a7df7e073d5e` 的 canonical Public Platform CI `31730498450` / job `94549484609`：SUCCESS；artifact `9193694750`，SHA-256 `231cf7729f383c655f70cc9fa35cc9588c14770ba9b5dc09b2e26b527ccdd824`。"
if ci_anchor not in text:
    raise SystemExit("stage completion CI anchor missing")
if "31730498450" not in text:
    text = text.replace(ci_anchor, ci_anchor + "\n" + ci_line, 1)
stage_completion.write_text(text.rstrip() + "\n", encoding="utf-8")

# 3) Add final closeout CI evidence to the R4-04 node record.
text = node_record.read_text(encoding="utf-8")
node_anchor = "- R4 stage 因真实数据库矩阵补齐而正式 `DONE`；R5-01 仅开放为 `READY`，本节点未提前实现 R5。"
node_line = "- formal closeout HEAD `f7bc0101a4c443e1fc6e95d2e604a7df7e073d5e` 的 Public Platform CI run `31730498450` / job `94549484609`：`SUCCESS`；artifact `9193694750`，SHA-256 `231cf7729f383c655f70cc9fa35cc9588c14770ba9b5dc09b2e26b527ccdd824`。"
if node_anchor not in text:
    raise SystemExit("R4-04 closeout anchor missing")
if "31730498450" not in text:
    text = text.replace(node_anchor, node_anchor + "\n" + node_line, 1)
node_record.write_text(text.rstrip() + "\n", encoding="utf-8")

# 4) Root README: keep the concise R4 summary and append the final closeout CI evidence to that one record.
text = root_readme.read_text(encoding="utf-8")
lines = text.splitlines()
matches = [i for i, line in enumerate(lines) if line.startswith("- R4 Persistence 基础设施阶段已正式关闭为 `DONE`。")]
if len(matches) != 1:
    raise SystemExit(f"unexpected root README R4 summary count: {len(matches)}")
i = matches[0]
if "31730498450" not in lines[i]:
    lines[i] = lines[i] + " 最终 closeout HEAD `f7bc0101a4c443e1fc6e95d2e604a7df7e073d5e` 的 Public Platform CI `31730498450` / job `94549484609` 同样 `SUCCESS`，artifact `9193694750`，SHA-256 `231cf7729f383c655f70cc9fa35cc9588c14770ba9b5dc09b2e26b527ccdd824`。"
root_readme.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")

# 5) R5 index: record the verified R4 closeout evidence without pinning a stale future R5 branch base.
text = r5_readme.read_text(encoding="utf-8")
r5_anchor = "- R5-01 开始时必须从 `rewrite/r4-persistence-foundation` 最终 closeout HEAD 独立建分支；该 HEAD 的最终 Public Platform CI 必须先为 `SUCCESS`。"
r5_line = "- 已验证 R4 closeout evidence：HEAD `f7bc0101a4c443e1fc6e95d2e604a7df7e073d5e` 的 Public Platform CI run `31730498450` / job `94549484609` 为 `SUCCESS`；R5-01 实际开工时仍必须重新读取 `rewrite/r4-persistence-foundation` 当前最终 HEAD 后再建独立分支。"
if r5_anchor not in text:
    raise SystemExit("R5 baseline anchor missing")
if "31730498450" not in text:
    text = text.replace(r5_anchor, r5_anchor + "\n" + r5_line, 1)
r5_readme.write_text(text.rstrip() + "\n", encoding="utf-8")

# Remove transient files from the final tree before staging.
for transient in [ROOT / ".github/r4-docs-correction.py", ROOT / ".github/workflows/r4-docs-correction.yml"]:
    if transient.exists():
        transient.unlink()

allowed = {
    "README.md",
    "docs/modular-rewrite/R04-persistence-foundation/README.md",
    "docs/modular-rewrite/R04-persistence-foundation/R04-04-port-adapter-注册.md",
    "docs/modular-rewrite/R04-persistence-foundation/R04-stage-completion.md",
    "docs/modular-rewrite/R05-competition-routing-persistence/README.md",
}

subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "diff", "--cached", "--check"], check=True)
net = subprocess.run(["git", "diff", "--name-only", BASE], check=True, text=True, capture_output=True).stdout.splitlines()
if set(net) != allowed:
    raise SystemExit(f"unexpected final net diff: {net}")

# Final semantic assertions.
if "`DONE`" not in stage_readme.read_text(encoding="utf-8").split("## 阶段状态", 1)[1].split("##", 1)[0]:
    raise SystemExit("R4 stage status is not DONE")
if "阶段标记 `VERIFYING`" in stage_completion.read_text(encoding="utf-8"):
    raise SystemExit("stale VERIFYING sentence remains")
if "31730498450" not in root_readme.read_text(encoding="utf-8"):
    raise SystemExit("root README missing final closeout CI")
print("R4 docs correction prepared; final net diff is exactly five documentation files.")

# Trigger update after workflow exists; no semantic change.
