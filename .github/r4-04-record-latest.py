from pathlib import Path

path = Path("docs/modular-rewrite/R04-persistence-foundation/R04-04-port-adapter-注册.md")
text = path.read_text(encoding="utf-8")
anchor = "Windows 2025 / Rust 1.88.0 / Node 22 hard gate run"
idx = text.find(anchor)
if idx < 0:
    raise RuntimeError("R4-04 final hard-gate summary anchor missing")
section = text.find("## 验证\n\n")
if section < 0:
    raise RuntimeError("R4-04 verification section missing")
insert_at = section + len("## 验证\n\n")
latest = (
    "- 第九次 implementation run `31689306408`：候选 rewrite、11 个同源旧 owner verifier 迁移、最小硬门禁均通过，`npm run verify:architecture` 全链 PASS；随后 `npm run verify:frontend` fail-fast，workspace Clippy/tests、frozen/docs/commit 未执行，远端仍无生产源码提交。\n"
    "- frontend diagnostic run `31689940421` 精确定位失败为 `verify-teams-players-service.mjs`：机械地把 composition 外原 `ActiveDatabase` 会话边界替换成 `PersistenceStore`，导致 Teams/Players facade 泄漏 concrete persistence。该 verifier 是有效架构门禁，未删除或放宽。\n"
    "- session-boundary audit run `31690075828` 枚举基线 composition 外全部 20 处 `ActiveDatabase` 引用，并确认多项 Service verifier 明确禁止 `PersistenceStore`/`PostgresStore`/`football_persistence_postgres` 泄漏。恢复方案建立 crate-private、零运行时包装的 `DatabaseSession = PersistenceStore` type alias：services/use-cases 只依赖 `DatabaseSession`，40 个 Application Port impl 仍直接 `for PersistenceStore`，旧 `ActiveDatabase` struct 与 `transition_store` 不恢复。\n"
    "- full frontend diagnostic run `31690313018` 使用与主 hard gate 相同候选树生成链并加入 `DatabaseSession` 边界后，`npm run verify:persistence-adapters` 与完整 `npm run verify:frontend` 均 PASS。该诊断未替代 workspace Clippy/tests；最终 hard gate 仍需从头执行完整最小门禁与 stage regression。\n\n"
)
if "31690313018" in text:
    raise RuntimeError("R4-04 latest verification history already recorded")
text = text[:insert_at] + latest + text[insert_at:]
path.write_text(text, encoding="utf-8", newline="\n")
