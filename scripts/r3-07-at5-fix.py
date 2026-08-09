from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

research_verifier = ROOT / "scripts/verify-research-service.mjs"
text = research_verifier.read_text(encoding="utf-8")

obsolete = 'check(service.includes("fn finalize_successful_research"), "ResearchService 缺少 Research 成功收口职责");\n'
if text.count(obsolete) != 1:
    raise RuntimeError("AT5 verifier obsolete finalizer assertion mismatch")
text = text.replace(obsolete, "", 1)

anchor = 'check(!facade.includes("fn finalize_p4_research_task"), "AT5 后仍残留仅供旧 workbench 的 facade finalizer 转发");\n'
if text.count(anchor) != 1:
    raise RuntimeError("AT5 verifier worker-owner assertion anchor mismatch")
text = text.replace(
    anchor,
    anchor
    + 'check(p4Transitions.includes("fn finalize_successful_research"), "P4 Research worker 丢失可复用的成功收口职责");\n',
    1,
)
research_verifier.write_text(text, encoding="utf-8", newline="\n")

prediction_verifier = ROOT / "scripts/verify-prediction-service.mjs"
text = prediction_verifier.read_text(encoding="utf-8")
old_sources = 'const facade=read("crates/application/src/services/prediction/facade.rs"); const service=read("crates/application/src/services/prediction/service.rs"); const adapter=read("crates/application/src/composition/adapters/prediction.rs"); const ports=read("crates/application/src/ports/prediction/mod.rs"); const useCases=read("crates/application/src/use_cases/prediction/mod.rs"); const snapshotUseCase=read("crates/application/src/use_cases/prediction/p4_snapshot/mod.rs"); const legacyOrchestration=read("crates/application/src/p4_orchestration.rs"); const legacyWorkbench=read("crates/application/src/p4_workbench.rs"); const packageJson=JSON.parse(read("package.json")); const frontend=read("scripts/verify-frontend.mjs");\n'
new_sources = 'const facade=read("crates/application/src/services/prediction/facade.rs"); const service=read("crates/application/src/services/prediction/service.rs"); const adapter=read("crates/application/src/composition/adapters/prediction.rs"); const ports=read("crates/application/src/ports/prediction/mod.rs"); const useCases=read("crates/application/src/use_cases/prediction/mod.rs"); const snapshotUseCase=read("crates/application/src/use_cases/prediction/p4_snapshot/mod.rs"); const legacyOrchestration=read("crates/application/src/p4_orchestration.rs"); const researchFacade=read("crates/application/src/services/research/facade.rs"); const packageJson=JSON.parse(read("package.json")); const frontend=read("scripts/verify-frontend.mjs");\n'
if text.count(old_sources) != 1:
    raise RuntimeError("AT5 Prediction verifier source anchor mismatch")
text = text.replace(old_sources, new_sources, 1)
old_boundary = 'for(const method of ["read_p4_match_workspace","read_p4_task_workspace"]) check(!legacyWorkbench.includes(`fn ${method}`),`p4_workbench.rs 仍持有只读 Prediction workspace 职责：${method}`); check(legacyWorkbench.includes("fn resolve_p4_conflict"),"R3-07 冲突写入职责被意外移除");\n'
new_boundary = 'check(!existsSync(join(root,"crates/application/src/p4_workbench.rs")),"AT5 后旧 p4_workbench.rs 仍残留跨 Prediction/Research 混合 owner"); check(researchFacade.includes("fn resolve_p4_conflict"),"R3-07 冲突写入职责未迁入 Research facade");\n'
if text.count(old_boundary) != 1:
    raise RuntimeError("AT5 Prediction verifier workbench boundary anchor mismatch")
text = text.replace(old_boundary, new_boundary, 1)
prediction_verifier.write_text(text, encoding="utf-8", newline="\n")

decision = ROOT / "crates/application/src/use_cases/research/p4_manual_conflict/decision.rs"
text = decision.read_text(encoding="utf-8")
for old, new, label in [
    ('            ""select_evidence""\n', '            r#""select_evidence""#\n', "select_evidence serialization assertion"),
    ('            ""accept_unknown""\n', '            r#""accept_unknown""#\n', "accept_unknown serialization assertion"),
]:
    if text.count(old) != 1:
        raise RuntimeError(f"AT5 decision test anchor mismatch: {label}")
    text = text.replace(old, new, 1)
decision.write_text(text, encoding="utf-8", newline="\n")

print("AT5 patches applied: verifier ownership and exact serialization assertions")
