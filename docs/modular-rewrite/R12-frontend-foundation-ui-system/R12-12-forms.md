# R12-12 Forms

状态：`VERIFYING`。基线`085097ab2d1264775ac953cf72655c25c3fa35b7`，唯一分支`rewrite/r12-frontend-foundation-ui-system`。用户2026-10-25授权“收尾11开始12”；01～11 DONE，13～19 BLOCKED。不轮询CI。

## 前置与11收尾

11精确提交`085097ab2d1264775ac953cf72655c25c3fa35b7`的[run40188280192](https://github.com/uniquenesssta/123/actions/runs/40188280192)、job121153622426、attempt1全部步骤成功。完整日志310,654字符，45组Rust结果均ok/0 failed，环境ignored仍为ignored。前端91步骤在验证与打包各通过一次，868项控件、1,344项token、1,104项Shell/工作区、136项告警和17视口通过；Rust fmt/Clippy/工作区测试、release/MSI/NSIS及启动日志通过。artifact12596479924未过期，ZIP17,315,852字节，下载SHA-256 `0162e0ddbfb10ec2bfd5f3d11076d7812c6d4b6ab9bfc92188d51a980692fe3c`与API摘要及上传日志一致。MSI8,388,608字节、NSIS5,922,297字节。报告`windows-acceptance-20261025-112007.json` startup PASS：7有效条目、0无效行、0运行错误，会话`0a18fbe6-4628-438a-af64-b562ff264271`内read_workspace_state/save_workspace_state/bootstrap三组trace逐一配对。原ZIP与完整日志保留在仓库上一级`R12-11-windows-evidence.zip`、`r12-11-ci-full.log`。一次交接核验，无轮询。startup不替代Windows Full/真实PG/XLSX/私有固定回归。

## 设计与职责

[Figma Controls](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=3-3)，设计分支精确40109ceec8372391a7680e7e86c1265fda06ab3d，已只读核对高保真Text Field57:313、Confirmation280:2814、Typed Danger280:3600、Inline Alert303:4259的代码、截图和文档。无Figma写入/新母版/临时远程资产/新依赖。[真实映射](R12-12-component-mappings.json)列出七类目录、节点、调用方和复用SVG指纹；人工映射未声称Code Connect自动同步。

- Form Field组合稳定id、Label、控件slot和Hint/Error；控制值归原生控件。Label13/18/500，Hint/Error12/16/400，间距6px。Required沿native required，不新增未设计的星号。隐藏标签仍参与可访问命名。
- Field Label/Hint/Error分别拥有文字呈现，Error经原iconSlot使用精确16px Alert Circle；消息变更只修改本组件描述引用，保留外部aria-describedby、草稿和焦点。
- Validation Summary复用已接受Inline Alert危险表面、图标和文字；错误列表为宿主结果的原生链接组合，没有伪称独立Figma母版。Enter/点击只在指定表单区域内寻找控件并聚焦，支持Select触发器、禁用/缺失项。
- Confirmation Input只投影原trim(value)===expected且大小写敏感的名称门槛；短暂touched/composing归本实例，取消监听/重置定时器复用EventScope。宿主还可传保护门槛；空expected失败关闭，disabled/祖先禁用/Busy/composition中不可启用确认。
- Form Actions只复用11按钮：取消在前，确认在后，Busy/Loading禁用两者。它不执行请求、取消、业务校验或存储。
- 唯一Forms样式入口由main导入；目录index.ts仅显式导出。没有新跨模块owner，仍31。

## 真实接入与兼容

数据库连接区域的地址/连接数/超时用真实原生字段与原id/action；地址仍password、autocomplete=off，数值范围/默认值原样。既有连接错误投影为摘要，不回显连接串/凭据，并通过原click委托定位原字段。没有新增连接请求。

数据库清空请求只替换字段与动作区视图，去掉已替代的五条旧CSS规则及main中的重复名称比较。实际executeDatabaseReset函数逐字与基线一致，含原名称二次判断、api.resetDatabase、工作区清除和重载。ModalController通过通用data-confirmation-button稳定id挂接控制器，替换内容前销毁，恢复原草稿后再计算门槛；没有硬编码数据库动作到通用控制器。

既有workspace确认和Shell native confirmation采用同一Form Actions API，所有动作名、默认取消焦点、后果说明保持。workspace仍role=region；本任务没有把它冒充新模态容器。异步模态Submitting/Error/未知结果流程仍属于14及实际接口验收；其它Feature页面的领域表单未被本节点泛化改写。

## 验证证据

| 验证 | 结果 |
|---|---|
| 最终Windows Chromium Forms | 4组各68，合计272 PASS；DPR1；真实CDP输入/Enter/多行与原生FormData/reset；r12-12-forms-final.log |
| 完整前端 | npm run verify:frontend exit0，91步骤，TSC/Vite；r12-12-frontend-final.log |
| 原有回归 | 868 Primitives、1344 Tokens、1104 Shell/Workspace、136告警、17原截图视口；无改PNG或阈值 |
| 架构 | npm run verify:architecture exit0，31 owner；r12-12-architecture.log |
| Rust格式 | cargo fmt --all -- --check exit0 |
| 模型保护 |18指纹不变，聚合d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57 |
| 静态影响审计 | main具名函数仅requestDatabaseReset视图改变；executeDatabaseReset逐字不变；API/DTO/schema/Rust/锁文件/生产依赖/持久化/模型均无修改 |

最终完整Forms证据在仓库上一级`logs/r12-12-forms-1792931575632`，四张截图和forms-results.json；Temp `football-r12-forms-ABkpQl`。最终源码前一轮最小验证的同代码明暗960/1440截图已人工逐项复核，Figma错误图标、标签、提示、输入、摘要及动作区无裁切/错位。这里只确认实际DPR1，不覆盖全部DPI、屏幕阅读器和操作系统IME。

初次生成验证脚本因模板插值、随后因Unicode URL pathname错误停止，均保留失败日志；改用真实fileURLToPath后执行成功，未跳过检查。截图发现fixture宽泛section规则影响摘要表面，已收窄到直接子section；生产旧确认CSS也已退出，最后全量回归覆盖这些修复。全部本地浏览器/build调用使用既有保留产物preload，未做递归删除。原>500kB构建提示仍存在，未放宽阈值：最终182模块，main-BDhA2Xrh.css439.03kB/gzip69.20；main-BNFRIlfs.js617.02kB/gzip175.58；index-BJQUF_Kr.js6.31kB/gzip2.58。

非Markdown共38路径，聚合SHA-256 `6eb43b7fbae0e6805edc19b5a7bff7e4b48b51aa722c7fab8e263b1b7169a6e0`（上一级r12-12-source-proof.json）。父目录helper/日志/ZIP不进入Git。

## 实际文件清单

合计44路径：29新增、15修改；移动/重命名：无；文件删除：无。

### 新增

- [docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-12-component-mappings.json](R12-12-component-mappings.json)
- [docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-12-forms.md](R12-12-forms.md)
- [scripts/forms/verifyForms.mjs](../../../scripts/forms/verifyForms.mjs)
- [src/ui/forms/__tests__/formsBrowserFixture.ts](../../../src/ui/forms/__tests__/formsBrowserFixture.ts)
- [src/ui/forms/confirmation-input/confirmationInput.css](../../../src/ui/forms/confirmation-input/confirmationInput.css)
- [src/ui/forms/confirmation-input/confirmationInputController.ts](../../../src/ui/forms/confirmation-input/confirmationInputController.ts)
- [src/ui/forms/confirmation-input/confirmationInputView.ts](../../../src/ui/forms/confirmation-input/confirmationInputView.ts)
- [src/ui/forms/confirmation-input/index.ts](../../../src/ui/forms/confirmation-input/index.ts)
- [src/ui/forms/field-error/fieldError.css](../../../src/ui/forms/field-error/fieldError.css)
- [src/ui/forms/field-error/fieldErrorView.ts](../../../src/ui/forms/field-error/fieldErrorView.ts)
- [src/ui/forms/field-error/index.ts](../../../src/ui/forms/field-error/index.ts)
- [src/ui/forms/field-hint/fieldHint.css](../../../src/ui/forms/field-hint/fieldHint.css)
- [src/ui/forms/field-hint/fieldHintView.ts](../../../src/ui/forms/field-hint/fieldHintView.ts)
- [src/ui/forms/field-hint/index.ts](../../../src/ui/forms/field-hint/index.ts)
- [src/ui/forms/field-label/fieldLabel.css](../../../src/ui/forms/field-label/fieldLabel.css)
- [src/ui/forms/field-label/fieldLabelView.ts](../../../src/ui/forms/field-label/fieldLabelView.ts)
- [src/ui/forms/field-label/index.ts](../../../src/ui/forms/field-label/index.ts)
- [src/ui/forms/form-actions/formActions.css](../../../src/ui/forms/form-actions/formActions.css)
- [src/ui/forms/form-actions/formActionsView.ts](../../../src/ui/forms/form-actions/formActionsView.ts)
- [src/ui/forms/form-actions/index.ts](../../../src/ui/forms/form-actions/index.ts)
- [src/ui/forms/form-field/formField.css](../../../src/ui/forms/form-field/formField.css)
- [src/ui/forms/form-field/formFieldMessage.ts](../../../src/ui/forms/form-field/formFieldMessage.ts)
- [src/ui/forms/form-field/formFieldView.ts](../../../src/ui/forms/form-field/formFieldView.ts)
- [src/ui/forms/form-field/index.ts](../../../src/ui/forms/form-field/index.ts)
- [src/ui/forms/index.css](../../../src/ui/forms/index.css)
- [src/ui/forms/validation-summary/index.ts](../../../src/ui/forms/validation-summary/index.ts)
- [src/ui/forms/validation-summary/validationSummary.css](../../../src/ui/forms/validation-summary/validationSummary.css)
- [src/ui/forms/validation-summary/validationSummaryController.ts](../../../src/ui/forms/validation-summary/validationSummaryController.ts)
- [src/ui/forms/validation-summary/validationSummaryView.ts](../../../src/ui/forms/validation-summary/validationSummaryView.ts)

### 修改

- [architecture/module-boundaries.json](../../../architecture/module-boundaries.json)
- [docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md](../../football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md)
- [docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-11-ui-primitives.md](R12-11-ui-primitives.md)
- [docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md](README.md)
- [docs/TESTING.md](../../TESTING.md)
- [README.md](../../../README.md)
- [scripts/verify-database-reset.mjs](../../../scripts/verify-database-reset.mjs)
- [scripts/verify-global-visual-system.mjs](../../../scripts/verify-global-visual-system.mjs)
- [src/app/modal.ts](../../../src/app/modal.ts)
- [src/components/databaseSetup.ts](../../../src/components/databaseSetup.ts)
- [src/main.ts](../../../src/main.ts)
- [src/styles/app.css](../../../src/styles/app.css)
- [src/ui/overlays/confirmation-dialog/confirmationDialog.css](../../../src/ui/overlays/confirmation-dialog/confirmationDialog.css)
- [src/ui/overlays/confirmation-dialog/confirmationDialogView.ts](../../../src/ui/overlays/confirmation-dialog/confirmationDialogView.ts)
- [src/ui/primitives/text-input/textInputView.ts](../../../src/ui/primitives/text-input/textInputView.ts)

## 未验与回退

12自身Rust Clippy/工作区测试、Windows打包/MSI/NSIS、实际Tauri启动及日志，需本次提交后精确head/run独立核验；11的成功只作12前置，未当作12结果。真实PG、受保护历史/不可变账本并发回滚、XLSX、Windows Full、私有固定回归及真实API/凭据取消E2E仍按最终环境验收。未执行真实连接/清空，未伪造API或业务结果。

以已完整验收的085097ab2d1264775ac953cf72655c25c3fa35b7为回退点，发布后通过对应原子提交revert恢复；不手工复制，不留双实现。12自身Windows核验前保持VERIFYING，13不开始。
