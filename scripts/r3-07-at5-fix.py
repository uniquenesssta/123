from pathlib import Path

GENERATOR = Path(__file__).resolve().with_name("r3-07-at5.py")
text = GENERATOR.read_text(encoding="utf-8")

anchor = '''replace_once(
    "scripts/verify-research-service.mjs",
    '''check(facade.includes("fn finalize_p4_research_task"), "Application Research facade 缺少人工裁决复用的 Research 收口入口");
''',
'''

replacement = '''replace_once(
    "scripts/verify-research-service.mjs",
    '''check(service.includes("fn finalize_successful_research"), "ResearchService 缺少 Research 成功收口职责");
check(facade.includes("fn finalize_p4_research_task"), "Application Research facade 缺少人工裁决复用的 Research 收口入口");
''',
'''

if text.count(anchor) != 1:
    raise RuntimeError("AT5 verifier fix anchor mismatch")
text = text.replace(anchor, replacement, 1)

old_new_block = '''    '''check(service.includes("fn resolve_p4_conflict"), "ResearchService 缺少人工冲突裁决职责");
check(facade.includes("pub async fn resolve_p4_conflict"), "Application Research facade 缺少公共人工冲突裁决入口");
check(facade.includes("ApplicationResult<P4TaskWorkspace>"), "resolve_p4_conflict 返回契约发生变化");
check(!service.includes("fn finalize_successful_research"), "AT5 后仍残留仅供旧 workbench 的 ResearchService finalizer 转发");
check(!facade.includes("fn finalize_p4_research_task"), "AT5 后仍残留仅供旧 workbench 的 facade finalizer 转发");
''',
'''

new_new_block = '''    '''check(service.includes("fn resolve_p4_conflict"), "ResearchService 缺少人工冲突裁决职责");
check(facade.includes("pub async fn resolve_p4_conflict"), "Application Research facade 缺少公共人工冲突裁决入口");
check(facade.includes("ApplicationResult<P4TaskWorkspace>"), "resolve_p4_conflict 返回契约发生变化");
check(!service.includes("fn finalize_successful_research"), "AT5 后仍残留仅供旧 workbench 的 ResearchService finalizer 转发");
check(!facade.includes("fn finalize_p4_research_task"), "AT5 后仍残留仅供旧 workbench 的 facade finalizer 转发");
check(p4Transitions.includes("fn finalize_successful_research"), "P4 Research worker 丢失可复用的成功收口职责");
''',
'''

if text.count(old_new_block) != 1:
    raise RuntimeError("AT5 verifier replacement block mismatch")
text = text.replace(old_new_block, new_new_block, 1)

GENERATOR.write_text(text, encoding="utf-8", newline="\n")
print("AT5 generator patched: Research success finalization is verified at p4_worker owner")
