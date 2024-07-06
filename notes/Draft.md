## DepotAnalyzer

**DepotMatchData** 中定义的 roi 为 `[0, 0, 1066, 599]`

换算到 2560x1440 下为 `[0, 0, 2132, 1198]`



Maa 中 1280x720 分辨率下任务中定义的物品 roi 为 `[0, 190, 156, 190]`

`start_y` 为 `190`，宽高周期分别为 `156` 和 `190`

换算到 2560x1440 下：

`start_y` 为 `380`，宽高周期为 `312` 和 `380`

这个 `start_y` 恰好位于物品的中心位置。



将图像转换为灰度值，按 y 轴拍扁计算平均值，于是可以视作一个信号，而这个信号的关键组成部分就是包含着由物品排列产生的周期信号。

借助傅里叶变换，可以得到其相位，也即第一列物品的 x 中心坐标。

![tmp_original](assets/tmp_original.png)

![tmp_gray](assets/tmp_gray.png)

![tmp_hist_x](assets/tmp_hist_x.png)

归一化：

![tmp_hist_x_normalized](assets/tmp_hist_x_normalized.png)

---

```
---- vision::matcher::single_matcher::test::test_devices stdout ----
#### testing device MUMU ####
== testing start_start.png with start.png ==
[SingleMatcher]: image: 1920x1080, template: 46x43, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.18581371s, Extremes { max_value: 1207.2903, min_value: 0.0, max_value_location: (821, 500), min_value_location: (937, 1001) }
[SingleMatcher]: success!
Some(Rect { x: 937, y: 1001, width: 46, height: 43 })
== testing wakeup_wakeup.png with wakeup.png ==
[SingleMatcher]: image: 1920x1080, template: 294x90, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.246691s, Extremes { max_value: 8321.12, min_value: 0.0, max_value_location: (1072, 841), min_value_location: (813, 721) }
[SingleMatcher]: success!
Some(Rect { x: 813, y: 721, width: 294, height: 90 })
== testing main_base.png with main.png ==
[SingleMatcher]: image: 1920x1080, template: 141x111, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.2600485s, Extremes { max_value: 10195.935, min_value: 152.61916, max_value_location: (760, 74), min_value_location: (22, 908) }
[SingleMatcher]: failed
None
== testing main_mission.png with main.png ==
[SingleMatcher]: image: 1920x1080, template: 130x115, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.2577141s, Extremes { max_value: 10827.955, min_value: 152.61916, max_value_location: (1573, 74), min_value_location: (780, 902) }
[SingleMatcher]: failed
None
== testing main_operator.png with main.png ==
[SingleMatcher]: image: 1920x1080, template: 173x121, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.25271866s, Extremes { max_value: 10906.95, min_value: 152.61916, max_value_location: (1259, 76), min_value_location: (1110, 924) }
[SingleMatcher]: failed
None
== testing main_squads.png with main.png ==
[SingleMatcher]: image: 1920x1080, template: 118x99, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.26568112s, Extremes { max_value: 10301.004, min_value: 152.61916, max_value_location: (861, 73), min_value_location: (774, 896) }
[SingleMatcher]: failed
None
== testing main_recruit.png with main.png ==
[SingleMatcher]: image: 1920x1080, template: 143x73, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.26766172s, Extremes { max_value: 7349.7046, min_value: 154.99184, max_value_location: (1059, 73), min_value_location: (1321, 922) }
[SingleMatcher]: failed
None
== testing notice_close.png with notice.png ==
[SingleMatcher]: image: 1920x1080, template: 56x51, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.2715552s, Extremes { max_value: 7421.4766, min_value: 0.0, max_value_location: (402, 686), min_value_location: (1622, 1006) }
[SingleMatcher]: success!
Some(Rect { x: 1622, y: 1006, width: 56, height: 51 })
== testing back.png with mission.png ==
[SingleMatcher]: image: 1920x1080, template: 179x63, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.2676161s, Extremes { max_value: 12156.099, min_value: 158.92026, max_value_location: (1122, 106), min_value_location: (977, 980) }
[SingleMatcher]: failed
None
== testing main_base.png with start.png ==
[SingleMatcher]: image: 1920x1080, template: 141x111, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.26034796s, Extremes { max_value: 9308.418, min_value: 158.92026, max_value_location: (953, 905), min_value_location: (1117, 959) }
[SingleMatcher]: failed
None
== testing main_mission.png with start.png ==
[SingleMatcher]: image: 1920x1080, template: 130x115, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.25657475s, Extremes { max_value: 9308.418, min_value: 158.92026, max_value_location: (1744, 899), min_value_location: (1314, 953) }
[SingleMatcher]: failed
None
== testing main_operator.png with start.png ==
[SingleMatcher]: image: 1920x1080, template: 173x121, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.26107916s, Extremes { max_value: 9308.418, min_value: 471.52676, max_value_location: (197, 922), min_value_location: (1349, 959) }
[SingleMatcher]: failed
None
== testing main_squads.png with start.png ==
[SingleMatcher]: image: 1920x1080, template: 118x99, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.26686472s, Extremes { max_value: 9184.669, min_value: 158.92026, max_value_location: (1421, 895), min_value_location: (696, 947) }
[SingleMatcher]: failed
None
== testing main_recruit.png with start.png ==
[SingleMatcher]: image: 1920x1080, template: 143x73, method: SumOfSquaredErrors, matching...
[SingleMatcher]: cost: 0.26422253s, Extremes { max_value: 8181.9126, min_value: 158.92026, max_value_location: (17, 923), min_value_location: (1257, 960) }
[SingleMatcher]: failed
None
```

