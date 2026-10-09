# 全产品原型集成修复记录

**2026-10-09 修复检查点：INT-01 的96处默认导航缺口已实际补齐并回读；34个默认明暗画面272/272个非选中入口具有正确产品路由，缺口与错误目标均0。整体仍未通过；INT-02、INT-03、INT-04待修复。默认导航中另有8条既存主题不连续，保留归入INT-02。仅验证存储动作，未作完整浏览器事件回放。详见[修复记录](FIGMA_PROTOTYPE_INTEGRATION_REPAIRS.md)。**

## INT-01 完成范围

基线：`86d1ec5e54e98fb657518d476e5f0af8c0696436`；唯一分支 `ui-design-system`；Figma `PN0Whgu6HLWIHx4Mv6aHfu` / Screens `3:5`。

实际补92个无点击入口、替换4个仅说明的管理入口，共96个实例。原有非点击反应保留。新增连接使用对应Light/Dark模块默认页，不转移对象、草稿、请求、权限或凭据；仅修改验收列明的34个默认画面。无数据库、保存中、结果未知等特殊状态保持原接线。现有933个根节点、主组件、视觉几何和应用源码未变。

独立回读34页：272个非选中导航全有产品目标，缺口0、错误路由0。检查184个可见Disabled实例，自身/祖先指针旁路0。该数量仅为本次34默认页范围，不代替原全量3738个实例。8条原有主题不连续仍归INT-02；修复未声称主题问题已完成。未进行浏览器悬停后事件回放。

## 下一批

按INT-03与INT-02联合处理来源身份与主题，再处理INT-04；整体NOT_PASSED保持。不得以统一默认目标替代来源表单、草稿或未知结果。

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
