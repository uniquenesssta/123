# R12-09 Files、Storage、Diagnostics、Clipboard 与 Timing

状态：`VERIFYING`。基线`c16f2c07419b5f637f0e2694e983360ff7c31747`；唯一分支`rewrite/r12-frontend-foundation-ui-system`。用户2026-10-25授权“08ci已通过 开始09”。01～08 DONE，10～19 BLOCKED，不轮询CI，不开始10。

## 前置核验与范围

08精确提交`c16f2c07419b5f637f0e2694e983360ff7c31747`的[Windows run40137847917](https://github.com/uniquenesssta/123/actions/runs/40137847917)、job121003070447、attempt1全部SUCCESS。2026-10-25交接时一次性核对head/分支、每个步骤、完整308,239字符job日志与下载产物，没有轮询。artifact12578236452未过期，ZIP15,874,690字节，SHA-256 `c3a01fcba21572f180010352b4866f202c7127d1cc7fadfb1adbefe7ee3b05bc`与API摘要及上传日志一致。原91步骤前端/24项Transport/1,104项Shell/工作区、136项告警/17视口、Rust fmt/Clippy/workspace tests、release/MSI/NSIS与真实启动均通过。报告`windows-acceptance-20261024-221745.json`为startup PASS：7条有效记录、0无效行、0运行错误；会话`308155e9-e864-40a5-81d3-f33a6e8778c6`内bootstrap、read_workspace_state、save_workspace_state三组trace开始/完成逐项配对。MSI8,384,512字节，NSIS5,916,019字节；ZIP保留在仓库上一级`R12-08-windows-evidence.zip`。startup不替代Windows Full/真实PG最终验收，原待验项继续保留。

依据R12任务书、总纲执行订正、R00文件/171命令/类型清单及实际调用方。核对已安装`@tauri-apps/plugin-dialog 2.7.1`的`index.d.ts`：open/save返回路径或null，directory/multiple/recursive选项按原生声明传递。没有UI、样式、图标、Figma资源或业务页面视觉变化；沿05～07已接入设计，不因09内部能力创建新画板。无新生产依赖、runner、workflow、数据库专项入口或Rust改动。

## 职责、入口与状态

| 模块 | 唯一职责和真实调用 |
|---|---|
| platform/files | fileDialogAdapter唯一导入原生plugin-dialog；openFile/saveFile/selectDirectory只选择路径，原api九种文件选择方法传递原过滤器、默认路径，命令继续读写内容 |
| platform/storage | PreferenceStore集中local/session Storage副作用，延迟解析浏览器存储；应用保留自己的键与领域归一化，SessionPersistence继续拥有主题回退策略 |
| storage/storageKey | 统一原命名空间，六个原键字符串保持逐字不变，不建跨Feature键/值store |
| storage/versionedJsonStore | 显式启用的版本信封、校验与迁移；先解码验证再一次setItem，未来版本/损坏/迁移失败保留原字节，原六键不转信封、不自动迁移 |
| platform/diagnostics | 原traceSummary迁为唯一logSummary；有界、非抛出、脱敏投影复用08逻辑，Transport仅持有特定命令生命周期与结果元信息 |
| platform/clipboard | ClipboardAdapter唯一浏览器剪贴板入口，保持原生receiver/文本/权限拒绝，实际复制成功后原调用方才提示成功 |
| platform/timing | wallTime/monotonicTime区分时间戳与耗时；RequestSequence由各调用方独立实例化；DebouncedTask持有单一防抖回调，native timer仍由已有EventScope释放 |

所有index.ts只显式导出。Platform不依赖App/Feature/UI，Transport通过平台公开出口使用诊断和时钟。原main直接Storage/clipboard、api直接plugin-dialog、旧日志摘要路径实现已移除；没有临时转发或第二状态缓存。browser.local-preferences owner更新到PreferenceStore，键/业务默认值仍归各调用方。新增browser.debounce和browser.request-sequence两个逐实例owner，合计31项，历史last_updated_task保持原R3-02兼容值。

## 行为与兼容

原九种文件对话框选项、路径原字节、取消null和错误拒绝保持；新增目录选择固定multiple=false/directory=true/recursive=false，不引入任何目录遍历或文件删除。文件选择与剪贴板适配器只交付结果，不拥有UI状态、业务重试或原生撤销；取消对话框返回null，业务调用仍经原runBusy/既有调用方生命周期。没有声称能够撤销已执行的OS剪贴板写入或原生对话框。

主题dark/light原字符串、分析包ID原字符串、重置标记session原`1`、球队/球员筛选与导航上下文原plain JSON保持，字段归一化和游标排除仍在业务owner；损坏JSON只处理明确键，合法非object数据仍原样保留并回退null。PreferenceStore原严格写入保留配额/访问错误；Session继续捕获并使用原默认主题。版本化能力只给显式调用者使用，没有改变任何现有持久格式/Schema或数据库迁移。

工作区250ms和API示例320ms防抖时长保持；替换、手动保存、clear、stop/destroy使旧回调失效。生命周期AsyncScope继续拥有AbortController，RequestSequence仅统一单调请求ID；双方球队/单边名单保留独立渠道、球队ID复核和迟到结果丢弃。UI/业务状态不进入Timing。Toast仍按单调时钟保留原3200ms暂停/恢复。Transport日志phase、JSONL字段、开始/终结顺序、Issue例外、敏感正文摘要上限和公开错误不变。

171命令名/注册顺序、全部业务invoke方法、参数/DTO/返回、src/types.ts、Rust、Schema/迁移/配置、模型资产和UI CSS保持。诊断通过纯迁移保持原逻辑，不对实际业务参数/返回做脱敏或修改。

## 验证

- 原运行诊断入口：新增25项生产Platform行为测试，原24项Transport继续通过；九种原API文件选项逐项验证。
- TypeScript --noEmit通过；原24生命周期、18会话行为通过。
- 完整verify:architecture通过，18 Feature、11 Rust crate、31状态owner；cargo fmt --all -- --check通过。
- npm run verify:frontend完整通过：89个原验证脚本＋TypeScript＋Vite共91步骤；25项Platform＋24项Transport、原24生命周期/18会话/21路由/16工作区状态竞态、1,104项Shell/工作区、136项告警和17视口全部通过。最终日志为仓库上一级r12-09-frontend-final-2.log。
- 原命令两套检查器均通过171命令/顺序/冻结Hash。独立AST核对179个原API方法：170个非文件选择方法体逐字不变，九种对话框方法原选项由生产模块测试逐项比对。原traceSummary与新logSummary规范化文本完全一致。
- 18项模型保护资产聚合SHA-256仍d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57；受保护源码、类型/Schema/迁移、依赖锁、UI CSS和图标无差异。Vite150模块，main-CMIfZmKw.css保持，主JS607.28kB（gzip172.01kB），既有500kB提示未放宽。
- 最终33条非Markdown路径差异（含摘要迁移旧路径）规范化SHA-256为`d27bf2d040b96ff87dc919b24d2a2e7a7da48ef770e62d2c9e1e90f33a48f321`；验证后复核一致。

失败尝试保留：r12-09-frontend-1.log exit13，受限环境原浏览器启动unsettled top-level await；授权浏览器环境的r12-09-warning-recheck.log原136项复查通过。第二次全量r12-09-frontend-final.log在详情历史失败，根因为原拼接源码夹具缺少迁移后的RequestSequence/preferenceKey依赖；同步补齐详情/阵容两处真实生产依赖，没有改业务实现、测试断言/像素阈值或PNG基线。r12-09-shell-recheck.log原1,104项及阵容滚动复查通过，之后完整r12-09-frontend-final-2.log exit0。所有测试从首命令启用保留产物预载，未批量删除测试目录。

## 待验、限制与回退

09自身Rust Clippy/workspace tests、Windows release/MSI/NSIS、实际启动和日志配对待本节点提交后原CI验证；08通过不算09通过。实际OS文件对话框/剪贴板权限和业务导入导出完整链路需Windows交互环境核验，注入native边界测试不算真实OS交互PASS。真实PG/XLSX/历史保护四项/不可变账本并发回滚/Windows Full/私有P4/P7固定回归沿已有最终封包新库安排保留待验。

回退点c16f2c0；用受控Git回退本节点。原偏好未转换格式，无新增持久Schema。原Vite主chunk超500kB提示保持。10尚未开始。

## 实际文件清单

按Git重命名识别共38项文件变更、39个物理路径：15新增、22修改、1次纯移动、无额外删除。未识别重命名时等价16 A/22 M/1 D；唯一旧路径是下列纯迁移源。

### 新增

- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-09-files-storage-diagnostics-clipboard-and-timing.md`
- `scripts/platform/verifyPlatformBehavior.mjs`
- `src/platform/clipboard/clipboardAdapter.ts`
- `src/platform/clipboard/index.ts`
- `src/platform/diagnostics/index.ts`
- `src/platform/files/fileDialogAdapter.ts`
- `src/platform/files/index.ts`
- `src/platform/storage/index.ts`
- `src/platform/storage/preferenceStore.ts`
- `src/platform/storage/storageKey.ts`
- `src/platform/storage/versionedJsonStore.ts`
- `src/platform/timing/clock.ts`
- `src/platform/timing/debouncedTask.ts`
- `src/platform/timing/index.ts`
- `src/platform/timing/requestSequence.ts`

### 修改

- `README.md`
- `architecture/module-boundaries.json`
- `architecture/state-ownership.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-08-tauri-transport.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md`
- `scripts/architecture/verifyProtectedImports.mjs`
- `scripts/tauri/transportSources.mjs`
- `scripts/verify-analysis-history-workflow.mjs`
- `scripts/verify-api-runtime-diagnostics.mjs`
- `scripts/verify-inline-workspace-and-resource-scroll.mjs`
- `scripts/verify-lineup-scroll-continuity.mjs`
- `scripts/verify-match-lineup-chain.mjs`
- `src/api/client.ts`
- `src/app/lifecycle/asyncScope.ts`
- `src/app/session/sessionPersistence.ts`
- `src/app/workspace/workspacePersistence.ts`
- `src/main.ts`
- `src/platform/tauri/invokeResponse.ts`
- `src/platform/tauri/invokeTrace.ts`
- `src/ui/feedback/toast/toastController.ts`

### 移动/重命名

- `src/platform/tauri/traceSummary.ts` → `src/platform/diagnostics/logSummary.ts`；实现逐字保持，唯一旧路径退出，无兼容转发。

### 其他删除

无。
