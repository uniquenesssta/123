# R17-06 真实环境、工作簿与 Windows Full

状态：部分实证完成；BLOCKED，未验收完成。用户授权“收尾01开始06”，长期提交推送授权有效；基线2101ce145d58d1516fdb5b252140b0e16d6edae8，沿用rewrite/r17-verification-system。R17-01自身CI/官方双包/startup已正式收尾。本项不把Automated或本次数据库自动测试当作Windows Full，不进入R18。

## 真实环境与授权

现有127.0.0.1:5432的应用账号没有建库权限；最初CREATE DATABASE被拒绝，没有清空或改写日常football_model。用户随后选择“自动启动临时测试实例”：使用已安装的PostgreSQL 18.4，在O:/Project/FOODBALL/tmp/r17-06-postgres/data建立独立实例，只监听127.0.0.1:55437。既有三个repository契约要求名称包含test，保留这些检查，使用football_r17_test_acceptance_20261107。初始化后在该实例将同名验收库重建为普通账号r17_test_owner所有；最终全部PG及XLSX以该无SUPERUSER/CREATEDB/CREATEROLE/REPLICATION/BYPASSRLS权限的账号通过。初始化进程核对data_directory并记录启动身份/OID，普通账号再核对相同启动身份与库OID，不额外授予系统设置读取权限。每轮沿原destructive_reset测试建立清洁迁移基线。最后实例已停止，必要QA数据和DPAPI保护的随机密码保留以便后续依赖验收；没有明文密码文件残留。

进程TEMP/TMP使用O:/Project/FOODBALL/tmp，已单独提醒。没有新增生产依赖、runner、workflow、迁移或持久生产数据库；临时编排脚本只在项目证据中留存，不进入仓库或公开凭据。

首次诊断记录（已由下方11/08新证据推进）：用户指定O:/Project/FOODBALL/123作为本地P4/P7校验位置。首次检查时该目录停在f060748/R14；随后按用户要求快进远端并切到已有R17分支e651b423，与123 r3对齐。用户明确本机保存完整P4/P7；不能凭公开分支或下拉框判断本机资产不存在。已在123实际ApplicationService注册表调用P4/P7，并调用原P4演练入口：两者均返回“模型运行时不可用，请接入私有或外部ModelProvider”，12个入口中本次成功预测为0。诊断用原默认请求，只证明当前程序未加载真实引擎，不是模型固定回归或Full。临时诊断源码已逐项移除、原lib字节恢复，工作区干净；原回执在logs/r17-06-20261107/123-actual-runtime-probe.json。用户已明确“暂不运行真实API和凭据测试”，未执行计费API或Windows Credential Manager写入/删除。

## 确认缺陷与修复

| 问题 | 修复与唯一归属 | 保留边界 |
| --- | --- | --- |
| SQL NULL摘要导致整张运行历史无法读取，Application又强制将历史摘要当作完整模型输出 | persistence/runs/read保留NULL；Application的ModelRunHistorySummary专用于可缺失的历史概率；TS注明summary可空 | 不造0概率；模型执行的PredictionSummary仍严格；错误类型仍拒绝；原界面已用可选链显示缺失，无UI改版 |
| SQL NULL惯用脚使月度导出、球员档案及AI工作包读取失败 | monthly_player/read导出空单元格；catalog player行结构保留Option，既有枚举映射为Unknown | 不改写数据库，不猜测左右脚；原已记录值不变 |
| Full说明中的中文示例路径不存在 | examples目录及三份XLSX恢复实际中文名，README随目录移动；原工作簿字节完全相同 | 4项纯移动，无额外文件删除；真实生产解析器新增2项文件读取回归 |

新测试分别位于原Application组合根测试、历史适配测试、spreadsheet-io测试及既有PG target；私有模型实现、概率公式、171个命令、数据库Schema、Cargo/前端依赖版本、UI和PNG基线未变。

真实执行也暴露旧夹具不符合当前契约：回放断言误把T-1h窗口起点当截止；直接阻塞PID计数漏掉锁队列的间接等待；证据重复归属、任务trace复用及第二次导入复用阵容时间违反唯一约束。现在采用生产返回的实际截止，保留相等/后一微秒的原断言；递归观察同一父锁的全部等待链；给独立任务分配独立研究/trace/冲突/路由，并验证相互决策不串线；并发导入用新版本时间。没有删除测试或修改0014/0018等约束。新增manual_peer夹具独立文件负责构造第二个合法上下文。

原静态Lineups守卫同步要求递归阻塞链及原取消/混合并发断言；类型/端口扫描和数据库测试指纹按审阅后的真实源码更新，保留先前接受记录。没有通过跳过检查获得通过。

## 已执行证据

证据根：O:/Project/FOODBALL/logs/r17-06-20261107。各次失败原日志分别保留在attempt01～06目录，最终结果在根目录，不把较早失败或编译结果改写成成功。

| 层级 | 实际结果 | 范围限制 |
| --- | --- | --- |
| PostgreSQL repository/综合测试 | 20个既有target、40个原ignored真实测试全部通过；另先执行原清空重建测试；46次迁移成功、150张业务/基础表读回 | PostgreSQL18.4临时实例；不是日常库兼容迁移或私有模型成功 |
| BA-02四个测试 | match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent、match_scope_inference_and_lineup_pair_transaction_are_atomic、structured_match_events_are_queryable_and_revision_aware、p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable分别PASS | 原四项逐名实跑，模型相关概率是明确测试夹具 |
| 实际XLSX→Application→PG→重建应用连接→读回 | 原Application lib target显式执行1个ignored场景：1156条球队/球员链路及807条比赛/阵容链路写入；读回4队、72球员、864能力、3比赛、12阵容、216阵容明细 | 读取仓库真实工作簿，经原应用导入/预检/提交；非JSON预览替代、非桌面UI Full |
| 重连及重复提交 | 原应用对象断开并销毁，新应用连接后查回相同batch/source SHA；球员批次重复提交返回原结果、比赛批次重复提交拒绝，业务数量不增加 | 证明该应用层持久化/幂等边界；未冒充Windows进程崩溃或真实网络断开 |
| 本地完整前端 | 98步退出0，原工作流/完整挂载/浏览器检查与TypeScript/Vite通过 | 含历史NULL/部分摘要不造0概率的原工作流回归、TypeScript和Vite；未更新PNG |
| 本地完整Rust | 格式、Clippy -D warnings及工作区773项通过；默认41项ignored | 默认ignored仍列出40个PG及新增1个Application XLSX；它们在本次另有显式真实执行证据，不能把默认ignored改报通过 |
| 架构与保护 | 35项完整架构通过；原保护/迁移/Windows契约及模型边界通过 | 原35项架构、Windows验收契约/迁移兼容/公开模型边界/保护资产检查分列，不代替真实Full |
| Windows/CI/包 | 01已收尾；06月度修复e651b423的精确push CI42384401592成功，官方artifact13580480729摘要、双包及原始startup日志已接受；本次Windows验收路径修复的新提交CI仍须单独核对 | 未执行Windows Full、重新安装或数字签名验证；不轮询CI或PR |

真实PG命令统一沿用cargo test --locked -p football-persistence-postgres --test <target> -- --ignored --nocapture --test-threads=1；逐项命令、退出码及原日志SHA见postgres-acceptance.json。Application真实工作簿操作、批次身份及读回见application-workbooks.log和application-workbooks-exit.json。普通Rust测试数与显式真实测试数不相加重复计算。

## 本次续接：官方证据与真实月度回导

bff86e4首批Automated证据已正式接受：[CI42272039325](https://github.com/uniquenesssta/123/actions/runs/42272039325)，Windows job128078771173。官方ZIP为52,495,615字节，SHA256 5078037b8e4aa70f088d8fb50e3320a880b1a3c52648c940b3cda28bb7950d88；MSI 18,079,744字节、NSIS 15,709,690字节，原始startup读出7条/0非法/0运行错误。下载、包摘要及重新分析回执保存在O:/Project/FOODBALL/logs/r17-06-ci-42272039325/acceptance.json。没有安装本地包或验证签名。

此前“未找到匹配push CI”的回执源于PR限定的连接器接口，不能据此判定push不存在。本次用GitHub原始push接口按精确SHA查询一次并核验完成任务，原观察保留且标为被纠正；没有轮询。此次月度修复产生的新提交与bff86e4证据分开，不继承其CI通过状态。

在同一Application lib的原真实工作簿场景中增加球队/球员月度导出、预检、提交和重连读回。导出默认skip，原样回导业务更新数为0；QA副本只将球队/球员基础行明确设为update，各改变一个实际字段（简称/身高），保留历史行skip。4支球队、72名球员更新且新增数为0，两项改变跨重连读回一致，原4队/72人/864能力、3比赛/12阵容/216明细数量不变。独立ZIP/单元格比较确认只改变指定action与两个字段，证据见monthly-workbook-comparison.json及monthly-xlsx目录。历史记录仍只允许新增生效记录，未放宽规则。

该场景先复现了真实缺陷：已有球队尚无team_profiles行时，月度update仅插入默认档案，丢失填写的简称等扩展字段。唯一team_package/write所有者现将同一组填写值及clear语义用于首次INSERT和冲突UPDATE，保留原缺失值默认、事务、别名及校验。失败原日志monthly-missing-profile-regression.log保留；修复后原20个PG target/40项及扩展后的Application真实工作簿场景再次通过。未新增Schema、迁移、生产依赖、runner或workflow。

本次Rust fmt/Clippy -D warnings/工作区测试仍为773通过、41默认ignored，真实PG/XLSX显式结果单列；35项架构通过。前端src/scripts/styles与bff86e4相同，沿用其98步前端及官方CI证据，未重复改UI或PNG。QA实例每轮停止，日常库未触及；初次验收副本的路径、默认skip、历史更新误用、空单元格以及未用导入等失败记录均保留，没有计入通过结果。

## 2026-11-08：Windows实际验收入口续接

e651b423自身[CI42384401592](https://github.com/uniquenesssta/123/actions/runs/42384401592)/job128438634683已完成成功，官方artifact13580480729摘要匹配。ZIP 52,493,491字节，SHA256 49f15ceefe5c6a34f9132bae7948ce9f8b582728ed8ff5f3ee0e6c542fd00b74；MSI 18,079,744字节、NSIS 15,711,408字节，原始startup读出7条/0非法/0运行错误，完整回执在O:/Project/FOODBALL/logs/r17-06-ci-42384401592/acceptance.json。本次仅按精确SHA查询一次push，其后读取该完成run的任务与产物；不轮询PR或CI。未执行本地安装或签名验证，此Automated结论不继承给当前脚本修复的新提交。

实际复现本地入口错误：在独立123 r3目录传入ProjectRoot，旧验收器仍拼接“项目源码”，在启动任何阶段前失败。原scripts/windows/acceptance-paths.psm1现在唯一负责源码/项目根解析：独立checkout默认以源码根为运行根，旧“项目源码”打包目录保留外层运行根，显式ProjectRoot优先；缺少验收契约或双重源码根时明确拒绝。CI原布局保持支持。Windows脚本中的路径读取、切换和日志写入改用LiteralPath，覆盖带空格、中文和方括号的目录；原试验还复现了Add-Content通配路径导致日志失败，失败日志单独保留。

同一模块修复release定位的错误退路：只接受项目两种既有客户端文件名，不再把目录中任意最新EXE当作应用。窗口显示行为保持不变；原启动身份、新日志集合、有界重试、数据库安全与Full操作组均保留。

沿原verify-windows-path-contract入口增加20项Windows PowerShell实际行为检查，测试夹具位于scripts/windows/verify-acceptance-paths.ps1，不新增生产runner、package命令、workflow或依赖。原verify-node-process-compatibility失败传播入口保留7项，并增加独立显式目录、默认源码目录、默认旧打包目录3项真实脚本调用；合计10项通过，确保环境/依赖失败码37正确传播，且后续构建或应用不启动。夹具EXE仅用于路径选择，未执行；故障注入证明验收器行为，不证明业务Full成功。

完整98步前端（TypeScript/Vite和原浏览器门禁）通过；最终受影响路径/Windows契约/10项失败传播另行重验通过；35项架构通过。Rust、前端业务和数据库树均未改变，沿用e651b423已接受的773普通Rust/41默认ignored、40项显式PG和真实工作簿证据，不重复跑相同成功场景。本批证据根O:/Project/FOODBALL/logs/r17-06-20261108；进程TEMP/TMP仍在项目tmp，必要证据保留，无用临时产物推送后归入tmp/old并逐文件实际清理。

## 2026-11-08：找回私有资产、真实应用运行与CI修正

此前只检查指定checkout及常见下载位置，漏掉Windows重定向下载目录。本次读取实际Downloads设置后找到用户保留的完整源归档；归档SHA256为9ba3fceb738cb4097149a734a5105f3790dea9f0f18a0d88edd6c0c41e5571db。按白名单只向123内原受Git忽略的模型、默认资源、契约和Schema位置恢复54个文件；未改变源归档，123 r3及公开提交均不包含私有实现、参数或Golden资产。路径、逐文件摘要、恢复及构建回退清单仅保存在项目本地证据O:/Project/FOODBALL/logs/r17-06-private-20261108。

私有原始回归先发现一项正向T-N测试夹具把已有canonical时点重复放入legacy组，触发了正确的重复拒绝。仅修正被忽略的私有cfg(test)夹具分组，保留负向重复测试；所有生产模型Rust字节、参数、Profile和Golden资产与源归档一致。最终P4单测8项、P4 Golden Master10项、P7单测5项全部通过。原失败日志与单项测试修正摘要保留；不把这23项混入公开工作区测试数。临时workspace及私有Cargo构建路径在finally内逐字节恢复。

当前公开ApplicationService此前硬编码公开注册表、启动规则目录及默认演练请求，仅替换registry仍无法接上真实模型。新增显式ApplicationModelRuntime，由宿主提供registry、匹配启动目录、默认模板和默认ModelRequest；在创建服务/连接数据库前复用原规则包校验和提供器校验。RulesService拥有不可变目录，PredictionService拥有可选默认请求，原Facade只转发；未增加文件扫描、自动加载私有路径或全局可变状态。公开new()保持原外部入口与Unavailable行为。原默认dry-run先查注册再构造默认请求、唯一adapter及不计时政策保留，守卫适配显式宿主输入，未删除原断言。新增4项行为测试覆盖提供器确实被调用、启动数据使用宿主输入、错参数/缺提供器/非法默认请求提前拒绝。

本地一次性QA宿主将原参数与单独的Profile按原模型要求组合，登记12个真实提供器，通过当前123 r3的ApplicationService分别成功调用P4、P7；未修改模型公式。随后在已批准55437隔离PostgreSQL，原execute_prediction完成真实路由、计算、规则身份及model.runs写入，read_run和列表查回两次成功运行，销毁应用后新建应用连接，完整持久化文档逐项一致。计算内存值与JSON读回按1e-12精度核对，重连后的文档仍要求完整相等；首轮仅JSON末位差异的失败证据保留。回执runtime-host-database.json/log记录实际run ID、规则摘要和输出身份；未触及日常库或调用API。

已对原导入3场样例分别检查P4/P7的正式门禁：已有阵容自身的采集时间早于标注窗口，当前6次评估均不允许正式运行，未绕过门禁或补造比赛结果。因此以上是真实服务层直接推演及持久化证据，尚不是execute_prediction_from_match、复盘→结算→分析/决定→报告的同对象证明。临时QA规则另用独立版本覆盖原时间窗口，不更改模型参数；不能把路由可用等同于数据完整。

上一提交ebef8903的精确push CI42393007728失败，原因是路径测试将RUNNER~1短路径和runneradmin长路径直接按字符串比较。只修测试Equal：先用Windows完整路径规范化，再按系统大小写规则比较；新增等价写法与不同路径拒绝两项，合计22项原生路径检查通过。真实8.3目录读回也已补验；生产路径解析、安全拒绝与原10项失败传播保持。原CI失败日志与本地复现保存，新提交的CI和安装包不继承旧提交结论，不轮询。

最终公开工作区格式、Clippy -D warnings和777项Rust测试通过，41项默认ignored保留；35项架构、完整98步前端及TypeScript/Vite通过。另用已有隔离原生lib target和原Tauri命令表，6项Windows WebView启动/命令检查通过，所有可写状态在项目QA目录，未连接日常库。首次验收构建漏带已有FOOTBALL_NATIVE_LOG_ACCEPTANCE_BUILD开关，导致Common Controls入口点加载失败；补正确开关后通过，没有更换依赖或系统。该原生证据只验证公开默认应用启动与外部入口未假报成功，私有模型服务层证据单列，仍不抵销Full。

未新增生产依赖、迁移、package命令、持续runner或workflow；一次性本地QA宿主/编排仅留在项目tmp及证据目录。完整私有资产、依赖缓存和停止的QA实例继续保留；公开工作区只交付通用装配接口、测试、清单、路径测试修正及记录。真实API/系统凭据仍按用户明确要求暂缓。R17-06和R18状态不变。

## 11/09 续接：私有 Windows 实机与精确验收边界

此前 `c8f85f4dfd0d33ec5e5ede04f61a2687d537a914` 的精确 push [CI42407666655](https://github.com/uniquenesssta/123/actions/runs/42407666655)、job128508767325已成功；官方artifact13589197273的52,493,709字节ZIP SHA256为03d802203be888ed05b65a46e6c0eec8d2c010fa0790b5c252a4c89d351b8108，与下载结果一致。原MSI18,083,840字节、NSIS15,712,805字节及startup原日志已接受；startup重新分析为7条、0非法、0运行错误。不是本地安装或Windows Full证据。本批新提交CI另验，不轮询。

证据根 `O:/Project/FOODBALL/logs/r17-06-full-20261109`。本地仅在123使用已恢复的原私有资产构建已有隔离lib原生测试窗口：真实Tauri命令表、真实Windows WebView、12个真实P4/P7提供器；前端加载123 r3本批修复源码。临时workspace/dev-dependency/原生测试装配在构建后逐项恢复，123受跟踪文件干净。54项私有资产复核不变，既有P4测试夹具例外仍仅cfg(test)，生产前缀与归档相同；未修改公式、参数、Profile或Golden，私有内容不进入公开仓库。此装配不是公开默认程序或正式安装候选。

### 原生链路暴露的三项公开缺陷

| 缺陷 | 修复归属及验证 | 保留边界 |
| --- | --- | --- |
| 按创建时间倒排后Map保留末项，导致同key旧规则覆盖新规则；正式推演、路由绑定、P4研究三处相同问题 | rules/rulePackageSelection唯一选择函数，调用方保留原active/模型过滤与优先级；30项实际生产HTML/选择回归，修前失败、修后通过 | 不将版本号大小当创建时间；不修改原规则或模型内容 |
| 原生明确拒绝模型输入/参数后仍保存未知运行回执，导致不可恢复的锁定 | prediction恢复分类仅识别锚定的原生拒绝；无已返回run ID才清本数据库回执并标failed；33项原工作流与挂载浏览器覆盖 | 网络未知错误、已返回run ID后读回失败仍保留unknown；不自动重发 |
| Full分析器只看命令名，P7也会被算成正式P4 | Windows验收契约增加p4身份；同会话、同trace、同操作的请求启动参数与成功完成须关联；九类合成日志含P7/缺身份/串trace/串会话/无启动/错误操作拒绝 | 合成测试不是真实Full；startup不增加模型要求，原错误等级与缺项门禁保留 |

### 同一 QA 对象的实际结果

所有球队、球员、历史、当前比赛和2-1赛后输入均为明确标注的隔离QA样例，不是实际足球比赛资料；概率和复盘结论由真实模型/原业务生成。原球员XLSX经原生预检/提交返回11/07既有批次的1156条插入回执，属于幂等重放，不是本次新增1156条。当前比赛按实际系统时间新建、采集阵容；历史样例通过原生命令形成，不回填数据库时间或关闭完整度检查。

P4实际门禁允许后，原引擎拒绝：没有manual_base_lambda，Profile为WAITING_ALPHA_CALIBRATION。只读检查114个本机归档中的206个默认Profile，未找到已校准版本；用户已明确暂无补充来源。这里的α校准属于基础期望进球λ的建模资格，不是安装模型；固定回归通过或提供器已载入不能代替。没有人为改CALIBRATED/eligibility、补造λ或产生成功P4运行。

P7在原完整度门禁允许后，经原界面真实execute_prediction_from_match完成计算、落库、历史详情；同场手工QA赛果/实际阵容经对象及比分确认生成复盘。实际对象：match51b99b39-f919-4d4c-8a46-ac3c3fafcb9f → run5adef493-968c-4570-b221-971b116344fa → reviewa8dbd96b-6951-42b7-a20a-9257af930fcf；复盘source_run_id与该run一致。

正式结算正确阻塞：运行快照runtime/none且冻结模型/Profile等身份不符，不能改贴real标签。分析UI依正式结算数保持阻断；独立原生后台完整分析任务实跑succeeded，返回1个QA运行/复盘样本，不作为真实模型准确率或UI Full。应用内原确认流程实际保存不可变报告a3b81761-0769-4423-bfee-3548d174965c，overall_status为blocked，不能因报告保存成功就算发布通过。

随后关闭并重启Windows原生进程：模型运行、复盘及不可变报告完整文档分别相等，另含提供器和QA隔离核验共5项通过。多次脚本定位/续接失败证据独立保留；自定义Select和原生确认对话框均通过产品原控件操作，未为脚本改产品UI。native-scoped-acceptance.json与native-restart-readback/report.json记录对象、源日志SHA和读回结果。4个原始会话日志原样合并仅分析覆盖：255条/34种完成操作、0运行错误；新版Full分析器仍明确缺失正式P4，退出1/BLOCKED。跨会话QA覆盖不抵销正式同对象Windows Full。

本批最终完整98步前端、TypeScript/Vite和35项架构退出0；33项推演工作流、30项规则生产视图及原挂载浏览器全部通过。首次前端在TypeScript回执getter控制流窄化处失败，改为原恢复存储直接读取后完整重跑通过，失败日志保留。最终结果见frontend-final-exit.json和architecture-final-exit.json。Rust生产树及锁文件相对此前c8f85f4d未变，复用该树已执行的格式/Clippy/777项普通测试（41默认ignored）及23项私有回归，不写成本批重跑。无生产依赖、迁移、命令、工作流、CSS或PNG基线变化。隔离PostgreSQL最终已停止，日常库和真实API/系统凭据未触碰。

## BA账本与场景交接

| BA | 本次实证 | 尚未关闭 |
| --- | --- | --- |
| 02 | 四项原综合测试分别通过 | 本项四个数据库回归义务已实证；不连带核销Full |
| 03～06 | 当前删除/引用/强删/配对/锁等待/取消/原子回滚/12插入1结束等原契约真实执行 | 桌面同对象完整操作仍随Full核对 |
| 07 | 两份实际XLSX及球队/球员月度导出回导经应用提交并跨连接读回，重复提交边界实跑 | 其余原生文件对话框、复盘包及桌面同状态Full覆盖仍需按原账本确认，不整组核销 |
| 08 | 运行身份、快照复用、事务故障回滚及任务幂等等原PG断言通过 | 私有正式运行及真实客户端未知响应恢复 |
| 09～11 | 原受控工作流回归与PG范围分开 | 真实API、用户会话取消、Windows凭据由用户明确暂缓，不报通过 |
| 12 | 原综合目标内不可变/回滚和工作流持久状态实跑 | 正式复盘→结算→分析/候选→J报告同对象链及B接入契约未获本次完整证明 |
| 13 | 已从本机完整归档恢复受保护资产；23项私有回归通过；真实P4/P7通过当前ApplicationService计算、路由落库、历史与重连读取 | 公开默认桌面仍明确Unavailable；本地私有Windows装配/P7同对象链及重启读回见11/09；正式P4、真实结算和Windows Full仍阻塞 |
| 14 | 20target逐项及应用真实XLSX补验完成 | Windows Full及最终安装候选证据仍必需 |

V1：初始账号无建库权限、专用库名被既有守卫拒绝、指定路径没有私有运行时分别如实记录；没有擅用日常库或放行Stub。V2：原PG并发/取消/回滚、实际工作簿提交及重建应用读回通过，保留各自边界。V3：01的startup和本次自动证据均没有被记成Full；外部必需项仍阻塞R17阶段出口。

## 继续条件与回退

下一步仍为R17-06：真实模型、固定回归、私有Windows装配及P7同对象链/重启读回已有实证；取得合格P4基础λ/Profile与真实结算证据后补正式P4及Windows Full。Windows入口路径阻塞已修复；API/系统凭据仍按用户要求暂缓。123须与对应受测提交一致；不能沿用旧程序结果。R18保持BLOCKED。

本批源码可回退至2101ce14；无迁移、用户配置或日常数据变更。临时新库保留测试导入数据，实例停止；后续需要重置仍只限该已批准隔离库。成功推送后仅按本批清单将无用临时文件归入O:/Project/FOODBALL/tmp/old，再逐文件实际清理并独立核空；保留证据、受依赖QA目录和稳定依赖缓存。
