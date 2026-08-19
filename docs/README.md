# cocos4-rust 文档索引

更新时间：2026-05-27

本目录记录 `cocos4-rust` 对 `cocos4` 的复刻、重构、测试和完整性校验状态。当前结论是：Rust 工程已经恢复可编译，通过 1812 个单元测试和 all-targets clippy 严格门禁，并已跑通原版 `cocos4` 的 Vec3 Jest 子集对照；但还不能宣称“每个 cocos4 功能和接口 100% 兼容”。仍需继续做原版 `cocos4` 全量 Jest 对照、逐接口签名表和真实运行场景对照。

## 文档

- [使用与帮助文档](usage.md)
- [性能对比测试报告](performance-comparison-report.md)
- [接口与功能对齐审计](interface-parity-audit.md)
- [接口 symbol 对照基线摘要](interface-symbol-map-summary.md)
- [接口 symbol 对照 CSV](interface-symbol-map.csv)
- [项目完整性检查](completeness-check.md)

## 本轮验证摘要

| 项目 | 当前结果 |
|---|---:|
| `cocos4/tests` 功能测试文件数 | 163 |
| `cocos4-rust` Rust 源文件数 | 330 |
| `cocos4-rust` public symbol 粗扫描数 | 6040 |
| `cocos4-rust` public symbol map 扫描数 | 6003 |
| `node scripts/compare-cocos4-vec3-tests.mjs` | 原版 Vec3 Jest 4 passed；Rust Vec3 38 passed |
| `cargo test -- --list` | 1812 tests, 0 benchmarks |
| `cargo test` | 1812 passed, 0 failed |
| `cargo test --features "gfx-wgpu storage-disk platform-real"` | 1812 passed, 0 failed |
| `cargo test --features js-runtime-real game` | 131 passed, 0 failed |
| `cargo clippy --all-targets -- -D warnings` | 通过 |

## 已修复的完整性阻塞

- `src/game/game.rs` 中 JS bootstrap prelude 的 Rust raw string delimiter 被 `"#GameCanvas"` 提前截断，导致库无法编译；已改为更高阶 raw string。
- `base::Value` 增加 `Int(i64)` 兼容层，并将 `Float` 调整为 JS 数字友好的 `f64` 存储，解决 AGI mini-game 状态序列化编译错误。
- `AtomRegistry` 注册接口改为泛型 factory，兼容闭包直接注册。
- 修复 `Match3Atom` 初始棋盘生成、越界宝石索引、跑酷积分累计和无强制目标维度自动完成问题。
- `Vec3` 补齐 `len`、`length_sqr`、`multiply_scalar`、`lerp_vecs`、`equals`、`scale_and_add`、`generate_orthogonal`、`slerp` 等原版测试覆盖到的接口，并增加 Rust 镜像测试。

## 当前不能视为完成的原因

- 原版 `cocos4` 依赖已通过 `npm ci --ignore-scripts` 恢复，Vec3 Jest 子集已通过；但原版 163 个测试文件尚未全量执行。
- 直接通过 `tsx` import `cocos/core/math/vec3.ts` 仍会因 `internal:constants` 模块别名失败；当前采用 Jest 环境和 `cc.decorator` mapper 执行原版子集。
- 已生成 `docs/interface-symbol-map.csv` 作为 TypeScript/JavaScript 导出符号到 Rust public symbol 的逐项机器基线；但它仍未证明参数、返回值和行为等价，也未覆盖 C++ native 内部 public 方法。
