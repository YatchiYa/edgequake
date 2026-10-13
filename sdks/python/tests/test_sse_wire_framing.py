"""Exercise SDK parsers with actual SSE response bytes, not unframed mock lines."""

import httpx
import pytest
from pydantic import BaseModel

from edgequake._streaming import AsyncSSEStream, SSEStream


class Event(BaseModel):
    chunk: str


WIRES = [
    b'data:{"chunk":"hello"}\n\ndata:[DONE]\n\n',
    b'data: {"chunk":\ndata: "hello"}\n\ndata: [DONE]\n\n',
    b'\xef\xbb\xbfdata:{"chunk":"hello"}\r\n\r\ndata:[DONE]\r\n\r\n',
    b': heartbeat\nid: 42\nevent: token\nretry: 500\ndata:{"chunk":"hello"}\n\n',
    b'data:{"chunk":"hello"}\r\rdata:[DONE]\r\r',
]


@pytest.mark.parametrize("wire", WIRES)
def test_sync_wire_event_framing(wire: bytes) -> None:
    response = httpx.Response(200, content=wire)
    assert [event.chunk for event in SSEStream(response, Event)] == ["hello"]


@pytest.mark.asyncio
@pytest.mark.parametrize("wire", WIRES)
async def test_async_wire_event_framing(wire: bytes) -> None:
    response = httpx.Response(200, content=wire)
    assert [event.chunk async for event in AsyncSSEStream(response, Event)] == ["hello"]


def test_sync_preserves_legacy_unterminated_final_event() -> None:
    response = httpx.Response(200, content=b'data: {"chunk":"hello"}')
    assert SSEStream(response, Event).to_string() == "hello"


@pytest.mark.asyncio
async def test_async_preserves_legacy_unterminated_final_event() -> None:
    response = httpx.Response(200, content=b'data: {"chunk":"hello"}')
    assert await AsyncSSEStream(response, Event).to_string() == "hello"


def test_empty_data_is_invalid_json_not_a_heartbeat() -> None:
    from edgequake._errors import StreamError

    response = httpx.Response(200, content=b'data:\n\n')
    with pytest.raises(StreamError, match="Failed to parse SSE data"):
        list(SSEStream(response, Event))


@pytest.mark.asyncio
async def test_async_empty_data_is_invalid_json_not_a_heartbeat() -> None:
    from edgequake._errors import StreamError

    response = httpx.Response(200, content=b'data:\n\n')
    with pytest.raises(StreamError, match="Failed to parse SSE data"):
        [event async for event in AsyncSSEStream(response, Event)]
