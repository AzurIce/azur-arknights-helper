首先识别 BattleStartAll



```json
"BattleStartAll": {
        "action": "DoNothing",
        "algorithm": "JustReturn",
        "next": [
            "BattleStartNormal",
            "BattleStartRaid",
            "BattleStartExercise",
            "BattleStartAdverse",
            "BattleStartSimulation",
            "BattleStartTrials",
            "BattleStartTrialsRaid",
            "BattleStartOCR"
        ]
    },
    "BattleStartNormal": {
        "baseTask": "BattleStart"
    },
    "BattleStartRaid": {
        "baseTask": "BattleStart"
    },
    "BattleStartExercise": {
        "baseTask": "BattleStart"
    },
    "BattleStartAdverse": {
        "baseTask": "BattleStart"
    },
    "BattleStartTrials": {
        "baseTask": "BattleStart"
    },
    "BattleStartTrialsRaid": {
        "baseTask": "BattleStart"
    },
    "BattleStartOCR": {
        "baseTask": "BattleStart",
        "algorithm": "OcrDetect",
        "text": [
            "开始",
            "行动"
        ],
        "roi": [
            999,
            398,
            222,
            237
        ]
    },
    "BattleStartSimulation": {
        "action": "ClickSelf",
        "roi": [
            417,
            567,
            444,
            153
        ],
        "postDelay": 2000
    },
```

