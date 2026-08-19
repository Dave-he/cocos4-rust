# cocos4-rust 接口 symbol 对照基线

生成日期：2026-05-27

本文件由 `scripts/generate-interface-symbol-map.py` 生成。它是逐接口复核的机器基线，不是 100% 兼容声明。

## 覆盖范围

- cocos4 侧：扫描 `../cocos4/cocos` 与 `../cocos4/exports` 中的 `export class/enum/interface/function/const/let/var/type/namespace` 以及单行 `export { ... }`。
- cocos4-rust 侧：扫描 `src/**/*.rs` 中的 `pub struct/enum/trait/fn/type/const/mod`。
- 当前未解析 TypeScript 装饰器元数据、类成员方法、C++ native 内部 public 方法，也未做参数/返回值语义等价证明。

## 总览

| 项目 | 数量 |
|---|---:|
| cocos4 exported symbols | 4489 |
| cocos4-rust pub symbols | 6003 |
| CSV rows | 9876 |

## 状态分布

| 状态 | 数量 | 含义 |
|---|---:|---|
| same-name | 877 | 规范化名称完全一致，仍需人工核对签名和行为 |
| same-name-candidate | 132 | 名称一致但模块不在当前等价映射内，只能作为复核入口 |
| renamed-candidate | 207 | 字符串相似的候选项，只能作为复核入口 |
| missing | 3273 | 当前未找到 Rust public symbol 候选 |
| rust-only | 5387 | Rust 侧新增或内部公开符号，未匹配原版导出 |

## 缺失最多的 cocos4 模块

| 模块 | missing 数量 |
|---|---:|
| rendering | 526 |
| core | 461 |
| animation | 458 |
| gfx | 287 |
| physics | 232 |
| 2d | 166 |
| native-binding | 144 |
| asset | 127 |
| physics-2d | 127 |
| 3d | 93 |
| render-scene | 84 |
| scene-graph | 75 |
| particle | 64 |
| serialization | 57 |
| dragon-bones | 56 |
| ui | 53 |
| spine | 48 |
| terrain | 42 |
| tween | 33 |
| tiledmap | 27 |

## cocos4 导出最多的模块

| 模块 | exported symbols |
|---|---:|
| rendering | 695 |
| core | 641 |
| gfx | 499 |
| animation | 493 |
| physics | 275 |
| asset | 234 |
| render-scene | 223 |
| 2d | 196 |
| native-binding | 151 |
| physics-2d | 146 |
| 3d | 142 |
| scene-graph | 116 |
| ui | 75 |
| particle | 69 |
| serialization | 67 |
| dragon-bones | 62 |
| spine | 51 |
| tween | 50 |
| terrain | 45 |
| primitive | 39 |

## cocos4-rust pub 符号最多的模块

| 模块 | pub symbols |
|---|---:|
| renderer | 1785 |
| core | 767 |
| scene | 519 |
| math | 403 |
| agi_minigame | 355 |
| physics | 303 |
| network | 177 |
| base | 168 |
| 2d | 144 |
| ui | 124 |
| 3d | 120 |
| platform | 113 |
| physics_2d | 93 |
| xr | 77 |
| gi | 74 |
| audio | 70 |
| application | 68 |
| game | 66 |
| tween | 66 |
| particle | 58 |

## 使用方式

```bash
python3 scripts/generate-interface-symbol-map.py
```

详细逐项结果见 `docs/interface-symbol-map.csv`。人工二次校验时，应把 `same-name`、`same-name-candidate` 和 `renamed-candidate` 继续升级为 `same` / `renamed-equivalent` / `partial` / `missing`，并补充测试证据。
