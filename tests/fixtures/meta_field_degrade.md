# 降级示范
<!-- mddag: {"id":"degrade-01","status":"frozen","edges":[{"to":"ghost-01","relation":"depend"},{"to":"target-01","relation":"depends"},{"relation":"depend"}],"tags":"not-array"} -->
四个字段级降级（status/边×2/tags），节点仍有效；ghost-01 边后报 E-REF-NOT-FOUND。
# 正常目标
<!-- mddag: {"id":"target-01","status":"draft"} -->
被引用目标。
