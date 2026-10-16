"""Streaming HTTP errors retain server diagnostics and release their response."""

import httpx
import pytest

from edgequake._config import ClientConfig
from edgequake._errors import ApiError
from edgequake._transport import AsyncTransport, SyncTransport


class SyncBody(httpx.SyncByteStream):
    def __init__(self, fail=False):
        self.closed = False
        self.fail = fail

    def __iter__(self):
        if self.fail:
            raise httpx.ReadError("broken error body")
        yield b'{"message":"query rejected","code":"QUERY_INVALID"}'

    def close(self):
        self.closed = True


class AsyncBody(httpx.AsyncByteStream):
    def __init__(self, fail=False):
        self.closed = False
        self.fail = fail

    async def __aiter__(self):
        if self.fail:
            raise httpx.ReadError("broken error body")
        yield b'{"message":"query rejected","code":"QUERY_INVALID"}'

    async def aclose(self):
        self.closed = True


@pytest.mark.parametrize("status", [400, 500])
def test_sync_stream_error_closes_response_and_preserves_details(status):
    body = SyncBody()
    transport = SyncTransport(ClientConfig())
    transport._client.close()
    transport._client = httpx.Client(base_url="http://test", transport=httpx.MockTransport(
        lambda request: httpx.Response(status, stream=body)))
    try:
        with pytest.raises(ApiError) as error:
            transport.stream("POST", "/query/stream")
        assert body.closed
        assert error.value.message == "query rejected"
        assert error.value.code == "QUERY_INVALID"
    finally:
        transport._client.close()


@pytest.mark.asyncio
@pytest.mark.parametrize("status", [400, 500])
async def test_async_stream_error_closes_response_and_preserves_details(status):
    body = AsyncBody()
    transport = AsyncTransport(ClientConfig())
    await transport._client.aclose()
    transport._client = httpx.AsyncClient(base_url="http://test", transport=httpx.MockTransport(
        lambda request: httpx.Response(status, stream=body)))
    try:
        with pytest.raises(ApiError) as error:
            await transport.stream("POST", "/query/stream")
        assert body.closed
        assert error.value.message == "query rejected"
        assert error.value.code == "QUERY_INVALID"
    finally:
        await transport._client.aclose()


def test_sync_stream_error_read_failure_still_closes():
    body = SyncBody(fail=True)
    transport = SyncTransport(ClientConfig())
    transport._client.close()
    transport._client = httpx.Client(base_url="http://test", transport=httpx.MockTransport(
        lambda request: httpx.Response(400, stream=body)))
    try:
        with pytest.raises(httpx.ReadError, match="broken error body"):
            transport.stream("POST", "/query/stream")
        assert body.closed
    finally:
        transport._client.close()


@pytest.mark.asyncio
async def test_async_stream_error_read_failure_still_closes():
    body = AsyncBody(fail=True)
    transport = AsyncTransport(ClientConfig())
    await transport._client.aclose()
    transport._client = httpx.AsyncClient(base_url="http://test", transport=httpx.MockTransport(
        lambda request: httpx.Response(400, stream=body)))
    try:
        with pytest.raises(httpx.ReadError, match="broken error body"):
            await transport.stream("POST", "/query/stream")
        assert body.closed
    finally:
        await transport._client.aclose()


def test_sync_success_stream_remains_caller_owned():
    body = SyncBody()
    transport = SyncTransport(ClientConfig())
    transport._client.close()
    transport._client = httpx.Client(base_url="http://test", transport=httpx.MockTransport(
        lambda request: httpx.Response(200, stream=body)))
    try:
        response = transport.stream("POST", "/query/stream")
        assert not body.closed
        assert not response.is_stream_consumed
        response.close()
        assert body.closed
    finally:
        transport._client.close()


@pytest.mark.asyncio
async def test_async_success_stream_remains_caller_owned():
    body = AsyncBody()
    transport = AsyncTransport(ClientConfig())
    await transport._client.aclose()
    transport._client = httpx.AsyncClient(base_url="http://test", transport=httpx.MockTransport(
        lambda request: httpx.Response(200, stream=body)))
    try:
        response = await transport.stream("POST", "/query/stream")
        assert not body.closed
        assert not response.is_stream_consumed
        await response.aclose()
        assert body.closed
    finally:
        await transport._client.aclose()
