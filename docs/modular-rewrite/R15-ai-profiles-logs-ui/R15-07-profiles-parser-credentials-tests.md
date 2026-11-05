# R15-07 配置、示例解析、凭据与连接测试

状态：DONE。2026-11-05启动时基线0f65d593664f70c7a0464e27e52f4dfee875d037，起始工作区干净，沿用rewrite/r15-ai-profiles-logs-ui。01与本批07自身CI、官方双包及startup均已正式验收，07精确提交42418cb5/运行41887780319详末节收尾。BA-11真实外部边界保留。用户已授权启动R15-12。

## 实际归属与保留

| 原调用/状态 | 唯一owner与公开出口 | 真实命令/保留行为 |
| --- | --- | --- |
| main.openAiProfiles/selectedProfile/creating/目录载入 | ProfilesFeature、ProfilesState | listOpenAiProfiles；设备本机配置不依赖业务数据库。选择编辑、当前启用、密钥存在和测试分别显示；不静默换绑缺失配置 |
| 表单读取/草稿/保存 | profileDrafts、profileStorage、ProfileOperations | saveOpenAiProfile原DTO；同profile与临时new独立安全草稿，显式确认后提交；原回执只更新原对象，后续编辑不被清空，新建回执不抢B |
| API Example防抖/解析/应用 | ProfileExample、ProfileControls | parseOpenAiApiExample；320ms防抖、原请求序号/DOM/文本/profile/修订/页面守卫。只预览，再明确应用到草稿，保存另需确认；不执行curl |
| 原activate/clear/delete | ProfileOperations、ProfileActions | setActiveOpenAiProfile/clearOpenAiProfileKey/deleteOpenAiProfile；确认绑定原ID、元数据版本、编辑修订及页面，防双提交。留空保留、替换与清除分开；已有会话不改绑 |
| 保存并测试/局部状态 | ProfileOperations | testOpenAiProfile；先保存，再测试原profile及保存版本；确认受理后关闭，局部状态继续，实际可浏览B。无虚构取消接口/进度 |
| 旧页面与控件增强 | profileView、ProfileControls | 复用FIP Button/Select/TextInput/Number/Textarea/FormField、现有确认与图标；输入焦点、选区、分区滚动及临时DOM字段由控件owner处理 |
| 后端异步测试提交 | 原OpenAiProfileStore.update_test_state | 增expected_updated_at私有参数，成功与失败均仅提交原profile快照；参数/密钥更换、原对象删除、已提交新结果使旧响应失效。四项真实函数Rust回归在openai_profiles/test_freshness.rs |

公开出口src/features/openai-profiles/index.ts。main只组合、转发和路由/Workspace衔接；旧全局状态、解析/保存/测试函数与事件分支退出，唯一旧路径src/pages/openai.ts由实际Feature替代退出。原Windows凭据、解析器、后端元数据格式和错误/协议边界复用。无新增生产依赖、runner、workflow、命令、Schema、迁移或模型变更。类型清单只更新Rust用量摘要/扫描文件数，类型定义与公共兼容数量未变。

## 设计与状态

使用Impeccable4.7.1的Operate/harden及craft-floor；启动器默认缓存不可写，已先说明直接读取项目批准资料，项目内0.1.17检测另验。不存在PRODUCT.md/DESIGN.md，不编造设计上下文或另立主题。

Figma文件PN0Whgu6HLWIHx4Mv6aHfu，session7da85654-6d25-4e4a-accc-77da841dd79c。批准记录ui-design-system/FIGMA_SCREEN_OPENAI_STATE.json，recordCommit857af9f35df624e76ec1c83ed2256a92918b9288；实际取得Profiles647:166715、Example Chat654:177329、Models654:181253及Tests652:170058高保真上下文/截图。38状态/24确认与说明是预置设计证据，不计真实配置操作。

实际Example为次级页内编辑状态，原任务卡“次级弹窗”误记已订正；按实际Figma呈现，不新造弹窗，不改Figma。原操作确认仍使用既有批准组件。26/700标题、14/700分区、13/500控件、13/400输入、12/400正文，32px动作/40px输入、12px表单组间距；标准248px目录/12px工作区间距，窄屏按容器收缩。表单/目录各自滚动，动作栏在表单视口外。明暗与1440×900、960×720、1280×720、1920×1080分列，复用本地FIP图标，无临时资产URL/图片替代。

## 原ID与验收

| 原ID | 已实现及验证入口 |
| --- | --- |
| 07目录 | 真实读取/空与失败/刷新/选择/新建；设备作用域，无数据库不禁用；原路由测试及完整挂载 |
| 08编辑 | 每配置安全草稿、修订、格式/范围/名称校验、显式保存、A写入后切B不串线；主动作保持可达 |
| 09解析 | 原防抖与过期防护保留，新增显式应用；失焦不重复解析，换配置退出原示例。手工更改连接/模型后退出旧模板，避免后端再解析覆盖手工值 |
| 10凭据 | 临时输入/原凭据调用，API Key不在ViewModel/持久草稿；确认绑定原对象。跨存储失败按unknown处理，先查原配置，不能据目录缺席声称密钥已清除 |
| 11测试 | 原profile/原保存修订、后端时间快照、A/B隔离、已保存与测试失败分离；成功投影到原保存版本，当前新草稿不借用旧结果 |

profileStorage仅白名单安全草稿/必要ID/操作恢复记录，新preferenceKey(openai-profiles-drafts)，不把它当正式配置库。仅后端已脱敏模板进入安全草稿，保留YOUR_API_KEY占位语义，不能用日志脱敏标记替换后又当新密钥。原始示例和临时密钥只存在活动DOM输入与必要凭据调用，切页/切配置/关闭释放；普通Workspace不捕获本页，旧openai-api-example快照逐项退出。已保存密钥从不回传。

unknown不自动重试；完整目录/密钥存在读回也不证明跨存储全部成功或回滚。核验共用目录最新序列，旧查回不能覆盖新读取/已确认写入；新一轮核验永久使旧解除确认失效。人工确认只解除本地门禁，不重发、不撤销、不制造终态。

## 证据与实际失败

证据根O:/Project/FOODBALL/logs/r15-07-20261105，最终准确状态/工具/受测清单以validation.json、source-manifest.json及delivery.json为准，未生成的结果不视为通过。

- baseline-save-race.json执行原生产函数与受控延迟，明确复现A保存后切B却回填A。新真实Feature流程最终23项在既有verify-openai-profile-ui入口；不在测试重写状态机。
- 实际生产层级25项、原路由21项；完整挂载最终36项含真实键盘/点击/确认取消、A/B草稿、解析迟到、保存/测试/unknown、导航密钥释放及四尺寸明暗。只替代外部transport，不能当真实服务成功。
- Windows原生12项：真实本机配置读取、Rust解析、明确应用、共享数字输入、布局、草稿/跨页/临时密钥清理与元数据不变。native/report.json及逐会话startup；没有保存/激活/删除真实配置、写入/删除系统密钥或发送计费测试，BA-11仍待。
- Rust格式/严格Clippy及工作区757项测试通过，含4项新后端快照检查；native-build.log为本批debug客户端构建。类型/生产构建与完整96步前端门禁分别记录，不以其中一项替代其他项。
- workflow-initial.log保留安全草稿恢复ID类型遗漏失败，修复白名单可空字符串恢复。browser-initial保留失焦重复解析；browser-final保留确认等待阻碍真实B输入；browser-confirmation保留切换后示例区未退出。对应修复后完整挂载通过，未放宽行为标准。
- frontend-full因原类型用量清单摘要漂移失败；原生成器更新且审查只变Rust扫描摘要/文件数。frontend-final发现旧main函数名守卫，切到真实ProfileExample并维持请求/DOM/文本/修订失效约束；不重开R12或移除生命周期检查。

## 剩余边界、回退与清理

BA-11真实用户Windows Credential Manager写入/替换/删除及权限失败/回滚故障、授权API最小请求和真实旧会话引用读回，仍沿已批准安排由R17-06关闭，R18不得忽略。模拟密钥/回执、原生解析和元数据读取不核销这些外部项；不称Full或本地安装包验收。

回退到0f65d593仅撤回本批代码/记录，旧配置与凭据格式没有迁移；本批若实际用户保存配置属独立业务写入，代码回退不能撤销。安全草稿/未知回执保留，不批量清用户状态。

进程TEMP/TMP继续project-temp.ps1指向O:/Project/FOODBALL/tmp；已单独提醒。用户再次确认唯一tmp/old特定清理长期授权，每次成功推送后核对路径、清单、链接并逐文件实际删除，移除空目录、保留old根、独立复查0；不再让用户手动删除。01旧临时产物已实际清空116,775文件/12.2GB；本批清理须以推送后的实际报告为准，必要证据/依赖/稳定缓存保留。下一项R15-12，未启动。

最终补验：沿原隐藏字段语义，实际保存DTO强制api_workspace_web_search_mode=disabled，历史auto不能绕过，已补生产回归。同行密钥输入与32px按钮按实际中心对齐，浏览器核对输入400/标签500字重与几何。最终完整96步前端（23项流程、25项层级、36项挂载）、757项Rust及12项Windows原生/startup结果分别见validation.json；旧函数名与隐藏字段守卫迁移到真实owner/实际DTO，没有放宽禁联网或生命周期规则。

frontend-complete.log保留几何断言失败：原断言比较输入内层与下拉外层的顶边，边框产生1px差异，但实际中心一致。最终改验中心差小于1px，并保留密钥同行中心、控件尺寸和400/500字重断言；完整96步重新运行通过。诊断坐标见layout-probe/geometry.json。

## 2026-11-05正式收尾

用户授权“收尾07开始12”，本次一次性核对42418cb555de577f50e1b25314e9b66183de4adc、rewrite/r15-ai-profiles-logs-ui、push及ci.yml：运行[41887780319](https://github.com/uniquenesssta/123/actions/runs/41887780319)，job126776786668，completed/success。此前文中“未启动12”描述是本批原交付时状态，现已由用户启动12。

官方artifact13355059463，ZIP52,470,175字节，SHA256为9c509ad48ce8d8f919a3b03940b14236b790a5801cb6c44255184aaef2e5b30a，与官方digest一致；MSI18,063,360字节和NSIS15,694,461字节均核对名称、非零长度、文件头与SHA256。逐项值及官方文件保存在O:/Project/FOODBALL/logs/r15-07-ci-41887780319/acceptance.json。官方运行日志按原startup分析器通过：7条、0非法行、0运行错误。

未在本机安装这两个包、未核验签名，不把官方打包或startup当Full。旧delivery已补formalAcceptance。原凭据与计费API未验项仍由BA-11/R17-06关闭。12的CI/安装包须使用12自己的提交，不能复用07结果。
