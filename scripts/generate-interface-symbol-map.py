#!/usr/bin/env python3
"""Generate a conservative public-symbol parity baseline.

The output is intentionally a verification aid, not a compatibility claim.
It compares exported TypeScript/JavaScript symbols from ../cocos4 with Rust
`pub` symbols from this crate, using exact normalized-name matching first and
string-similarity candidates only as review hints.
"""

from __future__ import annotations

import csv
import difflib
import re
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
COCOS4_ROOT = REPO_ROOT.parent / "cocos4"
DOCS_DIR = REPO_ROOT / "docs"
CSV_PATH = DOCS_DIR / "interface-symbol-map.csv"
SUMMARY_PATH = DOCS_DIR / "interface-symbol-map-summary.md"

TS_JS_DIRS = ("cocos", "exports")
TS_JS_GLOBS = ("*.ts", "*.js")

TS_DECL_RE = re.compile(
    r"^\s*export\s+"
    r"(?:(?:default|declare)\s+)?"
    r"(?:(abstract)\s+)?"
    r"(?:(const)\s+)?"
    r"(class|enum|interface|function|const|let|var|type|namespace)\s+"
    r"([A-Za-z_$][\w$]*)"
)
TS_REEXPORT_RE = re.compile(
    r"^\s*export\s+(type\s+)?\{\s*([^}]+)\s*\}"
)
RUST_DECL_RE = re.compile(
    r"^\s*pub\s+"
    r"(?:(?:\([^)]*\)|crate|super|self)\s+)?"
    r"(struct|enum|trait|fn|type|const|mod)\s+"
    r"([A-Za-z_][A-Za-z0-9_]*)"
)

MODULE_EQUIVALENTS = {
    "2d": {"2d", "ui"},
    "3d": {"3d", "scene"},
    "animation": {"core", "3d", "tween"},
    "asset": {"core"},
    "core": {"core", "base", "math"},
    "dragon-bones": {"dragon_bones"},
    "game": {"game", "application"},
    "gfx": {"renderer"},
    "input": {"input"},
    "native-binding": {"bindings"},
    "physics": {"physics"},
    "physics-2d": {"physics_2d"},
    "primitive": {"primitive"},
    "render-scene": {"scene", "renderer"},
    "rendering": {"renderer"},
    "scene-graph": {"core", "scene"},
    "serialization": {"serialization"},
    "spine": {"spine"},
    "terrain": {"terrain"},
    "tiledmap": {"tiled_map"},
    "tween": {"tween"},
    "ui": {"ui", "2d"},
    "xr": {"xr"},
}


@dataclass(frozen=True)
class Symbol:
    side: str
    kind: str
    name: str
    path: str
    line: int
    module: str
    signature: str

    @property
    def key(self) -> str:
        return normalize(self.name)


def normalize(name: str) -> str:
    return re.sub(r"[^a-z0-9]", "", name.lower())


def iter_files(root: Path, dirs: tuple[str, ...], globs: tuple[str, ...]) -> list[Path]:
    files: list[Path] = []
    for directory in dirs:
        base = root / directory
        if not base.exists():
            continue
        for glob in globs:
            files.extend(base.rglob(glob))
    return sorted({path for path in files if path.is_file()})


def module_from_cocos4_path(path: Path) -> str:
    rel = path.relative_to(COCOS4_ROOT).as_posix()
    parts = rel.split("/")
    if parts[0] == "exports":
        return f"exports/{Path(rel).stem}"
    if len(parts) >= 2:
        return parts[1]
    return parts[0]


def module_from_rust_path(path: Path) -> str:
    rel = path.relative_to(REPO_ROOT).as_posix()
    parts = rel.split("/")
    if len(parts) >= 2 and parts[0] == "src":
        return parts[1]
    return parts[0]


def modules_match(cocos4_module: str, rust_module: str) -> bool:
    if cocos4_module == rust_module:
        return True
    return rust_module in MODULE_EQUIVALENTS.get(cocos4_module, set())


def extract_ts_exports() -> list[Symbol]:
    symbols: list[Symbol] = []
    for path in iter_files(COCOS4_ROOT, TS_JS_DIRS, TS_JS_GLOBS):
        rel = path.relative_to(COCOS4_ROOT).as_posix()
        try:
            lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        except OSError:
            continue
        for idx, line in enumerate(lines, start=1):
            decl = TS_DECL_RE.match(line)
            if decl:
                abstract_kw, const_kw, kind, name = decl.groups()
                full_kind = " ".join(x for x in (abstract_kw, const_kw, kind) if x)
                symbols.append(
                    Symbol(
                        side="cocos4",
                        kind=full_kind,
                        name=name,
                        path=rel,
                        line=idx,
                        module=module_from_cocos4_path(path),
                        signature=line.strip(),
                    )
                )
                continue

            reexport = TS_REEXPORT_RE.match(line)
            if reexport:
                is_type, body = reexport.groups()
                kind = "reexport type" if is_type else "reexport"
                for item in body.split(","):
                    cleaned = item.strip()
                    if not cleaned or cleaned.startswith("//"):
                        continue
                    if " as " in cleaned:
                        exported = cleaned.rsplit(" as ", 1)[1].strip()
                    else:
                        exported = cleaned.split()[0].strip()
                    exported = exported.strip("{}")
                    if re.match(r"^[A-Za-z_$][\w$]*$", exported):
                        symbols.append(
                            Symbol(
                                side="cocos4",
                                kind=kind,
                                name=exported,
                                path=rel,
                                line=idx,
                                module=module_from_cocos4_path(path),
                                signature=line.strip(),
                            )
                        )
    return symbols


def extract_rust_pub_symbols() -> list[Symbol]:
    symbols: list[Symbol] = []
    for path in sorted((REPO_ROOT / "src").rglob("*.rs")):
        rel = path.relative_to(REPO_ROOT).as_posix()
        try:
            lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        except OSError:
            continue
        for idx, line in enumerate(lines, start=1):
            decl = RUST_DECL_RE.match(line)
            if not decl:
                continue
            kind, name = decl.groups()
            symbols.append(
                Symbol(
                    side="cocos4-rust",
                    kind=kind,
                    name=name,
                    path=rel,
                    line=idx,
                    module=module_from_rust_path(path),
                    signature=line.strip(),
                )
            )
    return symbols


def best_candidate(source: Symbol, rust_keys: dict[str, list[Symbol]]) -> tuple[str, Symbol | None]:
    if not source.key:
        return "missing", None
    if source.key in rust_keys:
        exact_targets = rust_keys[source.key]
        for target in exact_targets:
            if modules_match(source.module, target.module):
                return "same-name", target
        return "same-name-candidate", exact_targets[0]

    module_keys = {
        sym.key
        for values in rust_keys.values()
        for sym in values
        if modules_match(source.module, sym.module)
    }
    close = difflib.get_close_matches(source.key, module_keys, n=1, cutoff=0.86)
    if close:
        return "renamed-candidate", rust_keys[close[0]][0]

    all_close = difflib.get_close_matches(source.key, rust_keys.keys(), n=1, cutoff=0.92)
    if all_close:
        return "renamed-candidate", rust_keys[all_close[0]][0]

    return "missing", None


def write_outputs(cocos4_symbols: list[Symbol], rust_symbols: list[Symbol]) -> None:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    rust_by_key: dict[str, list[Symbol]] = defaultdict(list)
    for symbol in rust_symbols:
        rust_by_key[symbol.key].append(symbol)

    used_rust: set[tuple[str, int, str]] = set()
    rows: list[dict[str, str]] = []
    for source in cocos4_symbols:
        status, target = best_candidate(source, rust_by_key)
        if target:
            used_rust.add((target.path, target.line, target.name))
        rows.append(
            {
                "status": status,
                "cocos4_symbol": source.name,
                "cocos4_kind": source.kind,
                "cocos4_module": source.module,
                "cocos4_path": source.path,
                "cocos4_line": str(source.line),
                "rust_symbol": target.name if target else "",
                "rust_kind": target.kind if target else "",
                "rust_module": target.module if target else "",
                "rust_path": target.path if target else "",
                "rust_line": str(target.line) if target else "",
                "notes": (
                    "name-normalized exact match in equivalent module"
                    if status == "same-name"
                    else "name-normalized exact match in a different module; manual review required"
                    if status == "same-name-candidate"
                    else "similar-name candidate only; manual signature and behavior review required"
                    if status == "renamed-candidate"
                    else "no public Rust symbol candidate found by current scanner"
                ),
                "cocos4_signature": source.signature,
                "rust_signature": target.signature if target else "",
            }
        )

    for target in rust_symbols:
        identity = (target.path, target.line, target.name)
        if identity in used_rust:
            continue
        rows.append(
            {
                "status": "rust-only",
                "cocos4_symbol": "",
                "cocos4_kind": "",
                "cocos4_module": "",
                "cocos4_path": "",
                "cocos4_line": "",
                "rust_symbol": target.name,
                "rust_kind": target.kind,
                "rust_module": target.module,
                "rust_path": target.path,
                "rust_line": str(target.line),
                "notes": "Rust public symbol not matched to an exported cocos4 symbol",
                "cocos4_signature": "",
                "rust_signature": target.signature,
            }
        )

    fieldnames = [
        "status",
        "cocos4_symbol",
        "cocos4_kind",
        "cocos4_module",
        "cocos4_path",
        "cocos4_line",
        "rust_symbol",
        "rust_kind",
        "rust_module",
        "rust_path",
        "rust_line",
        "notes",
        "cocos4_signature",
        "rust_signature",
    ]
    with CSV_PATH.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    status_counts = Counter(row["status"] for row in rows)
    cocos_module_counts = Counter(symbol.module for symbol in cocos4_symbols)
    rust_module_counts = Counter(symbol.module for symbol in rust_symbols)
    missing_by_module = Counter(
        row["cocos4_module"] for row in rows if row["status"] == "missing"
    )

    summary = [
        "# cocos4-rust 接口 symbol 对照基线",
        "",
        "生成日期：2026-05-27",
        "",
        "本文件由 `scripts/generate-interface-symbol-map.py` 生成。它是逐接口复核的机器基线，不是 100% 兼容声明。",
        "",
        "## 覆盖范围",
        "",
        "- cocos4 侧：扫描 `../cocos4/cocos` 与 `../cocos4/exports` 中的 `export class/enum/interface/function/const/let/var/type/namespace` 以及单行 `export { ... }`。",
        "- cocos4-rust 侧：扫描 `src/**/*.rs` 中的 `pub struct/enum/trait/fn/type/const/mod`。",
        "- 当前未解析 TypeScript 装饰器元数据、类成员方法、C++ native 内部 public 方法，也未做参数/返回值语义等价证明。",
        "",
        "## 总览",
        "",
        "| 项目 | 数量 |",
        "|---|---:|",
        f"| cocos4 exported symbols | {len(cocos4_symbols)} |",
        f"| cocos4-rust pub symbols | {len(rust_symbols)} |",
        f"| CSV rows | {len(rows)} |",
        "",
        "## 状态分布",
        "",
        "| 状态 | 数量 | 含义 |",
        "|---|---:|---|",
        f"| same-name | {status_counts['same-name']} | 规范化名称完全一致，仍需人工核对签名和行为 |",
        f"| same-name-candidate | {status_counts['same-name-candidate']} | 名称一致但模块不在当前等价映射内，只能作为复核入口 |",
        f"| renamed-candidate | {status_counts['renamed-candidate']} | 字符串相似的候选项，只能作为复核入口 |",
        f"| missing | {status_counts['missing']} | 当前未找到 Rust public symbol 候选 |",
        f"| rust-only | {status_counts['rust-only']} | Rust 侧新增或内部公开符号，未匹配原版导出 |",
        "",
        "## 缺失最多的 cocos4 模块",
        "",
        "| 模块 | missing 数量 |",
        "|---|---:|",
    ]
    for module, count in missing_by_module.most_common(20):
        summary.append(f"| {module} | {count} |")

    summary.extend(
        [
            "",
            "## cocos4 导出最多的模块",
            "",
            "| 模块 | exported symbols |",
            "|---|---:|",
        ]
    )
    for module, count in cocos_module_counts.most_common(20):
        summary.append(f"| {module} | {count} |")

    summary.extend(
        [
            "",
            "## cocos4-rust pub 符号最多的模块",
            "",
            "| 模块 | pub symbols |",
            "|---|---:|",
        ]
    )
    for module, count in rust_module_counts.most_common(20):
        summary.append(f"| {module} | {count} |")

    summary.extend(
        [
            "",
            "## 使用方式",
            "",
            "```bash",
            "python3 scripts/generate-interface-symbol-map.py",
            "```",
            "",
            "详细逐项结果见 `docs/interface-symbol-map.csv`。人工二次校验时，应把 `same-name`、`same-name-candidate` 和 `renamed-candidate` 继续升级为 `same` / `renamed-equivalent` / `partial` / `missing`，并补充测试证据。",
            "",
        ]
    )
    SUMMARY_PATH.write_text("\n".join(summary), encoding="utf-8")


def main() -> None:
    if not COCOS4_ROOT.exists():
        raise SystemExit(f"cocos4 repo not found: {COCOS4_ROOT}")

    cocos4_symbols = extract_ts_exports()
    rust_symbols = extract_rust_pub_symbols()
    write_outputs(cocos4_symbols, rust_symbols)
    print(f"wrote {CSV_PATH.relative_to(REPO_ROOT)}")
    print(f"wrote {SUMMARY_PATH.relative_to(REPO_ROOT)}")
    print(f"cocos4_exported_symbols={len(cocos4_symbols)}")
    print(f"rust_pub_symbols={len(rust_symbols)}")


if __name__ == "__main__":
    main()
