# 全产品原型集成修复记录

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
