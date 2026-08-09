from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERIFIER = ROOT / "scripts/verify-research-service.mjs"
text = VERIFIER.read_text(encoding="utf-8")

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

VERIFIER.write_text(text, encoding="utf-8", newline="\n")
print("AT5 verifier patched: successful Research finalization stays owned by p4_worker")
