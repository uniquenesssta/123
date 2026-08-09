from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected exactly one status marker in {path}, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


root_old = "- clean 源码头 `7e3f43d805b22fceffc6a367392ad9fa1eabef36` 已删除 2A 与 warning-cleanup 的临时 workflow / Python 脚本。完整 Public Platform CI 仍需在最终状态提交上通过后才能关闭 2A；当前不得将 R3-06 或 R3-07 标记为 DONE / READY。"
root_new = "- Atomic Task 2A 已正式关闭为 `DONE`。最终验收提交 `443286b269cc6f34318bcf9ea60a86697f7a64a8` 的 Public Platform CI run `31268125289` / Windows Automated job `93129475772` 已全部通过：architecture、frontend、17 个截图回归视口、TypeScript、Vite production build、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Tauri Windows release 构建与 release 客户端运行日志验收均为 PASS；Application tests 33/33 通过。artifact `9025087726` 大小 `14245255` 字节，SHA-256 `f69a988b6832c5af18af661ea3e436ffeb48212d9a7f67c049676356376180ae`。18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按既有安全设计保持 `ignored`，未记为已执行。2A、warning cleanup、inventory refresh 与 Clippy 修复使用的临时 workflow / 脚本均已清理；R3-06 继续为 `IN_PROGRESS`，R3-07 继续为 `BLOCKED`。"
replace_once("README.md", root_old, root_new)

stage_old = "- 2A 与 warning-cleanup 的临时 workflow / Python 脚本均已删除。当前仍等待最终状态提交上的完整 Public Platform CI；在该硬门禁通过前，R3-06 保持 `IN_PROGRESS`，R3-07 保持 `BLOCKED`。"
stage_new = "- Atomic Task 2A 已正式关闭为 `DONE`。最终验收提交 `443286b269cc6f34318bcf9ea60a86697f7a64a8` 的 Public Platform CI run `31268125289` / Windows Automated job `93129475772` 已通过 architecture、完整 frontend、17 个截图回归视口、TypeScript、Vite、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Tauri Windows release 构建与 release runtime 日志验收；Application tests 33/33 通过。artifact `9025087726` 大小 `14245255` 字节，SHA-256 `f69a988b6832c5af18af661ea3e436ffeb48212d9a7f67c049676356376180ae`。18 个真实 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 继续按既有安全设计保持 `ignored`，未记为已执行。所有 2A 临时 workflow / Python 脚本均已删除；R3-06 仍为 `IN_PROGRESS`，R3-07 仍为 `BLOCKED`，可继续 R3-06 下一 Atomic Task。"
replace_once("docs/modular-rewrite/R03-application-services/README.md", stage_old, stage_new)

print("R3-06 Atomic Task 2A status closed in canonical docs")
