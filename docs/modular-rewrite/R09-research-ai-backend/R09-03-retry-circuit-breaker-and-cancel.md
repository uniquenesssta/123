# R9-03 Retry、Circuit Breaker 与 Cancel 实施记录

状态：`VERIFYING`。用户2026-10-10“收尾02开始03”授权；uniquenesssta/123、0.23.0、唯一阶段分支rewrite/r9-research-ai-backend。02精确05055d1/run37954536795全SUCCESS并DONE，五文档收尾/03基线59af3f2ec65030a28150a3042d3b0673d65a48a8 `[skip ci]`。04～11BLOCKED。

### 1. 目标

- 已把原重试预算/等待、熔断状态与本地取消令牌归到唯一resilience职责；协议编排留原client，不改业务策略。

### 2. 现状与来源

- 原client.rs混合三协议、三个重复退避select、CircuitState/三状态方法、取消错误；原cancellation.rs只持本地令牌。
- 原GatewayAttemptSink、Application/Tauri Registry、Responses远端取消/轮询继续原owner；R9-10另处理Registry。

### 3. 目标文件与目录

- `resilience/{retry,circuit_breaker,cancellation}.rs` 与仅登记/显式export的mod.rs；旧cancellation.rs删除。
- 按总纲顶部订正使用三个实际职责文件，不为单一可独立职责机械创建空子目录。

### 4. 文件职责边界

- retry持原次数上限、指数退避与可取消等待，不持模型/错误重试资格/协议/账本。
- CircuitBreaker持单一Mutex<CircuitState>及check/success/failure；配置仍唯一GatewayConfig，不克隆第二政策对象。
- CancellationToken持原Arc<AtomicBool+Notify>共享克隆及原Cancelled错误；不持网络响应ID/Registry。

### 5. 输入

- 原max_retries、retry_base_delay_ms/retry_index；GatewayError及借用CircuitBreakerConfig；原CancellationToken。

### 6. 输出

- 原每模型max_retries+1预算、原Duration/Ok或Cancelled、原CircuitOpen/Ok及同一令牌公共接口。

### 7. 允许依赖

- 已锁定std、tokio1.52.3 sync/time/macros与现有GatewayError/Config；依赖/feature/锁文件保持。

### 8. 禁止依赖

- resilience不调用transport、key_provider、HTTP、账本/数据库、Domain预测、Tauri、UI或Responses远端取消。

### 9. 状态所有权

- CircuitBreaker实例唯一持consecutive_failures/open_until，三协议共用；令牌原Arc生命周期唯一持cancelled/notify。
- 原architecture/state-ownership登记这两个实际owner，配置仍GatewayConfig，Semaphore仍client原请求scope。

### 10. 副作用边界

- retry仅原tokio sleep及取消等待；circuit仅原锁内内存状态；token仅原SeqCst原子位/notify_waiters。
- 所有网络调用、远端cancel/poll与attempt sink仍原client/transport，不新增IO、日志或持久化事件。

### 11. 异常路径

- 原四类Network/Timeout/RateLimit/ProviderUnavailable计数，与retryable位独立；其余15类不计数且不清零。
- 到阈值设置原open_seconds窗口，下一入口拒绝；到期或在途成功清零；不新增半开单探针政策。
- 原sink.record错误先于熔断状态更新；原Cancelled/CircuitOpen完整错误、metadata与优先级保持。

### 12. 并发/异步/生命周期

- 原三个入口只在请求开始check；在途失败可能打开熔断但当次原重试继续，不增加每次retry检查。
- 原select分支顺序、公平选择、网络future丢弃及OwnedSemaphorePermit请求scope保持。
- 令牌先创建Notified再查取消位；SeqCst/notify_waiters/幂等cancel、克隆共享及独立token保持，丢弃等待不取消token。

### 13. 兼容要求

- 正式研究只Responses；Plain双协议仍不发送tools/web/schema/token字段。
- Structured自动web-search兼容回退、模型fallback资格/去重、每模型预算、attempt_offset/饱和及事件顺序全部保持。
- 原公共Gateway/CancellationToken API、DTO、Schema、配置键/profile、错误文案/日志/UI/版本0.23.0保持。

### 14. 实施步骤

1. 已核实02修复自身Windows全SUCCESS，五文档收尾59af3f2，用户授权03；原35 unit/16 contract为基线。
2. 已迁移三职责及唯一export，原生产体和整个client剩余逻辑完成重路由等价核对。
3. 原unit新增12、原contract新增5；原测试保持，无新target/runner/workflow/数据库。
4. 原兼容传输验证器补唯一owner/三入口/政策/唤醒顺序，6破坏探针拒绝恢复。
5. 静态/保护验证及记录完成；提交自身WindowsCI开始后停止轮询，待下次核实精确结果。

### 15. 切换入口

- crate根继续公开CancellationToken原名；client直接持CircuitBreaker并调用方法，三个等待共用wait_retry。
- 不留下Gateway转发方法、旧取消入口、第二状态或第二退避公式；远端取消不是本地token副作用。

### 16. 删除清单

- 删除旧cancellation.rs；client移除CircuitState、check_circuit/record_success/record_failure、retry_delay/cancelled_error和三重复退避select。
- 原其他client方法和原contract16测试/9具名helper与实现逐token保留；不删除行为/断言。

### 17. 最小验证

- 实际83/83原静态检查、完整verify:architecture、Rustfmt1.88源check、171命令、18保护资产、46迁移/18PG静态及diff PASS。
- 生产3熔断体/原token全部/退避与错误/重路由剩余client/251literal等价，6探针拒绝恢复。
- 源码预期Gateway47/contract21、Application126/Persistence175，新增12+5须本项Windows实际运行，不能记为已PASS。

### 18. 阶段回归

- 原Public Platform CI：完整frontend/contracts/TypeScript/Vite/17视口、fmt/Clippy -D warnings/workspace tests、Windows release/MSI/NSIS/启动。
- 本地仅源码静态，未执行Linux/macOS Cargo/编译/单测/loopback/浏览器动态；真实PG/历史四项/账本并发回滚/有效XLSX/Full/私有固定回归仍最终新库待验。

### 19. 失败停止条件

- 自身Windows失败只修相关链路，03不得DONE/启动04；公共契约/保护资产/静态门禁失败先修复，不放宽或略过。

### 20. 回退点

- 03基线59af3f2ec65030a28150a3042d3b0673d65a48a8，源码等于02精确05055d1/run37954536795成功；受控revert03整体并恢复原唯一owner，不复制双实现、不覆盖用户修改。

### 21. 根 README 摘要记录

- 已同步02DONE/03VERIFYING、5A/10M/1D、职责/等价/12+5测试及实际静态/Windows待验边界。

### 22. docs 阶段节点详细记录

- 已创建03实施记录和准确全文件清单、报告/工具/回退；阶段索引03VERIFYING、04～11BLOCKED，R9整体IN_PROGRESS。

### 23. 完成标准

- 代码/静态/文档已完成；仍须本项精确Windows完整SUCCESS才能03DONE。
- 未执行真实PG等延期项目不冒充PASS，阶段不创建完成记录或提前启动04。

## 完整实际文件清单

相对59af3f2基线 `git diff --no-renames --name-status` 含新增：**5A / 10M / 1D**，无整文件移动/重命名。

新增（5）：

- `crates/research-gateway/src/resilience/mod.rs`
- `crates/research-gateway/src/resilience/retry.rs`
- `crates/research-gateway/src/resilience/circuit_breaker.rs`
- `crates/research-gateway/src/resilience/cancellation.rs`
- `docs/modular-rewrite/R09-research-ai-backend/R09-03-retry-circuit-breaker-and-cancel.md`

修改（10）：

- `README.md`
- `architecture/domain-type-inventory.json`
- `architecture/state-ownership.json`
- `crates/research-gateway/src/client.rs`
- `crates/research-gateway/src/lib.rs`
- `crates/research-gateway/tests/gateway_contract.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md`
- `docs/modular-rewrite/R09-research-ai-backend/README.md`
- `scripts/verify-api-compatible-transport.mjs`

移动/重命名：无。删除（1）：`crates/research-gateway/src/cancellation.rs`，无旧壳。

Domain使用扫描1110→1113、usageDigest `3f2e1c1e006acf971cc176490eeaa23d4517fe20f3f72a702ea8079ded5000f4`；365/300/sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd`、Application402/43Ports、PG根35/46迁移、171命令与18保护资产保持。原state契约增加两个已有运行时状态的实际owner登记，不新增运行时状态或变更契约id/version。

## 原目标新增测试与真实结果边界

原Gateway35 unit与16 contract保留。新增12 unit：retry4覆盖0/10/u32MAX预算、指数0/1/2/11/12/MAX与毫秒溢出、正常/预取消等待、pending取消/丢弃；cancellation4覆盖Send+Sync/克隆/独立、首次poll前取消、注册及未poll多等待者、丢弃与重复取消；circuit4覆盖全部19category、原4计数不看retryable/其余保持、阈值/原完整错误/到期复位、成功复位、8并发失败与u32饱和。超时界限使用原tokio time，未启用test-util或暂停时钟，到期测试仅测试内设置已过期Instant。

新增5 contract通过公开Gateway验证：Formal失败打开三入口共享熔断；两轮成功复位避免非连续失败误熔断；sink取消后停止重试/模型fallback、保留offset8且许可释放；Persistence sink失败先于circuit mutation；挂起IO与并发排队取消后无额外请求、future Drop实际执行且容量恢复。所有请求为原mock或明确pending future，无真实API/OS凭据副作用，无新target或loopback。

本地已PASS的是静态，不是Rust单测。源码预期Gateway47/contract21、Application126/Persistence175须自身Windows实跑。保护聚合 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`，PG静态聚合 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93` 保持。

报告：`/workspace/scratch/eb298ad5cdcb/r903-static-checks.json`、`r903-architecture.log`、`r903-equivalence.json`、`r903-negative-probes.json`。83/83既有静态、完整architecture、Rustfmt1.88源码format/check、原保护资产/数据库静态/命令契约、git diff --check均PASS。6探针分别破坏退避接入/指数上限/计数类别/复位/通知全部等待者/通知创建顺序，原增强兼容传输验证器全部拒绝，逐字恢复后通过。无Clippy抑制或测试删减。

生产三熔断方法体/两个纯helper/原令牌全部tokens保持；client重内联三等待并反向还原owner路由后全体其余tokens保持；原contract16测试及25具名函数集合（含原helper/trait实现）保持，251条生产literal保持。没有声称将不同协议的fallback或工具兼容策略统一；本项明确没有生产政策变化。

## 工具、取舍与继续位置

Context7按Cargo.lock tokio1.52.3核对Notify，但目录仅给latest；准确docs.rs1.52.3页面访问失败，已通过GitHub读取官方tokio-rs/tokio的tokio-1.52.3标签notify.rs，确认Notified在创建后即能收到notify_waiters且不必先poll。原顺序原样迁移，不增加enable/notify_one、新库或依赖升级。Mermaid Chart已展示真实闭合/熔断/到期/在途成功与取消生命周期，不引入半开探针；Create State仅保存核实的足球稳定model96f03270-c236-46de-99fa-db85d2fbf4ce，合成摘要不替代源码及CI。

沿用原Windows workflow，精确head与Windows job确认开始后停止轮询，预计20～30分钟。下一轮先核实03自身结果：成功收尾、失败修相关完整链路；04必须另有授权且门禁通过才能开始。真实PostgreSQL、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041删除风险仍最终封包新库待验；ignored与公开unavailable stub不计PASS。R9整体IN_PROGRESS，无阶段完成记录。
