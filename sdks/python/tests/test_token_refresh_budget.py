"""JWT refresh must be independent of transient HTTP retry budgets."""

import httpx
import pytest

from edgequake._config import ClientConfig
from edgequake._errors import UnauthorizedError
from edgequake._transport import AsyncTransport, SyncTransport


@pytest.mark.parametrize("max_retries", [0, 1, 3])
@pytest.mark.parametrize("refreshed_status", [200, 401])
def test_sync_refresh_resends_once(max_retries: int, refreshed_status: int) -> None:
    tokens = []
    refreshes = []

    def refresh(token: str) -> str:
        refreshes.append(token)
        return "fresh"

    def handler(request: httpx.Request) -> httpx.Response:
        tokens.append(request.headers["authorization"])
        return httpx.Response(401 if len(tokens) == 1 else refreshed_status, json={"ok": True})

    transport = SyncTransport(
        ClientConfig(
            base_url="https://example.invalid",
            jwt="expired",
            max_retries=max_retries,
            on_token_refresh=refresh,
        )
    )
    transport._client.close()
    transport._client = httpx.Client(
        base_url="https://example.invalid", transport=httpx.MockTransport(handler)
    )
    try:
        if refreshed_status == 401:
            with pytest.raises(UnauthorizedError):
                transport.request("GET", "/health")
        else:
            assert transport.request("GET", "/health").status_code == 200
    finally:
        transport.close()
    assert tokens == ["Bearer expired", "Bearer fresh"]
    assert refreshes == ["expired"]


@pytest.mark.asyncio
@pytest.mark.parametrize("max_retries", [0, 1, 3])
@pytest.mark.parametrize("refreshed_status", [200, 401])
async def test_async_refresh_resends_once(max_retries: int, refreshed_status: int) -> None:
    tokens = []
    refreshes = []

    def refresh(token: str) -> str:
        refreshes.append(token)
        return "fresh"

    def handler(request: httpx.Request) -> httpx.Response:
        tokens.append(request.headers["authorization"])
        return httpx.Response(401 if len(tokens) == 1 else refreshed_status, json={"ok": True})

    transport = AsyncTransport(
        ClientConfig(
            base_url="https://example.invalid",
            jwt="expired",
            max_retries=max_retries,
            on_token_refresh=refresh,
        )
    )
    await transport._client.aclose()
    transport._client = httpx.AsyncClient(
        base_url="https://example.invalid", transport=httpx.MockTransport(handler)
    )
    try:
        if refreshed_status == 401:
            with pytest.raises(UnauthorizedError):
                await transport.request("GET", "/health")
        else:
            assert (await transport.request("GET", "/health")).status_code == 200
    finally:
        await transport.close()
    assert tokens == ["Bearer expired", "Bearer fresh"]
    assert refreshes == ["expired"]
