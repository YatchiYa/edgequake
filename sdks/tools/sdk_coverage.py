#!/usr/bin/env python3
"""Compare OpenAPI operations with the ten SDKs and generate missing methods.

Usage:
  python3 sdks/tools/sdk_coverage.py            # write manifest + markdown report
  python3 sdks/tools/sdk_coverage.py --generate # also emit operations_generated sources
  python3 sdks/tools/sdk_coverage.py --check    # exit 1 if any HTTP operation is missing
"""

from __future__ import annotations

import argparse
import json
import keyword
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SNAPSHOT = ROOT / "edgequake_webui/openapi/openapi.snapshot.json"
MANIFEST = ROOT / "sdks/operation-manifest.json"
REPORT = ROOT / "sdks/tools/coverage-report.md"

SDK_ROOTS = {
    "python": ROOT / "sdks/python/edgequake",
    "typescript": ROOT / "sdks/typescript/src",
    "rust": ROOT / "sdks/rust/src",
    "go": ROOT / "sdks/go",
    "java": ROOT / "sdks/java/src/main",
    "kotlin": ROOT / "sdks/kotlin/src/main",
    "csharp": ROOT / "sdks/csharp/src",
    "php": ROOT / "sdks/php/src",
    "ruby": ROOT / "sdks/ruby/lib",
    "swift": ROOT / "sdks/swift/Sources",
}

GENERATED = {
    "python": ROOT / "sdks/python/edgequake/resources/operations_generated.py",
    "typescript": ROOT / "sdks/typescript/src/resources/operations-generated.ts",
    "rust": ROOT / "sdks/rust/src/resources/operations_generated.rs",
    "go": ROOT / "sdks/go/operations_generated.go",
    "java": ROOT / "sdks/java/src/main/java/io/edgequake/sdk/resources/OperationsGenerated.java",
    "kotlin": ROOT / "sdks/kotlin/src/main/kotlin/io/edgequake/sdk/resources/OperationsGenerated.kt",
    "csharp": ROOT / "sdks/csharp/src/EdgeQuakeSDK/OperationsGenerated.cs",
    "php": ROOT / "sdks/php/src/OperationsGenerated.php",
    "ruby": ROOT / "sdks/ruby/lib/edgequake/operations_generated.rb",
    "swift": ROOT / "sdks/swift/Sources/EdgeQuakeSDK/OperationsGenerated.swift",
}

SKIP_DIRS = {"node_modules", "target", "bin", "obj", "__pycache__", "dist", ".phpunit.cache"}
HTTP = ("get", "post", "put", "patch", "delete", "head", "options")
PY_KEYWORDS = set(keyword.kwlist) | {"id"}
RUST_KEYWORDS = {"type", "self", "move", "async", "await", "box", "ref", "mod", "fn", "match"}
GO_KEYWORDS = {"type", "func", "map", "range", "chan", "select", "defer", "go", "var"}


def load_operations() -> list[dict]:
    spec = json.loads(SNAPSHOT.read_text())
    ops: list[dict] = []
    for path, item in spec.get("paths", {}).items():
        if not isinstance(item, dict):
            continue
        for method, op in item.items():
            if method not in HTTP or not isinstance(op, dict):
                continue
            params = []
            for p in op.get("parameters") or item.get("parameters") or []:
                if isinstance(p, dict) and p.get("in") == "path":
                    params.append(p["name"])
            if not params:
                params = re.findall(r"\{([^}]+)\}", path)
            kind = "http"
            blob = json.dumps(op)
            if "/ws" in path or "websocket" in path.lower():
                kind = "websocket"
            elif "text/event-stream" in blob or path.rstrip("/").endswith("/stream"):
                kind = "sse"
            oid = op.get("operationId") or f"{method}_{path.strip('/').replace('/', '_')}"
            ops.append(
                {
                    "operationId": oid,
                    "method": method.upper(),
                    "path": path,
                    "pathParams": params,
                    "kind": kind,
                }
            )
    return ops


def iter_sources(root: Path) -> str:
    chunks: list[str] = []
    if not root.exists():
        return ""
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        if path.suffix.lower() not in {".py", ".ts", ".rs", ".go", ".java", ".kt", ".cs", ".php", ".rb", ".swift"}:
            continue
        if "operations_generated" in path.name or "operations-generated" in path.name or "OperationsGenerated" in path.name:
            continue
        try:
            chunks.append(path.read_text(errors="ignore"))
        except OSError:
            continue
    return "\n".join(chunks)


def covered(source: str, path: str) -> bool:
    return path in source


def normalize_call_path(path: str) -> str:
    """Collapse interpolations so `{user_id}` and `$id` compare as the same segment."""
    path = path.split("?", 1)[0]
    def swift_interp(match: re.Match[str]) -> str:
        inner = match.group(1).strip()
        if inner == "qs" or inner.startswith("?"):
            return ""
        return "{}"

    path = re.sub(r"\\\(([^)]*)\)", swift_interp, path)
    path = re.sub(r"\$\{[^}]+\}", "{}", path)
    path = re.sub(r"#\{[^}]+\}", "{}", path)
    path = re.sub(r"\{[^}]+\}", "{}", path)
    path = re.sub(r"\$[A-Za-z_][A-Za-z0-9_]*", "{}", path)
    path = re.sub(r"%[sdv]", "{}", path)
    path = re.sub(r"/{2,}", "/", path)
    if len(path) > 1:
        path = path.rstrip("/")
    return path


def openapi_index(ops: list[dict]) -> dict[str, set[str]]:
    index: dict[str, set[str]] = {}
    for op in ops:
        if op["kind"] == "websocket":
            continue
        index.setdefault(normalize_call_path(op["path"]), set()).add(op["method"])
    return index


_CALL_RE = re.compile(
    r"(?:\.(?P<dot>get|post|put|patch|delete|del|head|options)"
    r"|(?<![A-Za-z0-9])_(?P<under>get|post|put|patch|delete|del|head|options))"
    r"(?:_raw|Raw|Async|NoContent|MultipartMany)*"
    r"(?:<[^>\n]{0,120}>)?\s*\(",
    re.IGNORECASE,
)
_CONST_RE = re.compile(
    r"""\b([A-Z][A-Z0-9_]*)\s*=\s*(['"])(/[^'"]+)\2"""
)
_DEF_RE = re.compile(r"\b(def|fun|func|fn)\s*$")


def _read_string(text: str, i: int) -> tuple[str, int]:
    quote = text[i]
    i += 1
    buf: list[str] = []
    while i < len(text):
        char = text[i]
        if quote == "`" and char == "$" and i + 1 < len(text) and text[i + 1] == "{":
            depth = 1
            i += 2
            start = i
            while i < len(text) and depth:
                if text[i] == "{":
                    depth += 1
                elif text[i] == "}":
                    depth -= 1
                i += 1
            inner = text[start : i - 1].lstrip()
            buf.append("" if inner.startswith(("qs", "?")) else "{}")
            continue
        if char == "\\" and i + 1 < len(text):
            nxt = text[i + 1]
            if nxt == "(":
                buf.append("\\(")
                i += 2
                continue
            if quote != "`":
                buf.append(nxt)
                i += 2
                continue
        if char == quote:
            return "".join(buf), i + 1
        buf.append(char)
        i += 1
    return "".join(buf), i


def _constant_at(constants: list[tuple[int, str, str]], name: str, pos: int) -> str:
    value = "{}"
    for index, const_name, const_value in constants:
        if index < pos and const_name == name:
            value = const_value
    return value


def _expand_constants(raw: str, constants: list[tuple[int, str, str]], pos: int) -> str:
    def repl(match: re.Match[str]) -> str:
        return _constant_at(constants, match.group(1), pos)

    return re.sub(r"#\{([A-Z][A-Z0-9_]*)\}", repl, raw)


def _join_concat(text: str, end: int, path: str) -> tuple[str, int]:
    """Join `" /prefix/" + id + "/suffix"` into one template."""
    i = end
    while True:
        match = re.match(r"\s*(?:\+|\.)\s*", text[i : i + 8])
        if not match:
            return path, i
        nxt = i + match.end()
        if nxt >= len(text):
            return path, i
        if text[nxt] in "'\"`":
            extra, after = _read_string(text, nxt)
            path += extra
            i = after
            continue
        ident = re.match(r"[A-Za-z_$@][A-Za-z0-9_$.]*", text[nxt:])
        if not ident:
            return path, i
        path += "{}"
        i = nxt + ident.end()
        if i < len(text) and text[i] == "(":
            depth = 0
            while i < len(text):
                if text[i] == "(":
                    depth += 1
                elif text[i] == ")":
                    depth -= 1
                    if depth == 0:
                        i += 1
                        break
                i += 1


def _first_path(text: str, paren: int, constants: list[tuple[int, str, str]]) -> str | None:
    depth = 0
    i = paren
    limit = min(len(text), paren + 600)
    while i < limit:
        char = text[i]
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth == 0:
                return None
        elif char.isalpha() or char == "_":
            ident = re.match(r"[A-Za-z_][A-Za-z0-9_]*", text[i:])
            if ident is None:
                i += 1
                continue
            value = _constant_at(constants, ident.group(0), i)
            if value.startswith("/"):
                return normalize_call_path(value)
            i += ident.end()
            continue
        elif char in "'\"`":
            raw, end = _read_string(text, i)
            if raw.startswith("/") or raw.startswith("#{") or raw.startswith("{}"):
                expanded = _expand_constants(raw, constants, i)
                if not expanded.startswith("/"):
                    i = end
                    continue
                expanded, end = _join_concat(text, end, expanded)
                return normalize_call_path(expanded)
            i = end
            continue
        i += 1
    return None


def scan_phantoms(ops: list[dict]) -> list[tuple[str, int, str, str, str]]:
    """Return (file, line, verb, path, detail) for hand-written calls the snapshot rejects."""
    index = openapi_index(ops)
    found: list[tuple[str, int, str, str, str]] = []
    for lang, root in SDK_ROOTS.items():
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file():
                continue
            if any(part in SKIP_DIRS or part in {"tests", "test", "e2e"} for part in path.parts):
                continue
            if path.suffix.lower() not in {".py", ".ts", ".rs", ".go", ".java", ".kt", ".cs", ".php", ".rb", ".swift"}:
                continue
            name = path.name
            if "operations_generated" in name or "operations-generated" in name or "OperationsGenerated" in name:
                continue
            if name.endswith("_test.go") or name.endswith(".test.ts") or name.startswith("test_"):
                continue
            try:
                text = path.read_text(errors="ignore")
            except OSError:
                continue
            constants = [(match.start(), match.group(1), match.group(3)) for match in _CONST_RE.finditer(text)]
            for match in _CALL_RE.finditer(text):
                prefix = text[max(0, match.start() - 12) : match.start()]
                if _DEF_RE.search(prefix):
                    continue
                verb_raw = (match.group("dot") or match.group("under") or "").lower()
                verb = "DELETE" if verb_raw == "del" else verb_raw.upper()
                call_path = _first_path(text, match.end() - 1, constants)
                if not call_path or not call_path.startswith("/"):
                    continue
                methods = index.get(call_path)
                if methods and verb in methods:
                    continue
                if methods:
                    detail = "verb mismatch, server has " + ", ".join(sorted(methods))
                else:
                    detail = "path not in OpenAPI"
                line = text.count("\n", 0, match.start()) + 1
                rel = str(path.relative_to(ROOT))
                found.append((rel, line, verb, call_path, detail))
    found.sort()
    return found


def ident(raw: str, lang: str) -> str:
    name = re.sub(r"[^0-9A-Za-z_]", "_", raw)
    name = re.sub(r"_+", "_", name).strip("_")
    if not name or name[0].isdigit():
        name = "op_" + name
    reserved = set()
    if lang in {"python", "ruby"}:
        reserved = PY_KEYWORDS
    elif lang == "rust":
        reserved = RUST_KEYWORDS
    elif lang == "go":
        reserved = GO_KEYWORDS
    if name in reserved:
        name = name + "_"
    if lang in {"java", "kotlin", "csharp", "swift", "typescript"}:
        parts = name.split("_")
        name = parts[0] + "".join(p[:1].upper() + p[1:] for p in parts[1:] if p)
        if lang in {"go", "csharp"}:
            name = name[:1].upper() + name[1:]
    if lang == "go":
        parts = [p for p in name.split("_") if p]
        name = "".join(p[:1].upper() + p[1:] for p in parts) or "Op"
    if lang == "csharp":
        parts = [p for p in re.sub(r"[^0-9A-Za-z_]", "_", raw).split("_") if p]
        name = "".join(p[:1].upper() + p[1:] for p in parts) or "Op"
    return name


def unique_names(ops: list[dict], lang: str) -> dict[str, str]:
    used: dict[str, int] = {}
    out: dict[str, str] = {}
    for op in ops:
        base = ident(op["operationId"], lang)
        n = used.get(base, 0) + 1
        used[base] = n
        out[op["operationId"] + op["method"] + op["path"]] = base if n == 1 else f"{base}_{n}"
    return out


def quote_path(path: str) -> str:
    return path.replace("\\", "\\\\").replace('"', '\\"')


def py_path(op: dict) -> str:
    rest = op["path"]
    pieces: list[str] = []
    for p in op["pathParams"]:
        head, rest = rest.split("{" + p + "}", 1)
        if head:
            pieces.append(json.dumps(head))
        pieces.append(f"_path_param({ident(p, 'python')})")
    if rest:
        pieces.append(json.dumps(rest))
    return " + ".join(pieces) if pieces else '""'


def py_method(name: str, op: dict, async_def: bool) -> str:
    params = op["pathParams"]
    args = ", ".join(f"{ident(p, 'python')}: str" for p in params)
    prefix = "async def" if async_def else "def"
    sig = f"    {prefix} {name}(self"
    if args:
        sig += ", " + args
    path_expr = py_path(op)
    if op["kind"] == "websocket":
        sig += ") -> str:"
        return "\n".join(
            [
                sig,
                f'        """{op["method"]} {op["path"]}"""',
                f"        return _socket_url(self._transport.base_url, {path_expr})",
                "",
            ]
        )
    sig += ", *, query: dict | None = None, body: dict | None = None) -> Any:"
    events = "True" if op["kind"] == "sse" else "False"
    invoke = "await self._invoke" if async_def else "self._invoke"
    return "\n".join(
        [
            sig,
            f'        """{op["method"]} {op["path"]}"""',
            f"        return {invoke}({json.dumps(op['method'])}, {path_expr}, query, body, {events})",
            "",
        ]
    )


def emit_python(ops: list[dict], names: dict[str, str]) -> str:
    methods = []
    async_methods = []
    for op in ops:
        key = op["operationId"] + op["method"] + op["path"]
        methods.append(py_method(names[key], op, False))
        async_methods.append(py_method(names[key], op, True))
    return "\n".join(
        [
            '"""Generated OpenAPI operations. Do not edit by hand.',
            "",
            "Regenerate: python3 sdks/tools/sdk_coverage.py --generate",
            '"""',
            "",
            "from __future__ import annotations",
            "",
            "from typing import Any",
            "from urllib.parse import quote, urlencode",
            "",
            "from edgequake.resources._base import AsyncResource, SyncResource",
            "",
            "",
            "def _path_param(value: object) -> str:",
            '    """Encode one path parameter and keep slashes for wildcard segments."""',
            '    return "/".join(quote(part, safe="") for part in str(value).split("/"))',
            "",
            "",
            "def _prepare(path: str, query: dict | None) -> str:",
            "    if not query:",
            "        return path",
            "    joined = urlencode({key: value for key, value in query.items() if value is not None})",
            "    if not joined:",
            "        return path",
            "    return path + ('&' if '?' in path else '?') + joined",
            "",
            "",
            "def _events(text: str) -> list[str]:",
            '    """SSE data lines. One parser for every generated stream route."""',
            "    events: list[str] = []",
            '    for block in text.split("\\n\\n"):',
            "        for line in block.splitlines():",
            '            if not line.startswith("data:"):',
            "                continue",
            "            data = line[5:].lstrip()",
            '            if data and data != "[DONE]":',
            "                events.append(data)",
            "    return events",
            "",
            "",
            "def _decode(response: Any, events: bool) -> Any:",
            "    if events:",
            "        return _events(response.text)",
            "    if response.status_code == 204 or not response.content:",
            "        return None",
            "    try:",
            "        return response.json()",
            "    except Exception:",
            "        return response.text",
            "",
            "",
            "def _socket_url(base: str, path: str) -> str:",
            '    root = str(base).rstrip("/")',
            '    if root.startswith("https://"):',
            '        root = "wss://" + root[len("https://") :]',
            '    elif root.startswith("http://"):',
            '        root = "ws://" + root[len("http://") :]',
            "    return root + path",
            "",
            "",
            "class OperationsResource(SyncResource):",
            '    """Every OpenAPI operation that the hand-written resources do not already mention."""',
            "",
            "    def _invoke(self, method: str, path: str, query: dict | None, body: dict | None, events: bool) -> Any:",
            "        response = self._transport.request(method, _prepare(path, query), json=body, params=None)",
            "        return _decode(response, events)",
            "",
            *methods,
            "",
            "class AsyncOperationsResource(AsyncResource):",
            '    """Async twin of OperationsResource."""',
            "",
            "    async def _invoke(self, method: str, path: str, query: dict | None, body: dict | None, events: bool) -> Any:",
            "        response = await self._transport.request(method, _prepare(path, query), json=body, params=None)",
            "        return _decode(response, events)",
            "",
            *async_methods,
            "",
        ]
    )


def ts_path(op: dict) -> str:
    cursor = op["path"]
    pieces: list[str] = []
    for param in op["pathParams"]:
        head, cursor = cursor.split("{" + param + "}", 1)
        if head:
            pieces.append(json.dumps(head))
        pieces.append("encodePathParam(" + ident(param, "typescript") + ")")
    if cursor:
        pieces.append(json.dumps(cursor))
    return " + ".join(pieces) if pieces else '""'


def emit_typescript(ops: list[dict], names: dict[str, str]) -> str:
    methods = []
    for op in ops:
        key = op["operationId"] + op["method"] + op["path"]
        name = names[key]
        args = ", ".join(f"{ident(param, 'typescript')}: string" for param in op["pathParams"])
        path_expr = ts_path(op)
        if op["kind"] == "websocket":
            sig = f"{name}({args})" if args else f"{name}()"
            methods.append(
                "\n".join(
                    [
                        f"  /** {op['method']} {op['path']} */",
                        f"  {sig}: string {{",
                        f"    return this.transport.websocketUrl({path_expr});",
                        "  }",
                        "",
                    ]
                )
            )
            continue
        extra = "query?: Record<string, string>, body?: unknown"
        sig = f"{name}({args}, {extra})" if args else f"{name}({extra})"
        events = "true" if op["kind"] == "sse" else "false"
        methods.append(
            "\n".join(
                [
                    f"  /** {op['method']} {op['path']} */",
                    f"  {sig}: Promise<unknown> {{",
                    f"    return this.call({json.dumps(op['method'])}, {path_expr}, query, body, {events});",
                    "  }",
                    "",
                ]
            )
        )
    helper = """
  private async call(
    method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE",
    path: string,
    query?: Record<string, string>,
    body?: unknown,
    events = false,
  ): Promise<unknown> {
    if (events) {
      const chunks: unknown[] = [];
      const target = appendQuery(path, query);
      const stream = method === "GET" ? this._streamSSE<unknown>(target) : this._streamSSE<unknown>(target, body);
      for await (const event of stream) chunks.push(event);
      return chunks;
    }
    return this.transport.request({ method, path, query, body });
  }
"""
    return (
        "/** Generated OpenAPI operations. Do not edit by hand.\n"
        " *\n"
        " * Regenerate: python3 sdks/tools/sdk_coverage.py --generate\n"
        " */\n"
        'import { Resource } from "./base.js";\n\n'
        "function encodePathParam(value: string): string {\n"
        '  return String(value).split("/").map((part) => encodeURIComponent(part)).join("/");\n'
        "}\n\n"
        "function appendQuery(path: string, query?: Record<string, string>): string {\n"
        "  if (!query) return path;\n"
        "  const qs = new URLSearchParams(\n"
        "    Object.entries(query).filter(([, value]) => value != null) as [string, string][],\n"
        "  ).toString();\n"
        '  return qs ? path + (path.includes("?") ? "&" : "?") + qs : path;\n'
        "}\n\n"
        "export class OperationsResource extends Resource {\n"
        + helper
        + "".join(methods)
        + "}\n"
    )


def rust_path_lines(op: dict) -> list[str]:
    pieces: list[str] = []
    cursor = op["path"]
    for param in op["pathParams"]:
        head, cursor = cursor.split("{" + param + "}", 1)
        if head:
            pieces.append(f"path.push_str({json.dumps(head)});")
        pieces.append(f"path.push_str(&encode_path_param({ident(param, 'rust')}));")
    if cursor:
        pieces.append(f"path.push_str({json.dumps(cursor)});")
    return ["        let mut path = String::new();", *[f"        {line}" for line in pieces]]


def rust_method(name: str, op: dict) -> str:
    params = op["pathParams"]
    args = ", ".join(f"{ident(p, 'rust')}: &str" for p in params)
    if op["kind"] == "websocket":
        sig = f"    pub fn {name}(&self"
        if args:
            sig += ", " + args
        sig += ") -> String {"
        return "\n".join(
            [
                f"    /// {op['method']} {op['path']}",
                sig,
                *rust_path_lines(op),
                "        socket_url(self.client.base_url(), &path)",
                "    }",
                "",
            ]
        )
    sig = f"    pub async fn {name}(&self"
    if args:
        sig += ", " + args
    sig += ", query: Option<&HashMap<String, String>>, body: Option<&serde_json::Value>) -> Result<String> {"
    events = "true" if op["kind"] == "sse" else "false"
    return "\n".join(
        [
            f"    /// {op['method']} {op['path']}",
            sig,
            *rust_path_lines(op),
            f"        self.call(Method::{op['method']}, path, query, body, {events}).await",
            "    }",
            "",
        ]
    )


def emit_rust(ops: list[dict], names: dict[str, str]) -> str:
    methods = []
    for op in ops:
        key = op["operationId"] + op["method"] + op["path"]
        methods.append(rust_method(names[key], op))
    return (
        "//! Generated OpenAPI operations. Do not edit by hand.\n"
        "//!\n"
        "//! Regenerate: `python3 sdks/tools/sdk_coverage.py --generate`\n\n"
        "use std::collections::HashMap;\n\n"
        "use reqwest::Method;\n\n"
        "use crate::client::EdgeQuakeClient;\n"
        "use crate::error::Result;\n\n"
        "pub struct OperationsResource<'a> {\n"
        "    pub(crate) client: &'a EdgeQuakeClient,\n"
        "}\n\n"
        "impl<'a> OperationsResource<'a> {\n"
        "    async fn call(\n"
        "        &self,\n"
        "        method: Method,\n"
        "        mut path: String,\n"
        "        query: Option<&HashMap<String, String>>,\n"
        "        body: Option<&serde_json::Value>,\n"
        "        events: bool,\n"
        "    ) -> Result<String> {\n"
        "        if let Some(query) = query {\n"
        "            let qs: String = query\n"
        "                .iter()\n"
        "                .map(|(k, v)| format!(\"{}={}\", encode_query_param(k), encode_query_param(v)))\n"
        "                .collect::<Vec<_>>()\n"
        "                .join(\"&\");\n"
        "            if !qs.is_empty() {\n"
        "                path.push(if path.contains('?') { '&' } else { '?' });\n"
        "                path.push_str(&qs);\n"
        "            }\n"
        "        }\n"
        "        let text = self.client.exchange_text(method, &path, body).await?;\n"
        "        if events { Ok(parse_events(&text)) } else { Ok(text) }\n"
        "    }\n\n"
        + "".join(methods)
        + "}\n\n"
        "fn parse_events(text: &str) -> String {\n"
        "    let mut events = Vec::new();\n"
        "    for block in text.split(\"\\n\\n\") {\n"
        "        for line in block.lines() {\n"
        "            let Some(data) = line.strip_prefix(\"data:\") else { continue };\n"
        "            let data = data.trim();\n"
        "            if !data.is_empty() && data != \"[DONE]\" {\n"
        "                events.push(data.to_string());\n"
        "            }\n"
        "        }\n"
        "    }\n"
        "    events.join(\"\\n\")\n"
        "}\n\n"
        "fn socket_url(base: &str, path: &str) -> String {\n"
        "    let root = base.trim_end_matches('/');\n"
        "    let root = if let Some(rest) = root.strip_prefix(\"https://\") {\n"
        "        format!(\"wss://{rest}\")\n"
        "    } else if let Some(rest) = root.strip_prefix(\"http://\") {\n"
        "        format!(\"ws://{rest}\")\n"
        "    } else {\n"
        "        root.to_string()\n"
        "    };\n"
        "    format!(\"{root}{path}\")\n"
        "}\n\n"
        "fn encode_path_param(value: &str) -> String {\n"
        "    value.split('/').map(encode_query_param).collect::<Vec<_>>().join(\"/\")\n"
        "}\n\n"
        "fn encode_query_param(value: &str) -> String {\n"
        "    let mut out = String::new();\n"
        "    for b in value.bytes() {\n"
        "        match b {\n"
        "            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),\n"
        "            _ => out.push_str(&format!(\"%{b:02X}\")),\n"
        "        }\n"
        "    }\n"
        "    out\n"
        "}\n\n"
        "#[cfg(test)]\n"
        "mod tests {\n"
        "    use super::{parse_events, socket_url};\n\n"
        "    #[test]\n"
        "    fn sse_data_lines_and_socket_url() {\n"
        "        let text = \"data: {\\\"a\\\":1}\\n\\ndata: [DONE]\\n\\n: keep-alive\\n\\ndata: hi\\n\";\n"
        "        assert_eq!(parse_events(text), \"{\\\"a\\\":1}\\nhi\");\n"
        "        assert_eq!(\n"
        "            socket_url(\"http://localhost:8080/\", \"/ws/pipeline/progress\"),\n"
        "            \"ws://localhost:8080/ws/pipeline/progress\"\n"
        "        );\n"
        "        assert_eq!(\n"
        "            socket_url(\"https://api.example\", \"/ws/progress/t\"),\n"
        "            \"wss://api.example/ws/progress/t\"\n"
        "        );\n"
        "    }\n"
        "}\n"
    )


def go_method(name: str, op: dict) -> str:
    params = op["pathParams"]
    args = ", ".join(f"{ident(p, 'go')[:1].lower() + ident(p, 'go')[1:]} string" for p in params)
    # Go ident is Pascal; use camel local names
    locals = []
    for p in params:
        pascal = ident(p, "go")
        local = pascal[:1].lower() + pascal[1:]
        if local in GO_KEYWORDS:
            local = local + "Arg"
        locals.append((p, local))
    arglist = ", ".join(f"{local} string" for _, local in locals)
    fmt = op["path"]
    fmt_args = []
    for p, local in locals:
        fmt = fmt.replace("{" + p + "}", "%s")
        fmt_args.append(f"encodePathParam({local})")
    path_line = f'path := "{fmt}"' if not fmt_args else "path := fmt.Sprintf(\"" + fmt + "\", " + ", ".join(fmt_args) + ")"
    if op["kind"] == "websocket":
        sig = f"func (s *OperationsService) {name}("
        sig += arglist + ") string {" if arglist else ") string {"
        return "\n".join(
            [
                f"// {name} {op['method']} {op['path']}",
                sig,
                "    " + path_line,
                "    return socketURL(s.c.BaseURL(), path)",
                "}",
                "",
            ]
        )
    sig = f"func (s *OperationsService) {name}(ctx context.Context"
    if arglist:
        sig += ", " + arglist
    sig += ", query url.Values, body any) (string, error) {"
    events = "true" if op["kind"] == "sse" else "false"
    return "\n".join(
        [
            f"// {name} {op['method']} {op['path']}",
            sig,
            "    " + path_line,
            f"    return s.call(ctx, {json.dumps(op['method'])}, path, query, body, {events})",
            "}",
            "",
        ]
    )


def emit_go(ops: list[dict], names: dict[str, str]) -> str:
    methods = []
    for op in ops:
        key = op["operationId"] + op["method"] + op["path"]
        methods.append(go_method(names[key], op))
    return (
        "// Code generated by sdks/tools/sdk_coverage.py. DO NOT EDIT.\n"
        "package edgequake\n\n"
        "import (\n"
        '    "context"\n'
        '    "fmt"\n'
        '    "net/url"\n'
        '    "strings"\n'
        ")\n\n"
        "// OperationsService calls OpenAPI routes that the typed services do not already mention.\n"
        "type OperationsService struct{ c *Client }\n\n"
        + "func (s *OperationsService) call(ctx context.Context, method, path string, query url.Values, body any, events bool) (string, error) {\n"
        + "    if len(query) > 0 {\n"
        + "        if strings.Contains(path, \"?\") {\n"
        + "            path += \"&\" + query.Encode()\n"
        + "        } else {\n"
        + "            path += \"?\" + query.Encode()\n"
        + "        }\n"
        + "    }\n"
        + "    text, err := s.c.raw(ctx, method, path, body)\n"
        + "    if err != nil || !events {\n"
        + "        return text, err\n"
        + "    }\n"
        + "    return parseEvents(text), nil\n"
        + "}\n\n"
        + "\n".join(methods)
        + "\nfunc parseEvents(text string) string {\n"
        + "    var events []string\n"
        + "    for _, block := range strings.Split(text, \"\\n\\n\") {\n"
        + "        for _, line := range strings.Split(block, \"\\n\") {\n"
        + "            if !strings.HasPrefix(line, \"data:\") {\n"
        + "                continue\n"
        + "            }\n"
        + "            data := strings.TrimSpace(strings.TrimPrefix(line, \"data:\"))\n"
        + "            if data != \"\" && data != \"[DONE]\" {\n"
        + "                events = append(events, data)\n"
        + "            }\n"
        + "        }\n"
        + "    }\n"
        + "    return strings.Join(events, \"\\n\")\n"
        + "}\n\n"
        + "func socketURL(base, path string) string {\n"
        + "    root := strings.TrimRight(base, \"/\")\n"
        + "    switch {\n"
        + "    case strings.HasPrefix(root, \"https://\"):\n"
        + "        root = \"wss://\" + strings.TrimPrefix(root, \"https://\")\n"
        + "    case strings.HasPrefix(root, \"http://\"):\n"
        + "        root = \"ws://\" + strings.TrimPrefix(root, \"http://\")\n"
        + "    }\n"
        + "    return root + path\n"
        + "}\n\n"
        + "func encodePathParam(value string) string {\n"
        + "    parts := strings.Split(value, \"/\")\n"
        + "    for i, part := range parts {\n"
        + "        parts[i] = url.PathEscape(part)\n"
        + "    }\n"
        + "    return strings.Join(parts, \"/\")\n"
        + "}\n"
    )


def java_method(name: str, op: dict) -> str:
    params = op["pathParams"]
    args = []
    for p in params:
        j = ident(p, "java")
        if j in {"class", "default"}:
            j = j + "Arg"
        args.append((p, j))
    arglist = ", ".join(f"String {j}" for _, j in args)
    fmt = op["path"]
    for p, j in args:
        fmt = fmt.replace("{" + p + "}", "\" + encodePathParam(" + j + ") + \"")
    path_expr = '"' + fmt + '"'
    path_expr = path_expr.replace('"" + ', "").replace(' + ""', "")
    if op["kind"] == "websocket":
        sig = f"    public String {name}(" + (arglist if arglist else "") + ") {"
        return "\n".join(
            [
                f"    /** {op['method']} {op['path']} */",
                sig,
                f"        return http.websocketUrl({path_expr});",
                "    }",
                "",
            ]
        )
    sig = f"    public String {name}("
    bits = []
    if arglist:
        bits.append(arglist)
    bits.append("Map<String, String> query")
    bits.append("Object body")
    sig += ", ".join(bits) + ") {"
    events = "true" if op["kind"] == "sse" else "false"
    return "\n".join(
        [
            f"    /** {op['method']} {op['path']} */",
            sig,
            f"        return call({json.dumps(op['method'])}, {path_expr}, query, body, {events});",
            "    }",
            "",
        ]
    )


def emit_java(ops: list[dict], names: dict[str, str]) -> str:
    methods = [java_method(names[op["operationId"] + op["method"] + op["path"]], op) for op in ops]
    return (
        "package io.edgequake.sdk.resources;\n\n"
        "import io.edgequake.sdk.internal.HttpHelper;\n\n"
        "import java.net.URLEncoder;\n"
        "import java.nio.charset.StandardCharsets;\n"
        "import java.util.ArrayList;\n"
        "import java.util.Map;\n\n"
        "/** Generated OpenAPI operations. Do not edit by hand. */\n"
        "public class OperationsGenerated {\n"
        "    private final HttpHelper http;\n"
        "    public OperationsGenerated(HttpHelper http) { this.http = http; }\n\n"
        "    private String call(String method, String path, Map<String, String> query, Object body, boolean events) {\n"
        "        var text = http.exchange(method, appendQuery(path, query), body);\n"
        "        return events ? String.join(\"\\n\", parseEvents(text)) : text;\n"
        "    }\n\n"
        "    private static String appendQuery(String path, Map<String, String> query) {\n"
        "        if (query == null || query.isEmpty()) return path;\n"
        "        var pairs = new ArrayList<String>();\n"
        "        for (var entry : query.entrySet()) {\n"
        "            if (entry.getValue() == null) continue;\n"
        "            pairs.add(URLEncoder.encode(entry.getKey(), StandardCharsets.UTF_8)\n"
        "                + \"=\" + URLEncoder.encode(entry.getValue(), StandardCharsets.UTF_8));\n"
        "        }\n"
        "        if (pairs.isEmpty()) return path;\n"
        "        return path + (path.contains(\"?\") ? \"&\" : \"?\") + String.join(\"&\", pairs);\n"
        "    }\n\n"
        "    private static java.util.List<String> parseEvents(String text) {\n"
        "        var events = new ArrayList<String>();\n"
        "        for (var block : text.split(\"\\n\\n\", -1)) {\n"
        "            for (var line : block.split(\"\\n\", -1)) {\n"
        "                if (!line.startsWith(\"data:\")) continue;\n"
        "                var data = line.substring(5).trim();\n"
        "                if (!data.isEmpty() && !data.equals(\"[DONE]\")) events.add(data);\n"
        "            }\n"
        "        }\n"
        "        return events;\n"
        "    }\n\n"
        + "".join(methods)
        + "    private static String encodePathParam(String value) {\n"
        + "        var parts = value.split(\"/\", -1);\n"
        + "        var out = new StringBuilder();\n"
        + "        for (int i = 0; i < parts.length; i++) {\n"
        + "            if (i > 0) out.append('/');\n"
        + "            out.append(URLEncoder.encode(parts[i], StandardCharsets.UTF_8).replace(\"+\", \"%20\"));\n"
        + "        }\n"
        + "        return out.toString();\n"
        + "    }\n"
        + "}\n"
    )


def kt_method(name: str, op: dict) -> str:
    params = op["pathParams"]
    args = []
    for p in params:
        k = ident(p, "kotlin")
        args.append((p, k))
    arglist = ", ".join(f"{k}: String" for _, k in args)
    bits = []
    if arglist:
        bits.append(arglist)
    bits.append("query: Map<String, String?> = emptyMap()")
    bits.append("body: Any? = null")
    fmt = op["path"]
    for p, k in args:
        fmt = fmt.replace("{" + p + "}", "${encodePathParam(" + k + ")}")
    if op["kind"] == "websocket":
        sig_bits = arglist
        return "\n".join(
            [
                f"    /** {op['method']} {op['path']} */",
                f"    fun {name}({sig_bits}): String {{",
                f'        return socket("{fmt}")',
                "    }",
                "",
            ]
        )
    events = "true" if op["kind"] == "sse" else "false"
    return "\n".join(
        [
            f"    /** {op['method']} {op['path']} */",
            f"    fun {name}({', '.join(bits)}): String {{",
            f'        return call({json.dumps(op["method"])}, "{fmt}", query, body, {events})',
            "    }",
            "",
        ]
    )


def emit_kotlin(ops: list[dict], names: dict[str, str]) -> str:
    methods = [kt_method(names[op["operationId"] + op["method"] + op["path"]], op) for op in ops]
    return (
        "package io.edgequake.sdk.resources\n\n"
        "import io.edgequake.sdk.internal.HttpHelper\n\n"
        "/** Generated OpenAPI operations. Do not edit by hand. */\n"
        "class OperationsGenerated(private val http: HttpHelper) {\n"
        + "    private fun call(method: String, path: String, query: Map<String, String?>, body: Any?, events: Boolean): String {\n"
        + "        val text = http.exchange(method, appendQuery(path, query), body)\n"
        + "        return if (events) parseEvents(text) else text\n"
        + "    }\n\n"
        + "    private fun appendQuery(path: String, query: Map<String, String?>): String {\n"
        + "        val qs = query.filterValues { it != null }.entries.joinToString(\"&\") { \"${encodeQuery(it.key)}=${encodeQuery(it.value!!)}\" }\n"
        + "        if (qs.isEmpty()) return path\n"
        + "        return path + (if (path.contains('?')) \"&\" else \"?\") + qs\n"
        + "    }\n\n"
        + "    private fun parseEvents(text: String): String =\n"
        + "        text.split(\"\\n\\n\").flatMap { block ->\n"
        + "            block.lineSequence().mapNotNull { line ->\n"
        + "                if (!line.startsWith(\"data:\")) return@mapNotNull null\n"
        + "                line.removePrefix(\"data:\").trim().takeIf { it.isNotEmpty() && it != \"[DONE]\" }\n"
        + "            }\n"
        + "        }.joinToString(\"\\n\")\n\n"
        + "    private fun socket(path: String): String {\n"
        + "        val root = http.config.baseUrl.trimEnd('/')\n"
        + "        val ws = when {\n"
        + "            root.startsWith(\"https://\") -> \"wss://\" + root.removePrefix(\"https://\")\n"
        + "            root.startsWith(\"http://\") -> \"ws://\" + root.removePrefix(\"http://\")\n"
        + "            else -> root\n"
        + "        }\n"
        + "        return ws + path\n"
        + "    }\n\n"
        + "".join(methods)
        + "    private fun encodePathParam(value: String): String =\n"
        + "        value.split(\"/\").joinToString(\"/\") { java.net.URLEncoder.encode(it, Charsets.UTF_8).replace(\"+\", \"%20\") }\n\n"
        + "    private fun encodeQuery(value: String): String =\n"
        + "        java.net.URLEncoder.encode(value, Charsets.UTF_8)\n"
        + "}\n"
    )


def cs_method(name: str, op: dict) -> str:
    params = op["pathParams"]
    args = []
    for p in params:
        c = ident(p, "csharp")
        local = c[:1].lower() + c[1:]
        args.append((p, local))
    arglist = ", ".join(f"string {local}" for _, local in args)
    pieces: list[str] = []
    cursor = op["path"]
    for p, local in args:
        head, cursor = cursor.split("{" + p + "}", 1)
        if head:
            pieces.append(json.dumps(head))
        pieces.append(f"EncodePathParam({local})")
    if cursor:
        pieces.append(json.dumps(cursor))
    path_expr = " + ".join(pieces) if pieces else '""'
    if op["kind"] == "websocket":
        sig = f"public string {name}({arglist})" if arglist else f"public string {name}()"
        return "\n".join(
            [
                f"    /// <summary>{op['method']} {op['path']}</summary>",
                f"    {sig} => http.WebSocketUrl({path_expr});",
                "",
            ]
        )
    bits = [arglist] if arglist else []
    bits.append("Dictionary<string, string>? query = null")
    bits.append("object? body = null")
    events = "true" if op["kind"] == "sse" else "false"
    return "\n".join(
        [
            f"    /// <summary>{op['method']} {op['path']}</summary>",
            f"    public Task<string> {name}({', '.join(bits)})",
            "    {",
            f"        return Call({json.dumps(op['method'])}, {path_expr}, query, body, {events});",
            "    }",
            "",
        ]
    )


def emit_csharp(ops: list[dict], names: dict[str, str]) -> str:
    methods = [cs_method(names[op["operationId"] + op["method"] + op["path"]], op) for op in ops]
    return (
        "using System.Collections.Generic;\n"
        "using System.Linq;\n\n"
        "namespace EdgeQuakeSDK;\n\n"
        "/// <summary>Generated OpenAPI operations. Do not edit by hand.</summary>\n"
        "public class OperationsGenerated(HttpHelper http)\n"
        "{\n"
        + "    private async Task<string> Call(string method, string path, Dictionary<string, string>? query, object? body, bool events)\n"
        + "    {\n"
        + "        var text = await http.ExchangeAsync(method, AppendQuery(path, query), body);\n"
        + "        return events ? string.Join('\\n', ParseEvents(text)) : text;\n"
        + "    }\n\n"
        + "    private static string AppendQuery(string path, Dictionary<string, string>? query)\n"
        + "    {\n"
        + "        if (query is not { Count: > 0 }) return path;\n"
        + "        var qs = string.Join('&', query.Where(kv => kv.Value != null).Select(kv => Uri.EscapeDataString(kv.Key) + \"=\" + Uri.EscapeDataString(kv.Value)));\n"
        + "        if (qs.Length == 0) return path;\n"
        + "        return path + (path.Contains('?') ? \"&\" : \"?\") + qs;\n"
        + "    }\n\n"
        + "    private static IEnumerable<string> ParseEvents(string text)\n"
        + "    {\n"
        + "        foreach (var block in text.Split(\"\\n\\n\"))\n"
        + "        {\n"
        + "        foreach (var line in block.Split('\\n'))\n"
        + "        {\n"
        + "            if (!line.StartsWith(\"data:\")) continue;\n"
        + "            var data = line[5..].Trim();\n"
        + "            if (data.Length > 0 && data != \"[DONE]\") yield return data;\n"
        + "        }\n"
        + "        }\n"
        + "    }\n\n"
        + "".join(methods)
        + "    private static string EncodePathParam(string value) =>\n"
        + "        string.Join(\"/\", value.Split('/').Select(Uri.EscapeDataString));\n"
        + "}\n"
    )


def php_method(name: str, op: dict) -> str:
    params = op["pathParams"]
    args = []
    for p in params:
        args.append((p, ident(p, "php")))
    arglist = ", ".join(f"string ${n}" for _, n in args)
    pieces: list[str] = []
    cursor = op["path"]
    for p, n in args:
        head, cursor = cursor.split("{" + p + "}", 1)
        if head:
            pieces.append(json.dumps(head))
        pieces.append(f"$this->pathParam(${n})")
    if cursor:
        pieces.append(json.dumps(cursor))
    path_expr = " . ".join(pieces) if pieces else '""'
    if op["kind"] == "websocket":
        sig = f"public function {name}({arglist}): string" if arglist else f"public function {name}(): string"
        return "\n".join(
            [
                f"    /** {op['method']} {op['path']} */",
                f"    {sig}",
                "    {",
                f"        return $this->socket({path_expr});",
                "    }",
                "",
            ]
        )
    bits = [arglist] if arglist else []
    bits.append("?array $query = null")
    bits.append("?array $body = null")
    events = "true" if op["kind"] == "sse" else "false"
    return "\n".join(
        [
            f"    /** {op['method']} {op['path']} */",
            f"    public function {name}({', '.join(bits)}): string",
            "    {",
            f"        return $this->call({json.dumps(op['method'])}, {path_expr}, $query, $body, {events});",
            "    }",
            "",
        ]
    )


def emit_php(ops: list[dict], names: dict[str, str]) -> str:
    methods = [php_method(names[op["operationId"] + op["method"] + op["path"]], op) for op in ops]
    return (
        "<?php\n\n"
        "declare(strict_types=1);\n\n"
        "namespace EdgeQuake;\n\n"
        "/** Generated OpenAPI operations. Do not edit by hand. */\n"
        "class OperationsGenerated\n"
        "{\n"
        "    public function __construct(private readonly HttpHelper $http) {}\n\n"
        + "    private function call(string $method, string $path, ?array $query, ?array $body, bool $events): string\n"
        + "    {\n"
        + "        $text = $this->http->raw($method, $this->appendQuery($path, $query), $body);\n"
        + "        return $events ? implode(\"\\n\", $this->parseEvents($text)) : $text;\n"
        + "    }\n\n"
        + "    private function appendQuery(string $path, ?array $query): string\n"
        + "    {\n"
        + "        if (!$query) return $path;\n"
        + "        $qs = http_build_query(array_filter($query, fn($v) => $v !== null));\n"
        + "        if ($qs === '') return $path;\n"
        + "        return $path . (str_contains($path, '?') ? '&' : '?') . $qs;\n"
        + "    }\n\n"
        + "    private function parseEvents(string $text): array\n"
        + "    {\n"
        + "        $events = [];\n"
        + "        foreach (explode(\"\\n\\n\", $text) as $block) {\n"
        + "            foreach (explode(\"\\n\", $block) as $line) {\n"
        + "                if (!str_starts_with($line, 'data:')) continue;\n"
        + "                $data = trim(substr($line, 5));\n"
        + "                if ($data !== '' && $data !== '[DONE]') $events[] = $data;\n"
        + "            }\n"
        + "        }\n"
        + "        return $events;\n"
        + "    }\n\n"
        + "    private function socket(string $path): string\n"
        + "    {\n"
        + "        $root = rtrim($this->http->baseUrl(), '/');\n"
        + "        if (str_starts_with($root, 'https://')) $root = 'wss://' . substr($root, 8);\n"
        + "        elseif (str_starts_with($root, 'http://')) $root = 'ws://' . substr($root, 7);\n"
        + "        return $root . $path;\n"
        + "    }\n\n"
        + "".join(methods)
        + "    private function pathParam(string $value): string\n"
        + "    {\n"
        + "        return implode('/', array_map('rawurlencode', explode('/', $value)));\n"
        + "    }\n"
        + "}\n"
    )


def rb_method(name: str, op: dict) -> str:
    params = op["pathParams"]
    args = [ident(p, "ruby") for p in params]
    sig_args = ", ".join(f"{a}:" for a in args)
    pieces: list[str] = []
    cursor = op["path"]
    for p, a in zip(params, args):
        head, cursor = cursor.split("{" + p + "}", 1)
        if head:
            pieces.append(json.dumps(head))
        pieces.append(f"path_param({a})")
    if cursor:
        pieces.append(json.dumps(cursor))
    ruby_path = " + ".join(pieces) if pieces else '""'
    if op["kind"] == "websocket":
        sig = f"def {name}({sig_args})" if sig_args else f"def {name}"
        return "\n".join(
            [
                f"      # {op['method']} {op['path']}",
                f"      {sig}",
                f"        socket({ruby_path})",
                "      end",
                "",
            ]
        )
    bits = [sig_args] if sig_args else []
    bits.append("query: nil")
    bits.append("body: nil")
    events = "true" if op["kind"] == "sse" else "false"
    return "\n".join(
        [
            f"      # {op['method']} {op['path']}",
            f"      def {name}({', '.join(bits)})",
            f"        call({json.dumps(op['method'].lower())}, {ruby_path}, query, body, {events})",
            "      end",
            "",
        ]
    )


def emit_ruby(ops: list[dict], names: dict[str, str]) -> str:
    methods = [rb_method(names[op["operationId"] + op["method"] + op["path"]], op) for op in ops]
    return (
        "# frozen_string_literal: true\n\n"
        "# Generated OpenAPI operations. Do not edit by hand.\n"
        "require \"uri\"\n\n"
        "module EdgeQuake\n"
        "  class OperationsGenerated\n"
        "    def initialize(http)\n"
        "      @http = http\n"
        "    end\n\n"
        + "    def call(method, path, query, body, events)\n"
        + "      text = @http.call_raw(method, append_query(path, query), body)\n"
        + "      events ? parse_events(text).join(\"\\n\") : text\n"
        + "    end\n\n"
        + "    def append_query(path, query)\n"
        + "      return path unless query\n"
        + "      qs = URI.encode_www_form(query.compact)\n"
        + "      return path if qs.empty?\n"
        + "      path + (path.include?('?') ? '&' : '?') + qs\n"
        + "    end\n\n"
        + "    def parse_events(text)\n"
        + "      events = []\n"
        + "      text.to_s.split(\"\\n\\n\").each do |block|\n"
        + "        block.each_line do |line|\n"
        + "          next unless line.start_with?('data:')\n"
        + "          data = line.sub(/\\Adata:/, '').strip\n"
        + "          events << data unless data.empty? || data == '[DONE]'\n"
        + "        end\n"
        + "      end\n"
        + "      events\n"
        + "    end\n\n"
        + "    def socket(path)\n"
        + "      root = @http.base_url.to_s.sub(%r{/+$}, '')\n"
        + "      root = if root.start_with?('https://')\n"
        + "               'wss://' + root.delete_prefix('https://')\n"
        + "             elsif root.start_with?('http://')\n"
        + "               'ws://' + root.delete_prefix('http://')\n"
        + "             else\n"
        + "               root\n"
        + "             end\n"
        + "      root + path\n"
        + "    end\n\n"
        + "".join(methods)
        + "    def path_param(value)\n"
        + "      value.to_s.split('/', -1).map { |part| URI.encode_www_form_component(part).gsub('+', '%20') }.join('/')\n"
        + "    end\n"
        + "  end\n"
        "end\n"
    )


def swift_method(name: str, op: dict) -> str:
    params = op["pathParams"]
    args = []
    for p in params:
        args.append((p, ident(p, "swift")))
    arglist = ", ".join(f"{n}: String" for _, n in args)
    pieces: list[str] = []
    cursor = op["path"]
    for p, n in args:
        head, cursor = cursor.split("{" + p + "}", 1)
        if head:
            pieces.append(json.dumps(head))
        pieces.append(f"encodePathParam({n})")
    if cursor:
        pieces.append(json.dumps(cursor))
    path_expr = " + ".join(pieces) if pieces else '""'
    if op["kind"] == "websocket":
        sig = f"public func {name}({arglist}) -> String" if arglist else f"public func {name}() -> String"
        return "\n".join(
            [
                f"    /// {op['method']} {op['path']}",
                f"    {sig} {{",
                f"        socket({path_expr})",
                "    }",
                "",
            ]
        )
    bits = [arglist] if arglist else []
    bits.append("query: [String: String] = [:]")
    bits.append("body: [String: String]? = nil")
    events = "true" if op["kind"] == "sse" else "false"
    return "\n".join(
        [
            f"    /// {op['method']} {op['path']}",
            f"    public func {name}({', '.join(bits)}) async throws -> String {{",
            f"        try await call({json.dumps(op['method'])}, path: {path_expr}, query: query, body: body, events: {events})",
            "    }",
            "",
        ]
    )


def emit_swift(ops: list[dict], names: dict[str, str]) -> str:
    methods = [swift_method(names[op["operationId"] + op["method"] + op["path"]], op) for op in ops]
    return (
        "import Foundation\n\n"
        "/// Generated OpenAPI operations. Do not edit by hand.\n"
        "public final class OperationsGenerated: @unchecked Sendable {\n"
        "    private let http: HttpHelper\n"
        "    init(_ http: HttpHelper) { self.http = http }\n\n"
        + "    private func call(_ method: String, path: String, query: [String: String], body: [String: String]?, events: Bool) async throws -> String {\n"
        + "        let text = try await http.call(method, path: appendQuery(path, query), body: body)\n"
        + "        return events ? parseEvents(text).joined(separator: \"\\n\") : text\n"
        + "    }\n\n"
        + "    private func appendQuery(_ path: String, _ query: [String: String]) -> String {\n"
        + "        if query.isEmpty { return path }\n"
        + "        let qs = query.map { \"\\(encodeQuery($0.key))=\\(encodeQuery($0.value))\" }.joined(separator: \"&\")\n"
        + "        return path + (path.contains(\"?\") ? \"&\" : \"?\") + qs\n"
        + "    }\n\n"
        + "    private func parseEvents(_ text: String) -> [String] {\n"
        + "        text.components(separatedBy: \"\\n\\n\").flatMap { block in\n"
        + "            block.components(separatedBy: \"\\n\").compactMap { line -> String? in\n"
        + "                guard line.hasPrefix(\"data:\") else { return nil }\n"
        + "                let data = String(line.dropFirst(5)).trimmingCharacters(in: .whitespaces)\n"
        + "                return (data.isEmpty || data == \"[DONE]\") ? nil : data\n"
        + "            }\n"
        + "        }\n"
        + "    }\n\n"
        + "    private func socket(_ path: String) -> String {\n"
        + "        var root = http.config.baseUrl\n"
        + "        while root.hasSuffix(\"/\") { root.removeLast() }\n"
        + "        if root.hasPrefix(\"https://\") { root = \"wss://\" + root.dropFirst(8) }\n"
        + "        else if root.hasPrefix(\"http://\") { root = \"ws://\" + root.dropFirst(7) }\n"
        + "        return root + path\n"
        + "    }\n\n"
        + "".join(methods)
        + "    private func encodePathParam(_ value: String) -> String {\n"
        + "        value.split(separator: \"/\", omittingEmptySubsequences: false).map { encodeQuery(String($0)) }.joined(separator: \"/\")\n"
        + "    }\n\n"
        + "    private func encodeQuery(_ value: String) -> String {\n"
        + "        var allowed = CharacterSet.alphanumerics\n"
        + "        allowed.insert(charactersIn: \"-._~\")\n"
        + "        return value.addingPercentEncoding(withAllowedCharacters: allowed) ?? value\n"
        + "    }\n"
        + "}\n"
    )


EMITTERS = {
    "python": emit_python,
    "typescript": emit_typescript,
    "rust": emit_rust,
    "go": emit_go,
    "java": emit_java,
    "kotlin": emit_kotlin,
    "csharp": emit_csharp,
    "php": emit_php,
    "ruby": emit_ruby,
    "swift": emit_swift,
}


def classify(ops: list[dict]) -> dict[str, list[dict]]:
    sources = {lang: iter_sources(root) for lang, root in SDK_ROOTS.items()}
    missing = {}
    for lang, source in sources.items():
        missing[lang] = [op for op in ops if op["kind"] != "websocket" and not covered(source, op["path"])]
    return missing


def still_missing(lang: str, gaps: list[dict]) -> list[dict]:
    gen = GENERATED[lang]
    if not gen.exists():
        return gaps
    text = gen.read_text(errors="ignore")
    return [op for op in gaps if op["path"] not in text]


def write_report(ops: list[dict], missing: dict[str, list[dict]]) -> None:
    http_ops = [op for op in ops if op["kind"] != "websocket"]
    ws = [op for op in ops if op["kind"] == "websocket"]
    lines = [
        "# SDK OpenAPI coverage",
        "",
        f"Operations: {len(ops)}. HTTP: {len(http_ops)}. WebSocket out of band: {len(ws)}.",
        "",
        "Hand-written resources are counted first. Generated files then fill the rest.",
        "A path is covered when its template string appears in that SDK.",
        "",
    ]
    for lang, gaps in missing.items():
        left = still_missing(lang, gaps)
        lines.append(f"## {lang}")
        lines.append("")
        lines.append(f"- Hand-written mentions: {len(http_ops) - len(gaps)}")
        lines.append(f"- Generated methods: {len(gaps) + len(ws)}")
        lines.append(f"- HTTP paths still missing: {len(left)}")
        lines.append("- Unit tests: path-and-verb contract test")
        lines.append("- Live tests: shared scenario, skipped unless `EDGEQUAKE_E2E_URL` is a healthy API")
        if left[:8]:
            lines.append("- Still missing:")
            for op in left[:8]:
                lines.append(f"  - `{op['method']} {op['path']}`")
        lines.append("")
    REPORT.write_text("\n".join(lines))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generate", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--phantoms", action="store_true")
    args = parser.parse_args()
    ops = load_operations()
    missing = classify(ops)
    manifest = []
    for op in ops:
        row = dict(op)
        row["missing"] = {lang: any(op["path"] == g["path"] and op["method"] == g["method"] for g in gaps) for lang, gaps in missing.items()}
        manifest.append(row)
    MANIFEST.write_text(json.dumps(manifest, indent=2))
    print(f"operations={len(ops)} report={REPORT}")
    for lang, gaps in missing.items():
        print(f"  {lang}: missing {len(gaps)}")
    if args.generate:
        sockets = [op for op in ops if op["kind"] == "websocket"]
        for lang, gaps in missing.items():
            batch = gaps + sockets
            names = unique_names(batch, lang)
            text = EMITTERS[lang](batch, names)
            GENERATED[lang].parent.mkdir(parents=True, exist_ok=True)
            GENERATED[lang].write_text(text)
            print(f"wrote {GENERATED[lang]} ({len(batch)} methods)")
    write_report(ops, missing)
    if args.check:
        # Re-scan including generated files by reading them into the source.
        still = 0
        for lang, root in SDK_ROOTS.items():
            source = iter_sources(root)
            gen = GENERATED[lang]
            if gen.exists():
                source += "\n" + gen.read_text(errors="ignore")
            for op in ops:
                if op["kind"] == "websocket":
                    continue
                if not covered(source, op["path"]):
                    still += 1
                    print(f"MISSING {lang} {op['method']} {op['path']}")
        if still:
            raise SystemExit(f"{still} operations still missing")
        print("coverage check ok")
    if args.phantoms:
        hits = scan_phantoms(ops)
        for rel, line, verb, call_path, detail in hits:
            print(f"PHANTOM {rel}:{line} {verb} {call_path} ({detail})")
        if hits:
            raise SystemExit(f"{len(hits)} hand-written calls do not match OpenAPI")
        print("phantom check ok")


if __name__ == "__main__":
    main()
