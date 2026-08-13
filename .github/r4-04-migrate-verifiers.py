from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SIMPLE_TARGET_FILES = [
    "scripts/verify-ai-workspace-service.mjs",
    "scripts/verify-prediction-service.mjs",
    "scripts/verify-review-service.mjs",
    "scripts/verify-teams-players-service.mjs",
    "scripts/verify-research-service.mjs",
    "scripts/verify-lineups-service.mjs",
    "scripts/verify-release-service.mjs",
    "scripts/verify-exchange-service.mjs",
    "scripts/verify-analytics-service.mjs",
]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected 1 exact match, found {count}")
    return text.replace(old, new, 1)


for relative in SIMPLE_TARGET_FILES:
    text = read(relative)
    count = text.count("for ActiveDatabase")
    if count == 0:
        raise RuntimeError(f"{relative}: expected at least one ActiveDatabase adapter target")
    text = text.replace("for ActiveDatabase", "for PersistenceStore")
    text = text.replace("ActiveDatabase 适配", "PersistenceStore 适配")
    text = text.replace("ActiveDatabase 未实现", "PersistenceStore 未实现")
    write(relative, text)

composition_path = "scripts/verify-application-composition.mjs"
composition = read(composition_path)
composition = replace_once(
    composition,
    'check(\n  portRegistry.includes("pub(crate) struct ActiveDatabase"),\n  "活动数据库状态未归属端口模块",\n);',
    'check(\n  portRegistry.includes("use football_persistence_postgres::register_adapters;"),\n  "PostgreSQL adapter 注册入口未归属端口注册模块",\n);\ncheck(!portRegistry.includes("struct ActiveDatabase"), "端口注册模块仍保留 ActiveDatabase wrapper");',
    "application composition registration owner",
)
composition = replace_once(
    composition,
    'databaseService.includes("pub(crate) session: RwLock<Option<ActiveDatabase>>"),',
    'databaseService.includes("pub(crate) session: RwLock<Option<PersistenceStore>>"),',
    "application composition database session type",
)
write(composition_path, composition)

competition_path = "scripts/verify-competition-rules-service.mjs"
competition = read(competition_path)
competition = replace_once(
    competition,
    'const registry = read("crates/application/src/composition/port_registry.rs");',
    'const registry = read("crates/application/src/composition/port_registry.rs");\nconst competitionAdapter = read("crates/application/src/composition/adapters/competition.rs");\nconst rulesAdapter = read("crates/application/src/composition/adapters/rules.rs");\nconst adapterText = `${competitionAdapter}\\n${rulesAdapter}`;',
    "competition rules adapter owners",
)
competition = competition.replace("for ActiveDatabase", "for PersistenceStore")
competition = replace_once(
    competition,
    '  check(registry.includes(implementation), `组合根适配器缺少：${implementation}`);',
    '  check(adapterText.includes(implementation), `组合根具名适配器缺少：${implementation}`);',
    "competition rules implementation location",
)
needle = 'for (const implementation of [\n  "impl CompetitionHierarchyPort for PersistenceStore",\n  "impl RulePackagePort for PersistenceStore",\n  "impl RuleRoutingPort for PersistenceStore",\n]) {\n  check(adapterText.includes(implementation), `组合根具名适配器缺少：${implementation}`);\n}\n'
if competition.count(needle) != 1:
    raise RuntimeError("competition/rules migrated implementation block mismatch")
competition = competition.replace(
    needle,
    needle + 'check(!registry.includes("impl CompetitionHierarchyPort") && !registry.includes("impl RulePackagePort") && !registry.includes("impl RuleRoutingPort"), "Competition/Rules Port 实现重新堆叠到 port_registry.rs");\n',
    1,
)
write(competition_path, competition)

positive_remaining = []
for path in (ROOT / "scripts").glob("verify-*.mjs"):
    # R4-04's own verifier intentionally contains negative assertions banning the old
    # target. Those are required protection, not stale positive owner assumptions.
    if path.name == "verify-persistence-adapters.mjs":
        continue
    text = path.read_text(encoding="utf-8")
    for line_no, line in enumerate(text.splitlines(), 1):
        if (
            "for ActiveDatabase" in line
            or "session: RwLock<Option<ActiveDatabase>>" in line
            or "pub(crate) struct ActiveDatabase" in line
        ):
            positive_remaining.append(f"{path.name}:{line_no}:{line.strip()}")
if positive_remaining:
    raise RuntimeError("stale positive ActiveDatabase verifier assumptions remain:\n" + "\n".join(positive_remaining))

print("R4-04 stale verifier migration complete: 11 owner contracts moved from ActiveDatabase to PersistenceStore/named adapters; intentional negative legacy-target bans and service concrete-dependency bans retained")
