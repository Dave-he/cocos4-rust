# Engine vs App Responsibility Boundary (Cocos4 Runtime Compatibility)

This document defines a practical split for running `game-demo` (modern-systemjs chain).

## Current execution ownership matrix (authoritative)

- Engine (`cocos4-rust`)
  - Should own:
    - Runtime style detection and bootstrap contract (`GameBootstrapContract`, `start_with_bootstrap`).
    - `System.register` / `System.import` / `System.warmup` compatibility behavior.
    - `__require` and module-map behavior needed for bootstrap success.
    - `window`/`cc`/`canvas` host object stubs used by bootstrap source.
    - `application.js + game.js` stitching for modern-systemjs candidates (in package scan/build path).
    - `settings.json` and `settings.js` source injection strategy.
  - Must **not** own:
    - `wx/ks` concrete SDK implementations (App/平台层实现).
    - 文件下载、APK/Assets 读写、Activity 生命周期桥接（平台层实现）。

- App (`game-studio`)
  - Should own:
    - `game-path`/`zip/assets` 取包与路径规范化。
    - 平台权限、网络、文件系统和渲染窗口生命周期。
    - 启动失败后的 Web 回退策略（仅基于 native 返回码）。
  - Must **not** own:
    - 启动脚本语义兼容（`System/__require/cc/window` 注入）；
    - runtime 风格识别与 bootstrap 语义判定。

## 1) Engine layer (`cocos4-rust`) responsibilities

### A. Bootstrapping and bootstrap-style detection (required first)
- Parse/normalize bootstrap contract (`GameBootstrapContract`).
- Detect runtime style and entry (`legacy-cocos2d-js`, `modern-systemjs`, `bootstrap-only`).
- Validate bootstrap source by syntax/marker preflight.
- Execute bootstrap source through one JS runtime path (`js-runtime-mock` by default, `js-runtime-real` optional).
- Return actionable `GameBootstrapError` values when blocked.
- Keep start lifecycle API stable (`start_with_bootstrap`, `Game`, `GameConfig`, `GameEvent`).

### B. Open-source Cocos4 compatibility shims (bootstrap surface)
- `window`/`globalThis` baseline object injection.
- `cc` baseline namespace needed by bootstrap scripts.
- `System.register` + `System.import` compatibility path.
- `__require` compatibility path for mini-platform modules required by bootstrap source.
- `navigator`/`location` compatibility fields.
- `first-screen` compatibility object shape used by modern mini-chain.

### C. Module-level entry stitching
- Concatenate/prepare `application.js` + `game.js` for modern-systemjs when needed (already in game-studio currently).
- Keep module map semantics (`window.__moduleMap`) and `Application` extraction behavior.

### D. Minimal behavior for success criteria
- `game-demo` chain can reach `__initApp -> firstScreen -> System.import('./application.js') -> cc.game.init/run` path without runtime abort.

## 2) App layer (`game-studio`) responsibilities

### A. Platform/runtime service bridge
- Actual platform API implementations for `wx/ks`/third-party SDK:
  - login/share/recommend ads/audio/video etc.
  - file/cache/network/storage/device APIs specific to target platform.

### B. Resource and package loading
- ZIP/package path resolution and extraction strategy.
- Asset fetch/download/update strategy.
- Native rendering/input/audio/video/audio-context integrations.

### C. Event/logging/runtime telemetry
- Map platform lifecycle/events into engine where needed.
- Logging, crash/recover reporting, permissions.

## 3) Hard split rule

- **`cocos4-rust` is generic and game-engine-agnostic:** if logic can be validated from bootstrap source alone and reused across app targets, keep it in engine.
- **`game-studio` is platform-specific:** if logic depends on Android/iOS/mini-platform SDK behavior, keep in app.

## 4) Current immediate priority for `game-demo` to run

1. Ensure modern-systemjs bootstrap succeeds in both `js-runtime-mock` (default path) and `js-runtime-real`.
2. Keep app shim minimal (`wx/ks`) to pass bootstrap contract only.
3. After bootstrap reaches `cc.game.init/run`, improve deeper `cc` runtime behaviors only as needed by runtime execution logs.

## 5) Acceptance checklist (engine scope vs app scope)

- [ ] Modern chain preflight: modern game package with `game.js` and `application.js` can pass `probe_bootstrap_entry` in real-time.
- [ ] Modern chain bootstrap execution:
  - `System.warmup` can cache importMap/handlers/defaultHandler.
  - `System.import('./application.js')` resolves to exported `Application`.
  - `System.import('cc')` resolves bootstrap `cc` namespace.
- [ ] `__require` parity for game-demo modern chain:
  - `require('./kwaiadapter.js')` returns `{ ks: window.ks }`.
  - `require('src/system.bundle.js')`, `require("src/polyfills.bundle.js")`, `require('src/import-map.js')` do not crash and return safe objects.
  - `require('./main')`, `require('./main.js')`, `require('main.ts')` 等返回可空对象（不阻断 preflight）。
- [ ] `__globalAdapter` exists and `init()/adaptEngine()` are callable in mini-engine bootstrap scripts.
- [ ] `Application` extraction fallback:
  - `window.__moduleMap["application.js"].Application` and `window.Application` are both accepted entry points.
- [ ] App layer only:
  - Package analysis + path injection + 回退。
  - 运行失败时不改写 engine bootstrap 兼容策略。

## 6) Cocos4 Open-Source API Compatibility Ownership (enforced split)

### Engine (must be in `cocos4-rust`)
- Bootstrap and module-loader compatibility required by modern/legacy game entry contracts:
  - `System.register`, `System.import`, `System.warmup`, fallback handlers.
  - `__require`, `__moduleMap`, `__collectModuleCandidates`, `__lookupModuleFromMap`.
  - `__rollupPluginModLoBabelHelpers` and common Rollup Babel helpers compatibility.
- Core host/shim surface used during engine bootstrap:
  - `window`, `GameGlobal`, `canvas`, `screen`, `navigator`, `location`, `requestAnimationFrame`, timers.
  - `cc`, `cc.game`, `cc.director`, `cc.path`, `cc.view`, `cc.sys`, `cc._CCSettings`, `cc.loader`, `cc.AssetManager`, `cc.debug`.
  - `wx`/`ks` minimum bootstrap surface that cocos scripts call before engine runtime starts.
- Bootstrap contract and entry orchestration:
  - Detect runtime style (`legacy-cocos2d-js`, `modern-systemjs`, `bootstrap-only`).
  - Read and execute `main` + `settings` sources.
  - Produce machine-readable failure reasons and keep failure policy in engine.

### App (must stay outside cocos4 bootstrap semantics)
- Package and platform plumbing:
  - Zip/assets download/extract, path normalization (`assets://`/local path).
  - Activity/window lifecycle and permissions.
  - Network/security/device/storage policy.
- Native/JS runtime hosting:
  - JNI binding of lifecycle and settings APIs.
  - Error/status retrieval and fallback switch (`STARTED_SIMULATED`, `RUNTIME_UNAVAILABLE`, etc.).
- User-facing fallbacks:
  - Navigate to Web fallback when engine returns non-started status.
  - Toast/log display and UI-level diagnostics.

### Hard rule
- If an API/behavior can be validated solely from bootstrap source semantics and should be reusable across platforms, it belongs in engine.
- If behavior depends on Android/iOS/SDK platform capabilities, it belongs in app.
