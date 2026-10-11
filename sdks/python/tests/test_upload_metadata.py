"""Document titles must not leak into caller-owned upload metadata."""

from io import BytesIO
from unittest.mock import AsyncMock, MagicMock

import httpx
import pytest

from edgequake.resources.documents import AsyncDocumentsResource, DocumentsResource


@pytest.mark.parametrize("metadata", [{"source": "report"}, {"title": "original"}, {}])
def test_upload_does_not_mutate_reused_metadata(metadata: dict[str, str]) -> None:
    original = metadata.copy()
    transport = MagicMock()
    transport.upload.return_value = httpx.Response(200, json={"document_id": "doc-1"})
    documents = DocumentsResource(transport)
    documents.upload_file(BytesIO(b"first"), title="First", metadata=metadata)
    assert metadata == original
    assert transport.upload.call_args.kwargs["metadata"] == {**original, "title": "First"}
    documents.upload_file(BytesIO(b"second"), metadata=metadata)
    assert transport.upload.call_args.kwargs["metadata"] == original


@pytest.mark.asyncio
@pytest.mark.parametrize("metadata", [{"source": "report"}, {"title": "original"}, {}])
async def test_async_upload_does_not_mutate_reused_metadata(metadata: dict[str, str]) -> None:
    original = metadata.copy()
    transport = MagicMock()
    transport.upload = AsyncMock(return_value=httpx.Response(200, json={"document_id": "doc-1"}))
    documents = AsyncDocumentsResource(transport)
    await documents.upload_file(BytesIO(b"first"), title="First", metadata=metadata)
    assert metadata == original
    assert transport.upload.call_args.kwargs["metadata"] == {**original, "title": "First"}
    await documents.upload_file(BytesIO(b"second"), metadata=metadata)
    assert transport.upload.call_args.kwargs["metadata"] == original
