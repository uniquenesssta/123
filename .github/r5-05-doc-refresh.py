from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one replacement target, found {count}")
    file_path.write_text(text.replace(old, new, 1), encoding="utf-8")


root_old = "- R5-05 Route Resolution Reads 已完成生产 owner 切换并进入 `VERIFYING`：`resolve_competition_context` / scope validation 与 `resolve_route` / RouteDecision mapping 已从 legacy `competitions.rs` / `routing.rs` 收敛到 `adapters/competition/route_resolution/{context,route}/`；legacy `competitions.rs` 已删除，`routing.rs` 只保留 R5-06 model registration，未提前创建 `model_run_identity/`。old-owner PostgreSQL contract run `31869022228` / job `94974599427`、owner-switch/new-owner run `31869137395` / job `94974903779`、ownership gate `31869503602` / job `94975820076`、official inventory `31869537207` / job `94975903916` 均为 `SUCCESS`；第一轮 hard gate `31869573253` 因 R4-03 旧 mapping verifier 读取已删除 legacy path 而 fail-fast，推进 call-path 后第二轮 hard gate `31869727870` / job `94976380579` 为 `SUCCESS`，完整 architecture、模型保护、rustfmt、Persistence/Application check/tests 与同一 PostgreSQL 16 route/context contract 全部通过。Domain/Application/Tauri/Schema/0001–0046 migration、配置/错误/日志、前端行为、route algorithm/result、model identity、Cargo manifests/Cargo.lock、生产依赖与模型保护资产保持不变。18 个 broad PostgreSQL ignored tests 与 destructive reset 未执行，未触碰用户数据库。R5-06 继续 `BLOCKED`；详见 `docs/modular-rewrite/R05-competition-routing-persistence/R05-05-route-resolution-reads.md`。"
root_new = "- R5-05 Route Resolution Reads 已完成生产 owner 切换并进入 `VERIFYING`：`resolve_competition_context` / scope validation 与 `resolve_route` / RouteDecision mapping 已从 legacy `competitions.rs` / `routing.rs` 收敛到 `adapters/competition/route_resolution/{context,route}/`；legacy `competitions.rs` 已删除，`routing.rs` 只保留 R5-06 model registration，未提前创建 `model_run_identity/`。old-owner PostgreSQL contract run `31869022228` / job `94974599427`、owner-switch/new-owner run `31869137395` / job `94974903779`、ownership gate `31869503602` / job `94975820076`、official inventory `31869537207` / job `94975903916`、第二轮 hard gate `31869727870` / job `94976380579` 均为 `SUCCESS`。PR #30 首轮 fixed clean HEAD `bc93518f8436ee5175b4fea4af3bd32441c04e1d` 的 canonical run `31870124821` / job `94977359038` 在 Windows workspace Clippy `-D warnings` 因 typed `RouteRow.competition_kind` 从未消费而失败；修复仅删除该未使用 Row 字段和两条 SELECT 的冗余返回列，仍保留 `b.competition_kind` WHERE 过滤与 specificity/route result 语义，并用 official generator 刷新 inventory。inventory refresh run `31870479459` / job `94978235169` 为 `SUCCESS`；修复后的 Windows canonical run `31870519567` / job `94978336960` 为 `SUCCESS`，artifact `9243579284`，SHA-256 `a728b4806b18d034a34a79142dd926734e5461ece23f04ddf577039c7a1304e4`。Ubuntu 专项 Clippy fix gate `31870518531` 在 architecture/rustfmt 通过后因 runner 缺少系统 `glib-2.0` 而停止，workspace Clippy 与 PostgreSQL contract 未在该 gate 完成，未记为通过；同一源码修复已由 canonical Windows acceptance 验证。Domain/Application/Tauri/Schema/0001–0046 migration、配置/错误/日志、前端行为、route algorithm/result、model identity、Cargo manifests/Cargo.lock、生产依赖与模型保护资产保持不变。18 个 broad PostgreSQL ignored tests 与 destructive reset 未执行，未触碰用户数据库。transient helper 已清理，最终 clean PR canonical CI 仍需对最终固定 HEAD 通过；R5-06 继续 `BLOCKED`。详见 `docs/modular-rewrite/R05-competition-routing-persistence/R05-05-route-resolution-reads.md`。"
replace_once("README.md", root_old, root_new)

record_anchor = "\n## 兼容性与未变范围\n"
record_insert = """
## PR #30 canonical 验证与修复

- PR #30 首轮 fixed clean HEAD `bc93518f8436ee5175b4fea4af3bd32441c04e1d` 的 Public Platform CI run `31870124821` / Windows job `94977359038` 为 `FAILURE`。architecture 已通过，失败发生在 Windows automated acceptance 的 workspace Clippy `-D warnings`：typed `RouteRow.competition_kind` 自 owner switch 后从未被 Domain mapper 或 source 判定消费，因此触发 `dead_code`。
- 修复没有使用 `#[allow]`、没有放宽门禁，也没有改变 route algorithm/result。仅删除 `RouteRow.competition_kind` 未使用字段，以及 explicit package / binding candidate SELECT 中对应的冗余返回列；automatic candidate 的 `b.competition_kind = $4` WHERE 过滤、Stage > Season > Competition > CompetitionKind Default specificity、priority/created_at/id 排序、有效期与模型过滤均保持不变。
- 该字段删除改变 PostgreSQL mapping usage digest 后，full architecture 正确发现 inventory drift；只使用官方 `node scripts/generate-domain-type-inventory.mjs` 刷新。run `31870479459` / job `94978235169` 为 `SUCCESS`。
- Ubuntu 专项修复 gate run `31870518531` 在 R5 ownership、full architecture 与 canonical rustfmt 通过后，workspace Clippy 因 runner 缺少系统 `glib-2.0` / `glib-2.0.pc` 停止；PostgreSQL contract 按 fail-fast 未执行。该环境阻塞未记为源码或测试通过，也未通过安装新生产依赖绕过。
- 同一修复源码随后由 canonical Public Platform CI run `31870519567` / Windows job `94978336960` 完成验证并为 `SUCCESS`；artifact `9243579284`，大小 `13929568` 字节，SHA-256 `a728b4806b18d034a34a79142dd926734e5461ece23f04ddf577039c7a1304e4`。该 run 证明 Windows architecture 与完整 automated acceptance（含 workspace Clippy/tests/Tauri release/runtime）通过。
- 上述成功 run 对修复验证有效，但当时分支仍含临时修复 workflow；临时 workflow 已在随后提交中全部删除。R5-05 仍保持 `VERIFYING`，最终 clean HEAD 必须再次通过 canonical Public Platform CI 才允许 fixed-head merge。
"""
replace_once(
    "docs/modular-rewrite/R05-competition-routing-persistence/R05-05-route-resolution-reads.md",
    record_anchor,
    record_insert + record_anchor,
)

stage_old = "- R5-05 当前为 `VERIFYING`，R5-06 继续 `BLOCKED`；clean PR 与合并后/final canonical Windows 门禁尚未完成。"
stage_new = """- PR #30 首轮 fixed clean HEAD `bc93518f8436ee5175b4fea4af3bd32441c04e1d` 的 canonical run `31870124821` / job `94977359038` 在 Windows workspace Clippy `-D warnings` 因未使用的 typed `RouteRow.competition_kind` 失败；没有合并失败 tree。
- 修复仅删除未消费 Row 字段与冗余 SELECT 返回列，保留 `b.competition_kind` WHERE 过滤和全部 route specificity/result 语义；official inventory refresh run `31870479459` / job `94978235169` 为 `SUCCESS`。
- Ubuntu 专项 gate `31870518531` 在 architecture/rustfmt 通过后受 runner 缺 `glib-2.0` 系统库阻塞，workspace Clippy/PG contract 未完成；同一源码的 canonical Windows run `31870519567` / job `94978336960` 为 `SUCCESS`，artifact `9243579284` SHA-256 `a728b4806b18d034a34a79142dd926734e5461ece23f04ddf577039c7a1304e4`。
- transient helper 已清理；R5-05 当前仍为 `VERIFYING`，R5-06 继续 `BLOCKED`。最终 clean PR canonical、固定 HEAD merge、merged stage CI 与 final closeout canonical Windows 门禁仍需完成。"""
replace_once(
    "docs/modular-rewrite/R05-competition-routing-persistence/README.md",
    stage_old,
    stage_new,
)
