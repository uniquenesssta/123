import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
const root=process.cwd(), base='8db5f460f25887edac6a6bf95932de6c46164e9a';
const run=(c,a,capture=false)=>execFileSync(c,a,{cwd:root,encoding:'utf8',stdio:capture?['ignore','pipe','inherit']:'inherit'});
const read=p=>readFileSync(join(root,p),'utf8');
const write=(p,s)=>{const a=join(root,p);mkdirSync(dirname(a),{recursive:true});writeFileSync(a,s.replace(/\r\n/g,'\n'),'utf8');};
run('git',['merge-base','--is-ancestor',base,'HEAD']);
const staged=new Set(run('git',['diff','--name-only',base+'..HEAD'],true).trim().split(/\r?\n/).filter(Boolean));
if(staged.size!==2||!staged.has('scripts/r1-04-apply.mjs')||!staged.has('.github/workflows/r1-04-apply.yml'))throw Error('unexpected R1-04 staging history');
if(run('git',['status','--porcelain'],true).trim())throw Error('working tree is not clean');
const lib=read('src-tauri/src/lib.rs');
const m=lib.match(/\.invoke_handler\(tauri::generate_handler!\[([\s\S]*?)\]\)\n\s*\.run/);if(!m)throw Error('handler not found');
const body=m[1].trimEnd(), names=[...body.matchAll(/commands::([a-z0-9_]+)/g)].map(x=>x[1]);
if(names.length!==171||new Set(names).size!==171)throw Error('command count changed');
write('src-tauri/src/bootstrap/mod.rs','mod application;\nmod command_registry;\nmod error;\nmod state;\n\npub(crate) use state::AppState;\n\npub(crate) fn run() { application::run(); }\n');
write('src-tauri/src/bootstrap/error.rs','use std::io;\n\npub(crate) const STARTUP_ERROR_MESSAGE: &str = "足球赛事模型平台启动失败";\n\npub(crate) fn io_error(error: impl ToString) -> io::Error { io::Error::other(error.to_string()) }\n\npub(crate) fn expect_startup(result: tauri::Result<()>) { result.expect(STARTUP_ERROR_MESSAGE); }\n');
write('src-tauri/src/bootstrap/state.rs',`use super::error;
use crate::{issue_log::IssueLogStore, openai_profiles::OpenAiProfileStore, runtime_log::RuntimeLogStore, workspace_state::WorkspaceStateStore};
use football_application::ApplicationService;
use football_research_gateway::CancellationToken;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tauri::Manager;
use tokio::sync::Mutex;

pub struct AppState {
    pub service: Arc<ApplicationService>,
    pub config_path: PathBuf,
    pub issue_log: Arc<IssueLogStore>,
    pub runtime_log: Arc<RuntimeLogStore>,
    pub openai_profiles: Arc<OpenAiProfileStore>,
    pub workspace_state: Arc<WorkspaceStateStore>,
    pub api_workspace_requests: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

pub(crate) fn install<R: tauri::Runtime>(app: &mut tauri::App<R>) -> Result<(), std::io::Error> {
    let config_dir = app.path().app_config_dir().map_err(error::io_error)?;
    let runtime_log = Arc::new(RuntimeLogStore::discover(&config_dir));
    let _ = runtime_log.record("info", "application", "application_started", None, serde_json::json!({
        "config_directory": config_dir.display().to_string(),
        "runtime_log_path": runtime_log.path().display().to_string(),
        "runtime_log_relative_path": runtime_log.relative_display_path(),
        "runtime_log_relative_directory": r".\\logs",
        "runtime_log_session_id": runtime_log.session_id(),
    }));
    app.manage(AppState {
        service: Arc::new(ApplicationService::new()),
        config_path: config_dir.join("database.json"),
        issue_log: Arc::new(IssueLogStore::new(config_dir.join("issue-log.json"))),
        runtime_log,
        openai_profiles: Arc::new(OpenAiProfileStore::new(config_dir.join("openai-profiles.json"))),
        workspace_state: Arc::new(WorkspaceStateStore::new(config_dir.join("workspace-state.json"))),
        api_workspace_requests: Arc::new(Mutex::new(HashMap::new())),
    });
    Ok(())
}
`);
write('src-tauri/src/bootstrap/command_registry.rs',`use crate::commands;

pub(crate) fn register<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
${body}
    ])
}
`);
write('src-tauri/src/bootstrap/application.rs',`use super::{command_registry, error, state};

pub(crate) fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| -> Result<(), Box<dyn std::error::Error>> { state::install(app)?; Ok(()) });
    error::expect_startup(command_registry::register(builder).run(tauri::generate_context!()));
}
`);
write('src-tauri/src/lib.rs','mod bootstrap;\nmod commands;\nmod config;\nmod file_store;\nmod issue_log;\nmod openai_profiles;\nmod runtime_log;\nmod workspace_state;\n\n#[cfg_attr(mobile, tauri::mobile_entry_point)]\npub fn run() { bootstrap::run(); }\n');
let commands=read('src-tauri/src/commands.rs');
commands=commands.replace(/\nuse crate::issue_log::IssueLogStore;[\s\S]*?pub struct AppState \{[\s\S]*?\n\}\n/,'\npub(crate) use crate::bootstrap::AppState;\n');
if(!commands.includes('pub(crate) use crate::bootstrap::AppState;'))throw Error('AppState migration failed');
write('src-tauri/src/commands.rs',commands);
const cc=JSON.parse(read('architecture/command-contract.json'));cc.sources.tauri_registration.file='src-tauri/src/bootstrap/command_registry.rs';write('architecture/command-contract.json',JSON.stringify(cc,null,2)+'\n');
const mb=JSON.parse(read('architecture/module-boundaries.json'));mb.rust.tauri_host={public_entry:'src-tauri/src/lib.rs::run',composition_entry:'src-tauri/src/bootstrap/mod.rs::run',builder:'src-tauri/src/bootstrap/application.rs',state:'src-tauri/src/bootstrap/state.rs::AppState',command_registry:'src-tauri/src/bootstrap/command_registry.rs',command_root:'src-tauri/src/commands.rs',target_task:null};mb.tauri_commands.registration_owner='src-tauri/src/bootstrap/command_registry.rs';write('architecture/module-boundaries.json',JSON.stringify(mb,null,2)+'\n');
const so=JSON.parse(read('architecture/state-ownership.json')), states=new Map(so.states.map(x=>[x.id,x]));
Object.assign(states.get('tauri.app-state'),{owner:'src-tauri/src/bootstrap/state.rs::AppState',writers:['src-tauri/src/bootstrap/state.rs::install'],transition:null});
states.get('tauri.database-config-path').writers=['src-tauri/src/bootstrap/state.rs::install'];
write('architecture/state-ownership.json',JSON.stringify(so,null,2)+'\n');
write('scripts/verify-command-contract.mjs',read('scripts/verify-command-contract.mjs').replace('src-tauri/src/lib.rs','src-tauri/src/bootstrap/command_registry.rs'));
let stage=read('docs/modular-rewrite/R01-architecture-composition/README.md');stage=stage.replace('| R1-04 | Tauri 组合根 | READY | 待创建 | 待执行 | 待执行 |','| R1-04 | Tauri 组合根 | VERIFYING | 待创建 | 最小门禁执行中 | Windows Automated 待验证 |').replace('`R1-04 Tauri 组合根`','`R1-04 Tauri 组合根：完成正式 Windows 自动化门禁并关闭节点`');write('docs/modular-rewrite/R01-architecture-composition/README.md',stage);
let readme=read('README.md'), anchor='- R1-03 已建立 `src/bootstrap/` 浏览器组合根并切换 `index.html` 唯一入口；', at=readme.indexOf(anchor), end=readme.indexOf('\n',at);if(at<0)throw Error('README anchor missing');readme=readme.slice(0,end+1)+'- R1-04 已建立 `src-tauri/src/bootstrap/` Tauri 组合根，拆分 Builder、全局状态、171 条命令注册和启动错误映射；当前状态为 `VERIFYING`，正式 Windows Automated 通过前 R1-05 保持 `BLOCKED`。\n'+readme.slice(end+1);write('README.md',readme);
run('cargo',['fmt','--all']);run('npm',['run','verify:architecture']);run(process.execPath,['scripts/verify_command_contract.mjs']);run(process.execPath,['scripts/verify-command-contract.mjs']);run(process.execPath,['scripts/verify_protected_assets.mjs']);run('cargo',['fmt','--all','--','--check']);run('cargo',['check','--locked','-p','football-match-model-desktop']);run('git',['diff','--check']);
run('git',['config','user.name','github-actions[bot]']);run('git',['config','user.email','41898282+github-actions[bot]@users.noreply.github.com']);run('git',['add','README.md','architecture','docs/modular-rewrite/R01-architecture-composition/README.md','scripts/verify-command-contract.mjs','src-tauri/src/bootstrap','src-tauri/src/commands.rs','src-tauri/src/lib.rs']);run('git',['commit','-m','refactor(r1): establish Tauri composition root']);run('git',['push','origin','HEAD:new-B']);