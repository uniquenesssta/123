# R14-17 应用内验收与报告

状态：DONE（2026-11-04已核验本批精确CI/官方包/startup；原外部边界保留）。用户授权“收尾16开始17”，基线349dcda9c021a9ab839d14e346dfa2a3ad9aeccb，沿用rewrite/r14-prediction-review-analytics-ui；开始时工作区干净。16自身CI41489495436、官方artifact13165974429摘要、MSI/NSIS与startup已正式核验DONE。这里只实施17；下一主交付R15-01“会话、草稿与请求全生命周期”，未启动。

## 已核对的责任与实际边界

ReleaseFeature拥有数据库隔离的草稿、原请求、确认消费、执行/未知恢复；ReleaseQuery拥有独立列表/选择序列及不可变报告；releaseDraft校验原始输入，releaseReport校验DTO身份；Request/Report/Page View负责组合现有FIP组件；ReleaseControls复用NumberInput控制器；ReleaseActions绑定原请求保护确认，ReleaseDetails复用原生DialogController承载只读JSON。main只组合与转发，退出refreshReleaseAcceptance、optionalNumber、runReleaseAcceptance、openReleaseAcceptanceRun及两个旧全局报告变量；单个旧src/pages/release.ts退出，由Feature接管，未增加转发壳。

保留原三条API命令、请求/返回DTO、Schema、账本、模型与生产依赖。明确复用后端公开public_shell_and_runtime模式；外部模型运行时警告不被固定fixture文案或历史通过掩盖。性能/成本窗口1～365整数，非有限/负预算拒绝，空预算与0分开，前端不静默夹取输入。检查状态与成本/性能数值只消费真实后端，不重算发布门禁。

目录初次加载不把最新历史报告自动当本次结果；所选历史、当前请求、返回结果和草稿分别保存。读取B先清A，迟到A不覆盖B；新请求的警告/失败/未知不能沿用历史绿色状态。提交前读取目录，确认随草稿/导航/数据库身份失效，双击只发送一次。收到报告ID后重读原账本及指纹；读回失败可按已知ID恢复，不重跑。回执完全丢失时，现有接口没有客户端幂等键或可核实的原请求标识，只列出新增可见报告供只读核对，保持未知和写锁，绝不猜测归属。

当前历史DTO不保留完整原始性能窗口；不从新草稿填充历史字段，不伪造可重建的请求。已有查看完整报告/证据保留，没有可调用的原发布报告导出命令，不新增伪导出或Git部署。报告只读弹窗清楚显示原ID、hash和完整JSON，关闭/焦点/滚动复用共享控制器。

## 设计与技能

使用Impeccable Operate/harden、Figma design-to-code；文件PN0Whgu6HLWIHx4Mv6aHfu，session59cc846f-cc42-4552-b150-a5761b4e3747。读取ui-design-system的FIGMA_SCREEN_RELEASE_STATE.json（旧设计记录只供状态映射，实际以当前节点为视觉依据）。已取得高保真Overview597:123771、Running597:128181、ReportA597:133278；报告初次读取及完整弹窗599:134979超时，改读明确子节点。Release main稀疏响应未用于直接实现，随后完整ReportA高保真成功；DialogBody599:134981高保真与截图成功，保留680×570外框、62/460/48头/滚动/动作区及原FIP共享组件。四指标、六分区、32px数字控件/动作、40px文字输入、8px间距、26/700标题、22/700指标、14/700分区标题按现有token；颜色/字重不另立方向。复用现有精确Shell/加减/关闭图标，无临时Figma资源URL。

Impeccable context入口因默认引擎缓存不可写未运行，已单独说明；项目无PRODUCT.md/DESIGN.md，以现有批准记录/组件/token直接核对，未重跑context或伪造上下文。已安装项目引擎0.1.17可用，初次检测结果为空，最终检测另存。

## 验证进度及失败修复

22项真实生产流程检查、27项六分区/工作区断言、原21项路由/历史行为及34项Windows完整挂载检查通过。外部API使用明确QA夹具；没有把fixture报告当真实PG。已接入原verify-release-acceptance/verify-browser-bootstrap和verifyApprovedDesign，不新增runner、workflow或放宽检查。

首次浏览器入口暴露共享夹具缺少release→model映射（第二次误以为route支持第二参数未生效），已按实际夹具契约补路由，原等待与就绪门槛不变。页面复验发现通用详情将ID/hash折叠且偏离S11只读JSON，改复用批准Dialog；确认返回未知后原生确认已关闭，测试旧的重复取消前提已订正，未削弱未知写锁。完整96步已通过；最后历史错误、弹窗细节和PG精度修复由差异复验覆盖，未声称同一套96步在每次小修后重复全跑。自身CI/包未验前不标DONE。

证据根O:/Project/FOODBALL/logs/r14-17-20261104；明暗1440×900、960×720、1280×720、1920×1080，100%缩放，保留实际输入、点击、键盘/滚动和截图。R14-17-D1覆盖未运行/处理中/警告/阻断/失败/历史，D2覆盖草稿不改旧报告/原请求恢复；V1旧成功后失败不冒称当前通过，V2公开提供器警告真实保留，V3完整报告/证据只读且不创建Git标签。

BA-12真实新库不可变触发器/跨Port事务/有样本全链及Full、私有模型固定回归继续由R17-06承接；本批真实报告创建和读回若通过单独记账，不抵销未执行的触发器攻击或Full。Rust/crates/src-tauri未改，可复用16同源码成功CI；本批原生运行用既有同后端二进制加载最新dist，不冒称重编Rust。回退349dcda9只撤销代码，不能抹掉新写入的不可变报告。每次成功推送后实际清空唯一O:/Project/FOODBALL/tmp/old，逐文件清单、不跟随链接，必要日志与复用缓存保留。

## 最终证据与交接（2026-11-04）

| 原入口/状态 | 实际归属与真实调用 | 结果/证据层级 |
| --- | --- | --- |
| refreshReleaseAcceptance / 路由载入 | ReleaseQuery.list → listReleaseAcceptanceRuns(50)；从不自动把目录首项当新结果 | 首次/刷新/失败保留目录、未取得数量不补零，原路由21项继续通过 |
| openReleaseAcceptanceRun / 六分区 / inspector | ReleaseQuery.select/publish + releaseFeaturePage/reportView；readReleaseAcceptanceRun原ID | A/B迟到、B错误/重试、来源与hash、历史字段、六分区27项和37项完整挂载 |
| optionalNumber / runReleaseAcceptance / 全局两个报告变量 | releaseDraft + ReleaseFeature.prepare/run，原runReleaseAcceptance(request) | 25项真实生产流程（仅外部端口夹具）；窗口/预算/确认/双击、未知和跨库隔离均执行真实owner |
| show-release-check-evidence / show-release-acceptance-json | ReleaseActions/Details，原报告/检查身份 + 680×570共享原生Dialog | 原ID/hash完整展示，JSON滚动与原生Escape/焦点恢复，纯只读不发布Git |
| result unknown / known report recovery | releaseStorage捕获原库，ReleaseFeature.recover → 原报告ID/指纹 | Windows重启后真实查回，0重发；无返回ID时不借新报告猜成功 |

完整96步frontend-final.log通过，含原保护资产/171命令/架构/64状态、21路由和原页面回归；其中当时37项增量历史场景尚未载入，原挂载34项通过。最终差异：workflow-ledger.log的25项、browser-final-pass.log的37项、types-ledger.log、build-ledger.log以及原生14项/只读精度8项通过。新normalizer只改变已证实的账本精度/JSONB键序投影，未重算检查或丢弃字段。Impeccable最终检测[]。Rust源码与349dcda9完全一致，复用16精确CI成功；调试可执行SHA256 edc7a9c4fa0d9e091c98f692ba38a65dbe39aae6769347d76af03ff9a460b0e2未变，加载最新dist实测；没有重编Rust、重新安装或Full声明。

### Windows真实报告与边界

实际Windows NT 10.0.19045，原生WebView视口1360×880、100%缩放，业务传输无mock。仅创建 **1份** 带R14-17任务标记的实际报告：bb22381a-99e8-4ccd-80ab-63cc64b21dd9，SHA256 52561481cea50d12a17d1fe665f5156f0f22ca48b1dd78d20e9bfea6a3638774。原请求单日/周期预算均0；共13检查，8通过/4警告/1阻断。真实阻断integration_contracts_a_to_i：当前库缺B接入契约；按BA-12由R17-06新库/连续迁移验收处理，R18不得核销。不手工补伪契约、跳过迁移或改变后端检查。

首次原生辅助脚本误匹配说明中的“警告”而过早读目录，日志证明尚未调用run_release_acceptance；preflight-retry-proof.json记录原写调用0、匹配报告0后才重新验证。第二次实际调用已成功写入，随后暴露本批strict JSON比较的真实问题：运行回执时间2026-11-04T12:56:39.851061400Z，PG读回2026-11-04T12:56:39.851061Z；原报告ID/hash一致，但PG只保留微秒。

修复sameReleaseReport：仅将两个chrono UTC时间按实际PG存储精度归一到6位小数，规范JSONB对象键序；保留ID、hash、全部成本/检查/状态/证据/数组次序比较。一微秒差异、同hash下成本变化仍拒绝。25项行为覆盖纳秒→微秒真实例、一微秒/字段改变拒绝和对象键序/数组次序差别；原实际报告已重启按已知ID查回（native-resume.log，14项），并以真实回执时间/真实库读回运行生产比较函数只读复验（native-precision-readonly.log，8项）。该时间投影不是重新捕获完整原回执，不冒称修复后又执行新请求成功。

自动审批拒绝了额外第二份不可变报告的创建（认为超过一次真实写入授权）；未执行该草案，改为上述只读方式。原报告完整JSON/预算/检查归属/关闭/焦点/680×570滚动和前后读回未变已验证；本批没有第二份报告，也未进行真实库UPDATE/DELETE触发器攻击或私有模型/Full。新的不可逆正向再执行未做，不作为已验证项目。

native-startup-final.json为最后只读会话startup/pass：5条日志、2完成操作、0无效/运行错误；真实run成功与首次失败/恢复的各会话原日志均保留。source-manifest.json/validation.json保存最终源码和构建指纹，delivery.json保存发布SHA/一次性CI状态与清理报告。截图位于browser-final-pass/*.png和native/*.png；Figma完整报告弹窗截图figma-full-report.png已核对，Compact截图请求超时，不将超时记通过；窄屏由原批准几何与37项实际布局/可达性检查覆盖。

### 接下来

17自身CI/官方安装包按新提交另验，禁止轮询或拿16结果代替。R14仍待本批正式出口；下一主交付 **R15-01 会话、草稿与请求全生命周期**，未启动，需R14出口及用户启动授权。BA-12其余新库/事务/有样本全链和私有固定回归保留原R17-06承接。本批回退只撤代码，不能撤销已创建的不可变报告。

## 2026-11-04 正式收尾

提交be02451a962549c4ff23d4a6c27fdfe5376c57d8，自身push CI41835857983、job126594452224 success；官方artifact13328354678，ZIP SHA256 e976cc78a655e86303f540225d324a8b7d968742b20941ac738f92abe4c8e733匹配官方digest。MSI/NSIS格式有效，包内startup PASS（7记录、0无效行/运行错误）。详情见[R14阶段完成记录](R14-stage-completion.md)，证据O:/Project/FOODBALL/logs/r14-17-ci-41835857983/acceptance.json。无数字签名/本地安装/Full主张。用户已明确启动R15-01，阶段分支从此提交建立；旧交付时的“未启动”仅为当时状态。
