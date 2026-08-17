# R06-06 — Abilities 与 Dynamic Tags

## 状态

`VERIFYING`

## 进入基线

- R6-05：`DONE`。
- 唯一阶段分支：`rewrite/r6-entity-catalog-persistence`。
- 节点起点：`f48841a8a50f51cd71e09969d858b95e7d9ba9fa`。

## 已实施职责边界

- `adapters/catalog/abilities/dimensions/`：能力维度 query、typed Row 与 Domain mapper。
- `adapters/catalog/abilities/observations/`：能力观察输入策略、写 SQL、typed Row 与 Domain mapper；Player Detail 只保留读取 SQL并复用该 Row/mapper owner。
- `adapters/catalog/dynamic_tags/definitions/`：动态标签定义读取。
- `adapters/catalog/dynamic_tags/tags/`：动态标签输入策略、写入、单条读取、时点列表、typed Row 与 mapper。
- `adapters/catalog/dynamic_tags/contribution/`：比赛贡献计算编排与纯评分组件分责。
- 旧 `dynamic_tags.rs` 单文件 owner 已删除；`player_catalog.rs` 已退出 R6-06 Abilities 职责。

## 保持不变的契约

- `PlayerSignalPort` 的 Ability、Dynamic Tag 与 Match Contribution 方法签名及 Application adapter 调用语义不变。
- Domain DTO、Schema、0001–0046 migration、配置、错误文案、默认战术角色/位置映射、历史 P4/运行引用和模型保护资产未主动修改。
- 未新增生产依赖。

## 验证

- implementation gate run `32043516067` 已实际通过 R6-06 ownership、R6-03～R6-05 retained ownership、完整 architecture、模型保护、数据库基线、rustfmt、Persistence unit tests、Application check、R6-06 PostgreSQL contract 编译及 PostgreSQL 16 专用测试库真实执行。
- 阶段 hard gate 与 clean canonical Public Platform CI 尚未执行，因此当前保持 `VERIFYING`。

## 尚未执行

- 用户现有 PostgreSQL 数据真实写入/sample 验收与 Windows Full 人工交互验收，继续留到最终统一验收。

## 回退点

- `f48841a8a50f51cd71e09969d858b95e7d9ba9fa`。
