from pathlib import Path
import os

run_id = os.environ["GITHUB_RUN_ID"]
env_failed = os.environ["R4_03_ENV_FAILED_RUN"]
publish_failed = os.environ["R4_03_PUBLISH_FAILED_RUN"]
cleanup_failed = os.environ["R4_03_CLEANUP_FAILED_RUN"]

record_path = Path("docs/modular-rewrite/R04-persistence-foundation/R04-03-row-映射基础规范.md")
record = record_path.read_text(encoding="utf-8")
anchor = "- 第二次调度 run `31665943169`：修复方案本身未执行，workflow 因嵌套 verifier 源码破坏 YAML block 缩进在调度阶段失败，0 个 job、无生产源码变化。\n"
insert = anchor + f"- 第三次 hard gate run `{env_failed}`：R4-03 最小门禁全部通过，`npm run verify:architecture` 通过；`npm run verify:frontend` 在既有 `typescript` 开发依赖未安装时于 `verify-searchable-select-diagnostics.mjs` 失败。该 run 按 fail-fast 未执行 workspace Clippy/tests、未提交生产源码。随后确认项目既有 `npm run setup` 会通过 `ensure-node-dependencies.mjs --install-only` 准备锁定依赖，因此恢复执行只补齐标准环境准备，不新增或升级依赖。\n- 第四次 hard gate run `{publish_failed}`：最小门禁、`npm run setup`、完整 architecture/frontend、workspace Clippy `-D warnings`、workspace tests、冻结范围、文档写入与 transient cleanup 全部通过；最终发布步骤因临时 workflow 将 npm cache 放在仓库根 `/.npm-cache`，触发严格 clean-worktree 门禁而失败。该缓存不在 `.gitignore` 中，未提交生产源码。恢复执行把 npm cache 恢复到项目既定的源码根目录上一级，并保留 clean-worktree 硬门禁。\n- 第五次 hard gate run `{cleanup_failed}`：最小门禁、完整 stage regression、冻结范围与文档写入再次全部通过；发布前 cleanup 将 `git ls-files --others --exclude-standard` 放在精确暂存之前，导致本任务应新增的 `mapping/*.rs`、`competition_kind.rs`、专项 verifier 与节点记录本身被当作非法 untracked 拒绝。该 run 未进入 commit。恢复执行改为先只暂存精确 R4-03 allowlist，再要求剩余 untracked 为 0，并对 staged diff 做精确集合校验；门禁强度不降低。\n"
if record.count(anchor) != 1:
    raise RuntimeError(f"task record recovery anchor expected once, found {record.count(anchor)}")
record = record.replace(anchor, insert, 1)
record_path.write_text(record, encoding="utf-8", newline="\n")

stage_path = Path("docs/modular-rewrite/R04-persistence-foundation/README.md")
stage = stage_path.read_text(encoding="utf-8")
old = "- run `31665585062` 因 verifier 转义错误与共享调用面漏扫停止；run `31665943169` 因临时 workflow YAML 缩进错误在调度前停止；两次均未提交生产源码。"
new = old + f" run `{env_failed}` 随后通过 R4-03 最小门禁与完整 architecture，但因临时 runner 未先执行既有 `npm run setup`，frontend 在缺少已声明的 `typescript` 开发依赖时停止；workspace Clippy/tests 未执行且仍未提交生产源码。run `{publish_failed}` 已通过全部质量、冻结范围、文档与 transient cleanup 门禁，最终仅因 npm cache 临时落在仓库根而被 clean-worktree 发布门禁拒绝，仍未提交生产源码。run `{cleanup_failed}` 再次通过全部质量、范围与文档门禁，但 cleanup 在精确暂存前检查 untracked，误将本任务应新增文件视为非法项并停止，仍未提交生产源码。"
if stage.count(old) != 1:
    raise RuntimeError(f"stage recovery anchor expected once, found {stage.count(old)}")
stage = stage.replace(old, new, 1)
stage_path.write_text(stage, encoding="utf-8", newline="\n")

readme_path = Path("README.md")
readme = readme_path.read_text(encoding="utf-8")
old = "前两次临时执行分别因 verifier 生成错误与 workflow YAML 缩进错误停止且均未提交生产源码；"
new = old + f"第三次 run `{env_failed}` 已通过最小门禁与 architecture，但 frontend 因临时 runner 未先准备既有 `typescript` 开发依赖而停止，Clippy/workspace tests 按 fail-fast 未执行；第四次 run `{publish_failed}` 已通过完整质量与范围门禁，仅因 npm cache 临时落在仓库根而被严格 clean-worktree 发布门禁拒绝；第五次 run `{cleanup_failed}` 再次通过完整质量与范围门禁，但 cleanup 在精确暂存前检查 untracked，误将任务应新增文件本身拒绝；"
if readme.count(old) != 1:
    raise RuntimeError(f"root README recovery anchor expected once, found {readme.count(old)}")
readme = readme.replace(old, new, 1)
readme_path.write_text(readme, encoding="utf-8", newline="\n")

print(
    f"R4-03 recovery history recorded; final successful run={run_id}, "
    f"dependency-setup failure={env_failed}, publication-cache failure={publish_failed}, "
    f"cleanup-order failure={cleanup_failed}"
)
