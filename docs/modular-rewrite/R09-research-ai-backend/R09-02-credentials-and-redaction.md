# R9-02 Credentials 与 Redaction 实施记录

状态：`DONE`（修复自身Windows run37954536795全SUCCESS）。2026-10-09用户“收尾01开始02”授权。项目uniquenesssta/123、版本0.23.0；唯一R9分支 `rewrite/r9-research-ai-backend`，基线/回退 `b80f09e6488af18435c8e281395ddf7962c48eb5` 为01五文档收尾 `[skip ci]`，源码等于精确f3da62d/run37933341557全SUCCESS。02必须自身精确WindowsCI，不继承01或源码预期为02执行PASS；当前03已授权READY、04～11BLOCKED，R9整体IN_PROGRESS。

## 实际来源、为何修改与职责切换

原credentials.rs同时实现ApiKey输入校验/内存/Debug、异步来源选择、目标及三个档案操作、平台IO/原生缓冲与编码；api_example.rs同时解析协议和实现持久化模板脱敏。现credentials/mod只登记和显式export；key只持密钥生命周期，provider只持原ApiKeyProvider/Default来源选择，store持档案目标守卫与三个原公开操作，windows持全部原Windows原生IO及原非Windows映射，blob持无IO编码解码，error持共享MissingCredential映射，redaction持无IO递归脱敏、占位符及canonical curl。没有每函数一个空目录、第二provider、兼容转发壳或跨模块共享State。

原lib.rs的六凭据导出、ApiKeyProvider async参数/返回/Send+Sync、DefaultApiKeyProvider Debug/Default/Clone/Copy保持；实际调用方Application、Tauri profiles、共享transport不需改名。api_example保留解析、端点/双协议/候选/Token字段/警告和瞬时api_key DTO；只改为通过credentials出口一次调用sanitized_api_example。旧credentials.rs及解析器内四个脱敏helper/占位常量删除，唯一原职责owner已切换。整个client与crate根逐字保持；03后的协议、重试/circuit/cancel、正式研究/普通聊天、会话与diagnostics本轮不推进。

Windows native read/write/delete/exists的8个cfg函数逐token保持；CredReadW/CredWriteW/CredDeleteW/CredFree、普通凭据/LOCAL_MACHINE持久化、target/username/blob清理、1168不存在=false、重复delete不存在=Ok、OS错误与恢复原样保持。不增加真实凭据写入测试或调用用户凭据，所有新增测试均纯函数、读取明确缺席变量或在非法输入返回前硬停；原IO编译由自身Windows覆盖。

## 四个明确安全修复与兼容边界

1. 原ApiKey输入String虽及时zeroize，但拒绝的normalized副本未清理。两者现由锁定zeroize1.9.0的Zeroizing持有；原输入仍及时zeroize，成功用mem::take把normalized唯一所有权交给ApiKey；失败/临时销毁自动清理。原ApiKey String布局、Drop清理、固定Debug掩码、trim/空值/2560字节/Unicode空白政策和全部错误保持，不新增Serialize/Deserialize/Clone或公开expose。
2. 原save在目标校验失败前直接丢弃明文String。现目标检查前用Zeroizing持有，失败自动清理，成功take交给ApiKey；目标错误仍先于密钥错误，无新native调用或错误优先级变化。原240字节与ASCII字母数字及-_/与点号目标政策保持；target不是文件系统路径，原允许../openai不擅自改策略。
3. 原UTF-16两解码分支的units和临时String未清理。现四处Zeroizing自动清理；解码优先级、UTF16启发前8对、零终止、UTF8两端零/trim/嵌入零拒绝、偶数字节UTF16 fallback、原MissingCredential完整文案均保持。Rust拥有的原native bytes/blob清理保持，不擅自修改OS拥有的指针/释放策略。
4. 原固定Authorization占位符及sk-/sk_启发只处理完整字符串；compatible Bearer若回显在嵌套正文（含空格）或对象键中可能保留。现把本次已提取的非空非占位Bearer借用传入唯一脱敏owner，递归替换已知key在字符串/对象键中的出现，兼容任何提供器前缀；原启发/placeholder/canonical格式保持，无提取key的普通数据完全保持。只有含该已知凭据的模板数据刻意改变；raw body与瞬时candidate.api_key仍原样，模板不把真实key当作metadata写入。本项不是未知任意密钥检测器，未新增日志内容、持久化状态、配置键或真实API请求。

正式研究仍只Responses；普通问答无tools/web search/schema/token字段串线；现有配置/profile、rollback、密钥掩码、本机元数据格式、连接状态、Tauri命令与UI无源码变动。GatewayError category/DTO/provider_status/provider_code/用户提示/recovery及107条原生产literal保持。公开contract仅artifacts登记从旧单文件改到8个实际owner路径，contract/schema/release版本及全部政策/commands不变，不是公共DTO或协议升级。

## 完整实际文件清单

按基线到最终工作树 `git diff --no-renames --name-status` 并含新增文件核对：**9A / 9M / 1D**，无整文件移动/重命名。

### 新增（9）

- `crates/research-gateway/src/credentials/blob.rs`
- `crates/research-gateway/src/credentials/error.rs`
- `crates/research-gateway/src/credentials/key.rs`
- `crates/research-gateway/src/credentials/mod.rs`
- `crates/research-gateway/src/credentials/provider.rs`
- `crates/research-gateway/src/credentials/redaction.rs`
- `crates/research-gateway/src/credentials/store.rs`
- `crates/research-gateway/src/credentials/windows.rs`
- `docs/modular-rewrite/R09-research-ai-backend/R09-02-credentials-and-redaction.md`

### 修改（9）

- `README.md`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `contracts/openai-profile-ui-contract.json`
- `crates/research-gateway/src/api_example.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md`
- `docs/modular-rewrite/R09-research-ai-backend/README.md`
- `scripts/verify-openai-profile-ui.mjs`

### 移动/重命名

无。旧多职责单文件按owner拆开，不以整文件改名掩盖边界。

### 删除（1）

- `crates/research-gateway/src/credentials.rs`；不保留重复实现或forwarding shell。

架构清单只更新ApiKeyProvider.owner到provider.rs；Domain生成器登记扫描1103→1110及rustUsageDigest `8342f5a4f4779b3a60a206e5018c479b47e6bab70bdcad054d57c8b331d182e8`。365Domain/300数据库映射/sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd`、Application402/43Ports、171命令、PG根35/46迁移与18保护资产保持。

## 原测试与12项新增边界测试

原4凭据与3 API Example测试逐token保留，原Gateway其余/transport8项、contract16全部保留。新测试仍原Gateway unit target；源码预期 **35**（23+12），contract **16**、Application **126**、Persistence **175**；须本项Windows实际确认，不记源码预期为PASS。

- key +2：空/无效完整原错误与Debug/Display/JSON错误无输入回显；2560/2561字节、Unicode字节、两端trim及嵌入Unicode空白。
- provider +2：桌面/大小写/尾空格server模式在变量读取前拒绝；显式server缺席变量保持原category/recovery。只读取明确缺席专用fixture变量，不修改进程环境或引入并发共享测试状态。
- store +2：240/241字节及原字符政策；三个公开操作非法目标均在nativeIO前停止，目标错误优先与非法密钥错误无回显。不给真实凭据读写/删除增加测试副作用。
- blob +2：UTF8两端NUL/trim、UTF16终止与非ASCIIfallback；空/空白/零/非法UTF8/孤立代理项/嵌入零的完整原错误及无provider metadata。
- redaction +3：嵌套array/object及对象键内已提取的compatible key、前后文本保留、固定认证头；原sk启发/placeholder兼容；无key和空可选key的普通JSON及canonical原样。
- 原api_example +1：通过公开parse_api_example验证任意compatible Bearer仍提取到瞬时字段，model/协议保持，嵌套模板实际只保留占位符，覆盖真实调用接入而非只测helper。

## 实际验证与报告

实际静态PASS：83/83既有源码检查（原5浏览器项留Windows）、完整npm run verify:architecture、原兼容API/transport/Domain/源码卫生、18保护资产指纹、171命令、46迁移/18PG静态、Rustfmt1.88源码format/check、git diff --check。保护聚合 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`；PG静态聚合 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`。只做非Windows源码/静态，没有Linux/macOS Cargo、编译、单测、loopback或浏览器动态。

8个破坏探针：拒绝normalized清理丢失、save早返输入清理丢失、桌面环境隔离反转、target边界漂移、native释放丢失、UTF16临时清理丢失、已提取key接入丢失、嵌套字符串脱敏丢失。均被原增强openai-profile-ui验证器拒绝，并恢复所有文件后原门禁通过；不新增runner/workflow/target/数据库/依赖或独立持续回归体系。

等价核对：8原native cfg函数、8原政策/生命周期函数、原provider implementation、4凭据与3示例测试、3 canonical/placeholder/heuristic函数逐token保持；解码去除显式RAII包装后逻辑tokens保持；107生产literal保持；crate根和client逐字保持。四项有意安全差异已逐条记录，不谎称整体完全无行为变化。

报告：`/workspace/scratch/eb298ad5cdcb/r902-static-checks.json`、`r902-architecture.log`、`r902-equivalence.json`、`r902-negative-probes.json`。首轮83门禁82通过，最后一条新增test改动后Domain使用摘要漂移；已在最终格式化源码后重跑官方生成器及83/完整architecture全部通过，不放宽门禁。

待自身Windows：原前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace（含12项新增unit）、release/MSI/NSIS/启动。沿用Public Platform CI，核实自身精确head与Windows job开始后停止轮询；02VERIFYING、03～11BLOCKED。真实PostgreSQL、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共unavailable stub不计PASS。

## 工具、设计决定与回退

Context7按实际Cargo.lock zeroize1.9.0准确版本核对Zeroizing::new/Drop、Deref/DerefMut与alloc String/Vec；初次resolve说明问题写1.8.2但随源码立即确认为1.9.0，实际查询与应用均1.9.0。Windows-sys0.61原系统调用只迁移、没有新增API；不升级外部依赖或照搬latest接口。Mermaid Chart已展示示例→唯一脱敏→原metadata，以及Windows/server provider→ApiKey→共享HTTP真实链路。Create State只保存足球稳定model `96f03270-c236-46de-99fa-db85d2fbf4ce`；合成摘要不替代Git/精确CI，也不能把Clippy误记为本地已执行。

回退到 `b80f09e6488af18435c8e281395ddf7962c48eb5`，受控revert本项源码/路径清单/原门禁和文档，恢复旧唯一owner与公开export；不手工复制双实现、不覆盖用户修改、不变更历史数据。下一步核实02自身精确Windows：成功才能收尾，失败只修完整相关链路；03仅用户授权后启动。真实PostgreSQL、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共unavailable stub不计PASS。


## 首轮 Windows 失败与本次修复（2026-10-09）

首轮精确HEAD `98d542fe183cefa94aadb05a66235e1c51ef3867` / [run37944445186](https://github.com/uniquenesssta/123/actions/runs/37944445186) / job113866973203已FAILURE，2026-10-09T14:35:28Z结束。日志唯一Rust错误位于credentials/provider.rs:88～91，新增测试使用 `format!("{:?}", DefaultApiKeyProvider)`，Rust1.88 Clippy建议 `format!("{DefaultApiKeyProvider:?}")`；`-D warnings`使uninlined_format_args阻断lib test检查。前端/contracts/TS/Vite/17视口已成功，Rustfmt阶段已进入Clippy，但workspace单测、Windows release/MSI/NSIS/启动没有执行；不能计Gateway35或后续交付为PASS。失败过程报告 `D:\a\123\123\logs\windows-acceptance-20261009-143026.txt`；失败artifact11623485055，671字节，SHA256 `c190abf69b58d3daa6f4f2c1d3512270e9100ff6c183f3399996a21b15106fdb`，不是完整成功验收包。

本次只将该Debug断言的format变量内联；预期字符串仍DefaultApiKeyProvider，测试名/数量/条件与生产实现保持。检查全部02凭据/示例格式化调用，没有其他新增的同类可内联标识符参数遗漏；原配置字段访问及repeat/OS错误表达式不是该处标识符形式。沿用原源码卫生门禁的已知Clippy回归检查机制，给provider owner增加可匹配原写法/任意换行与可选尾逗号的拒绝断言；没有移除测试、允许告警、改变production Debug或泛化新的验证体系。

本次修复相对首轮98d542f的完整清单：**8M / 0A / 0D / 0移动重命名**（此前19文件的初始实施清单仍有效，以下为额外修复提交）：

- `README.md`
- `architecture/domain-type-inventory.json`
- `crates/research-gateway/src/credentials/provider.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md`
- `docs/modular-rewrite/R09-research-ai-backend/R09-02-credentials-and-redaction.md`
- `docs/modular-rewrite/R09-research-ai-backend/README.md`
- `scripts/verify-rust-source-hygiene.mjs`

provider生产段与原提交逐字一致，完整文件仅一个format表达式不同；原Gateway35测试（12新增）/contract16、Application126/Persistence175数量保持，production error/config/template 107literal等价及原职责核对仍通过。其他源码、UI/Tauri/Application/PG/Domain、契约/Schema、依赖/锁、workflow保持；清单仅同步rustUsageDigest `f343a8846085754e5bce01af9eee24dc327a4a475df054273826375ecc57294c`，Domain365/300/声明摘要/扫描1110保持。

实际本地静态：83/83既有检查、完整verify:architecture、Rustfmt1.88源check、18保护资产/171命令/46迁移18PG静态、diff、原等价通过。三破坏探针还原原失败表达式、跨行表达式、尾逗号表达式，均被原增强卫生门禁拒绝，并逐字恢复后通过；未启动Rust代码或非Windows动态。报告 `/workspace/scratch/eb298ad5cdcb/r902-fix-review.json`、`r902-fix-static-checks.json`、`r902-fix-architecture.log`、`r902-fix-equivalence.json`。

Context7按已确认Rust1.88.0查询Clippy，但仅返回master的lint开发/配置内容，没有提供准确版本证据；不用该结果冒充1.88验证。该具体修复以精确Windows 1.88日志中的Clippy建议为依据，无不确定第三方API或升级。本次单测试格式修复不改变架构数据链，沿用上一轮真实Mermaid图；Create State按已核实足球稳定model保存新失败/修复/待验信息。

本次修复回退点98d542f；整个02回退点仍b80f09e。本项保持VERIFYING，03～11BLOCKED；修复提交自身Windows CI确认开始后停止轮询，成功后再收尾，失败继续只修相关链路。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承删除风险仍最终新库待验，ignored/公共unavailable stub不计PASS。


## R9-02 精确 Windows 收尾（2026-10-10，DONE）

精确修复 `05055d1fd90e9a19ee4bc791e769018ca700dd14` / [Windows run `37954536795`](https://github.com/uniquenesssta/123/actions/runs/37954536795) / job `113901577714` 全 SUCCESS，2026-10-10 00:15:32北京时间完成。Gateway **35/35**、contract **16/16**、Application **126/126**、Persistence **175/175**，新增12项及原凭据/示例/传输测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动7条记录/3完成操作均通过。证据artifact `11628233815`，14,037,356字节，SHA-256 `9a738defde297f2ff10bc0218885efa72d9e7f2c9ad4ff999bcf237cafc0e45a`。

首轮 Clippy 失败已由该修复自身精确 Windows 关闭。当前02 DONE；用户“收尾02开始03”授权同一R9分支03 READY，04～11 BLOCKED，R9整体IN_PROGRESS。本次只同步既有五份文档，源码/验证器/依赖保持，使用 `[skip ci]` 复用上述精确源码结果；此前VERIFYING叙述为实施历史。真实PG、历史四项/账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored和公共stub不计PASS。
