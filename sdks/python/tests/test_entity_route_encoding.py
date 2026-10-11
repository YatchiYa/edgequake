"""Entity names must remain one literal route segment in SDK requests."""

import httpx
import pytest

from edgequake._config import ClientConfig
from edgequake._errors import NotFoundError
from edgequake._transport import AsyncTransport, SyncTransport
from edgequake.resources.graph import AsyncEntitiesResource, EntitiesResource
from edgequake.types.graph import EntityUpdate

NAMES = [
    ("AC/DC", "AC%2FDC"),
    ("C#", "C%23"),
    ("A?B", "A%3FB"),
    ("100%REAL", "100%25REAL"),
    ("PLAIN", "PLAIN"),
]
OPERATIONS = [
    ("get", ""),
    ("update", ""),
    ("delete", "?confirm=true"),
    ("neighborhood", "/neighborhood?depth=1"),
]


@pytest.mark.parametrize("name,encoded", NAMES)
@pytest.mark.parametrize("operation,suffix", OPERATIONS)
def test_sync_entity_routes_keep_literal_names(name, encoded, operation, suffix) -> None:
    requests = []

    def handler(request: httpx.Request) -> httpx.Response:
        requests.append(request)
        return httpx.Response(404, json={"message": "owned test response"})

    transport = SyncTransport(ClientConfig(base_url="https://example.invalid", max_retries=0))
    transport._client.close()
    transport._client = httpx.Client(
        base_url="https://example.invalid", transport=httpx.MockTransport(handler)
    )
    resource = EntitiesResource(transport)
    args = (name, EntityUpdate(description="updated")) if operation == "update" else (name,)
    try:
        with pytest.raises(NotFoundError):
            getattr(resource, operation)(*args)
    finally:
        transport.close()
    assert requests[0].url.raw_path == f"/api/v1/graph/entities/{encoded}{suffix}".encode()


@pytest.mark.asyncio
@pytest.mark.parametrize("name,encoded", NAMES)
@pytest.mark.parametrize("operation,suffix", OPERATIONS)
async def test_async_entity_routes_keep_literal_names(name, encoded, operation, suffix) -> None:
    requests = []

    def handler(request: httpx.Request) -> httpx.Response:
        requests.append(request)
        return httpx.Response(404, json={"message": "owned test response"})

    transport = AsyncTransport(ClientConfig(base_url="https://example.invalid", max_retries=0))
    await transport._client.aclose()
    transport._client = httpx.AsyncClient(
        base_url="https://example.invalid", transport=httpx.MockTransport(handler)
    )
    resource = AsyncEntitiesResource(transport)
    args = (name, EntityUpdate(description="updated")) if operation == "update" else (name,)
    try:
        with pytest.raises(NotFoundError):
            await getattr(resource, operation)(*args)
    finally:
        await transport.close()
    assert requests[0].url.raw_path == f"/api/v1/graph/entities/{encoded}{suffix}".encode()
