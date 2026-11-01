# R14-07 运行记录与血缘

状态：VERIFYING。本地适用验证完成，精确07提交的CI/官方摘要/双包/包内startup待验收。用户授权收尾06并开始07，持续推送授权有效，不轮询PR/CI。

分支 `rewrite/r14-prediction-review-analytics-ui`，基线 `8e2d444ba4ab0cd6f596776d0a72afc5446dd64a`，开始时工作区干净。06代码5171f779、精确CI41297701399、官方摘要、MSI/NSIS和包内startup已独立验收DONE，不用06证据替代07。

## 实施与保留映射

- `src/features/runs` 接管运行目录的状态、选择、详情和历史可见性流程；Session/BootstrapState继续独占最近运行缓存，不再建立第二份业务列表。
- List/Detail/技术追踪继续走现有 `list_recent_runs` / `read_run`；移除使用现有 `hide_model_run_history`。API wire、171命令、领域类型和数据库结构保持原契约。只在既有readRun JSON里加回已有数据库列 `history_hidden_at/history_hidden_reason`，让未知响应可以通过原ID正向核实。
- `src/pages/prediction.ts` 的六条紧凑历史也消费Runs公开接口与同一状态；查看进入S05原运行，移除/失败/待核实在原入口同步更新。退出被替代的 `src/pages/runs.ts`。BA-15最后登记的页面内部过渡边实际迁出，没有扩大导入例外。
- 复用Prediction公开只读投影和原始结果展示，保留完整矩阵、阈值、输入审计和技术内容。缺模型、规则、指纹、时间、比分或概率均不补值；实际0继续显示0。历史输入就绪标记不授予当前执行许可。
- A/B详情有请求序号、页面世代和数据库世代约束；B读取失败清空A内容。数据库A→B→A也不能接受A的旧响应。同库刷新不使已打开的原记录失效。
- 确认绑定原run和列表版本，提交前读取原ID并比对模型/窗口/指纹；取消、确认失效或保存恢复信息失败时零写入。每个数据库内按run锁定，防止双击重复提交。
- 隐藏只改可见性。恢复信息先于写入持久化，存储键隔离数据库且不保存密码。响应丢失时保留原ID和原血缘，只读查回；目录中消失、null或缺少可见性字段都不等于核实成功。只有原ID、有效隐藏时间和相同血缘同时成立才移出Session缓存并结束恢复，绝不自动重发隐藏。
- Session阻止较早的目录/摘要响应恢复已隐藏项；数据库切换时新库不会继承旧库的过滤缓存。

## 已批准Figma的接入

沿用文件 `PN0Whgu6HLWIHx4Mv6aHfu` / Screens S05，实施前已读取根Frame完整上下文、截图及状态。List `516:48546`、RecordDetail `516:49577`；FirstLoad `516:48857`、RefreshError `516:49289`、DetailLoading `516:49865`、DetailError `516:50009`；HideSubmitting `521:54576`、HideError `521:54636`。无新设计系统、依赖或擅自重画。

标题26/700/34，面板标题14/700/20，控件13/500/18，正文12/400/18，次要信息11/400/16，比分16/600/22，指标28/600/34；沿用现有中英文字体、FIP tokens和明暗主题。面板内边距12、间隙8/12、圆角16；七列按批准比例对齐。列表行桌面88、紧凑112；窄窗口详情指标变2×2。头部和刷新动作固定，正文独立滚动。逐张截图发现并修复旧表格高度上限造成的卡片溢出，以及旧页面高度规则对紧凑窗口的干扰；新增真实元素边界断言。

## 验证分层

证据根目录：`O:/Project/FOODBALL/logs/r14-07-20261101`。

| 层级 | 当前结果 | 实际证明范围 |
| --- | --- | --- |
| 生产流程测试 | 26项PASS，verifyRunsWorkflow | 运行真实Feature、AppSession和BootstrapState；仅外部API/本地存储受控。覆盖竞态、取消、零写入、重复提交、未知结果、血缘错配、缓存与数据库隔离、实际0和缺值 |
| 完整挂载浏览器 | 最终55项PASS，browser-final/results.json；共享S03再验40项PASS，prediction-final/results.json | 真实main/Shell/事件和生产组件；仅传输为明确QA样本。A～F，1440×900/960×720/1280×720/1920×1080，明暗主题、加载/刷新失败/详情失败、完整矩阵、确认/取消/未知结果查回和焦点返回 |
| Rust格式、Clippy和工作区测试 | 748项PASS、40项显式ignored，rust.log | 本批后端只读字段变化重新检查和测试；显式ignored的PG集成没有执行，不能记为真实PG通过 |
| TypeScript | PASS | 类型检查，不代替浏览器或原生运行 |
| 完整前端门禁/生产构建 | 完整96步exit0，frontend.log；最终干净构建build-final.log exit0 | 静态、行为、浏览器、类型、生产构建分别记录；保留既有大分块提示，不调阈值掩盖 |
| Impeccable | exit0，impeccable.json为[] | 本批Runs和共享只读投影源码，无配置或内联忽略，不代替交互验收 |
| Windows原生 | native-final/report.json PASS | 刚构建的Tauri/WebView2、真实invoke、无QA注入，无业务写入；界面与list_recent_runs真实返回均为0条，未伪造运行记录 |
| startup日志 | PASS，native-final/logs中的acceptance.json | 22条有效日志、7个完成操作，0无效/错误日志；startup档案，不是Full |
| 真实PG隐藏/私有成功矩阵/Full | 未验证成功 | 当前真实库无历史运行；没有向业务库造样本。BA-08仍按原批准R17-06承接，不用QA样本或startup核销 |

既有ignored PG断言 `prediction_input_audit_fields_are_persisted_and_immutable` 已改用实际hide入口，比较隐藏前后的id、模型/参数、输入指纹、输入审计和输出，检查readRun仍可读而目录不再列出。因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 未执行；该样本也不提供完整私有矩阵/复盘/报告链的证明，R17-06仍需实际关联样本逐项读回。

## 临时产物和回退

本批后续进程均先加载 `O:/Project/FOODBALL/project-temp.ps1`，TEMP/TMP为 `O:/Project/FOODBALL/tmp`。最初一次旧bootstrap验证漏配TEMP，产生明确的本批 `C:/Users/uniqu/AppData/Local/Temp/r13-07-browser-PyLDK8`，已迁至 `O:/Project/FOODBALL/tmp/old/r14-07-initial-bootstrap`，见initial-temp-archive.json，不能清理其他用户Temp内容。

临时归档唯一位置是 **`O:/Project/FOODBALL/tmp/old`**，按用户纠正执行；必要证据保留logs，归档路径及清理结果单独列清单，不使用 `O:/Project/FOODBALL/old`。禁止递归删除或把移动称为删除。

代码回退点8e2d444b。本批无迁移、无模型算法/私有资产变化，不因回退代码改写用户数据。公开推送后单次核精确07代码SHA，CI/官方摘要/双包/包内startup未验收前保持VERIFYING。下一项 **R14-08 复盘选择、事实与包提交**，未启动。

最终局部变更（公开历史入口通知和新源码排版）后重跑26项生产状态测试、55项S05挂载及40项S03挂载，TypeScript、架构/样式/状态/命令/类型清单及受影响静态检查再次PASS。最终Vite先把旧dist单项归入tmp/old，再在不存在旧dist时构建，避免递归删除；构建产物与完整前端末尾的JS/CSS内容指纹一致。
