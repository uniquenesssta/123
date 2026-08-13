from pathlib import Path
import os

run_id = os.environ["GITHUB_RUN_ID"]
env_failed = os.environ["R4_03_ENV_FAILED_RUN"]

record_path = Path("docs/modular-rewrite/R04-persistence-foundation/R04-03-row-映射基础规范.md")
record = record_path.read_text(encoding="utf-8")
anchor = f"- 第二次调度 run `31665943169`：修复方案本身未执行，workflow 因嵌套 verifier 源码破坏 YAML block 缩进在调度阶段失败，0 个 job、无生产源码变化。\n"
insert = anchor + f"- 第三次 hard gate run `{env_failed}`：R4-03 最小门禁全部通过，`npm run verify:architecture` 通过；`npm run verify:frontend` 在既有 `typescript` 开发依赖未安装时于 `verify-searchable-select-diagnostics.mjs` 失败。该 run 按 fail-fast 未执行 workspace Clippy/tests、未提交生产源码。随后确认项目既有 `npm run setup` 会通过 `ensure-node-dependencies.mjs --install-only` 准备锁定依赖，因此恢复执行只补齐标准环境准备，不新增或升级依赖。\n"
if record.count(anchor) != 1:
    raise RuntimeError(f"task record recovery anchor expected once, found {record.count(anchor)}")
record = record.replace(anchor, insert, 1)
record_path.write_text(record, encoding="utf-8", newline="\n")

stage_path = Path("docs/modular-rewrite/R04-persistence-foundation/README.md")
stage = stage_path.read_text(encoding="utf-8")
old = "- run `31665585062` 因 verifier 转义错误与共享调用面漏扫停止；run `31665943169` 因临时 workflow YAML 缩进错误在调度前停止；两次均未提交生产源码。"
new = old + f" run `{env_failed}` 随后通过 R4-03 最小门禁与完整 architecture，但因临时 runner 未先执行既有 `npm run setup`，frontend 在缺少已声明的 `typescript` 开发依赖时停止；workspace Clippy/tests 未执行且仍未提交生产源码。"
if stage.count(old) != 1:
    raise RuntimeError(f"stage recovery anchor expected once, found {stage.count(old)}")
stage = stage.replace(old, new, 1)
stage_path.write_text(stage, encoding="utf-8", newline="\n")

readme_path = Path("README.md")
readme = readme_path.read_text(encoding="utf-8")
old = "前两次临时执行分别因 verifier 生成错误与 workflow YAML 缩进错误停止且均未提交生产源码；"
new = old + f"第三次 run `{env_failed}` 已通过最小门禁与 architecture，但 frontend 因临时 runner 未先准备既有 `typescript` 开发依赖而停止，Clippy/workspace tests 按 fail-fast 未执行；"
if readme.count(old) != 1:
    raise RuntimeError(f"root README recovery anchor expected once, found {readme.count(old)}")
readme = readme.replace(old, new, 1)
readme_path.write_text(readme, encoding="utf-8", newline="\n")

print(f"R4-03 recovery history recorded; final successful run={run_id}, dependency-setup failure={env_failed}")
