# R01 架构契约与空壳组合根阶段完成记录

- 阶段状态：`DONE`
- `new-B` 分支起点：`36d34ba1ff73cbec575cf58594aa8c0329669496`
- R1-05 前置基线：`f930b81a720ff1f87bb0e63cb39c9b14c6a2a23f`
- R1-05 实施提交：`f2df52e9fb3a2506c6991089e7d8bbdc5868e326`
- R1 正式验收提交：`08803725dcd9f403ffc25552c27d2a9c0d3acd2d`
- 正式验收源码树：`e5b93e149270168bd4626e34b5a514c8779c0d51`
- 目标平台：Windows

## 1. 阶段完成范围

- R1-01：建立模块边界与状态所有权契约。
- R1-02：建立模块边界、状态所有权和受保护导入的机器验证门禁。
- R1-03：建立浏览器唯一组合根、模块注册和受控生命周期。
- R1-04：建立 Tauri Builder、全局状态、命令注册和启动错误映射组合根。
- R1-05：建立 Application 组合根、兼容服务门面、模型注册表和持久化端口注册入口。
- R1-01 至 R1-05 均已完成各自最小验证和 Windows 阶段回归，状态全部为 `DONE`。

## 2. 阶段验证结论

R1-05 的正式 Windows Automated 同时作为 R1 阶段出口回归，在提交 `08803725dcd9f403ffc25552c27d2a9c0d3acd2d` 上实际通过：

- Workflow run：`31073166446`。
- Windows job：`92525208547`。
- 架构边界验证：通过。
- Application 组合根专项验证：通过；3 个职责目录、28 个 Rust 源文件、唯一模型注册所有者和唯一 PostgreSQL 具体导入所有者均符合契约。
- `npm run verify:frontend`：通过，包含 TypeScript、静态契约、17 个截图视口和 Vite 生产构建。
- `npm run verify:rust`：通过，包含 Cargo.lock、rustfmt、Clippy `-D warnings` 和工作区测试。
- 公共 Tauri 命令契约：171 条命令一致。
- 模型保护资产：18 个文件指纹一致，私有 P4/P7 资产继续缺席。
- Tauri Windows release：通过，生成 MSI 与 NSIS bundle。
- release 客户端：首次启动即建立运行日志，客户端启动与状态载入通过。
- 运行日志验收：`PASS`，7 条记录、3 个完成操作，覆盖率与错误扫描通过。

验证证据：

- Artifact ID：`8956912712`。
- Artifact 名称：`windows-automated-delivery-evidence-08803725dcd9f403ffc25552c27d2a9c0d3acd2d`。
- Artifact 大小：`14117884` 字节。
- Artifact SHA-256：`495d3c5e29f2b474e97b98f89dd64b6175cc6e9496dd77681c0a567e00c60016`。

## 3. 兼容性结论

- 未改变公共 Tauri 命令名称、顺序、参数或返回类型。
- 未改变公共 `ApplicationService`、`ModelRegistry`、DTO 或错误语义。
- 未改变数据格式、数据库迁移、持久化结构或连接语义。
- 未改变配置、默认值、环境变量、日志等级、启动顺序、窗口行为或前端可观察行为。
- 未新增生产依赖；R1-05 未修改 `Cargo.toml` 或 `Cargo.lock`。
- 未修改真实模型实现、参数、Profile、固定资产或模型保护边界。

## 4. 延期到最终统一验收

以下验证尚未执行，并按既定策略保留到最终统一验收：

- 使用专用可写测试数据库的真实 PostgreSQL 实跑；当前 18 个 PostgreSQL 集成测试保持显式忽略。
- Windows Full 交互验收。
- 用户本机 Windows 10/11 实机验收。

这些延期项已明确登记，不阻塞 R1 阶段关闭或 R2 开始，也不得在后续阶段描述为已通过。

## 5. 基线与回退

- R1-05 局部回退点：R1-04 清洁基线 `f930b81a720ff1f87bb0e63cb39c9b14c6a2a23f`。
- R1 已验收源码基线：提交 `08803725dcd9f403ffc25552c27d2a9c0d3acd2d`，源码树 `e5b93e149270168bd4626e34b5a514c8779c0d51`。
- 不保留双实现、复制文件、临时 workflow、临时执行脚本或无退出计划的兼容层。

## 6. 下一阶段

R1 已完成并关闭。R2 可从已验收源码基线开始，在独立 Atomic Task 中执行；本次收口未实施任何 R2 源码或配置变更。
