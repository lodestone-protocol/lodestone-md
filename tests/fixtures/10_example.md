<!-- mddag: {"version":"1.3"} -->

# 流体实验记录
<!-- mddag: {"id":"exp-01","status":"aligned","tags":["experiment"]} -->
实验数据与原始观测记录。

# 采集参数快照
<!-- mddag: {"id":"exp-01-config","status":"aligned"} -->
采集环境如下：

```text
# 通道配置（位于围栏内，不构成节点边界）
channel-1: 压力传感器
channel-2: 流速计
```

# 实验结论
<!-- mddag: {"id":"concl-01","status":"converged","edges":[{"to":"exp-01","relation":"depend"},{"to":"exp-01-config","relation":"depend"}]} -->
基于 [实验记录](#流体实验记录) 的推导。

# 反例观察
<!-- mddag: {"id":"counter-01","status":"draft","edges":[{"to":"concl-01","relation":"refute"}]} -->
一次与结论冲突的观测。

# 后续计划
<!-- mddag: {"id":"plan-01","status":"draft","edges":[{"to":"old-note","relation":"depend"}]} -->
待定。
