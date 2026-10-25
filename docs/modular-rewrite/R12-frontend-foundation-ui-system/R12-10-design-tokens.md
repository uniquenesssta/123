# R12-10 Design Tokens

状态：`VERIFYING`。基线`98dc43eef5c34fcf7d15bb6201290a710b6f0a37`；唯一分支`rewrite/r12-frontend-foundation-ui-system`。用户2026-10-25授权“收尾09开始10”。01～09 DONE，11～19 BLOCKED；不轮询CI，不开始11。

## 前置核验

09精确提交`98dc43eef5c34fcf7d15bb6201290a710b6f0a37`的[Windows run40145377194](https://github.com/uniquenesssta/123/actions/runs/40145377194)、job121025458733、attempt1全部SUCCESS。2026-10-25交接时一次性核对head/分支、全部步骤、完整308,815字符job日志与下载产物，没有轮询。artifact12580543223未过期，ZIP15,883,914字节，SHA-256 `dec87ab81e3410641b600ef7ff3dbff845ec3ba9f3ce38101f3033d2adc48132`与API摘要及上传日志一致。91步骤前端、25项Platform/24项Transport、1,104项Shell/工作区、136项告警及17视口、Rust fmt/Clippy/workspace tests、release/MSI/NSIS、真实启动与日志通过。报告`windows-acceptance-20261025-000735.json`为startup PASS：7条有效记录、0无效行和运行错误；会话`48cd1943-b96a-44e2-aae8-f69fdaea7be8`内bootstrap/read_workspace_state/save_workspace_state三组trace开始/完成逐项配对。MSI8,384,512字节、NSIS5,915,087字节；ZIP在仓库上一级`R12-09-windows-evidence.zip`。startup不替代Windows Full/真实PG最终验收，原待验继续保留。

## 设计身份与依据

- 唯一设计分支`ui-design-system`当前提交`40109ceec8372391a7680e7e86c1265fda06ab3d`；读取该提交的`docs/README.md`、`FIGMA_COMPONENT_RECORD.md`、`FIGMA_FOUNDATION_COMPONENT_BACKLOG.md`及`FIGMA_PC_APP_SHELL.md`。最新batch08限定原型修复不改变193个共享设计变量，未用旧“Screens未开始”历史文字覆盖最新状态。
- 实际文件[Football Intelligence Platform](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=23-4)，Foundations页面0:1、稳定根23:4、Desktop dependencies100:2。设计转代码工具在编辑前读取完整高保真上下文及截图，再只读提取六个共享集合、所有模式/值/别名/作用域/WEB语法、文字和效果样式。没有改Figma、创建新文件或新组件，没有把原型专用控制变量带入代码。
- [真实提取与逐变量映射](R12-10-figma-tokens.json)保留193变量、13文字、6效果的实际ID与原值；每个变量登记唯一CSS名、类别路径和单位，保留原WEB codeSyntax作为来源。它是只读设计证据；运行时值只由分类CSS提供，不是第二套全局配置。Code Connect组件接入不适用于本次纯变量/样式映射，未声称自动同步。

## 唯一目录、入口与调用链

`src/main.ts -> ui/tokens/index.css -> 十类token CSS -> 现有Shell/反馈/确认框`。入口只导入，分类文件仅定义变量，不持有业务、DOM、网络或会话状态。原平面`colors.css`、`dimensions.css`已逐个明确路径迁出并替换；没有旧入口转发、重复FIP实现或全局store。架构契约登记唯一入口和实际设计快照，状态owner仍31项。

| 分类 | 唯一运行文件 | 来源/责任 |
|---|---|---|
| Color | color/colorTokens.css | 53原始色、43语义色、Icon Tone八模式；原FIP颜色接口是显式别名 |
| Typography | typography/typographyTokens.css | 13样式的family/size/weight/line-height/tracking，含数字与技术等宽字体 |
| Spacing | spacing/spacingTokens.css | 六级间距、各控件padding/gap及原space别名 |
| Size | size/sizeTokens.css | 布局/图标/控件/工作流尺寸，Icon Scale24/20/16、Control Scale32/40/48 |
| Radius | radius/radiusTokens.css | 基础、菜单、头像与控件各尺寸圆角 |
| Border | border/borderTokens.css | 控件描边、焦点描边与真实边框别名 |
| Shadow | shadow/shadowTokens.css | Light/Dark Subtle、Float双层、Dialog效果及原反馈/对话框别名 |
| Motion | motion/motionTokens.css | 原1.2s进度动效、easing/迭代/方向与reduced-motion |
| Z-index | z-index/layerTokens.css | 原Topbar1/Progress2/Feedback80；原生dialog保留浏览器top layer |
| Breakpoint | breakpoint/breakpointTokens.css | 原960×600最小Shell及现有响应式/大屏阈值元信息 |

193项保持分属Primitives53、Color43、Size78、Icon Scale3、Control Scale15、Icon Tone1。字面颜色/数字只在实际归属分类定义；变量别名按照真实ID映射并检查循环/缺项。Figma若干WEB语法被多个语义变量共享，代码为每个Figma变量保留独立FIP语义名，不靠CSS名碰撞合并不同责任。

## 兼容与范围决策

沿05～09现有`--fip-*`和Shell接口补齐，没有再建立第二套库。FIP前缀避免新Foundation值覆盖尚未进入R13～R15的历史Feature `--ui-*`调色板；旧业务页面、历史断点和其截图契约保持。后续业务页面接入与R18样式边界沿同一FIP库继续，不把本节点当全部页面重写完成。没有新增Feature CSS魔法值。

Shell/告警/反馈/确认框和已有32px按钮将对应的硬编码字体、边框、圆角、尺寸、层级、进度动效改读token，当前数值与交互保持；图标文件、蒙版几何和callsite未改。43项语义色双模式完备；尺寸/色调通过具名data-fip-control-scale、data-fip-icon-scale、data-fip-icon-tone属性局部覆盖，每个模式都含回到默认值的规则。原主题仍由AppSession唯一持有。

Motion、Z-index、Breakpoint不是193项中的新Figma变量。快照implementationContracts明确引用98dc43e的原运行代码/视觉契约，保留原值与浏览器层叠语义，不伪称新获Figma确认。CSS自定义属性不能直接用于原生媒体查询条件，因此只在普通min-width/min-height消费最小Shell尺寸；现有Feature的四个查询及1680×900规则不变，未引入不受支持的var媒体查询或预处理依赖。

保留用户确认的Dialog Light 0/10/24/4.5%与Dark0/12/28/26%，没有复制已作废的大扩散阴影。showShadowBehindNode原标记保存在快照；代码继续用不透明表面上的原生outer box-shadow表达。Inter/Cascadia Mono沿原系统字体和回退栈，不新增或下载字体文件，也不将字体family声明当作设备已安装字体的证明。

业务TS、171命令及注册顺序、API/DTO/Schema、持久键/格式、Rust/数据库/迁移、锁文件和模型保护资产均未变化；main差异只有CSS入口。无生产依赖、持续runner、workflow或数据库专项设施新增。

## 验证与真实证据

| 检查 | 结果 |
|---|---|
| 原verify-global-visual-system入口 | 193变量/13文字/6效果、所有模式/单位/别名、1,344项原生CSS计算值通过 |
| 实际浏览器 | 明暗1440×1200全色板已查看；所有变量计算来自真实浏览器，截图不是运行资产 |
| 完整verify:frontend | 91步骤通过：89原脚本＋TypeScript＋Vite；原1,104 Shell/工作区、136告警、17视口、25 Platform/24 Transport、24生命周期/18会话/21路由/16工作区检查保持 |
| 完整verify:architecture | 18 Feature、11 Rust crate、31状态owner通过 |
| Rust格式 | cargo fmt --all -- --check通过；Rust源码无变动 |
| 保护/差异 | 18保护项聚合SHA-256仍d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57；main业务逻辑、类型/命令/模型/平台/资源无差异 |

日志在仓库上一级`r12-10-frontend-final.log`、`r12-10-architecture.log`、`r12-10-tokens-final.log`；最终token图像在`C:/Users/uniqu/AppData/Local/Temp/football-r12-tokens-dFVXkB`。完整回归后仅加强百分比字距计算断言，原视觉入口再次1,344项通过，生产CSS未变。没有放宽像素阈值、改原PNG基线或跳过原步骤。所有测试从首命令启用保留产物预载，无批量删除。

首个只读Figma导出超过48,000字节返回限制，按safeToRetry=true分两批取得193项；无画布写入。适配旧Stage1验证器时曾漏设其CSS读取路径基准，出现root ReferenceError；已按原仓库运行根修复，专项及最终完整门禁通过。首轮token预览过短只展示部分色板，已增高到1200并查看全部43项明暗色板，没有改变原产品视口。

最终36条非Markdown路径差异（含两条退出路径）规范化SHA-256 `922e2c3cd36bdfcc5ffbc62315ce6d5491daa76b3a565c481d65ca62d42310a2`，验证后及暂存前复核。Vite149模块，CSS main-DpiPs_J5.css 416.76kB/gzip65.92kB；JS607.28kB/gzip172.01kB。CSS较09增加22.75kB用于完整变量/样式/模式，既有主JS超500kB提示保留。

## 待验、回退与下一项

10自身Windows Rust Clippy/workspace tests、release/MSI/NSIS、真实启动和日志配对待本节点提交后原CI独立核验；09通过不替代10通过，当前保持VERIFYING。真实PG/XLSX/历史保护四项/不可变账本并发回滚/Windows Full/私有P4/P7固定回归等沿最终封包新库安排保留待验。原型、注入adapter、token计算和startup均不替代这些业务验收。

回退基线98dc43e，使用受控Git回退本节点；无数据库或现有存储迁移。11尚未开始。

## 实际文件清单

相对98dc43e共42个路径：15新增、25修改、2个被替代的旧平面token路径退出；无整文件纯移动或额外删除。旧颜色/尺寸内容已按职责重写进十类目录，不能把它们误记成100%纯移动。

### 新增

- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-10-design-tokens.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-10-figma-tokens.json`
- `scripts/tokens/tokenStyles.mjs`
- `scripts/tokens/verifyDesignTokens.mjs`
- `src/ui/tokens/border/borderTokens.css`
- `src/ui/tokens/breakpoint/breakpointTokens.css`
- `src/ui/tokens/color/colorTokens.css`
- `src/ui/tokens/index.css`
- `src/ui/tokens/motion/motionTokens.css`
- `src/ui/tokens/radius/radiusTokens.css`
- `src/ui/tokens/shadow/shadowTokens.css`
- `src/ui/tokens/size/sizeTokens.css`
- `src/ui/tokens/spacing/spacingTokens.css`
- `src/ui/tokens/typography/typographyTokens.css`
- `src/ui/tokens/z-index/layerTokens.css`

### 修改

- `README.md`
- `architecture/module-boundaries.json`
- `contracts/global-visual-system-contract.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-09-files-storage-diagnostics-clipboard-and-timing.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md`
- `scripts/verify-entity-resource-center.mjs`
- `scripts/verify-global-visual-system.mjs`
- `scripts/verify-inline-workspace-and-resource-scroll.mjs`
- `scripts/verify-stage1-balanced-ui.mjs`
- `scripts/verify-team-player-management.mjs`
- `scripts/verify-warning-center.mjs`
- `src/app/shell/application-shell/applicationShell.css`
- `src/app/shell/content-region/contentRegion.css`
- `src/app/shell/sidebar/sidebar.css`
- `src/app/shell/topbar/topbar.css`
- `src/app/warnings/warning-center/warningCenter.css`
- `src/main.ts`
- `src/ui/feedback/blocking-message/blockingMessage.css`
- `src/ui/feedback/feedback-content/feedbackContent.css`
- `src/ui/feedback/task-activity/taskActivity.css`
- `src/ui/feedback/toast/toast.css`
- `src/ui/overlays/confirmation-dialog/confirmationDialog.css`
- `src/ui/primitives/button/button.css`

### 替代退出

- `src/ui/tokens/colors.css`
- `src/ui/tokens/dimensions.css`

### 纯移动/重命名

无。
