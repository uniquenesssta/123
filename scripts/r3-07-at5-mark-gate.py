from pathlib import Path
import os
import sys

ROOT = Path(__file__).resolve().parents[1]
run_id = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("GITHUB_RUN_ID")
if not run_id:
    raise RuntimeError("AT5 hard gate run id is required")

replacement = (
    f"Windows hard gate run `{run_id}` 已通过 Research/Database/Prediction 专项、Application Ports、"
    "完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。"
    "AT5 clean implementation 将在本次 hard gate 后由同一 workflow 提交；正式 Public Platform Windows Automated "
    "通过前 R3-07 保持 `IN_PROGRESS`。"
)

for relative in [
    "README.md",
    "docs/modular-rewrite/R03-application-services/README.md",
]:
    path = ROOT / relative
    text = path.read_text(encoding="utf-8")
    if text.count("AT5_HARD_GATE_PENDING") != 1:
        raise RuntimeError(f"AT5 README marker mismatch: {relative}")
    path.write_text(
        text.replace("AT5_HARD_GATE_PENDING", replacement, 1),
        encoding="utf-8",
        newline="\n",
    )

print(f"AT5 hard gate evidence recorded for run {run_id}")
