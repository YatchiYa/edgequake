"""Path and verb contract for generated OpenAPI operations. No server required."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "sdks/operation-manifest.json"
GENERATED = ROOT / "sdks/python/edgequake/resources/operations_generated.py"
SOURCE = ROOT / "sdks/python/edgequake"


def test_every_http_operation_path_and_generated_verb() -> None:
    ops = json.loads(MANIFEST.read_text())
    generated = GENERATED.read_text()
    blob = "\n".join(
        path.read_text(errors="ignore")
        for path in SOURCE.rglob("*.py")
        if "__pycache__" not in path.parts
    )
    for op in ops:
        if op["kind"] == "websocket":
            assert op["path"] in generated, op["path"]
            continue
        assert op["path"] in blob, op["path"]
        if op["missing"]["python"]:
            assert f"{op['method']} {op['path']}" in generated


def test_sse_parser_and_socket_url() -> None:
    from edgequake.resources.operations_generated import _events, _socket_url

    text = 'data: {"a":1}\n\ndata: [DONE]\n\n: keep-alive\n\ndata: hi\n'
    assert _events(text) == ['{"a":1}', "hi"]
    assert _socket_url("http://localhost:8080/", "/ws/pipeline/progress") == (
        "ws://localhost:8080/ws/pipeline/progress"
    )
    assert _socket_url("https://api.example", "/ws/progress/t") == "wss://api.example/ws/progress/t"
