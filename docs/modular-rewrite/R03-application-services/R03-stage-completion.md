# R03 Application Services 重写阶段完成记录

- 阶段状态：`DONE`
- R3 起点：`7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f`
- R3-10 最终 clean 验证 HEAD：`2ecebb9ab0076f27a20d46bc897e63c78aecae3d`
- R3 最终合并提交：`f400740a36e29ab5cf154728c5f240ee018707c6`
- 目标平台：Windows

## 1. 阶段完成范围

- R3-01：完成 Application Ports 设计与机器清单，建立按职责域划分的最小 Port 边界。
- R3-02：完成 Database Service 与数据库生命周期边界重写。
- R3-03：完成 Competition / Rules Services 重写。
- R3-04：完成 Teams / Players Services 重写。
- R3-05：完成 Lineups Service 重写。
- R3-06：完成 Prediction Service 重写，并保持模型执行仅经公开 Model API 边界。
- R3-07：完成 Research Service、Fact Pipeline、OpenAI Gateway、P4 Research worker 与人工冲突裁决重写。
- R3-08：完成 Review / Postmatch / Analytics Services 重写。
- R3-09：完成 Exchange / AI Workspace / Release Services 重写。
- R3-10：将 `ApplicationService` 收敛为兼容门面；删除根级 P4 orchestration 业务 owner，并将剩余生命周期、跨 Port 组装和兼容映射迁入独立 Service / Use Case / compatibility 模块。
- R3-01 至 R3-10 均已关闭为 `DONE`。

## 2. 最终 Application 边界

- Application Ports 共 15 个职责域、38 个最小能力 trait；不建立万能 Repository。
- 具体 PostgreSQL / SQLx 实现继续只由 composition adapter / port registry 持有，Service / Use Case 不直接依赖数据库实现细节。
- `ApplicationService` 保留既有公共方法、参数、返回 DTO 与 Tauri 调用面，只承担兼容门面和明确的会话边界委托，不再承载独立业务流程、P4 worker 状态、SQL、网络或模型算法。
- P4 orchestration 已拆入独立 `services/p4_orchestration/` 与 `use_cases/p4_orchestration/`；后台 job claim / dispatch / complete / fail、终态迁移和 worker 生命周期拥有明确 owner。
- Database connect / reset / bootstrap 生命周期进入 `use_cases/application_facade/`；Research 多 Port access 组装进入 ResearchService；Prediction recent-run DTO 映射进入独立 compatibility 模块。
- 各阶段删除的旧职责 owner 均不保留并行实现或空转发壳。

## 3. 验证结论

- R3-10 AT1 strict hard gate run `31591061641` / job `94095867258`：完整 architecture、38-Port、rustfmt、Application check、Application tests 33/33、Clippy `-D warnings` 全部通过。
- R3-10 AT2 strict hard gate run `31592634518` / job `94100834403`：ApplicationService facade 专项、Database / Competition-Rules / Research / Prediction / Application Composition、完整 architecture、Application check/tests 与 Clippy `-D warnings` 全部通过。
- 文档恢复验证在执行仓库既有 `npm run setup` 后通过完整 `verify:frontend`、17 个截图回归视口与完整 `verify:architecture`；历史 Database reset verifier 已迁移到当前 authoritative owner，原强确认、自动重连和 P4 worker 恢复断言未放宽。
- 最终 clean Public Platform CI run `31593758268` / Windows Automated job `94104353199` 在 HEAD `2ecebb9ab0076f27a20d46bc897e63c78aecae3d` 上全部通过：architecture、完整 Windows Automated acceptance 与 validation evidence upload 均为 `SUCCESS`。
- 最终 evidence artifact `9140937975`，大小 `13907479` 字节，SHA-256 `a67e78ee1272d9a953432292ee284118cffcc17325a9aefedf4367cec451ee75`。

## 4. 兼容性结论

- 未改变公共 ApplicationService 方法名、参数、返回 DTO、Tauri 公共命令或前端调用契约。
- 未改变 PostgreSQL SQL、migration、Schema、历史数据格式、配置、默认值、环境变量、错误语义或日志等级。
- 未修改模型实现、模型参数、Profile、私有资产或生产依赖；公开模型保护边界继续保持。
- R3-10 中历史验证器的变更仅迁移 authoritative owner；原行为、初始化、reset、Research、Prediction 与 P4 契约断言未删除或弱化。

## 5. 未执行与剩余验证

以下项目未描述为已通过：

- 18 个需要专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试；本阶段未配置专用测试数据库，因此保持既有 `ignored` 安全策略。
- destructive database reset 实跑；未对用户真实数据库执行破坏性验证。
- Windows Full 最终人工交互与私有 ModelProvider 环境中的固定模型回归继续按总任务书进入最终统一验收。

## 6. 阶段收口

- R3-10 PR #21 已以 merge commit 方式合并，最终合并提交为 `f400740a36e29ab5cf154728c5f240ee018707c6`。
- R3 已完成并关闭，不保留 R3 临时 workflow、并行 ApplicationService 业务 owner 或旧 P4 orchestration 根实现。
- 后续阶段从本 R3 完成基线继续，严格按总任务书顺序执行。
