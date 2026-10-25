# R12-11 UI Primitives

状态：`VERIFYING`。基线`e3dcf5dba39969eb11b1628248dd2fbe076261c7`；唯一分支`rewrite/r12-frontend-foundation-ui-system`。用户2026-10-25授权“已通过 开始11”。01～10 DONE；11已实现并通过本地Windows检查，自身完整Windows交付门禁待提交后独立核验；12～19 BLOCKED，不轮询。

## 前置核验

10精确提交`e3dcf5dba39969eb11b1628248dd2fbe076261c7`的[Windows run40174915845](https://github.com/uniquenesssta/123/actions/runs/40174915845)、job121113413620、attempt1全部SUCCESS。交接时按精确head/分支一次性读取全部步骤、完整309,077字符日志和artifact12592069389。下载ZIP15,872,541字节，SHA-256 `e15c8aab2e83d74206ef154c7176f5f17d9305f2ca394236a92b0efc10152181`与API及上传日志一致。前端91步骤（CI中验证和打包各跑一遍）、1,344项token、1,104项Shell/工作区、136项告警、17视口、Rust fmt/Clippy/工作区测试、release/MSI/NSIS、实际启动和日志通过。45组Rust结果均ok/0 failed；ignored环境用例仍为ignored，未冒充真实PG。报告`windows-acceptance-20261025-080139.json`为startup PASS：7有效条目、0无效行、0运行错误；会话`e7b74741-0968-4584-9bbd-ae36d991f4e1`内read_workspace_state/save_workspace_state/bootstrap三组trace逐一配对。MSI8,388,608字节、NSIS5,916,969字节。ZIP位于仓库上一级`R12-10-windows-evidence.zip`。没有轮询。startup不替代Windows Full/PG/XLSX/私有固定回归。

## 设计来源与边界

- 当前`ui-design-system`精确提交`40109ceec8372391a7680e7e86c1265fda06ab3d`未漂移。实际文件[Football Intelligence Platform · Controls](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=3-3)，页面`3:3`。
- 编辑前用Figma设计转代码工具读取Button/Primary、Secondary、Ghost、Danger、Icon、Loading、Text Field、Number、Textarea、Checkbox、Radio、Switch、Select、Badge、Spinner、Skeleton Block/Skeleton、Menu Separator的高保真代码和截图，再只读核对Switch/Radio的当前几何、语义绑定和opacity。没有改Figma或创建新的视觉母版，没有把历史Screens状态代替当前accepted记录。
- 真实节点、API/目录映射、20个新下载SVG与1个复用Spinner的原始根尺寸/字节数/SHA及部分几何/paint绑定在[唯一组件映射记录](R12-11-component-mappings.json)。共享193变量仍引用[10快照](R12-10-figma-tokens.json)，不重复创建token系统；未发布或声称Code Connect自动同步。
- Divider复用Menu Separator的1px水平线/4px上下空间。Tooltip采用Button/Icon约定的原生title，并补充独立aria-describedby文本；没有Figma自定义tooltip母版，不虚构新的浮层外观，也不声称原生title可在所有辅助设备上可视回放。
- 本节点实现字段的control slot（Figma label/helper visibility关闭）。标签、hint/error、校验摘要和表单编排仍归12，Searchable Combobox及其业务查询/草稿没有混入单选Select。未改Feature页面。

## 唯一模块与运行链

`main.ts -> ui/primitives/index.css -> control.css + 14类独立CSS`；目录index.ts仅显式re-export。原Shell/反馈/确认/告警的按钮由同一button/icon-button渲染，原加载图形改用同一spinner；没有保留第二份按钮/加载实现。所有原data-action和确认动作、关闭/取消含义及AppSession主题所有权保持。

| 目录 | 公开能力与局部责任 |
|---|---|
| button | Primary/Secondary/Ghost/Danger 32/40/48，leading/trailing、disabled；三类Loading组合Spinner。每次busy捕获当前宿主标签/禁用/宽度，结束/销毁恢复，拒绝无设计依据的Ghost Loading |
| icon-button | 三种方形命中区、16/20 Icon Slot，必填可访问名称与原生提示，原生disabled与可选aria-pressed |
| text-input | 40px真实input、前后图标、clear；保留输入/组合输入语义，disabled/readOnly不能经clear绕过 |
| number-input | 32px原生number，min/max/step/any、单位和共享Icon Button；由浏览器stepUp/Down/validity执行小数语义，不复制业务数值解析 |
| text-area | 原生多行/96或160初始高度、可选纵向resize、disabled/readOnly/error |
| checkbox | 原生checked/indeterminate，原生Space/标签点击与form reset，局部混合态控制 |
| radio | 原生同name互斥/方向键/disabled；Ring/Dot使用精确本地资产 |
| switch | 原生checkbox role=switch、整行label与说明，40×24 target、36×20 Track、16 Thumb；仅一个当前状态资产可见 |
| select | 原生select唯一保存form值；组件只拥有open/active/typeahead，Arrow/Home/End/Enter/Space/Escape/Tab，禁用选项/祖先、外部关闭、必选校验与reset同步 |
| badge | Filled/Outline×五tone，非交互状态标签，无删除动作 |
| divider | 原生hr的水平语义线及可选装饰隐藏 |
| tooltip | 原生title+辅助描述的挂载/恢复，无计时器或自建全局浮层 |
| spinner | 16/20/24×Default/On Accent，共享精确Track/Arc；800ms线性旋转，reduced motion静止 |
| skeleton | Compact/Default×List/Card/Table，由Block组合，无业务/输入状态，aria-hidden且静态，只供无旧内容的首次载入 |

原生input/select拥有表单值；host拥有业务、请求、取消和提交。组件不调用API、Tauri、Repository或持久化，不新增跨模块状态owner；原31项状态所有权不变。需要监听/短计时的控制器复用既有EventScope，destroy解除事件与reset/typeahead回调。重复destroy、destroy后操作及重挂载已验证。

Select只实现单选控件自身的option surface，使用既有层级token；开口按trigger计算屏幕内宽度/可用高度，底部不足向上翻转，外部滚动或resize关闭；自己的选项滚动不会误关闭。它不是14的通用Popover实现。

## 视觉/资源兼容决策

- 沿现有Inter与Windows回退字体，不下载新字体。字段按自身Figma控制规格13/14px，不误用Shell的紧凑11/12px文字样式。计算family声明不能证明设备安装了Inter；没有跨DPI全量验收声明。
- 20个新SVG逐个经Figma给出的curl下载来源取得，保留原始字节；原16px Spinner复用。Ring、Track、Thumb和focus轮廓不重绘，只用语义token投射当前主题paint。
- 原始Spinner导出含外描边，根尺寸17.25/21.5/25.75不变。局部max-width:none排除旧全局100%媒体规则压扁外沿。Plus根24.875保持，通过24px master容器和-0.875 offset缩放，不改SVG根width/height。
- 当前accepted Switch Disabled Thumb实际opacity为0.0042，精确保留导出，未擅自改成control token的0.42；native disabled与辅助名称仍完整。Radio边框与Switch低透明度paint只应用一次alpha。
- Textarea的旧全局规则使用!important；本节点只对`.ui-text-area[data-ui-primitive]`覆盖对应字体/边框/背景，不改Feature CSS。所有新增固定控件几何、12/14px字体、选项36px/上限288px与SVG外沿均能追溯Figma规格或明确的屏幕内定位约束，不作为新设计变量伪报。
- Reduced motion与forced colors保留可用状态；Skeleton无动画。原PNG基线、像素阈值、页面业务布局与滚动规则未修改。

## 实际验证

| 项目 | 结果/证据（仓库上一级日志） |
|---|---|
| 11原生Windows Chromium最小检查 | 14族，960×1800与1440×1500×Light/Dark，每组217，合计868；真实输入/键盘、form value/reset、禁用祖先、注入边界、加载宽度、viewport翻转、destroy/remount；`r12-11-primitives-accepted.log` |
| 完整现有前端 | `npm run verify:frontend` exit0，91步骤含TSC/Vite；`r12-11-frontend-accepted.log` |
| 原有核心回归 | 1,344 token，12×92=1,104 Shell/Workspace，136 Warning Center及17既有截图视口，25 Platform/24 Transport与原Session/Router等门禁通过；没有跳过或改阈值 |
| 架构/保护 | `npm run verify:architecture` exit0，31状态owner；18保护文件聚合SHA `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`；`r12-11-architecture-final.log`与完整前端日志 |
| Rust格式 | `cargo fmt --all -- --check` exit0；Rust、锁和模型保护目录无diff，Clippy/workspace/安装启动的本节点交付验证待自身CI |
| 最后非语义排版整理 | 仅将Plus静态导入移到文件顶部并移除多余空行；TSC/Vite再过。随后Select同步布局修复后的完整91步骤再次通过，最终产出main-BhjUIt0r.css/main-CzzBzp8J.js/index-BYhrr3FK.js，以`r12-11-frontend-accepted.log`为准 |
| Git/范围 | diff --check通过；main仅CSS入口变化，运行逻辑不变；无文件删除或移动，详下表与源码指纹 |

完整前端的原生组件证据在临时`football-r12-primitives-lqAfZz`（8张画面含下拉打开）及原`../logs/r12-11-primitives-1792925064827`；测试工具不新建持续runner/workflow。浅/深全控件画面及下拉几何已核对；这是Windows Chromium DPR=1证据，不代替所有DPI或11自身Tauri安装启动。

Vite实际158模块：CSS436.88kB（gzip68.85），main JS610.03kB（gzip172.93）及index JS6.31kB（gzip2.59）。原>500kB提示仍存在，没有提高阈值或将其记作失败。相比10，新增基础控件CSS和少量共享运行代码；没有依赖/锁变化。

最终非Markdown88路径SHA-256：`ff83e9617bbcdd8a51f5cd60c98d492ea87dbb268096e600bcf85a39caee0bed`，逐路径摘要保存在仓库上一级`r12-11-source-proof.json`。文档/链接另行检查。

## 失败、修复与执行限制

- 初次SVG尺寸断言使用旋转后的外接矩形不正确；转为计算尺寸后定位到原全局max-width压窄根，局部修复并保持精确期望。初次Skeleton纵向flex压缩高度已修复。
- 截图复核发现Switch/Radio多状态级联、Skeleton Avatar被横向flex拉宽、旧label/textarea样式覆盖以及Radio双重alpha；已修复，新增独立可见变体、头像几何、文字/错误色与alpha断言，最终868项通过。
- busy标签断言剔除SVG空白文本干扰，仍精确比较可见label；表单reset断言曾误写为不存在的子节点条件，已修正并补齐原生reset后的clear/stepper/mixed同步。
- 首轮完整前端在原Shell守卫匹配旧位置参数API时停止；守卫同步为同一对象API，1,104项实际浏览器断言保持；随后完整91步骤通过。
- 隔离执行账户返回Windows1909锁定，后续在批准的可用账户执行验证。一次测试误从仓库父目录启动，未运行测试；只逐项删除该次生成的一份明确路径日志，无项目文件删除。
- 一次额外global-visual验证未带本地保留preload，既有截图工具自动清理了临时浏览器profile。这是执行疏漏；没有项目文件删除。其后本地验证带保留preload，原产物保留；没有将该操作描述为用户允许的批量清理。

- 最后下拉打开截图发现横向尺寸仍错误：历史reduced-motion的全局0.01ms transition令旧几何参与瞬时测量/绘制。已在Select root/options局部设transition-property:none，并取消固定popup的100%宽度，由控制器测量唯一决定；新增reset关闭、明确280px宿主和popup与trigger等宽/横向边界断言，最终868项通过。临时诊断属性和布局等待均已移除，没有放宽期望。

## 实际文件清单

本节点相对e3dcf5d共94路径：72新增、22修改、0移动/重命名、0文件删除。目录index.ts为公开出口，fixture和两个验证支持文件沿原门禁组织，不新增持续测试设施。

### 新增（72）

- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-11-component-mappings.json`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-11-ui-primitives.md`
- `scripts/primitives/verifyPrimitives.mjs`
- `src/assets/controls/radio-default.svg`
- `src/assets/controls/radio-disabled-selected.svg`
- `src/assets/controls/radio-disabled.svg`
- `src/assets/controls/radio-hover.svg`
- `src/assets/controls/radio-selected.svg`
- `src/assets/controls/spinner-20.svg`
- `src/assets/controls/spinner-24.svg`
- `src/assets/controls/switch-off-default.svg`
- `src/assets/controls/switch-off-disabled.svg`
- `src/assets/controls/switch-off-focus.svg`
- `src/assets/controls/switch-off-hover.svg`
- `src/assets/controls/switch-off-pressed.svg`
- `src/assets/controls/switch-on-default.svg`
- `src/assets/controls/switch-on-disabled.svg`
- `src/assets/controls/switch-on-focus.svg`
- `src/assets/controls/switch-on-hover.svg`
- `src/assets/controls/switch-on-pressed.svg`
- `src/assets/icons/chevron-down.svg`
- `src/assets/icons/minus.svg`
- `src/assets/icons/plus.svg`
- `src/ui/primitives/__tests__/primitiveBrowserFixture.ts`
- `src/ui/primitives/badge/badge.css`
- `src/ui/primitives/badge/badgeView.ts`
- `src/ui/primitives/badge/index.ts`
- `src/ui/primitives/button/buttonController.ts`
- `src/ui/primitives/button/buttonView.ts`
- `src/ui/primitives/button/index.ts`
- `src/ui/primitives/checkbox/checkbox.css`
- `src/ui/primitives/checkbox/checkboxController.ts`
- `src/ui/primitives/checkbox/checkboxView.ts`
- `src/ui/primitives/checkbox/index.ts`
- `src/ui/primitives/control.css`
- `src/ui/primitives/controlAttributes.ts`
- `src/ui/primitives/divider/divider.css`
- `src/ui/primitives/divider/dividerView.ts`
- `src/ui/primitives/divider/index.ts`
- `src/ui/primitives/icon-button/index.ts`
- `src/ui/primitives/index.css`
- `src/ui/primitives/number-input/index.ts`
- `src/ui/primitives/number-input/numberInput.css`
- `src/ui/primitives/number-input/numberInputController.ts`
- `src/ui/primitives/number-input/numberInputView.ts`
- `src/ui/primitives/radio/index.ts`
- `src/ui/primitives/radio/radio.css`
- `src/ui/primitives/radio/radioView.ts`
- `src/ui/primitives/select/index.ts`
- `src/ui/primitives/select/select.css`
- `src/ui/primitives/select/selectController.ts`
- `src/ui/primitives/select/selectView.ts`
- `src/ui/primitives/skeleton/index.ts`
- `src/ui/primitives/skeleton/skeleton.css`
- `src/ui/primitives/skeleton/skeletonView.ts`
- `src/ui/primitives/spinner/index.ts`
- `src/ui/primitives/spinner/spinner.css`
- `src/ui/primitives/spinner/spinnerView.ts`
- `src/ui/primitives/switch/index.ts`
- `src/ui/primitives/switch/switch.css`
- `src/ui/primitives/switch/switchView.ts`
- `src/ui/primitives/text-area/index.ts`
- `src/ui/primitives/text-area/textArea.css`
- `src/ui/primitives/text-area/textAreaView.ts`
- `src/ui/primitives/text-input/index.ts`
- `src/ui/primitives/text-input/textInput.css`
- `src/ui/primitives/text-input/textInputController.ts`
- `src/ui/primitives/text-input/textInputView.ts`
- `src/ui/primitives/tooltip/index.ts`
- `src/ui/primitives/tooltip/tooltip.css`
- `src/ui/primitives/tooltip/tooltipController.ts`
- `src/ui/primitives/tooltip/tooltipView.ts`

### 修改（22）

- `README.md`
- `architecture/module-boundaries.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-10-design-tokens.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md`
- `scripts/verify-global-visual-system.mjs`
- `scripts/verify-inline-workspace-and-resource-scroll.mjs`
- `src/app/shell/sidebar/sidebarView.ts`
- `src/app/shell/topbar/serviceStatusView.ts`
- `src/app/shell/topbar/topbarView.ts`
- `src/app/warnings/warning-center/warningCenterView.ts`
- `src/main.ts`
- `src/ui/feedback/blocking-message/blockingMessageView.ts`
- `src/ui/feedback/task-activity/taskActivityView.ts`
- `src/ui/feedback/toast/toastView.ts`
- `src/ui/icons/iconSlot.css`
- `src/ui/icons/iconSlot.ts`
- `src/ui/overlays/confirmation-dialog/confirmationDialogView.ts`
- `src/ui/primitives/button/button.css`
- `src/ui/primitives/icon-button/iconButton.css`
- `src/ui/primitives/icon-button/iconButtonView.ts`

### 移动/重命名与删除

无。旧按钮/加载的重复markup/CSS仅在原文件内由唯一primitive调用替换，没有清理其他文件或目录。

## 未执行与后续门禁

11自身Windows CI/安装包/实际Tauri启动与日志待提交后按精确head/run独立核验；不沿用10为11结论，不轮询。真实PG、历史保护/账本并发回滚、真实XLSX、Windows Full、私有模型固定回归和真实API/凭据取消端到端等既有最终环境待验继续保留。本节点没有修改后端/协议/数据库/配置/Feature状态或生产依赖，不宣称这些待验通过。

## 回退点

`e3dcf5dba39969eb11b1628248dd2fbe076261c7`为11开始前通过完整Windows的10提交。发布后按独立原子提交revert恢复，不手工复制文件、不保留双实现。11自身完整Windows核验前保持VERIFYING，12不开始。
