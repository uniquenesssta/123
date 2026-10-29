# R12-14 CI修复：状态迁移来源与清单最后修改任务

日期：2026-10-29。状态：VERIFYING。沿原分支 rewrite/r12-frontend-foundation-ui-system，修复基线3e335ec217eaad902f44e2989cde46d3a457b767；不进入15，不轮询新CI。

## 精确失败与影响

[Public Platform CI run40734621493](https://github.com/uniquenesssta/123/actions/runs/40734621493) 的 head_sha 为3e335ec217eaad902f44e2989cde46d3a457b767。Windows job122979714554 在第6步 Verify R1 architecture boundaries 失败。原日志报 Database Service 验证失败：状态所有权契约未登记 R3-02 更新。第7步 Windows automated acceptance 被跳过，未执行其前端、Rust、打包和启动检查；上传步骤虽成功，但日志明确没有找到产物，不能当作交付成功。

本地读取原提交并直接运行 node scripts/verify-database-service.mjs，复现同一失败。原因：14收尾将 architecture/state-ownership.json 的 last_updated_task 从R3-02更新为R12-14；旧验证器将这个可更新的清单元信息当作R3迁移完成的永久证据。此前92步通过后又改了机器契约，只重跑通用owner/保护检查，漏了Database Service，因此此前记录不能代表3e335ec2最终树完整通过。此处明确订正，而非把CI失败解释为环境抖动。

## 修复与职责

- application.active-database 单独登记 migration_task=R3-02，表示当前DatabaseService.session归属的迁移来源。
- 保留全局 last_updated_task=R12-14，它只表示清单最近修改任务，后续任务可正常更新。
- Database Service 检查仍验证唯一真实owner、Service/UseCase/Port、重置预检和后端名称保护；迁移来源改为检查该状态自己的migration_task，不关闭任何原门禁。
- 新回归入口调用真实验证器，在独立子进程内仅替换本次测试的内存清单输入；不修改仓库文件。2个正例验证当前和后续清单修改，4个反例验证迁移来源缺失/错误、owner回退和状态缺失仍失败。新检查同时接入原verify:architecture和verify:frontend，不新增workflow或生产依赖。
- 生产src、CSS、Figma、Rust、API、DTO、数据库结构和模型资产没有修改；无文件删除。

## 验证与未验项

直接失败复现、修复后的Database Service、2正例/4反例、完整npm run verify:architecture均已通过。完整93步前端验收exit0（含342项浮层浏览器检查、TypeScript与Vite）；验证文件指纹一致；旧342项视觉和Rust结果保留其原范围，不替代本修复验证。本修复不重做UI或真实数据库破坏操作。

日志：O:/Project/FOODBALL/Temp/r12-14-ci-fix-architecture-final.log、r12-14-ci-fix-frontend.log。所有开发脚本使用项目临时目录：先加载 O:/Project/FOODBALL/project-temp.ps1，让进程级TEMP/TMP指向O:/Project/FOODBALL/Temp。

发布后仍须本修复精确提交的Windows CI与MSI/NSIS、startup证据。未通过或未执行项不计为PASS。不得用此修复启动15。回退点为原3e335ec2（已知CI失败基线，不称可验收基线）；回退使用本修复原子提交的git revert，不手工恢复或批量删除。

## 实际文件清单

新增2项：
- scripts/architecture/verifyDatabaseStateMetadata.mjs
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-14-ci-fix-state-provenance.md

修改8项：
- architecture/state-ownership.json
- package.json
- scripts/verify-database-service.mjs
- scripts/verify-frontend.mjs
- README.md
- docs/TESTING.md
- docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-14-feedback-and-overlays.md

移动/删除：无。合计10个路径。

## 最终本地验证

完整架构入口与93步前端均exit0，保留原门禁和Vite分块提示。5个可执行/契约文件的SHA-256从验收开始至提交前一致；本次验收后只补充Markdown结果，未再次修改机器契约。实际修复CI结果仍待本提交发布后验收，不冒称Windows打包/完整链路通过。
