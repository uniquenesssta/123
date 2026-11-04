# R15-01 会话、草稿与请求全生命周期

状态：VERIFYING。用户授权“收尾17 开始R15-01”；基线be02451a962549c4ff23d4a6c27fdfe5376c57d8。R14-17精确CI41835857983及官方双包/startup已核验DONE，见[R14完成记录](../R14-prediction-review-analytics-ui/R14-stage-completion.md)。本批新建唯一阶段分支rewrite/r15-ai-profiles-logs-ui，开始时无用户未提交修改；自身CI/包待推送后单次另验，不轮询。下一项R15-07，未实施。

## 实际调用与唯一职责

| 原入口/状态 | 当前owner与公开入口 | 真实链与保留边界 |
| --- | --- | --- |
| main.loadApiWorkspace、直接选择回填 | AiWorkspaceQuery，AiWorkspaceFeature.load/select | listOpenAiProfiles/listApiWorkspacePresets/listApiWorkspaceSessions(100)/playerCatalogReferenceData/readApiWorkspaceSession；目录与详情独立AsyncScope，按数据库、页面和每会话读取序号核对 |
| main中共用draft/profile/preset/context/search | AiWorkspaceState、aiWorkspaceStorage | 每个session:ID与临时new:UUID独立草稿；新preferenceKey(api-workspace-state)按已有脱敏数据库身份隔离。只存用户草稿/必要ID/搜索与回执，不存profile对象、密钥、数据库密码或消息账本副本 |
| main.sendApiWorkspaceMessage/pending | AiWorkspaceRequests | 唯一全局发送锁；sendApiWorkspaceMessage原DTO，发送前固定profile/preset/session/context/include_context/message/request_id，attachments保持空；跨await后重新核对来源，完成不强切回B或清B草稿 |
| 取消/未知/重试 | AiWorkspaceRequests + AiWorkspaceActions | cancelApiWorkspaceRequest仅允许原active request ID，接受取消仍待实际命令终态。确切后端取消回复与本地回填取消分开；未知先读原会话/目录，不猜最新session，不自动重发；人工确认仅解除本地发送锁，仍保留unknown原请求记录 |
| 归档确认与恢复 | AiWorkspaceArchive | archiveApiWorkspaceSession沿原归档/审计命令。确认绑定原ID、数据库、页面和修订，防双提交；丢回执保留unknown并禁重复，readApiWorkspaceSession仅在原记录status=archived时确认。目录缺席/仍active不能证明未执行 |
| 球队/球员AI入口 | AiWorkspaceActions.openTeam/openPlayer | 原readTeam/readPlayer与来源Feature.capture守卫，身份不匹配拒绝；新建独立草稿，资料只读且include_context默认false，须主动勾选，无自动发送 |
| 页面、消息、控件 | aiWorkspacePage/aiWorkspaceMessages/AiWorkspaceControls | 原消息/历史结构化审计/复制/搜索/归档/示例问题/设置/检查器/日志路径均保留。复用FIP Select/Checkbox/TextArea/TextInput/Button、现有图标和生命周期；加载/失败不显示另一会话正文 |

公开组合出口src/features/ai-workspace/index.ts。main只组合、转发和衔接原Router/Workspace/Modal/Clipboard；旧全局AI变量、读取/发送/来源函数与业务事件分支退出。唯一旧页面src/pages/apiWorkspace.ts被实际Feature替代退出；原API/client、171条命令、wire/Schema、历史迁移、Rust、生产依赖和模型资产未修改。OpenAI配置编辑与问题日志仍留后续批次，AI只持有本域只读查询投影。

原Workspace.controls缺少会话身份的旧消息仅一次恢复为未绑定新草稿，不猜原会话或上下文。AI页面暂停通用DOM快照回填，以Feature草稿为准；原Workspace记录不物理清除，模块section/sidebar继续复用。控件owner按实际data-ai-key保留输入焦点、选区和消息滚动；请求只滚动自身消息，长会话离开/加载期间不把占位区0位置覆盖原滚动。自动滚动复用motionScrollBehavior，尊重减少动态效果偏好；销毁只释放前端监听，不声称后端取消。

## 设计接入与原ID覆盖

Impeccable4.7.1，Operate/harden/craft-floor/polish；context启动器默认引擎缓存不可写，已明确说明回退。无PRODUCT.md/DESIGN.md，使用本项目既有批准资料、组件、token，未编造设计上下文；项目引擎0.1.17另作代码检测。

Figma文件PN0Whgu6HLWIHx4Mv6aHfu，session66ac3afe-25c7-47e2-bf5b-1b193a6d945e。S13批准checkpoint与50状态/24弹窗映射来自ui-design-system/FIGMA_SCREEN_API_WORKSPACE_STATE.json（recordCommit bb4a14d19d8dc4be458cbd502bc3621969107f7c），实际取得New628:152166、SessionA630:160562、History631:163284高保真上下文与截图；New初次超时后重试成功。未从稀疏上下文实现，未修改Figma或另立视觉方向。

沿批准1440×900、100%及明暗/960×720/1280×720/1920×1080范围：26/700标题、14/700分区、12/400消息、13/500按钮、32px动作、40px选择、96px文本域与8px组间距。设置区与消息/目录分别滚动，composer置于消息滚动区外并常驻；历史预览按设计隐藏composer。248px历史区、360px检查器沿批准布局，窄窗口检查器使用320px且同时收起历史区，以已有任务切换保持可达。复用既有Shell/chevron/check/search图标，无临时Figma资产URL或新图片资源。

| 原ID | 实现与覆盖 |
| --- | --- |
| 01目录 | 真实目录/搜索/选择/归档/已归档只读、加载/刷新失败、当前选择保持；V2/D2 |
| 02对话 | 消息绑定原session，迟到读不能覆盖新对象/新回答，历史审计只读且纯文本转义；V1～V3 |
| 03输入草稿 | 每会话/临时new独立，持久恢复/清空/示例问题；输入常驻、焦点/选区/滚动；D3 |
| 04请求 | 全局锁、pending只展示原会话、来源快照、A完成不导航B、未知回执保留；V1/V3 |
| 05取消 | 精确ID、取消受理≠终态、迟到取消不覆盖完成、未知不自动重发；V4 |
| 06上下文 | 主动勾选、原profile/preset/context锁定、缺失profile不换绑、发送时快照与历史消息不被新勾选改写；V4 |

## 验证与实际失败

证据根O:/Project/FOODBALL/logs/r15-01-20261104。baseline-regressions.log从原生产函数提取并复现3处失败：A完成→目录await切B后回填A、A/B/C逆序选择回填A、B草稿覆盖A。退出码1作为原始失败保留。BA-09以真实生产工作流外部边界受控延迟复验；不是在测试重写状态机。

- 生产流程最终31项：V1成功/失败/取消与B草稿、V2乱序/失败/数据库ABA、V3目录await窗口/旧详情覆盖新回答、V4目标取消/unknown查回/旧确认、重启恢复、归档不重复、来源只读、搜索与旧草稿兼容。incremental-final.log；workflow-final.log保留此前30项结果。
- 工作区层级26项，原路由21项；架构/契约/保护资产、类型/构建、完整前端及最终增量结果见validation.json。原检查入口已接入行为/挂载验证，没有新增runner、workflow或放宽门槛。
- Windows完整生产挂载使用明确QA外部传输边界，真实键盘/点击/取消确认、四尺寸明暗、长会话滚动与切页恢复；最终47项；结果和截图在browser-delivery及完整前端日志引用的ai-workspace-mounted。预设数据不当成真实库/API成功。
- Windows NT10.0.19045.0，1360×880、DPR1，既有同后端debug二进制加载本批dist。原生11项实际transport/数据库读取/草稿/跨页/控件/布局检查；真实库会话数0，未声称实际A/B或真实send/cancel/archive。报告native/report.json，startup逐日志分析；未本地安装本批包。
- 第一次原生观察器未捕获四条命令而失败；后端JSONL已证明命令真实完成，改用实际IPC生命周期作为证据，不以JS拦截计数代替原生命令。无业务写入，attempt-1-failure.json保留。
- 完整前端首次暴露旧页面制品/来源函数名守卫；按真实新owner保留历史契约并更新来源，不修改历史契约哈希。第二次发现迁移遗漏自动滚动的减少动态效果消费者，已恢复来源会话滚动与偏好并复验。原生截图发现本页span规则误画未勾选复选框，移除覆盖并补可视断言，未修改共享组件或关闭检查。

Rust与Cargo.lock受测树和be02451a一致，引用该提交CI41835857983结果；本批不冒称重新运行Rust。完整前端旧失败及最后差异复验分别保留，不声称每一小修都重跑整套。

## 外部边界、回退与交接

BA-09前端竞态本批修复；BA-10普通消息双autocommit/跨Port/取消不能撤销已提交写入已按真实源码确认并投影为unknown/可核验状态，未改变事务协议。请求ID不在消息DTO内，无法从一条相同文本或新目录项证明原请求完整成功；前端不伪造幂等或全回滚。真实PG并发/消息/尝试账本、授权API发送与取消、Windows凭据及缺失原配置实证仍沿BA-10/11由R17-06关闭；R18不得忽略。聊天文本不执行工具/schema或业务写入，但客户端消息账本可以保存用户消息。

回退代码到be02451a仅撤回本批前端与记录，不改数据库账本。新数据库隔离preference键不覆盖既有Workspace键；回退后需要的草稿/原请求记录应保留，不批量清除用户状态。自身CI/官方包收尾后，下一主交付R15-07，须新的启动指令。

临时目录进程级使用O:/Project/FOODBALL/project-temp.ps1，已单独提醒。用户本轮明确授信O:/Project/FOODBALL；不在其他非默认位置创建。最新AGENTS禁止批量删除，本批仅单个旧源码路径按明确路径退出；无用临时产物按本批清单归档至tmp/old供手动清理，最终日志、依赖和稳定缓存保留。清理结果以delivery.json为准，不称未执行的批量删除已完成。

最终补验修复：连续重绘未完成恢复时不捕获临时0滚动；用户新焦点优先于延迟恢复。请求离开数据库后仍将实际回执存入原数据库恢复记录，不写入新库UI；查回与所选详情共享每会话读取序号，旧查回和旧确认均失效，loading状态随正确原读回收口。对应生产回归此前30项，最后新增加载期间输入草稿回归，最终31项。

最终差异复验：完整前端96步（94个原入口、TypeScript及Vite）在frontend-delivery.log退出0，含当时30项流程/46项挂载检查。最终审阅再复现“读取详情期间输入草稿会使读取过期并留下loading”失败，typing-during-read-before.log保留退出1。详情读取改由会话、数据库、页面和读取序号守卫，不再让草稿编辑修订误取消同一详情；确认单仍按修订失效。incremental-final.log按最终源码通过31项生产流程、47项挂载与四尺寸明暗、层级/路由及类型/构建；没有把先前整套结果冒称最终微调后又完整重跑。原生复验显式进入chat后11项通过；attempt-2-failure.json保留历史页恢复导致脚本前提不成立的失败，未修改应用来满足测试。
