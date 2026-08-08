# 足球模型平台公开外壳

当前版本 **0.23.0**。本仓库是平台、数据库、数据准备、路由、工作台与外部模型调用入口的公开版本；真实 P4/P7 预测引擎、参数、Profile、固定比赛、私有研究提示词及模型专用固定回归资产不随仓库分发。

## 公开边界

- 保留 `crates/model-api`、模型 ID、路由、规则包入口、预测页面和历史数据结构。
- 使用 `crates/model-stub` 注册外部模型入口。未接入 ModelProvider 时，预测明确返回“运行时未分发”，不会静默回退、生成伪结果或使用隐藏默认参数。
- 公开规则包只保存外部提供器标识与通用输入输出契约；参数生成、校准、晋升和真实运行由私有或独立部署的 ModelProvider 负责。
- 私有资产由 `.gitignore` 和 `scripts/verify-public-model-boundary.mjs` 双重阻断。

## 构建与验证

```powershell
npm run setup
npm run verify:frontend
npm run verify:rust
node scripts/verify_protected_assets.mjs
node scripts/verify_command_contract.mjs
node scripts/verify_database_baseline.mjs
npm run tauri:dev
```

PostgreSQL 数据库基线必须使用名称包含 `test` 的专用、允许彻底清空的测试数据库：

```powershell
$env:FOOTBALL_TEST_DATABASE_URL="postgres://user:password@127.0.0.1:5432/football_test"
node scripts/run_database_baseline.mjs
```

Windows 可使用：

```text
启动平台.bat
验证平台.bat
验收平台.bat
```

`verify:frontend` 包含公开模型边界、Domain 类型清单漂移、Node 调用链兼容、Windows 路径契约、TypeScript、静态契约、截图和 Vite 生产构建。TypeScript 与 Vite 使用当前 Node 执行包内 JavaScript CLI，不直接启动 Windows `.cmd` 包装器。Windows 验收器从 `.cargo/target-location.json` 解析实际 Cargo target，并支持相对于项目根目录的 `LogDirectory`；应用 runtime 日志写入运行时根目录的 `logs`，开发态 runtime root discovery 可能解析为源码根目录上一级。`verify:architecture` 包含模块边界、状态所有权、受保护导入和 Domain 类型清单漂移门禁。`verify:rust` 包含 Cargo.lock 一致性、格式检查、Clippy 与工作区测试。`verify_protected_assets.mjs` 校验模型公开边界文件指纹、保护目录精确集合以及私有 P4/P7 资产缺席状态。`verify_command_contract.mjs` 校验前端调用、Rust 命令定义和 `generate_handler!` 注册集合一致，并拒绝缺失、重复、孤立或未授权动态命令。`verify_database_baseline.mjs` 校验 0001–0046 迁移连续性、内容指纹、SQLx 迁移入口、PostgreSQL 集成测试集合和关键不可变约束。`run_database_baseline.mjs` 在静态门禁通过后执行被忽略的 PostgreSQL 集成测试，并拒绝数据库名不含 `test` 的连接。
`Public Platform CI` 是 Windows 自动交付门禁：对 `main`、`new-*`、`rewrite/**` 的推送、Pull Request 和手动触发执行架构契约检查及 `scripts/windows-acceptance.ps1 -Mode Automated`，并保存验收日志和 release bundle 证据。云端 Automated 不替代最终真实 PostgreSQL、Windows Full 交互和用户本机验收。

Node 开发依赖固定安装和读取自源码根目录上一级的 `../node_modules`，npm 缓存固定使用 `../.npm-cache`；仓库根目录不再保存 Node 依赖目录。Cargo target 继续使用 `../.cargo-target`。

## 模块化重写执行记录

- `new-A` 已从 `main` 基线提交 `db79995873460688c15abb3497bf1c61b73ffb18` 建立。
- `new-B` 已从 `new-A` 提交 `36d34ba1ff73cbec575cf58594aa8c0329669496` 建立；R1-01 已创建模块边界与状态所有权契约并完成 Windows 自动化门禁，状态为 `DONE`，R1-02 已开放为 `READY`。
- R1-02 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，接入 `npm run verify:architecture`、前端聚合验证和 Windows CI 独立步骤；状态为 `DONE`，R1-03 已开放为 `READY`。
- R1-03 已建立 `src/bootstrap/` 浏览器组合根并切换 `index.html` 唯一入口；`src/main.ts` 仅保留既有业务实现并暴露受控生命周期。Windows workflow run `31012168809`、job `92326905405` 在提交 `a3b61088abaf0c9f052ecab09e040ea77bd8d344` 上通过，artifact `8933800016` 大小 `14117539` 字节，SHA-256 为 `4c28e5668b8b330cbab5b54516af1d70fe9f39c8299bb640da06a5b4442667f9`；状态为 `DONE`，R1-04 已开放为 `READY`。
- R1-04 已建立 `src-tauri/src/bootstrap/` Tauri 组合根，拆分 Builder、全局状态、171 条命令注册和启动错误映射；状态为 `DONE`。
- R1-05 已建立 `crates/application/src/` 下的 Application 组合根、兼容服务门面、模型注册表和持久化端口注册入口；默认模型注册与 PostgreSQL 具体导入均已收敛到唯一所有者，公共 API 和行为不变。专项、架构、前端、Rust 及正式 Windows Automated 全部通过；workflow run `31073166446`、job `92525208547` 在提交 `08803725dcd9f403ffc25552c27d2a9c0d3acd2d` 上通过，artifact `8956912712` 大小 `14117884` 字节，SHA-256 为 `495d3c5e29f2b474e97b98f89dd64b6175cc6e9496dd77681c0a567e00c60016`。状态为 `DONE`，R1 阶段已关闭。
- R2-01 已建立可机器复算的 Domain 类型与调用链清单、目标模块归属策略和 Serde 契约测试。清单登记 365 个公共兼容类型、20 个 Domain 来源文件、139 个 Rust 扫描文件和 299 个 PostgreSQL 映射类型；生成与全量门禁 run `31077537198`、job `92538743873` 已通过。正式 Windows Automated run `31078483578`、job `92541654912` 在最终实施提交 `2a6b9ea96a88168d6a751ebf48c2030512edaf24` 上通过；artifact `8959079579` 大小 `14118091` 字节，SHA-256 为 `6b75cc3abe2067472476cd0e7811b9fd9ee6f689f17cbc0eb346030775d0c9e2`。本节点未迁移或修改任何 `crates/domain/src` 生产类型，状态为 `DONE`，R2-02 已开放为 `READY`。
- R2-02 已将 14 个 Competition 类型和 8 个 Routing 类型从 Domain 根文件迁移到职责目录；正式 Windows Automated run `31088698579`、job `92574240109` 已通过，artifact `8963219366` 大小 `14117627` 字节，SHA-256 为 `5bc1481807b8fd378e8845c1a813e0cca3981681a5f5d89c572a54e74c973124`。状态为 `DONE`。
- R2-03 已将 Team 17、Player 21、Coach 9、Formation 8、Shared 17 共 72 个类型从 Domain 根文件迁移到职责目录；根级类型名、Serde、数据库映射、Application、Tauri DTO、模型边界和生产依赖保持不变。实施 run `31100515822` 已通过；正式 Windows Automated run `31110013068`、job `92645025258` 在与实施提交 `038ebd7096a78f7202d9c98e66e17d32701d343c` 同源码树的触发提交 `594940dca4c57aabfebdd768755ec27006ecaeb5` 上通过，artifact `8972168972` 大小 `14119500` 字节，SHA-256 为 `6ef2e064638cd17b214c66bbdea5ed752a08a1f0dc32002940e0f97d094cae5f`；运行报告为 PASS，7 条记录、3 个完成操作。状态为 `DONE`，R2-04 已开放为 `READY`。
- R2-04 已将 Lineup 16 个类型和 Match 3 个类型迁移到职责目录并删除旧 `crates/domain/src/lineup_chain.rs`；根级公共类型路径、Serde、数据库映射、Application、Tauri DTO、公共命令、生产依赖与模型保护边界保持不变。实施 workflow run `31151412918` 已通过专项 Serde 9/9、架构、前端、Rust、Clippy、workspace tests、精确变更集、legacy 删除、README 契约、transient 清理、提交后工作树和 push 后远端 HEAD 校验，并生成最终实施提交 `0aafe42d7ed08f8e78d71d44ccb6f8f58c425999`。正式 Windows Automated run `31153982572`、job `92789397631` 已在该最终提交上通过；artifact `8984980586` 大小 `14118155` 字节，SHA-256 为 `1e7224f4e7f713b0339e97fd114fa6dea2c0b2ecc9400789613fe872d660938c`。R2-04 状态为 `DONE`，R2-05 已开放为 `READY`。
- R2-05 已按两阶段迁移 Prediction 48 个类型与 Research 27 个类型。Prediction 独立提交 `2cd685b8057a1bce2f75e4c7f5b56aed1bf3d142` 的专项门禁通过；首次 run `31158780693` 中 Research 迁移与专项 Serde 11/11、完整 frontend 均通过，但完整 Rust 在 Clippy `-D warnings` 因 `prediction/orchestration/planning.rs` 两个未使用 import 停止，未提交 Research。已直接删除两个无效 import，不增加抑制；恢复 run `31159821513` 完成 Research 27 类型迁移、类型清单、架构、保护资产、frontend 与 Rust 全量回归。旧 6 个职责混合源文件均已删除，根级公共类型路径、Serde、数据库映射、Application、Tauri DTO、生产依赖和模型保护边界保持不变。正式 Windows Automated run `31171082098`、job `92842834091` 已在包含最终 R2-05 源码树的提交 `e328b4aa5a7737e6bb378abf8b891cd953b99f62` 上通过；artifact `8991618221` 大小 `14117154` 字节，SHA-256 为 `71320b8ef97e62be2fe2323327d21f4870476092ad024d7b8c2c26a4ade9dc59`。R2-05 状态为 `DONE`，R2-06 已开放。
- R2-06 已将 Review 48 个类型与 Postmatch 11 个类型迁移到 `review/`、`postmatch/` 职责目录并删除 5 个旧职责混合源文件；根级公共类型路径、Serde、数据库映射、Application、Tauri DTO、公共命令、生产依赖和模型保护边界保持不变。staged 与 Windows Automated 验收已通过，用户随后使用原有 PostgreSQL 数据库连接成功；数据库兼容修复保留历史数据与不可变资产，不清库、不覆盖旧版本、不削弱 fail-closed 或内容指纹保护。R2-06 状态为 `DONE`。
- R2-07 已将 Analytics 39、Exchange 54、AI Workspace 16、Release 9 共 118 个公共兼容类型迁移到职责目录，并删除 7 个旧职责混合根文件；Windows 本机格式、17/17 Serde、365 类型清单与架构门禁通过，用户确认状态为 `DONE`。
- R2-08 已将 `crates/domain/src/lib.rs` 收敛为 17 个模块声明、365 个显式公共兼容类型 re-export，并补齐 34 个既有公共根常量的显式兼容出口；根级 glob export 已清零，三个私有默认值实现迁入 `shared/defaults.rs`，确定性根出口生成/验证门禁已接入 `verify:architecture`。Windows 专项 run `31236344727` 已通过根出口、球队资料包和保护资产门禁并生成最终实现提交 `62b1f622b9c14b33dbaac850812a49c063ccb090`；用户本机阶段回归未见报错，上传 runtime 日志共 58 条且全部为 `info`，启动 `connection_error=null`，球队、阵容、分析、Postmatch 与 API 工作区读取均正常完成。R2-08 状态为 `DONE`，R2 阶段已关闭。
- R3 已从 R2 完成提交 `7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f` 建立独立分支 `new-C`。R3-01 Application Ports、R3-02 Database Service、R3-03 Competition / Rules Services 均已完成。R3-04 已将 35 个球队/球员/教练/实体引用职责拆入 Teams / Players Services，历史状态继续独立保留为 `VERIFYING`。R3-05 已删除旧 `crates/application/src/player_catalog.rs`，将剩余 19 个阵型/比赛/阵容/阵容预设职责迁入 `services/lineups/` 与 19 个对应 Use Cases，并以 `FormationPort`、`MatchCatalogPort`、`LineupPort`、`LineupPresetPort` 4 个既有 Ports 保持公共 Application/Tauri 契约；`read_match_exchange` 仅提升 workspace 可见性，SQL、参数、返回结构与数据库行为不变。clean 实施提交 `7e3fddeafcd32cc45e293fa9a7aeb05c7c66d4ec` 的 Public Platform CI run `31260698438` / job `93110942400` 已通过完整 Windows Automated；用户随后在最终分支完成 clean 工作区、rustfmt、Lineups 专项、architecture、Application 33/33、完整 frontend、完整 Rust/Clippy/workspace tests 与 `tauri:dev`。本机 runtime JSONL 共 280 条，除 3 条预期输入校验与 3 条公开模型运行时未分发的既有错误外，无 Lineups、SQL、migration、panic 或连接失败；预设保存、应用预检、双方阵容原子创建与 `ready_for_model=true` 阵容链均实际跑通。R3-05 状态为 `DONE`。R3-06 Prediction Service 已进入 `IN_PROGRESS`：Atomic Task 1 已完成 Prediction Core 模块化迁移；Atomic Task 2A 已完成 P4 planning / freeze readiness / read-only workspace 迁移及专项硬门禁。Research 冲突写入、Evidence/Fact 写入与联网 Research 执行仍明确保留给 R3-07，R3-06 尚未关闭。
- 开发依赖布局已外置：Node 依赖固定到 `../node_modules`、npm 缓存固定到 `../.npm-cache`，Cargo 构建输出继续使用 `../.cargo-target`；仓库根目录不保存依赖目录。
- R1-04 前置校正将 `crates/application/src/model_shell/mod.rs` 恢复为 Rust 1.88 标准排版，并只同步更新该文件的保护指纹与派生聚合值；导出集合、模型行为和保护范围均未变化。
- R1-04 已同步迁移 19 个既有验证器读取新的 Tauri 命令注册表或状态所有者，消除旧 `lib.rs` 路径造成的伪失败；产品代码和公共契约未改变。
- R1-04 Windows Automated 启动烟测改为按启动前日志路径集合识别新 session，并在首次 45 秒超时时最多重启一次；每次启动保留 stdout/stderr，连续两次超时仍硬失败。打包前后 EXE 的 A/B 运行均正常，产品入口、bundle、命令和业务行为未改变。
- R1-04 workflow run `31037323146`、job `92412650719` 在清理后代码树提交 `5cb66fdedbfcaf89c86a7124f8894bdc71a533c9` 上通过；artifact `8943939773` 大小 `14119217` 字节，SHA-256 为 `7562c9137d52040627a58d9c8e104c4053b9923983a16daae08e4361e9a78f2b`。Automated 报告为 PASS，7 条运行记录、0 条无效记录、0 个运行时错误，release 客户端首次启动即建立日志。
- R1-02 最终 workflow run `31001470224`、job `92291121763` 在提交 `28ec363babe4f3fbccd14693d0261febdc305458` 上通过；artifact `8929207011` 大小 `14117150` 字节，SHA-256 为 `e83b2ab9c6cb705d0bfd740c798673a45dc2a4cb0b7b35ddebe844bb40b13e88`，Automated 报告为 PASS，7 条运行记录、0 条无效记录、0 个运行时错误。
- 截图启动工具仅对 Chromium `DevToolsActivePort` 的 `EBUSY`、`ENOENT`、`EPERM` 和未完成端口内容执行最长 15 秒的有界重试；其他错误立即失败，截图差异阈值与门禁强度未放宽。
- `Public Platform CI` 现支持推送到 `main`、`new-*`、`rewrite/**`、Pull Request 和 `workflow_dispatch`，以 `windows-2025` 执行架构契约、前端、Rust、Tauri Windows release、release 客户端启动和运行日志扫描，并上传验证证据。
- R1-01 验证运行 `30989439570`、job `92251837163` 在提交 `fc02ad51d01229cb2ea62fc20f623910ba49de7f` 上通过；artifact `8924033934` 大小 `14115361` 字节，SHA-256 为 `85551aacdd43ba1e3516025ae510aefaaa8e11d61f433a701eaa884e292a47a1`，Automated 报告为 PASS，7 条运行记录、0 条无效记录、0 个运行时错误。
- 新增 `.gitattributes` 固定文本 LF 和二进制排除规则；相关验证器统一按 LF 规范读取冻结合同，避免 Windows 检出换行导致伪失败。冻结合同、迁移哈希、锁文件、生产依赖、公共命令、数据库结构和模型保护资产均未改变。
- R0-01 已冻结远端分支起点并建立 `docs/modular-rewrite/R00-baseline/` 节点记录。
- R0-02 已新增 `architecture/protected-assets.json` 和 `scripts/verify_protected_assets.mjs`，冻结 18 个公开模型边界及校验文件，聚合 SHA-256 为 `d2263a5ff09c8cf633a42b7bb35fffe3d42fb18648db4d12691817f51015c85c`。
- R0-03 已新增 `architecture/command-contract.json` 和 `scripts/verify_command_contract.mjs`，冻结 171 个公共命令、15 个 Rust 命令模块和前端调用边界。
- R0-04 已新增数据库静态基线与安全执行入口，冻结 0001–0046 共 46 个迁移，聚合 SHA-256 为 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`。真实 PostgreSQL 验证按用户要求留到最终统一验证。
- R0-05 workflow run `30910130867` 中，Linux `npm ci` 通过；Chromium 启动失败。Rust locked metadata 通过，但 `cargo fmt --check` 失败，Clippy 与 workspace tests 未执行。
- R0-06 workflow run `30912862564` 建立 Windows 基线。Windows release 构建和 RuntimeOnly startup 通过；startup report 为 PASS，7 条记录、3 个完成操作、0 个无效行、0 个运行时错误。
- R0-06 精确 Automated 暴露 Windows Node 调用问题及验收路径契约问题。
- R0-06.1 新增 `scripts/process/execution-context.mjs`、`scripts/process/node-package-cli.mjs` 和 `scripts/verify-node-process-compatibility.mjs`，关闭目录联接依赖同步与 `.cmd` 子进程调用缺口。Windows workflow run `30919764753` 中完整 frontend 通过，Automated 到达 Rust 阶段。
- R0-06.2 新增 `scripts/windows/acceptance-paths.psm1` 与 `scripts/verify-windows-path-contract.mjs`，支持根入口既有 `LogDirectory` 参数，并按 Cargo target 登记文件查找 release EXE。
- R0-06.2 Windows workflow run `30922384735` 中，完整 frontend、release 构建与 RuntimeOnly runner 均实际通过。release EXE 从项目根目录 `.cargo-target\release` 启动；startup report 为 PASS，7 条记录、3 个完成操作、0 个无效行、0 个运行时错误。
- R0-06.2 证据 artifact 为 `8898312587`，SHA-256 为 `d6ed06066aab354686f86938ec7c55f2c1f740e11a37e42a6a1b5edbbd53df63`。临时 workflow 的 job 最终因产品验证结束后的一条辅助中文日志精确匹配未命中而显示 failure；证据文件已复核，未将 workflow 总体描述为通过。
- R0-07 使用 Rust 1.88.0 rustfmt 对 42 个已诊断 Rust 文件进行纯格式规范化，实施提交为 `9e7be511ae2d97a0782fee1a2bea5e25d910d10d`；未触碰模型保护文件、依赖、锁文件、迁移或公共接口。
- R0-07 精确 workflow run `30961535208` 中，Cargo.lock 门禁、Cargo target 准备和 `cargo fmt --all -- --check` 通过。完整 `npm run verify:rust` 随后在 16 个 Clippy 错误处以退出码 `101` 结束，workspace tests 因 fail-fast 未执行。
- R0-07 精确验证 artifact 为 `8913160029`，SHA-256 为 `47712408cb9fbd37088f42cab92e71565b0c982d5c0492a78fb6c4ef2e53ad49`。
- R0-08 以 11 个 Rust 文件白名单关闭 Clippy 门禁，实施提交为 `919d62a2eaf95ade5ba1efa18924a9d578ef3f63`；没有添加 `allow`、放宽 `-D warnings`、修改依赖、迁移、模型边界或公共接口。
- R0-08 workflow run `30965687503` 中，持久化库测试 73/73、Tauri runtime log 专项 7/7、`cargo clippy --locked --workspace --all-targets -- -D warnings` 均通过。表格库完整测试为 11/12，通过替代验证确认除既有空白行用例外其余 11 项通过。
- R0-08 精确 workflow run `30966064295` 中，Cargo.lock、rustfmt 和 Clippy 连续通过；workspace tests 首个失败为 `openai_research::tests::built_in_gateway_is_strict_and_has_no_secret`，因此完整 `npm run verify:rust` 以退出码 `101` 结束。
- R0-08 最终应用 artifact 为 `8914718704`，SHA-256 为 `b8e75726c6ad53bdb4932ceb0bb3d35ff4554f306179178e6a566187723c6c60`；精确验证 artifact 为 `8914844238`，SHA-256 为 `05ee24344468b9613bf18c139ff7d3aabecb92e005f93afb1f9037ed7f21cede`。
- R0-09 先以 `--no-fail-fast` 完整扫描 workspace tests，确认失败集合只有 OpenAI 提示词大小写断言和表格稀疏测试工作簿两项。
- R0-09 仅修正两个测试契约，共新增 2 行；实施提交为 `50daa258af8ac8e09f8e4f5f428249fe670f2dd2`，未改变生产逻辑、公共接口、数据、配置、依赖、迁移或模型保护边界。
- R0-09 workflow run `30967448070` 中两个专项测试、185 个 workspace 测试以及精确 `npm run verify:rust` 全部通过；18 个 PostgreSQL 集成测试保持忽略并按用户要求留到最终统一验证。
- R0-09 最终 artifact 为 `8915431192`，SHA-256 为 `a745bded71179bb6542d3a06b5c65f61cdf48845b8f88193dc7ef0ac5c8fcadc`；临时 workflow、patch 与 Draft PR #6 均已清理或关闭。
- 用户于 2026-08-05 明确将目标平台收敛为 Windows，Linux Chromium 不再属于交付或阶段门禁；PostgreSQL 实跑、Windows Full 与用户本机实机验收统一延期到最终验收。R00 已完成并开放 R1，详见 `docs/modular-rewrite/R00-baseline/R00-stage-completion.md`。
- R0-01 至 R0-06.2 未修改前端或 Rust 业务源码；R0-07 只改变 Rust 排版；R0-08 仅实施行为等价的私有参数收束、Copy/借用修复、无效私有代码清理和测试编译补全；R0-09 只修改测试构造与断言。
- 当前执行环境未建立本地 Git 工作树，用户设备上的未提交与未跟踪文件不可见；远端分支操作不会覆盖这些本地内容。

## 0.23.0 变更记录

- 公开重写后的内置 P4 evidence、prematch snapshot、research output Schema 与研究 Prompt 使用独立不可变内容版本（`1.0.0+public.1` / `2.0.0+public.1`）；旧数据库中的同名旧版本继续保留，启动时不覆盖历史内容或放宽指纹校验。
- 删除真实 P4/P7 模型 crate、参数、Profile、固定比赛、模型专用契约、Schema、研究资源和验证脚本。
- 新增外部 `ModelProvider` Stub、通用模型边界契约、公开研究资源和明确不可用错误语义。
- 移除公开代码中的固定模型矩阵拓扑、固定比分单元数、校准常量、时间前推参数算法和自动参数候选生成。
- 保留赛事推演、模型路由、规则包、快照、复盘、分析和数据库入口；未连接外部提供器时不会执行预测。
- 新增 GitHub Actions 自动验证，减少本机重复下载和手工测试。

## 历史兼容记录

以下标题用于保留原项目的可追溯历史契约；不表示本次公开拆分重新执行了所有历史验收。

## 0.22.0 变更记录

接入点 H 的不可变赛后结算、证据评分和监控工作流保留。既有 UI 不重复重构；公开版本只调整模型执行边界。

## 0.19.0 变更记录

工作区 UI、双层导航和页面状态保持历史兼容。

## 0.15.0 变更记录

实体关系、球队与球员管理链路保持历史兼容。

## 0.14.0 变更记录

API 协作工作台、OpenAI Profile 与运行日志链路保持历史兼容。

## 0.13.5 变更记录

API 传输与诊断契约保持历史兼容。

## 0.13.4 变更记录

球队资料、导入和历史记录能力保持历史兼容。

## 0.13.3 变更记录

球队与球员管理修复保持历史兼容。

## 0.13.2 变更记录

球队与球员管理基础链路保持历史兼容。

## 历史能力索引

- 默认战术角色全链路：Excel、档案、阵容、复盘与来源审计。
- 球队完整资料包与 P4 输入就绪度历史工作流；历史数据记录包含 1248 条世界杯球员俱乐部关系增量补录。
- 强制删除全部资料使用 `football.force_purge` 审计边界；永久删除预检会处理陈旧标签页。
- 导入行子记录身份修复保留同一物理行内多实体身份。
- Windows 实机全链路验收阶段 5 由 `验收平台.bat` 启动。

## 验证事实与限制

R0-06.1 已关闭 Windows 目录联接依赖同步和 `.cmd` 子进程调用缺口；R0-06.2 已关闭 `LogDirectory` 参数和 Cargo release 查找路径缺口。Windows 完整 frontend、release 构建与 RuntimeOnly startup 均已有真实通过证据。R0-07 已关闭 Rust `cargo fmt --check` 阻塞；R0-08 已关闭 workspace all-targets Clippy `-D warnings` 阻塞；R0-09 已关闭全部已枚举 workspace tests 失败，精确 `npm run verify:rust` 现已通过。

R00 阶段已按 Windows-only 目标范围标记为 **DONE**。Linux Chromium 历史失败仅保留为非目标平台记录，不再阻塞后续重写。

PostgreSQL 实跑、Windows Full 和用户本机 Windows 10/11 实机验收仍保留最终统一验收；R2-06 节点已额外使用原 PostgreSQL 数据库完成连接验证。R3-02 已使用原数据库完成非破坏性 `tauri:dev` 运行时烟测，但真实 destructive reset 仍只允许在专用测试数据库执行。另保留 1 个 moderate npm vulnerability 和 Vite 大 chunk 警告。

已创建 `R00-stage-completion.md`、`R01-stage-completion.md` 与 `R02-stage-completion.md`。R1、R2 阶段均已关闭；R3-01 Application Ports、R3-02 Database Service、R3-03 Competition / Rules Services 与 R3-05 Lineups Service 状态为 `DONE`，R3-04 Teams / Players Services 的历史状态仍为 `VERIFYING`，R3-06 Prediction Service 为 `IN_PROGRESS`。详细状态见 `docs/modular-rewrite/R03-application-services/README.md`。


## R3-06 Prediction Service（IN_PROGRESS）

- R3-06 在独立分支 `rewrite/r3-06-prediction-service` 上实施，未修改 `new-C` 的 R3-05 已验收基线。Atomic Task 1 已将 Prediction Core 的推演执行、readiness、route preview、formal/shadow stored-match execution、dry-run 与运行历史职责迁入 `services/prediction/`、`use_cases/prediction/`，并通过既有 Ports 保持 ApplicationService / Tauri 公共调用语义；模型执行继续只经 `football-model-api` 边界，不修改或复制模型实现。
- Atomic Task 2A 仅迁移 Prediction 所属的 P4 horizon planning、freeze task list/read/events、freeze readiness、match/task workspace 只读职责；`resolve_p4_conflict`、联网 Research 执行、Evidence/Fact 写入和 Research artifact 写入仍保留给 R3-07，不因旧文件混合职责而提前迁移。
- 2A 专项 Windows run `31266144950` / job `93124468057` 已通过 Application Ports、完整 architecture、rustfmt、`cargo check --locked -p football-application` 与 `cargo test --locked -p football-application`，Application tests 33/33 通过。
- 模块化删除旧 `crates/application/src/prediction.rs` 后，确认并修复 3 个历史验证器的旧 owner 路径：默认战术角色、比赛工作流、历史比分验证器均改读当前 Prediction Service / Use Case 权威模块；原业务断言未删除或放宽，其中比赛工作流与历史验证改为递归扫描完整 Prediction 模块树。
- 2A 编译期确认的未使用 import 已直接清理，不增加 lint 抑制。warning-cleanup Windows run `31266871976` / job `93126329974` 已通过 Application Ports、architecture、rustfmt、`cargo clippy --locked -p football-application --all-targets -- -D warnings` 与 Application tests；测试专用 `P4Horizon` / `is_p4_model` 仅移入 `#[cfg(test)]` 作用域。
- clean 源码头 `7e3f43d805b22fceffc6a367392ad9fa1eabef36` 已删除 2A 与 warning-cleanup 的临时 workflow / Python 脚本。完整 Public Platform CI 仍需在最终状态提交上通过后才能关闭 2A；当前不得将 R3-06 或 R3-07 标记为 DONE / READY。

## R2-04 Lineup 与 Match

- Lineup 16 个类型和 Match 3 个类型已迁移到职责目录，旧 `lineup_chain.rs` 已删除。
- 根级类型路径、Serde、数据库映射、Application、Tauri DTO、公共命令和模型保护边界保持不变。
- 实施 workflow run `31151412918` 已通过；正式 Windows Automated run `31153982572` 已在最终实施提交 `0aafe42d7ed08f8e78d71d44ccb6f8f58c425999` 上通过，artifact `8984980586` 的 SHA-256 为 `1e7224f4e7f713b0339e97fd114fa6dea2c0b2ecc9400789613fe872d660938c`。R2-04 状态为 `DONE`，R2-05 为 `DONE`。

## R2-05 Prediction 与 Research

- Prediction 48 个类型与 Research 27 个类型已迁移到职责目录，旧 6 个职责混合源文件已删除。
- 正式 Windows Automated run `31171082098`、job `92842834091` 已通过；artifact `8991618221` 大小 `14117154` 字节，SHA-256 为 `71320b8ef97e62be2fe2323327d21f4870476092ad024d7b8c2c26a4ade9dc59`。
- R2-05 状态为 `DONE`，R2-06 已开放。

## R2-06 Review 与 Postmatch

- Review 48 个类型与 Postmatch 11 个类型已迁移到 `review/`、`postmatch/` 职责目录，旧 5 个职责混合源文件已删除。
- staged 与 Windows Automated 验收已通过；原 PostgreSQL 数据库兼容链已在保留历史数据、不可变资产与 fail-closed 保护的前提下完成验证。
- R2-06 状态为 `DONE`，R2-07 已开放。

## R2-07 Analytics、Exchange、AI 与 Release

- Analytics 39、Exchange 54、AI Workspace 16、Release 9 共 118 个公共兼容类型已迁移到职责目录，旧 7 个职责混合根文件已删除。
- Windows 本机格式、Serde 17/17、365 类型清单与架构门禁已通过；用户确认 R2-07 状态为 `DONE`。

## R2-08 Domain 根出口收敛

- `crates/domain/src/lib.rs` 仅保留 17 个业务模块声明、365 个显式公共兼容类型 re-export 与 crate 内默认值兼容转发，不再承载领域定义或默认值实现。
- 根级 `pub use module::*` 已全部删除；新增确定性生成器和静态验证器，`verify:architecture` 会拒绝 glob 回归、遗漏/重复出口和根文件业务实现。
- 现有 `football_domain::TypeName`、Serde、数据库映射、DTO、模型保护边界和生产依赖未改变。Windows 本机完整阶段回归已通过，R2-08 状态为 `DONE`，R2 阶段已关闭。
