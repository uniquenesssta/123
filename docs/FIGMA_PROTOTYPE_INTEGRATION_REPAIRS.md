# 全产品原型集成修复记录

## INT-04：工作区历史／重置首批

**INT-04 工作区历史／重置首批（2026-10-09）：限定存储动作解释与原生变量求值复验通过。17页中的113个既有预置状态已接入独立工作区历史和确认重置；全705状态现有的2587个模块／页面导航入口已接入清栈。新增2个明暗确认框、17个不发布的原型变量；当前1079根、705产品状态、356弹窗、17路由。其余592状态、未配对详情、真实输入／滚动／焦点和浏览器全事件回放仍待完成，整体 NOT_PASSED。恢复见[第四批数据](FIGMA_PROTOTYPE_REPAIR_BATCH04.json)。**

> 以下显隐与前三批文字为历史快照；当前范围以本段和第四批记录为准。既有193个设计变量和组件母版未改，原型专用变量现为23个。

历史使用既有详情动作及独立变量，不调用浏览器 BACK。保留基础页和最多八个详情；超出时丢弃最早详情，基础页仍保留。后退后打开新详情会截断前进分支。模块或页面入口把目标初始化为空栈，主题及直接显隐不写历史变量。History容器保留既有结构，只绑定两个子按钮的State。

重置先打开复用组件组成的明暗确认框。取消和关闭只执行CLOSE；确认进入本页中立预置并清空历史。工作包保留当前类别；主题、显隐和持久化业务数据不写入。此组对所有可用重置都保守确认，没有任意输入的dirty检测器。现有保存、执行、请求、删除及危险数据库操作没有被接成重置。

本批按精确节点限定覆盖113状态，不能写成全705状态历史／重置完成。705仅指现有页面与模块导航的清栈扫描。存储动作解释及变量求值不等于浏览器点击、真实表单／滚动／焦点持续性，旧禁用实例零动作统计也不覆盖新受条件保护的历史按钮。已有来源返回、主题和显隐结果保持独立依据。当前实读Topbar存在Expanded／Collapsed两种静态值；重要约束是没有Navigation变体变量绑定，显隐仍由原四个布尔变量控制，未强制改成单一变体。

## INT-04 续作：直接显隐修复及限定复验通过

**2026-10-10 INT-04 续作检查点：705个状态已改为直接显隐，保留677展开／28折叠的初始布局。全705状态分组回读通过；28个来源与2个独立档案返回、34个默认页272个导航入口、24个代表状态变量切换均无异常。来源基础资料截图正常。批量恢复664处顶栏页名、24处数据库提示，另修正1处样本页名。限定显隐复验通过，但工作区历史、统一重置和浏览器全事件回放仍未完成；整体 NOT_PASSED。恢复见[本次记录](FIGMA_PROTOTYPE_REPAIR_BATCH03R1.json)。**

重新读取发现旧方案的Shell／Topbar变体绑定会使嵌套页名和数据库提示恢复默认；隔离变量试验还发现展开入口动作丢失。本次固定Shell与Topbar结构，将二级栏容器和展开按钮分别绑定互补布尔变量，避免用户折叠时替换整套嵌套控件。样本的展开→折叠→展开已通过插件变量求值与独立动作回读，工作区宽度相差154。

705状态分8组完成转换与接线，记录修改前模板、节点与成功回执。保留原有控件动作不等于统一重置或历史已实现；第三批之前的重置正确性未由本轮证明。未增加产品路由或修改应用源码、共享组件母版。当前为1077根、705状态、17路由；新增4个不发布的布尔变量，原2个字符串变量暂保留。

分组复验结果：705状态的初始显隐、变量绑定、页名、数据库提示与双向按钮均0异常；28个来源档案返回对应主题首尔FC，2个独立档案仍返回球员目录；34个默认明暗页面272个入口路由／主题0异常。24个代表状态（含10个旧超时状态）经过折叠→展开→初始值恢复，控件属性与动作不变，工作区宽度差符合154px二级导航宽度。变量默认值最终恢复true／false／false／true。来源基础资料1440×900截图中文、表单、页名及返回入口正常。

一次性705状态和28来源状态请求超时，不计通过；本次结果来自后续10组／4组成功读取。插件变量求值与存储动作回读不等于浏览器点击回放。下一项：实现工作区详情历史和统一重置，并继续完整原型复验。

---

## 第三批：INT-04 导航显隐实施，复验未完成（2026-10-09）

**2026-10-09 第三批修复检查点：已为705个产品状态绑定导航显隐变量，并在展开／折叠两态保留顶栏控件。695个状态有完整显隐写入回执；另10个超时状态已回读绑定，仍须复核两态动作。705个状态的五类顶栏控件已完成属性／动作保留比较。复验接口超时、截图部分中文字缺失且浏览器原型页不可访问，本批复验尚未完成，整体仍为 NOT_PASSED。工作区前进／返回和统一重置尚未实施。详见[修复记录](FIGMA_PROTOTYPE_INTEGRATION_REPAIRS.md)和[第三批节点与恢复数据](FIGMA_PROTOTYPE_REPAIR_BATCH03.json)。**

本批基线提交 `62e894ff496bf2719e8485658badfccba0ea2e71`。本记录和第三批JSON、独立集成状态、README、两份计划及总状态在同一提交保存。应用源码和共享组件母版未修改，未新增产品页面。

### 实际实施

- 705个现有状态绑定导航显隐变量；保留677展开、28折叠的初始布局。点击折叠／展开时同时设置两个原型变量，后续绑定状态沿用用户选择，不用跳转到另一个业务预置画面代替显隐。
- 695个状态具有完整双态写入回执；10个超时状态已回读Shell／Topbar变量绑定和当前标题，但两态动作的完整回执缺失，不能跳过复核。节点清单在第三批JSON的timeoutReconciliation.collapseRange。
- 修正变体切换使顶栏位置文案、数据库状态提示及重置状态恢复默认的问题。705个状态的History、Back、Forward、Configuration、Reset五类控件在两态保留既有属性和动作；成功写入中进行了7,050次比较。这只保留已有行为，不代表实现了详情历史或统一重置。
- 新增独立的、不发布的FIP Prototype Workspace集合与两个STRING变量。已有193个设计变量、13个文字样式和6个效果样式未修改；总变量数增加到195。
- 临时验证根`734:251793`已删除，画布恢复1,077个根。验证用全局值最终恢复并回读为Expanded／Collapsed。

### 复验边界与阻塞

较大的235状态复验请求、12状态复验请求以及最后的折叠来源返回核对未成功返回，不能计为通过。浏览器原型页显示Site Unavailable，没有执行真实事件回放。连接器截图部分中文字缺字；节点文本回读仍完整包含“来自首尔 FC”和“返回首尔 FC 完整档案”，未将截图缺字推断为内容被删除，也未将其计为视觉复验通过。

83状态业务内容／动作比较仅准备了脚本，**没有执行完成**；本批没有新宣称全705状态普通主题图扫描、全部禁用实例检查或真实草稿／滚动保持通过。前两批的限定结果作为原有证据保留。

第三批JSON保存成功调用的修改前数据和节点回执；超时调用的回执缺口单独列出，不是完整离线Figma备份。

### 恢复顺序

先完成 INT-04 导航显隐复验（优先核对10个超时状态的两态动作及来源／导航回归），再实施工作区详情历史与重置，最后重复全产品验收。

先读取`docs/FIGMA_PROTOTYPE_REPAIR_BATCH03.json`和`docs/FIGMA_PROTOTYPE_INTEGRATION_STATE.json`，核对两个原型变量为Expanded／Collapsed、临时根已删除，再从列出的10个状态开始复验。不要重做已接入的705个状态，也不要把重置保护的“保留原动作”误认为“统一重置已实现”。

---


**2026-10-09 第二批修复检查点：INT-01 已修复；INT-03 的球队来源档案返回与 INT-02 的普通导航主题连续性已通过存储动作复验。705 个产品状态的普通主题变化候选为 0；28 个明暗来源档案保持首尔 FC 返回。整体仍为 NOT_PASSED，INT-04 待修复及全产品复验。未进行浏览器全事件回放或真实运行时验收。详见[修复记录](FIGMA_PROTOTYPE_INTEGRATION_REPAIRS.md)和[本批节点与复验数据](FIGMA_PROTOTYPE_REPAIR_BATCH02.json)。**

## 第二批：来源返回与普通主题连续性

基线提交 `5c71a8c099e14da541ff9c8198624a86a86c7232`。本批实际写入 Figma 后回读；没有修改应用源码或共享组件。

- INT-03：为 A01 新增 13 个球队来源资料/校验/保存状态，与原来源概览组成 14 个浅色状态；配对深色后共 28 个。各子区保留“来自首尔 FC”，返回对应主题的首尔 FC 完整档案。独立目录入口仍返回球员目录。13 个新增来源状态已加入原 S07 Review Hub，未新增路由或验收目录。
- INT-02：按原 156 条静态候选及其后续普通路径补齐 131 个深色业务状态，连接 262 个同业务状态主题切换按钮。131 份克隆的全部文字与来源逐一一致，保留失败、未选择、保存中、结果未知等业务语义。
- 原 156 条候选中，146 条改接同主题状态，10 条变为跳回当前顶层画面的冗余动作并移除。包含新增深色状态内部接线，本批共处理 1,523 条普通动作；该数不是独立缺陷数。
- 全部 17 阶段、705 个产品状态重新扫描，普通主题变化候选 0。28 个来源档案返回与子区目标检查 0 异常。实际截图检查浅/深来源基础资料、深色球队刷新、数据库连接、AI 历史、复盘未选择及 S07 验收目录。
- 对本批涉及的 288 个产品状态和 1 个验收目录补查：失效/越出 Screens 的导航目标 0；1,926 个可见 Disabled 实例自身及可点击祖先旁路 0。此计数是本批范围，不替代原 3,738 个的历史全量统计。
- 当前 1,077 个根：705 状态、354 弹窗、1 上下文菜单、17 验收目录；始终为 17 条产品路由。新增 144 个状态根和 13 个目录入口，本批直接修改节点 1,932 个（包含新增状态后代的接线，不全是旧画面）。

复验范围仍是存储动作及代表性截图；不包含弹窗主题、所有悬停/键盘事件的浏览器动态回放、任意草稿/来源组合或运行时事务。INT-04 全局返回/前进、折叠、重置仍 OPEN，整体 NOT_PASSED。

完整来源映射、131 组主题映射、旧/新目标、原 Theme 反应、克隆文字校验和逐页结果见 [第二批数据](FIGMA_PROTOTYPE_REPAIR_BATCH02.json)。这是节点级恢复记录，不是整个 Figma 文件离线备份；10 个已移除冗余动作只保留旧目标与索引，不声称具备完整原反应序列化备份。

下一批：INT-04，然后整体验收复验。

## 第一批历史记录

## INT-01 完成范围

基线：`86d1ec5e54e98fb657518d476e5f0af8c0696436`；唯一分支 `ui-design-system`；Figma `PN0Whgu6HLWIHx4Mv6aHfu` / Screens `3:5`。

实际补92个无点击入口、替换4个仅说明的管理入口，共96个实例。原有非点击反应保留。新增连接使用对应Light/Dark模块默认页，不转移对象、草稿、请求、权限或凭据；仅修改验收列明的34个默认画面。无数据库、保存中、结果未知等特殊状态保持原接线。现有933个根节点、主组件、视觉几何和应用源码未变。

独立回读34页：272个非选中导航全有产品目标，缺口0、错误路由0。检查184个可见Disabled实例，自身/祖先指针旁路0。该数量仅为本次34默认页范围，不代替原全量3738个实例。8条原有主题不连续仍归INT-02；修复未声称主题问题已完成。未进行浏览器悬停后事件回放。

## 第一批当时的后续计划（已由第二批进度更新）

当时计划先联合修复 INT-03 与 INT-02，再处理 INT-04；现前两项限定范围已完成，下一步为 INT-04。

## 回读证据

```json
{
  "roots": 34,
  "nonselected": 272,
  "gaps": [],
  "badTargets": [],
  "themeMismatch": [
    [
      "470:18992",
      "Page/review",
      "505:34022"
    ],
    [
      "490:24140",
      "Page/review",
      "505:34022"
    ],
    [
      "516:50441",
      "Page/review",
      "505:34022"
    ],
    [
      "579:125239",
      "Module/home",
      "453:2"
    ],
    [
      "579:125239",
      "Module/matches",
      "466:4436"
    ],
    [
      "579:125239",
      "Page/teams",
      "527:54630"
    ],
    [
      "579:125239",
      "Page/players",
      "550:69407"
    ],
    [
      "579:125239",
      "Page/lineup_presets",
      "570:86212"
    ]
  ],
  "disabledChecked": 184,
  "disabledViolations": [],
  "canvasRootCount": 933
}
```

## 原反应及新目标恢复表

格式：[被修改节点ID、所在根ID、新目标ID、修改前完整反应]。仅用于逐节点恢复，不是整份Figma离线备份。

```json
[
  [
    "I466:4437;380:1087;376:328",
    "466:4436",
    "589:112453",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I466:4437;380:1087;376:339",
    "466:4436",
    "608:135680",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I466:4437;380:1087;376:350",
    "466:4436",
    "628:152166",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I466:4437;380:1087;376:361",
    "466:4436",
    "673:178191",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I487:22279;380:1087;376:339",
    "487:22278",
    "608:135680",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I487:22279;380:1087;376:350",
    "487:22278",
    "628:152166",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I487:22279;380:1087;376:361",
    "487:22278",
    "673:178191",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I505:33974;380:1087;376:328",
    "505:33973",
    "589:112453",
    []
  ],
  [
    "I505:33974;380:1087;376:339",
    "505:33973",
    "608:135680",
    []
  ],
  [
    "I505:33974;380:1087;376:350",
    "505:33973",
    "628:152166",
    []
  ],
  [
    "I505:33974;380:1087;376:361",
    "505:33973",
    "673:178191",
    []
  ],
  [
    "I516:48547;380:1087;376:328",
    "516:48546",
    "589:112453",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I516:48547;380:1087;376:339",
    "516:48546",
    "608:135680",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I516:48547;380:1087;376:350",
    "516:48546",
    "628:152166",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I516:48547;380:1087;376:361",
    "516:48546",
    "673:178191",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I527:54631;380:1087;376:422",
    "527:54630",
    "589:112453",
    []
  ],
  [
    "I527:54631;380:1087;376:433",
    "527:54630",
    "608:135680",
    []
  ],
  [
    "I527:54631;380:1087;376:444",
    "527:54630",
    "628:152166",
    []
  ],
  [
    "I527:54631;380:1087;376:455",
    "527:54630",
    "673:178191",
    []
  ],
  [
    "I550:69408;380:1087;376:422",
    "550:69407",
    "589:112453",
    []
  ],
  [
    "I550:69408;380:1087;376:433",
    "550:69407",
    "608:135680",
    []
  ],
  [
    "I550:69408;380:1087;376:444",
    "550:69407",
    "628:152166",
    []
  ],
  [
    "I550:69408;380:1087;376:455",
    "550:69407",
    "673:178191",
    []
  ],
  [
    "I570:86213;380:1087;376:422",
    "570:86212",
    "589:112453",
    []
  ],
  [
    "I570:86213;380:1087;376:433",
    "570:86212",
    "608:135680",
    []
  ],
  [
    "I570:86213;380:1087;376:444",
    "570:86212",
    "628:152166",
    []
  ],
  [
    "I570:86213;380:1087;376:455",
    "570:86212",
    "673:178191",
    []
  ],
  [
    "I579:102497;380:1087;376:433",
    "579:102496",
    "608:135680",
    []
  ],
  [
    "I579:102497;380:1087;376:444",
    "579:102496",
    "628:152166",
    []
  ],
  [
    "I579:102497;380:1087;376:455",
    "579:102496",
    "673:178191",
    []
  ],
  [
    "I589:112454;380:1087;376:493",
    "589:112453",
    "466:4436",
    []
  ],
  [
    "I589:112454;380:1087;376:527",
    "589:112453",
    "608:135680",
    []
  ],
  [
    "I589:112454;380:1087;376:538",
    "589:112453",
    "628:152166",
    []
  ],
  [
    "I589:112454;380:1087;376:549",
    "589:112453",
    "673:178191",
    []
  ],
  [
    "I597:123772;380:1087;376:538",
    "597:123771",
    "628:152166",
    []
  ],
  [
    "I597:123772;380:1087;376:549",
    "597:123771",
    "673:178191",
    []
  ],
  [
    "I608:135681;380:1087;376:643",
    "608:135680",
    "673:178191",
    []
  ],
  [
    "I628:152167;380:1087;376:737",
    "628:152166",
    "673:178191",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "635:166209",
          "navigation": "OVERLAY",
          "transition": null,
          "resetVideoPosition": false,
          "resetScrollPosition": true
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "635:166209",
            "navigation": "OVERLAY",
            "transition": null,
            "resetVideoPosition": false,
            "resetScrollPosition": true
          }
        ],
        "trigger": {
          "type": "ON_CLICK"
        }
      }
    ]
  ],
  [
    "I647:166716;380:1087;376:737",
    "647:166715",
    "673:178191",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "656:178220",
          "navigation": "OVERLAY",
          "transition": null,
          "resetVideoPosition": false,
          "resetScrollPosition": true
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "656:178220",
            "navigation": "OVERLAY",
            "transition": null,
            "resetVideoPosition": false,
            "resetScrollPosition": true
          }
        ],
        "trigger": {
          "type": "ON_CLICK"
        }
      }
    ]
  ],
  [
    "I673:178192;380:1087;376:775",
    "673:178191",
    "466:4436",
    []
  ],
  [
    "I673:178192;380:1087;376:787",
    "673:178191",
    "527:54630",
    []
  ],
  [
    "I673:178192;380:1087;376:798",
    "673:178191",
    "589:112453",
    []
  ],
  [
    "I673:178192;380:1087;376:809",
    "673:178191",
    "608:135680",
    []
  ],
  [
    "I683:186759;380:1087;376:775",
    "683:186758",
    "466:4436",
    []
  ],
  [
    "I683:186759;380:1087;376:787",
    "683:186758",
    "527:54630",
    []
  ],
  [
    "I683:186759;380:1087;376:798",
    "683:186758",
    "589:112453",
    []
  ],
  [
    "I683:186759;380:1087;376:809",
    "683:186758",
    "608:135680",
    []
  ],
  [
    "I683:186759;380:1087;376:820",
    "683:186758",
    "628:152166",
    []
  ],
  [
    "I470:18993;380:1087;376:328",
    "470:18992",
    "589:113717",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I470:18993;380:1087;376:339",
    "470:18992",
    "608:137065",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I470:18993;380:1087;376:350",
    "470:18992",
    "630:152499",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I470:18993;380:1087;376:361",
    "470:18992",
    "674:178498",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I490:24141;380:1087;376:339",
    "490:24140",
    "608:137065",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I490:24141;380:1087;376:350",
    "490:24140",
    "630:152499",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I490:24141;380:1087;376:361",
    "490:24140",
    "674:178498",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I505:35346;380:1087;376:328",
    "505:35345",
    "589:113717",
    []
  ],
  [
    "I505:35346;380:1087;376:339",
    "505:35345",
    "608:137065",
    []
  ],
  [
    "I505:35346;380:1087;376:350",
    "505:35345",
    "630:152499",
    []
  ],
  [
    "I505:35346;380:1087;376:361",
    "505:35345",
    "674:178498",
    []
  ],
  [
    "I516:50442;380:1087;376:328",
    "516:50441",
    "589:113717",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I516:50442;380:1087;376:339",
    "516:50441",
    "608:137065",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I516:50442;380:1087;376:350",
    "516:50441",
    "630:152499",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I516:50442;380:1087;376:361",
    "516:50441",
    "674:178498",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "374:66",
          "navigation": "CHANGE_TO",
          "transition": null,
          "resetVideoPosition": false
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "374:66",
            "navigation": "CHANGE_TO",
            "transition": null,
            "resetVideoPosition": false
          }
        ],
        "trigger": {
          "type": "ON_HOVER"
        }
      }
    ]
  ],
  [
    "I527:60126;380:1087;376:422",
    "527:60125",
    "589:113717",
    []
  ],
  [
    "I527:60126;380:1087;376:433",
    "527:60125",
    "608:137065",
    []
  ],
  [
    "I527:60126;380:1087;376:444",
    "527:60125",
    "630:152499",
    []
  ],
  [
    "I527:60126;380:1087;376:455",
    "527:60125",
    "674:178498",
    []
  ],
  [
    "I550:80301;380:1087;376:422",
    "550:80300",
    "589:113717",
    []
  ],
  [
    "I550:80301;380:1087;376:433",
    "550:80300",
    "608:137065",
    []
  ],
  [
    "I550:80301;380:1087;376:444",
    "550:80300",
    "630:152499",
    []
  ],
  [
    "I550:80301;380:1087;376:455",
    "550:80300",
    "674:178498",
    []
  ],
  [
    "I574:102172;380:1087;376:422",
    "574:102171",
    "589:113717",
    []
  ],
  [
    "I574:102172;380:1087;376:433",
    "574:102171",
    "608:137065",
    []
  ],
  [
    "I574:102172;380:1087;376:444",
    "574:102171",
    "630:152499",
    []
  ],
  [
    "I574:102172;380:1087;376:455",
    "574:102171",
    "674:178498",
    []
  ],
  [
    "I579:125240;380:1087;376:433",
    "579:125239",
    "608:137065",
    []
  ],
  [
    "I579:125240;380:1087;376:444",
    "579:125239",
    "630:152499",
    []
  ],
  [
    "I579:125240;380:1087;376:455",
    "579:125239",
    "674:178498",
    []
  ],
  [
    "I589:113718;380:1087;376:493",
    "589:113717",
    "470:18992",
    []
  ],
  [
    "I589:113718;380:1087;376:527",
    "589:113717",
    "608:137065",
    []
  ],
  [
    "I589:113718;380:1087;376:538",
    "589:113717",
    "630:152499",
    []
  ],
  [
    "I589:113718;380:1087;376:549",
    "589:113717",
    "674:178498",
    []
  ],
  [
    "I597:124760;380:1087;376:538",
    "597:124759",
    "630:152499",
    []
  ],
  [
    "I597:124760;380:1087;376:549",
    "597:124759",
    "674:178498",
    []
  ],
  [
    "I608:137066;380:1087;376:643",
    "608:137065",
    "674:178498",
    []
  ],
  [
    "I630:152500;380:1087;376:737",
    "630:152499",
    "674:178498",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "635:166209",
          "navigation": "OVERLAY",
          "transition": null,
          "resetVideoPosition": false,
          "resetScrollPosition": true
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "635:166209",
            "navigation": "OVERLAY",
            "transition": null,
            "resetVideoPosition": false,
            "resetScrollPosition": true
          }
        ],
        "trigger": {
          "type": "ON_CLICK"
        }
      }
    ]
  ],
  [
    "I651:167035;380:1087;376:737",
    "651:167034",
    "674:178498",
    [
      {
        "action": {
          "type": "NODE",
          "destinationId": "656:178220",
          "navigation": "OVERLAY",
          "transition": null,
          "resetVideoPosition": false,
          "resetScrollPosition": true
        },
        "actions": [
          {
            "type": "NODE",
            "destinationId": "656:178220",
            "navigation": "OVERLAY",
            "transition": null,
            "resetVideoPosition": false,
            "resetScrollPosition": true
          }
        ],
        "trigger": {
          "type": "ON_CLICK"
        }
      }
    ]
  ],
  [
    "I674:178499;380:1087;376:775",
    "674:178498",
    "470:18992",
    []
  ],
  [
    "I674:178499;380:1087;376:787",
    "674:178498",
    "527:60125",
    []
  ],
  [
    "I674:178499;380:1087;376:798",
    "674:178498",
    "589:113717",
    []
  ],
  [
    "I674:178499;380:1087;376:809",
    "674:178498",
    "608:137065",
    []
  ],
  [
    "I684:187052;380:1087;376:775",
    "684:187051",
    "470:18992",
    []
  ],
  [
    "I684:187052;380:1087;376:787",
    "684:187051",
    "527:60125",
    []
  ],
  [
    "I684:187052;380:1087;376:798",
    "684:187051",
    "589:113717",
    []
  ],
  [
    "I684:187052;380:1087;376:809",
    "684:187051",
    "608:137065",
    []
  ],
  [
    "I684:187052;380:1087;376:820",
    "684:187051",
    "630:152499",
    []
  ]
]
```

