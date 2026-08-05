from __future__ import annotations

import json
import os
import re
import subprocess
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPO = os.environ.get('GITHUB_REPOSITORY', 'uniquenesssta/123')
TOKEN = os.environ.get('GITHUB_TOKEN', '')
EVIDENCE_HEAD = 'a3b61088abaf0c9f052ecab09e040ea77bd8d344'
RUN_ID = 31012168809
JOB_ID = 92326905405
ARTIFACT_ID = 8933800016
ARTIFACT_NAME = 'windows-automated-delivery-evidence-a3b61088abaf0c9f052ecab09e040ea77bd8d344'
ARTIFACT_SIZE = 14117539
ARTIFACT_DIGEST = 'sha256:4c28e5668b8b330cbab5b54516af1d70fe9f39c8299bb640da06a5b4442667f9'
ARTIFACT_EXPIRES = '2026-08-19T14:16:31Z'


def run(*args: str, capture: bool = False) -> str:
    result = subprocess.run(
        list(args), cwd=ROOT, check=True, text=True,
        encoding='utf-8', errors='strict',
        stdout=subprocess.PIPE if capture else None,
    )
    return result.stdout.strip() if capture else ''


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f'{label}: expected one literal match, got {count}')
    return text.replace(old, new, 1)


def api(path: str) -> dict:
    headers = {
        'Accept': 'application/vnd.github+json',
        'User-Agent': 'r1-03-close',
        'X-GitHub-Api-Version': '2022-11-28',
    }
    if TOKEN:
        headers['Authorization'] = f'Bearer {TOKEN}'
    req = urllib.request.Request(f'https://api.github.com/repos/{REPO}{path}', headers=headers)
    with urllib.request.urlopen(req, timeout=60) as response:
        return json.load(response)


def verify_evidence() -> None:
    run('git', 'merge-base', '--is-ancestor', EVIDENCE_HEAD, 'HEAD')
    staged_paths = set(run('git', 'diff', '--name-only', f'{EVIDENCE_HEAD}..HEAD', capture=True).splitlines())
    if staged_paths != {'scripts/r1-03-close.py', '.github/workflows/r1-03-close.yml'}:
        raise RuntimeError(f'unexpected staging paths before R1-03 close: {sorted(staged_paths)}')
    run_data = api(f'/actions/runs/{RUN_ID}')
    if run_data.get('head_sha') != EVIDENCE_HEAG or run_data.get('conclusion') != 'success':
        raise RuntimeError('R1-03 Windows workflow evidence is not successful for expected head')
    jobs = api(f'/actions/runs/{RUN_ID}/jobs?per_page=100').get('jobs', [])
    job = next((item for item in jobs if int(item.get('id', 0)) == JOB_ID), None)
    if job is None or job.get('conclusion') != 'success':
        raise RuntimeError('R1-03 Windows automated job is not successful')
    required = {
        'Verify R1 architecture boundaries',
        'Run Windows automated acceptance',
        'Upload validation evidence',
    }
    conclusions = {step.get('name'): step.get('conclusion') for step in job.get('steps', [])}
    missing = sorted(name for name in required if conclusions.get(name) != 'success')
    if missing:
        raise RuntimeError(f'R1-03 required steps not successful: {missing}')
    artifacts = api(f'/actions/runs/{RUN_ID}/artifacts?per_page=100').get('artifacts', [])
    artifact = next((item for item in artifacts if int(item.get('id', 0)) == ARTIFACT_ID), None)
    if artifact is None or artifact.get('expired'):
        raise RuntimeError('R1-03 evidence artifact missing or expired')
    if artifact.get('name') != ARTIFACT_NAME or int(artifact.get('size_in_bytes', 0)) != ARTIFACT_SIZE:
        raise RuntimeError('R1-03 evidence artifact metadata changed')
    if artifact.get('digest') != ARTIFACT_DIGEST:
        raise RuntimeError('R1-03 evidence artifact digest changed')


def update_root_readme() -> None:
    path = ROOT / 'README.md'
    text = path.read_text(encoding='utf-8')
    old = ('- R1-03 已建立 `src/bootstrap/` 浏览器组合根并切换 `index.html` 唯一入口；'
           '`src/main.ts` 仅保留既有业务实现并暴露受控生命周期，状态为 `VERIFYING`，'
           '正式 Windows Automated 通过前 R1-04 保持 `BLOCKED`。')
    new = ('- R1-03 已建立 `src/bootstrap/` 浏览器组合根并切换 `index.html` 唯一入口；'
           '`src/main.ts` 仅保留既有业务实现并暴露受控生命周期。Windows workflow run '
           f'`{RUN_ID}`、job `{JOB_ID}` 在提交 `{EVIDENCE_HEAD}` 上通过，artifact '
           f'`{ARTIFACT_ID}` 大小 `{ARTIFACT_SIZE}` 字节，SHA-256 为 '
           '`4c28e5668b8b330cbab5b54516af1d70fe9f39c8299bb640da06a5b4442667f9`；'
           '状态为 `DONE`，R1-04 已开放为 `READY`。')
    path.write_text(replace_once(text, old, new, 'root README R1-03 line'), encoding='utf-8', newline='\n')


def update_stage_index() -> None:
    path = ROOT / 'docs/modular-rewrite/R01-architecture-composition/README.md'
    text = path.read_text(encoding='utf-8')
    text = replace_once(
        text,
        '| R1-03 | 浏览器组合根 | VERIFYING | [`R01-03-浏览器组合根.md`](R01-03-浏览器组合根.md) | 浏览器组合根专项验证通过 | Windows Automated 待最终 HEAD 验证 |',
        f'| R1-03 | 浏览器组合根 | DONE | [`R01-03-浏览器组合根.md`](R01-03-浏览器组合根.md) | 浏览器组合根专项验证通过 | workflow run `{RUN_ID}` 通过 |',
        'stage R1-03 row',
    )
    text = replace_once(
        text,
        '| R1-04 | Tauri 组合根 | BLOCKED | 待创建 | 待执行 | 待执行 |',
        '| R1-04 | Tauri 组合根 | READY | 待创建 | 待执行 | 待执行 |',
        'stage R1-04 row',
    )
    old_section = '''## R1-03 当前结果

- 已建立浏览器唯一入口、应用创建、模块注册和 ApplicationHandle 生命周期模块；现有 Feature 实现未迁移。
- 浏览器入口及生命周期状态契约已切换，专项架构、TypeScript 和 Vite build 门禁通过。
- 当前状态为 `VERIFYING`；清理临时实施文件后的正式 Windows Automated 通过后才可开放 R1-04。

## 当前唯一可执行任务

`R1-03 浏览器组合根：完成最终 Windows 自动化门禁并关闭节点`
'''
    new_section = f'''## R1-03 完成结果

- 已建立浏览器唯一入口、应用创建、模块注册和 `ApplicationHandle` 生命周期模块；现有 Feature 实现未迁移。
- 浏览器入口及生命周期状态契约已切换，专项架构、TypeScript、Vite build 与正式 Windows Automated 均通过。
- workflow run `{RUN_ID}`、Windows job `{JOB_ID}` 在提交 `{EVIDENCE_HEAD}` 上通过；artifact `{ARTIFACT_ID}` 的 SHA-256 为 `4c28e5668b8b330cbab5b54516af1d70fe9f39c8299bb640da06a5b44442667f9`。
- 真实 PostgreSQL、Windows Full 和用户本机 Windows 实机验收继续保留到最终统一验收。
- R1-03 状态为 `DONE`；R1-04 开放为 `READY`，R1-05 继续 `BLOCKED`。

## 当前唯一可执行任务

`R1-04 Tauri 组合根`
'''
    path.write_text(replace_once(text, old_section, new_section, 'stage R1-03 result section'), encoding='utf-8', newline='\n')


def update_task_doc() -> None:
    path = ROOT / 'docs/modular-rewrite/R01-architecture-composition/R01-03-浏览器组合根.md'
    text = path.read_text(encoding='utf-8')
    text = replace_once(text, '- 任务状态：`VERIFYING`', '- 任务状态：`DONE`', 'task status')
    old = '''# 7. 待执行验证与状态

- 清理临时实施工作流后的最终 `new-B` HEAD 尚需通过正式 Windows `Public Platform CI` 全链路，因此当前状态保持 `VERIFYING`。
- 正式门禁必须重新覆盖架构、完整前端、Rust fmt/Clippy/workspace tests、Tauri Windows release、客户端启动和运行日志扫描。
- 真实 PostgreSQL、Windows Full 与用户本机实机验收仍按阶段规则保留到最终统一验收。
'''
    new = f'''## 7. Windows 自动化门禁与最终状态

- 候选提交：`{EVIDENCE_HEAD}`。
- `Public Platform CI` workflow run `{RUN_ID}`、Windows job `{JOB_ID}` 均为 `success`。
- 架构边界、完整前端、Rust fmt/Clippy/workspace tests、Tauri Windows release、release 客户端启动和运行日志扫描全部通过。
- 证据 artifact `{ARTIFACT_ID}`：`{ARTIFACT_NAME}`，大小 `{ARTIFACT_SIZE}` 字节，SHA-256 为 `4c28e5668b8b330cbab5b54516af1d70fe9f39c8299bb640da06a5b4442667f9`，保留至 `{ARTIFACT_EXPIRES}`。
- 真实 PostgreSQL、Windows Full 与用户本机实机验收仍按阶段规则保留到最终统一验收。
- R1-03 状态关闭为 `DONE`；R1-04 开放为 `READY`，R1-05 继续 `BLOCKED`。
'''
    path.write_text(replace_once(text, old, new, 'task final verification section'), encoding='utf-8', newline='\n')


def main() -> None:
    if run('git', 'status', '--porcelain', capture=True):
        raise RuntimeError('working tree must be clean')
    verify_evidence()
    update_root_readme()
    update_stage_index()
    update_task_doc()
    (ROOT / 'scripts/r1-03-close.py').unlink()
    (ROOT / '.github/workflows/r1-03-close.yml').unlink()
    npm = 'npm.cmd' if os.name == 'nt' else 'npm'
    run(npm, 'run', 'verify:architecture')
    run(npm, 'run', 'verify:frontend')
    changed = set(run('git', '-c', 'core.quotepath=false', 'diff', '--name-only', '--no-renames', capture=True).splitlines())
    expected_suffixes = {
        'README.md',
        'docs/modular-rewrite/R01-architecture-composition/README.md',
        'docs/modular-rewrite/R01-architecture-composition/R01-03-浏览器组合根.md',
        'scripts/r1-03-close.py',
        '.github/workflows/r1-03-close.yml',
    }
    if changed != expected_suffixes:
        raise RuntimeError(f'unexpected close paths: {sorted(changed)}')
    run('git', 'config', 'user.name', 'github-actions[bot]')
    run('git', 'config', 'user.email', '41898282+github-actions[bot]@users.noreply.github.com')
    run('git', 'add', '--all')
    run('git', 'commit', '-m', 'docs(r1): close R1-03 browser composition root')
    run('git', 'push', 'origin', 'HEAD:new-B')


if __name__ == '__main__':
    main()
