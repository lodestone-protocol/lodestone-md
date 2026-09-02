# Golden Fixture 清单（一致性语料，规范章节来源）

> 本清单是 fixtures/ 的规范章节索引：每个 fixture 反向标注其覆盖的
> Lodestone Protocol (MD-DAG) v1.3 Final 章节（防协议演进时语料漂移）。
> fixtures/*.md 保持字节稳定（头注释会改变解析结果，故来源标注集中于此）。

| Fixture | 覆盖章节 / 码 | 关键断言 |
|---|---|---|
| 10_example | §10（黄金基准） | chars 12/74/23/11/3；区间 5–5/9–15/19–19/23–23/27–27；全局图为空；E-REF-NOT-FOUND |
| docmeta_absent | §3.1（缺失路径） | 无文档级元数据无警告 |
| docmeta_version_mismatch | §3.1 + §11 | W-VERSION-MISMATCH，继续解析 |
| docmeta_bad_json | §3.1 + §11 | W-DOC-META，忽略之 |
| docmeta_dup_key | §3.1 + §11 | 文档级重复键 W-DOC-META |
| meta_dup_key | §5.2 + §11 | 节点级重复键 E-META-SYNTAX，节点无效 |
| meta_syntax_multiline | §5.1 + §11 | 跨行/未闭合 E-META-SYNTAX；不构成排除行 |
| meta_variant | §5.1 + §11 | 近似前缀变体 W-META-PLACEMENT；无元数据；不构成排除行 |
| meta_field_degrade | §5.3 + §11 | 字段层降级 E-META-FIELD（status/边×2/tags），节点仍有效 |
| dup_id | §4.2 E-DUP-ID | 同 id 全无效 + 不级联 + 引用报 E-REF-NOT-FOUND |
| slug_cases | §4.2 | 隐式 slug、空 slug → E-MISSING-ID、非法声明 id → E-MISSING-ID |
| edges_derive_fold | §7.2 | derive 转置 + 折叠 W-REDUNDANT-EDGE |
| cycle_global | §7.3 E-CYCLE | 循环边全集失效，其余子图正常（cc→cd 生效） |
| cycle_declared | §7.3 W-CYCLE-DECLARED | 声明层软环仅警告，无结构影响 |
| upstream_pending | §6 | aligned 节点非 aligned 上游 W-UPSTREAM-PENDING |
| boundary_fences | §4.1 | 缩进/容器/围栏/缩进代码块内标题不切分 |
| node0_preamble | §3.2 | 前言不进节点表；mddag 注释 W-META-PLACEMENT |
| body_blank_edges | §8.1 | 首尾空行剔除、内部空行保留、chars=0、尾随空格计入 |
| crlf_endings | §8.1 | CRLF 行号与 chars |
| nfc_violation | §4.2 W-NFC-VIOLATION | NFD 标题警告 |
| redundant_meta | §5.1 W-REDUNDANT-META | 采纳后后续注释忽略并告警 |
| no_nodes | §3.2/§4 | 无节点文档：空节点表、空图 |

§11 码覆盖率：E-MISSING-ID / E-META-SYNTAX / E-META-FIELD / E-DUP-ID /
E-REF-NOT-FOUND / E-CYCLE / W-VERSION-MISMATCH / W-DOC-META /
W-CYCLE-DECLARED / W-REDUNDANT-EDGE / W-META-PLACEMENT /
W-REDUNDANT-META / W-UPSTREAM-PENDING / W-NFC-VIOLATION — 14/14。
