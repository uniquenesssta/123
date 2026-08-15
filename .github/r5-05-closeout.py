from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one target, found {count}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


def regex_once(path: str, pattern: str, replacement: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.MULTILINE)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one regex target, found {count}")
    p.write_text(updated, encoding="utf-8")


root_line = (
    "- R5-05 Route Resolution Reads 已正式关闭为 `DONE`：competition context 与 route resolution persistence 已收敛到 "
    "`adapters/competition/route_resolution/{context,route}/`，legacy `competitions.rs` 已删除，legacy `routing.rs` 仅保留 R5-06 model registration；"
    "未提前创建 `model_run_identity/`。old/new owner PostgreSQL contract、R5 ownership、official Domain inventory 与 stage hard gate 均已通过。"
    "PR #30 首轮 clean canonical run `31870124821` / job `94977359038` 因 typed `RouteRow.competition_kind` 未使用而在 Windows workspace Clippy `-D warnings` 失败，失败 tree 未合并；"
    "修复仅删除未消费 Row 字段及冗余 SELECT 返回列，保留 `b.competition_kind` WHERE 过滤与 route specificity/result，official inventory refresh run `31870479459` / job `94978235169` 为 `SUCCESS`。"
    "同一修复源码 Windows canonical run `31870519567` / job `94978336960` 为 `SUCCESS`；最终 fixed clean HEAD `879dd724e9bace058d86387b04e97cbdb67843b7` 的 PR canonical run `31873912016` / job `94986637325` 为 `SUCCESS`，artifact `9244488927` SHA-256 `4cc15be2916477ea56a80080090c18b033c8eab1e6cfd2a4f4da948d1b24b754`。"
    "PR #30 已按 expected head squash merge 为 `d6b2d692dadbe546220eb0cb533bd9d4e207d230`；merged-stage canonical run `31875035071` / job `94989384122` 为 `SUCCESS`，artifact `9244773760` SHA-256 `f342644fd91e617331060360d47513a076594aa379f1e9e9493b287e2df97b08`。"
    "Ubuntu 专项 fix gate `31870518531` 受 runner 缺 `glib-2.0` 系统库阻塞，未把未完成的 workspace Clippy/PG contract 记为通过。18 个 broad PostgreSQL ignored tests 与 destructive reset 未在本节点执行，未触碰用户数据库。"
    "R5-06 已开放为 `READY`；本 closeout tree 仍必须通过一次 canonical Public Platform CI，成功后才是 R5-06 的有效起始基线。详见 `docs/modular-rewrite/R05-competition-routing-persistence/R05-05-route-resolution-reads.md`。"
)
regex_once("README.md", r"^- R5-05 Route Resolution Reads .*?$", root_line)

stage_path = "docs/modular-rewrite/R05-competition-routing-persistence/README.md"
replace_once(stage_path, "| R5-05 | Route Resolution Reads | VERIFYING |", "| R5-05 | Route Resolution Reads | DONE |")
replace_once(stage_path, "| R5-06 | Model Run Identity Reads | BLOCKED |", "| R5-06 | Model Run Identity Reads | READY |")
regex_once(
    stage_path,
    r"^- transient helper 已清理；R5-05 当前仍为 `VERIFYING`.*?$",
    "- R5-05 已按 fixed clean HEAD `879dd724e9bace058d86387b04e97cbdb67843b7` 通过 PR canonical run `31873912016` / job `94986637325`，并 squash merge 为 `d6b2d692dadbe546220eb0cb533bd9d4e207d230`；merged-stage canonical run `31875035071` / job `94989384122` 同样为 `SUCCESS`。R5-05 状态关闭为 `DONE`，R5-06 开放为 `READY`；本 closeout tree 仍需 canonical Public Platform CI 成功后才作为 R5-06 有效基线。",
)
stage_anchor = "\n## 兼容与限制\n"
stage_closeout = """
## R5-05 正式收口

- 最终 clean PR HEAD：`879dd724e9bace058d86387b04e97cbdb67843b7`；changed files 29，transient workflow 已全部清理。
- PR #30 final clean canonical run `31873912016` / Windows job `94986637325`：`SUCCESS`；artifact `9244488927`，大小 `13929542` 字节，SHA-256 `4cc15be2916477ea56a80080090c18b033c8eab1e6cfd2a4f4da948d1b24b754`。
- PR #30 使用 expected head `879dd724e9bace058d86387b04e97cbdb67843b7` squash merge；merge commit `d6b2d692dadbe546220eb0cb533bd9d4e207d230`，父提交精确为 R5-04 最终基线 `5a159e6ebcaeee0d25d833cd48042ffc9915a713`，tree 与 final clean PR tree 一致。
- merged-stage canonical run `31875035071` / Windows job `94989384122`：`SUCCESS`；artifact `9244773760`，大小 `13928853` 字节，SHA-256 `f342644fd91e617331060360d47513a076594aa379f1e9e9493b287e2df97b08`。
- R5-05 现关闭为 `DONE`，R5-06 开放为 `READY`。本次 closeout 文档提交形成的新 tree 仍必须通过 canonical Public Platform CI；只有该结果为 `SUCCESS`，此 tree 才可作为 R5-06 起始基线，本记录不预先宣称该结果。
"""
replace_once(stage_path, stage_anchor, stage_closeout + stage_anchor)

record_path = "docs/modular-rewrite/R05-competition-routing-persistence/R05-05-route-resolution-reads.md"
replace_once(record_path, "## 状态\n\n`VERIFYING`", "## 状态\n\n`DONE`")
regex_once(
    record_path,
    r"^R5-05 的 Route Resolution / Competition Context PostgreSQL persistence owner 已完成切换.*?$",
    "R5-05 的 Route Resolution / Competition Context PostgreSQL persistence owner 已完成切换，并已通过旧/new owner 同一 PostgreSQL 16 contract、R5 ownership、official Domain inventory、节点 hard gate、final clean PR canonical CI 与 merged-stage canonical CI；PR #30 已按固定 expected head squash merge。R5-05 状态关闭为 `DONE`，R5-06 开放为 `READY`。本 closeout 文档 tree 仍需一次 canonical Public Platform CI 成功后才是 R5-06 的有效起始基线。",
)
record_anchor = "\n## 兼容性与未变范围\n"
record_closeout = """
## Merge 与 closeout 证据

- final clean PR HEAD：`879dd724e9bace058d86387b04e97cbdb67843b7`；最终 permanent diff 为 29 个文件，`.github/workflows` 仅保留长期 `ci.yml`。
- final clean PR canonical Public Platform CI run `31873912016` / Windows job `94986637325`：`SUCCESS`。artifact `9244488927`，大小 `13929542` 字节，SHA-256 `4cc15be2916477ea56a80080090c18b033c8eab1e6cfd2a4f4da948d1b24b754`。
- PR #30 从 draft 转为 ready 后，使用 `expected_head_sha=879dd724e9bace058d86387b04e97cbdb67843b7` squash merge 成功；merge commit `d6b2d692dadbe546220eb0cb533bd9d4e207d230`，父提交为 `5a159e6ebcaeee0d25d833cd48042ffc9915a713`，tree 与 final clean PR HEAD 一致。
- merged-stage canonical Public Platform CI run `31875035071` / Windows job `94989384122`：`SUCCESS`。artifact `9244773760`，大小 `13928853` 字节，SHA-256 `f342644fd91e617331060360d47513a076594aa379f1e9e9493b287e2df97b08`。
- 首轮 PR canonical Clippy 失败与后续修复均已在上节如实保留；失败 tree 从未合并。Ubuntu 专项 gate 的 `glib-2.0` 环境阻塞也继续作为未完成验证记录保留。
- R5-05 关闭为 `DONE`，R5-06 开放为 `READY`。本 closeout 文档提交产生的新 tree 必须再通过 canonical Public Platform CI；只有该 CI 成功后，最终 stage HEAD 才可作为 R5-06 的有效起始基线，本记录不预先宣称该尚未执行的结果。
"""
replace_once(record_path, record_anchor, record_closeout + record_anchor)
regex_once(
    record_path,
    r"^R5-05 当前为 `VERIFYING`；只有 clean PR canonical CI、固定 HEAD merge、merged stage CI 与最终 closeout canonical CI 全部成功后，才可改为 `DONE` 并将 R5-06 改为 `READY`。$",
    "R5-05 已在 final clean PR canonical CI 与 merged-stage canonical CI 成功后关闭为 `DONE`，R5-06 已开放为 `READY`；closeout 文档 tree 仍需 canonical Public Platform CI 成功后才可作为 R5-06 的有效起始基线。",
)
