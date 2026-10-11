"""Retry-After values must not mask HTTP errors or break retry sleeps."""

import httpx
import pytest

from edgequake._errors import RateLimitedError, raise_for_status
from edgequake._transport import _get_retry_delay


@pytest.mark.parametrize("value", ["garbage", "-1", "NaN", "Infinity", "-Infinity"])
def test_invalid_retry_after_preserves_typed_error(value: str) -> None:
    response = httpx.Response(429, headers={"retry-after": value}, json={"message": "limited"})
    with pytest.raises(RateLimitedError) as error:
        raise_for_status(response)
    assert error.value.retry_after is None
    assert _get_retry_delay(response, 0) == 0.5


@pytest.mark.parametrize("value,expected", [("0", 0.0), ("5", 5.0), ("2.5", 2.5)])
def test_finite_retry_after_still_applies(value: str, expected: float) -> None:
    response = httpx.Response(429, headers={"retry-after": value}, json={"message": "limited"})
    with pytest.raises(RateLimitedError) as error:
        raise_for_status(response)
    assert error.value.retry_after == expected
    assert _get_retry_delay(response, 0) == expected


@pytest.mark.parametrize("value", ["garbage", "-1", "NaN", "Infinity", "-Infinity"])
def test_transport_recovers_after_invalid_retry_header(
    value: str, monkeypatch: pytest.MonkeyPatch
) -> None:
    from edgequake._config import ClientConfig
    from edgequake._transport import SyncTransport

    calls = []
    sleeps = []

    def handler(request: httpx.Request) -> httpx.Response:
        calls.append(request)
        if len(calls) == 1:
            return httpx.Response(429, headers={"retry-after": value}, json={"message": "limited"})
        return httpx.Response(200, json={"ok": True})

    transport = SyncTransport(ClientConfig(base_url="https://example.invalid", max_retries=1))
    transport._client.close()
    transport._client = httpx.Client(
        base_url="https://example.invalid", transport=httpx.MockTransport(handler)
    )
    monkeypatch.setattr("edgequake._transport.time.sleep", sleeps.append)
    try:
        assert transport.request("GET", "/health").json() == {"ok": True}
    finally:
        transport.close()
    assert len(calls) == 2
    assert sleeps == [0.5]
