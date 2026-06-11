# EdgeQuake API Documentation

**Version:** 0.7.0  
**Base URL:** `http://localhost:8080/api/v1`  
**Swagger UI:** `http://localhost:8080/swagger-ui`  
**OpenAPI JSON:** `http://localhost:8080/api-docs/openapi.json`

## Table of Contents

1. [Authentication](#authentication)
2. [Health & Monitoring](#health--monitoring)
3. [Documents](#documents)
4. [Query (RAG)](#query-rag)
5. [Knowledge Graph](#knowledge-graph)
6. [Entities](#entities)
7. [Relationships](#relationships)
8. [Chat](#chat)
9. [Conversations](#conversations)
10. [Messages](#messages)
11. [Folders](#folders)
12. [Models](#models)
13. [Pipeline](#pipeline)
14. [Tasks](#tasks)
15. [Costs](#costs)
16. [Tenants](#tenants)
17. [Workspaces](#workspaces)
18. [Lineage & Provenance](#lineage--provenance)
19. [PDF Processing](#pdf-processing)

---

## Authentication

EdgeQuake supports two authentication methods:

### Bearer Token (JWT)
```http
Authorization: Bearer <jwt_token>
```

### API Key
```http
X-API-Key: <your_api_key>
```

### Tenant & Workspace Headers
```http
X-Tenant-ID: <tenant_uuid>
X-Workspace-ID: <workspace_uuid>  # Optional
```

### Endpoints

#### POST /auth/login
Login with email and password to get JWT token.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Response:**
```json
{
  "token": "eyJ...",
  "refresh_token": "refresh_...",
  "user": {
    "user_id": "uuid",
    "email": "user@example.com",
    "role": "user"
  }
}
```

#### POST /auth/refresh
Refresh access token using refresh token.

**Request:**
```json
{
  "refresh_token": "refresh_..."
}
```

#### POST /auth/logout
Logout and invalidate tokens.

#### GET /auth/me
Get current user information.

---

## Health & Monitoring

#### GET /health
Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.7.0",
  "timestamp": "2026-04-21T00:00:00Z",
  "components": {
    "database": "healthy",
    "llm_provider": "healthy",
    "vector_storage": "healthy"
  },
  "build_info": {
    "git_commit": "abc123",
    "build_date": "2026-04-20",
    "rust_version": "1.95.0"
  }
}
```

#### GET /health/readiness
Readiness probe for Kubernetes.

#### GET /health/liveness
Liveness probe for Kubernetes.

#### GET /metrics
Prometheus metrics endpoint.

---

## Documents

#### POST /documents
Upload a new document for processing.

**Request:**
```json
{
  "content": "Document text content...",
  "title": "My Document",
  "metadata": {
    "author": "John Doe",
    "date": "2026-04-20"
  }
}
```

**Response:**
```json
{
  "document_id": "uuid",
  "task_id": "uuid",
  "status": "pending",
  "message": "Document queued for processing"
}
```

#### GET /documents
List all documents.

**Query Parameters:**
- `page`: Page number (default: 1)
- `limit`: Results per page (default: 20, max: 100)
- `sort`: Sort field (`created_at`, `title`, `status`)
- `order`: Sort order (`asc`, `desc`)
- `status`: Filter by status (`pending`, `processing`, `completed`, `failed`)

**Response:**
```json
{
  "documents": [
    {
      "document_id": "uuid",
      "title": "My Document",
      "status": "completed",
      "chunks_count": 15,
      "entities_count": 42,
      "relationships_count": 38,
      "created_at": "2026-04-20T10:00:00Z",
      "updated_at": "2026-04-20T10:05:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

---

## Query (RAG)

#### POST /query
Execute a RAG query with multi-mode retrieval.

**Request:**
```json
{
  "query": "What are the main features of EdgeQuake?",
  "mode": "hybrid",
  "conversation_id": "uuid",  // Optional - for conversation persistence
  "conversation_history": [   // Optional - if conversation_id not provided
    {
      "role": "user",
      "content": "Previous question"
    },
    {
      "role": "assistant",
      "content": "Previous answer"
    }
  ],
  "llm_provider": "openai",   // Optional
  "llm_model": "gpt-4o-mini", // Optional
  "context_only": false,
  "prompt_only": false,
  "enable_rerank": true,
  "rerank_top_k": 10,
  "max_results": 20,
  "system_prompt": "You are a helpful assistant.", // Optional
  "document_filter": {  // Optional
    "date_from": "2026-01-01T00:00:00Z",
    "date_to": "2026-12-31T23:59:59Z",
    "document_pattern": "report,summary"
  }
}
```

**Response:**
```json
{
  "answer": "EdgeQuake is a high-performance RAG system...",
  "mode": "hybrid",
  "conversation_id": "uuid",
  "reranked": true,
  "sources": [
    {
      "source_type": "chunk",
      "id": "chunk_id",
      "score": 0.95,
      "rerank_score": 0.98,
      "snippet": "Text snippet from the chunk...",
      "reference_id": 1,
      "document_id": "uuid",
      "file_path": "document_title.pdf",
      "start_line": 10,
      "end_line": 25,
      "chunk_index": 3
    },
    {
      "source_type": "entity",
      "id": "ENTITY_NAME",
      "score": 0.92,
      "snippet": "Entity description...",
      "reference_id": 2,
      "entity_type": "TECHNOLOGY",
      "degree": 15
    }
  ],
  "stats": {
    "embedding_time_ms": 45,
    "retrieval_time_ms": 120,
    "generation_time_ms": 850,
    "total_time_ms": 1015,
    "sources_retrieved": 25,
    "rerank_time_ms": 5,
    "tokens_used": 420,
    "tokens_per_second": 25.3,
    "llm_provider": "openai",
    "llm_model": "gpt-4o-mini"
  }
}
```

**Query Modes:**
- `naive`: Vector search only
- `local`: Entity-centric retrieval
- `global`: Community summaries
- `hybrid`: Local + global combined
- `mix`: Adaptive blend
- `bypass`: Direct LLM (no RAG)

#### POST /query/stream
Stream RAG query results with Server-Sent Events.

**Request:**
```json
{
  "query": "Explain EdgeQuake's architecture",
  "mode": "hybrid",
  "system_prompt": "Be concise.", // Optional
  "document_filter": {}, // Optional
  "llm_provider": "openai", // Optional
  "llm_model": "gpt-4o-mini" // Optional
}
```

**Response:** (Server-Sent Events)
```
event: context
data: {"chunks":15,"entities":42,"relationships":38}

event: chunk
data: {"text":"EdgeQuake "}

event: chunk
data: {"text":"is a "}

event: chunk
data: {"text":"high-performance RAG system"}

event: done
data: {"total_tokens":420,"duration_ms":850}
```

---

## Knowledge Graph

#### GET /graph
Get the knowledge graph (nodes and edges).

**Query Parameters:**
- `limit`: Maximum nodes to return (default: 100)
- `entity_types`: Comma-separated entity types to filter
- `min_degree`: Minimum node degree (connections)

**Response:**
```json
{
  "nodes": [
    {
      "id": "ENTITY_NAME",
      "label": "Entity Name",
      "entity_type": "TECHNOLOGY",
      "description": "Entity description...",
      "degree": 15,
      "community_id": 5
    }
  ],
  "edges": [
    {
      "source": "ENTITY_A",
      "target": "ENTITY_B",
      "relation_type": "USES",
      "weight": 0.95
    }
  ],
  "stats": {
    "total_nodes": 523,
    "total_edges": 1247,
    "returned_nodes": 100,
    "returned_edges": 234
  }
}
```

#### GET /graph/stream
Stream knowledge graph data.

#### GET /graph/nodes/{name}
Get a specific node by name.

#### GET /graph/labels/search
Search entity labels.

**Query Parameters:**
- `q`: Search query
- `limit`: Results limit (default: 10)

---

## Entities

#### GET /entities
List entities with filtering and pagination.

**Query Parameters:**
- `page`: Page number
- `limit`: Results per page
- `entity_type`: Filter by type
- `search`: Search in names/descriptions
- `min_degree`: Minimum connections

**Response:**
```json
{
  "entities": [
    {
      "name": "ENTITY_NAME",
      "entity_type": "TECHNOLOGY",
      "description": "Description...",
      "degree": 15,
      "community_id": 5,
      "source_documents": ["doc_id1", "doc_id2"],
      "created_at": "2026-04-20T10:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 523
  }
}
```

#### POST /entities
Create a new entity manually.

**Request:**
```json
{
  "name": "CUSTOM_ENTITY",
  "entity_type": "CONCEPT",
  "description": "A manually created entity",
  "metadata": {
    "custom_field": "value"
  }
}
```

#### GET /entities/{name}
Get entity details with relationships.

#### PATCH /entities/{name}
Update entity description or metadata.

**Request:**
```json
{
  "description": "Updated description",
  "metadata": {
    "updated_field": "new_value"
  }
}
```

#### DELETE /entities/{name}
Delete an entity (soft delete).

**Query Parameters:**
- `cascade`: Also delete relationships (default: false)

#### POST /entities/merge
Merge multiple entities into one.

**Request:**
```json
{
  "source_names": ["ENTITY_A", "ENTITY_B"],
  "target_name": "MERGED_ENTITY",
  "strategy": "keep_all" // or "prefer_first", "prefer_last"
}
```

#### GET /entities/{name}/neighborhood
Get entity neighborhood (connected entities).

**Query Parameters:**
- `depth`: Traversal depth (default: 1, max: 3)
- `limit`: Max nodes per level

---

## Relationships

#### GET /relationships
List relationships.

**Query Parameters:**
- `source`: Source entity name
- `target`: Target entity name
- `relation_type`: Relationship type
- `page`, `limit`: Pagination

#### POST /relationships
Create a relationship.

**Request:**
```json
{
  "source": "ENTITY_A",
  "target": "ENTITY_B",
  "relation_type": "USES",
  "weight": 0.8,
  "description": "Entity A uses Entity B",
  "metadata": {}
}
```

#### GET /relationships/{id}
Get relationship details.

#### PATCH /relationships/{id}
Update relationship.

#### DELETE /relationships/{id}
Delete relationship.

---

## Chat

#### POST /chat/completions
Chat completion endpoint (OpenAI-compatible).

**Request:**
```json
{
  "messages": [
    {
      "role": "user",
      "content": "Hello, how are you?"
    }
  ],
  "model": "gpt-4o-mini",
  "temperature": 0.7,
  "max_tokens": 1000,
  "stream": false
}
```

**Response:**
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-4o-mini",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "I'm doing well, thank you!"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

---

## Conversations

#### GET /conversations
List conversations for the authenticated user.

**Query Parameters:**
- `limit`: Results per page (default: 20, max: 100)
- `cursor`: Pagination cursor
- `sort`: Sort field (`updated_at`, `created_at`, `title`)
- `order`: Sort order (`asc`, `desc`)
- `filter_mode`: Filter by mode (comma-separated)
- `filter_archived`: Filter archived conversations
- `filter_pinned`: Filter pinned conversations
- `filter_folder_id`: Filter by folder
- `filter_search`: Search in titles

**Response:**
```json
{
  "items": [
    {
      "conversation_id": "uuid",
      "title": "Discussion about EdgeQuake",
      "mode": "hybrid",
      "created_at": "2026-04-20T10:00:00Z",
      "updated_at": "2026-04-20T11:00:00Z",
      "message_count": 15,
      "is_pinned": false,
      "is_archived": false,
      "folder_id": null
    }
  ],
  "pagination": {
    "next_cursor": "cursor_string",
    "prev_cursor": null,
    "total": 50,
    "has_more": true
  }
}
```

#### POST /conversations
Create a new conversation.

**Request:**
```json
{
  "title": "New Conversation",
  "mode": "hybrid",
  "system_prompt": "You are a helpful assistant.",
  "folder_id": "uuid" // Optional
}
```

**Response:**
```json
{
  "conversation_id": "uuid",
  "title": "New Conversation",
  "mode": "hybrid",
  "created_at": "2026-04-20T12:00:00Z",
  "updated_at": "2026-04-20T12:00:00Z",
  "message_count": 0
}
```

#### GET /conversations/{id}
Get conversation details.

#### PATCH /conversations/{id}
Update conversation (title, mode, system_prompt, etc.).

**Request:**
```json
{
  "title": "Updated Title",
  "is_pinned": true,
  "is_archived": false,
  "folder_id": "uuid"
}
```

#### DELETE /conversations/{id}
Delete a conversation.

#### POST /conversations/bulk-delete
Bulk delete conversations.

**Request:**
```json
{
  "conversation_ids": ["uuid1", "uuid2", "uuid3"]
}
```

---

## Messages

#### GET /conversations/{id}/messages
List messages in a conversation.

**Query Parameters:**
- `cursor`: Pagination cursor
- `limit`: Results per page (default: 50, max: 200)

**Response:**
```json
{
  "items": [
    {
      "message_id": "uuid",
      "conversation_id": "uuid",
      "role": "user",
      "content": "What is EdgeQuake?",
      "created_at": "2026-04-20T12:00:00Z",
      "tokens_used": 10,
      "parent_id": null
    },
    {
      "message_id": "uuid2",
      "conversation_id": "uuid",
      "role": "assistant",
      "content": "EdgeQuake is a high-performance RAG system...",
      "created_at": "2026-04-20T12:00:05Z",
      "tokens_used": 50,
      "duration_ms": 850,
      "parent_id": "uuid"
    }
  ],
  "pagination": {
    "next_cursor": null,
    "prev_cursor": null,
    "total": 2,
    "has_more": false
  }
}
```

#### POST /conversations/{id}/messages
Add a message to a conversation.

**Request:**
```json
{
  "role": "user",
  "content": "Tell me more about the architecture",
  "parent_id": "uuid" // Optional - for threaded conversations
}
```

**Response:**
```json
{
  "message_id": "uuid",
  "conversation_id": "uuid",
  "role": "user",
  "content": "Tell me more about the architecture",
  "created_at": "2026-04-20T12:01:00Z"
}
```

#### PATCH /messages/{message_id}
Update a message.

**Request:**
```json
{
  "content": "Updated message content",
  "tokens_used": 15,
  "duration_ms": 900
}
```

#### DELETE /messages/{message_id}
Delete a message.

---

## Folders

#### GET /folders
List folders.

#### POST /folders
Create a folder.

**Request:**
```json
{
  "name": "Work Conversations",
  "parent_id": null // Optional - for nested folders
}
```

#### PATCH /folders/{id}
Update folder.

#### DELETE /folders/{id}
Delete folder.

---

## Models

#### GET /models
List all available models (LLM and embedding).

**Response:**
```json
{
  "models": [
    {
      "model_id": "gpt-4o-mini",
      "provider": "openai",
      "type": "llm",
      "context_window": 128000,
      "max_output_tokens": 16384,
      "supports_vision": false,
      "supports_function_calling": true,
      "capabilities": {
        "streaming": true,
        "embeddings": false
      },
      "cost": {
        "input_per_1k_tokens": 0.00015,
        "output_per_1k_tokens": 0.0006
      }
    },
    {
      "model_id": "text-embedding-3-small",
      "provider": "openai",
      "type": "embedding",
      "dimension": 1536,
      "cost": {
        "per_1k_tokens": 0.00002
      }
    }
  ]
}
```

#### GET /models/llm
List LLM models only.

#### GET /models/embedding
List embedding models only.

#### GET /providers/{provider_id}
Get provider details and status.

**Response:**
```json
{
  "provider_id": "openai",
  "name": "OpenAI",
  "description": "Official OpenAI API",
  "available": true,
  "config_satisfied": true,
  "default_models": {
    "chat_model": "gpt-4o-mini",
    "embedding_model": "text-embedding-3-small",
    "embedding_dimension": 1536
  }
}
```

#### GET /models/{provider}/{model}
Get specific model details.

#### GET /providers/health
Check health status of all providers.

---

## Pipeline

#### GET /pipeline/status
Get document processing pipeline status.

**Response:**
```json
{
  "status": "running",
  "queue_size": 5,
  "processing_count": 2,
  "completed_count": 150,
  "failed_count": 3,
  "current_tasks": [
    {
      "task_id": "uuid",
      "document_id": "uuid",
      "stage": "entity_extraction",
      "progress": 45.5,
      "started_at": "2026-04-20T12:00:00Z"
    }
  ]
}
```

#### POST /pipeline/cancel
Cancel pipeline processing.

**Request:**
```json
{
  "document_id": "uuid" // Optional - cancel specific document
}
```

#### GET /pipeline/metrics
Get queue metrics.

**Response:**
```json
{
  "queue_length": 5,
  "avg_processing_time_ms": 15000,
  "throughput_per_hour": 48,
  "active_workers": 4,
  "pending_tasks": 5,
  "running_tasks": 2,
  "completed_tasks": 150,
  "failed_tasks": 3
}
```

---

## Tasks

#### GET /tasks
List background tasks.

**Query Parameters:**
- `status`: Filter by status (`pending`, `running`, `completed`, `failed`, `cancelled`)
- `task_type`: Filter by type
- `page`, `limit`: Pagination

**Response:**
```json
{
  "tasks": [
    {
      "task_id": "uuid",
      "task_type": "document_processing",
      "status": "running",
      "progress": 65.5,
      "created_at": "2026-04-20T12:00:00Z",
      "started_at": "2026-04-20T12:00:05Z",
      "metadata": {
        "document_id": "uuid",
        "stage": "relationship_extraction"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 158
  },
  "statistics": {
    "pending": 5,
    "running": 2,
    "completed": 150,
    "failed": 1
  }
}
```

#### GET /tasks/{id}
Get task details.

#### POST /tasks/{id}/cancel
Cancel a task.

#### POST /tasks/{id}/retry
Retry a failed task.

---

## Costs

#### GET /costs/summary
Get cost summary.

**Query Parameters:**
- `period`: Time period (`day`, `week`, `month`, `all`)
- `group_by`: Group by (`provider`, `model`, `operation`)

**Response:**
```json
{
  "total_cost": 15.45,
  "period": "month",
  "breakdown": [
    {
      "provider": "openai",
      "model": "gpt-4o-mini",
      "operation": "query",
      "cost": 8.50,
      "token_count": 142000,
      "request_count": 340
    },
    {
      "provider": "openai",
      "model": "text-embedding-3-small",
      "operation": "embedding",
      "cost": 6.95,
      "token_count": 3475000,
      "request_count": 2500
    }
  ]
}
```

#### GET /costs/pricing
Get model pricing information.

#### POST /costs/estimate
Estimate cost for an operation.

**Request:**
```json
{
  "operation": "query",
  "provider": "openai",
  "model": "gpt-4o-mini",
  "input_tokens": 1000,
  "output_tokens": 500
}
```

**Response:**
```json
{
  "estimated_cost": 0.45,
  "breakdown": {
    "input_cost": 0.15,
    "output_cost": 0.30
  }
}
```

---

## Tenants

#### POST /tenants
Create a new tenant.

**Request:**
```json
{
  "name": "Acme Corp",
  "plan": "enterprise",
  "settings": {
    "max_users": 50,
    "max_workspaces": 10
  }
}
```

#### GET /tenants
List tenants.

#### GET /tenants/{id}
Get tenant details.

#### PATCH /tenants/{id}
Update tenant.

#### DELETE /tenants/{id}
Delete tenant.

---

## Workspaces

#### POST /workspaces
Create a workspace.

**Request:**
```json
{
  "name": "Marketing Workspace",
  "description": "Workspace for marketing team",
  "llm_provider": "openai",
  "llm_model": "gpt-4o-mini",
  "embedding_provider": "openai",
  "embedding_model": "text-embedding-3-small",
  "embedding_dimension": 1536
}
```

#### GET /workspaces
List workspaces.

#### GET /workspaces/{id}
Get workspace details.

#### PATCH /workspaces/{id}
Update workspace.

**Request:**
```json
{
  "name": "Updated Name",
  "llm_model": "gpt-4o"
}
```

#### DELETE /workspaces/{id}
Delete workspace.

#### GET /workspaces/{id}/stats
Get workspace statistics.

**Response:**
```json
{
  "workspace_id": "uuid",
  "documents_count": 150,
  "chunks_count": 2500,
  "entities_count": 523,
  "relationships_count": 1247,
  "conversations_count": 45,
  "total_tokens_used": 1500000,
  "total_cost": 15.45
}
```

---

## Lineage & Provenance

#### GET /lineage/chunks/{chunk_id}
Get chunk details with lineage.

**Response:**
```json
{
  "chunk_id": "uuid",
  "content": "Chunk text content...",
  "document_id": "uuid",
  "document_title": "Document Title",
  "start_line": 10,
  "end_line": 25,
  "chunk_index": 3,
  "extracted_entities": ["ENTITY_A", "ENTITY_B"],
  "extracted_relationships": [
    {
      "source": "ENTITY_A",
      "target": "ENTITY_B",
      "relation_type": "USES"
    }
  ]
}
```

#### GET /lineage/entities/{entity_name}
Get entity provenance (source chunks).

**Response:**
```json
{
  "entity_name": "ENTITY_NAME",
  "entity_type": "TECHNOLOGY",
  "description": "Description...",
  "source_chunks": [
    {
      "chunk_id": "uuid",
      "document_id": "uuid",
      "document_title": "Document Title",
      "snippet": "Text mentioning the entity..."
    }
  ],
  "versions": [
    {
      "version": 1,
      "description": "First extraction",
      "extracted_at": "2026-04-20T10:00:00Z"
    },
    {
      "version": 2,
      "description": "Updated from new document",
      "extracted_at": "2026-04-20T11:00:00Z"
    }
  ]
}
```

#### GET /lineage/documents/{document_id}
Get full document lineage.

**Response:**
```json
{
  "document_id": "uuid",
  "title": "Document Title",
  "chunks": [
    {
      "chunk_id": "uuid",
      "chunk_index": 0,
      "entities": ["ENTITY_A", "ENTITY_B"],
      "relationships_count": 3
    }
  ],
  "extracted_entities": 15,
  "extracted_relationships": 12
}
```

---

## PDF Processing

#### POST /pdf
Upload a PDF document for processing with vision extraction.

**Request:** (multipart/form-data)
- `file`: PDF file
- `filename`: Filename (optional)
- `workspace_id`: Workspace UUID (optional)

**Response:**
```json
{
  "pdf_id": "uuid",
  "filename": "document.pdf",
  "status": "queued",
  "task_id": "uuid"
}
```

#### GET /pdf/{id}/status
Get PDF processing status.

**Response:**
```json
{
  "pdf_id": "uuid",
  "filename": "document.pdf",
  "status": "processing",
  "progress": {
    "current_page": 5,
    "total_pages": 10,
    "percentage": 50.0
  },
  "created_at": "2026-04-20T12:00:00Z",
  "started_at": "2026-04-20T12:00:05Z"
}
```

#### GET /pdf
List PDFs.

**Query Parameters:**
- `status`: Filter by status
- `page`, `limit`: Pagination

#### DELETE /pdf/{id}
Delete a PDF.

#### GET /pdf/{id}/progress
Get detailed progress with per-page status.

#### GET /pdf/{id}/content
Get extracted content (Markdown).

**Response:**
```json
{
  "pdf_id": "uuid",
  "markdown": "# Document Title\n\nExtracted content...",
  "pages": 10,
  "extracted_at": "2026-04-20T12:05:00Z"
}
```

---

## Common Response Codes

- `200 OK`: Successful request
- `201 Created`: Resource created successfully
- `204 No Content`: Successful request with no response body
- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Missing or invalid authentication
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource already exists or conflict
- `422 Unprocessable Entity`: Validation error
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: Service temporarily unavailable

---

## Rate Limits

Default rate limits:
- Authenticated users: 100 requests/minute
- Anonymous users: 20 requests/minute
- Document uploads: 10/minute
- Query requests: 30/minute

Rate limit headers:
- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Reset timestamp

---

## Pagination

List endpoints support cursor-based pagination:

**Request:**
```
GET /conversations?limit=20&cursor=base64_cursor_string
```

**Response:**
```json
{
  "items": [...],
  "pagination": {
    "next_cursor": "next_page_cursor",
    "prev_cursor": "prev_page_cursor",
    "total": 100,
    "has_more": true
  }
}
```

---

## Error Response Format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": {
      "field": "query",
      "issue": "Query cannot be empty"
    }
  }
}
```

---

## Supported LLM Providers

EdgeQuake supports multiple LLM providers via the `edgequake-llm` abstraction layer:

- **OpenAI**: GPT-4, GPT-3.5, text-embedding models
- **Anthropic**: Claude 3 Opus, Sonnet, Haiku
- **Google**: Gemini Pro, Gemini Flash (via Vertex AI)
- **Mistral AI**: Mistral Small, Large, Medium, Pixtral
- **xAI**: Grok models
- **OpenRouter**: Access to multiple providers
- **Ollama**: Local LLM inference
- **LM Studio**: Local LLM inference
- **Azure OpenAI**: Enterprise OpenAI deployment
- **Mock**: Testing provider (no API calls)

Configure via environment variables:
```
EDGEQUAKE_LLM_PROVIDER=mistral
MISTRAL_API_KEY=your_key_here
```

---

## Changelog

### v0.7.0 (2026-04-21)
- Added conversation persistence in query endpoint
- Query endpoint now accepts `conversation_id` parameter
- Automatic conversation history loading from database
- Messages saved after each query/response
- Fixed conversation context continuity issue
- Mistral AI models fully integrated and tested
- Comprehensive API documentation

### v0.6.2
- Added PDF vision extraction with OpenAI/Mistral
- Improved chunking and entity extraction
- Enhanced lineage tracking

---

## Notes

- All timestamps are in ISO 8601 format (UTC)
- UUIDs are in standard UUID format
- Binary data (PDFs) use multipart/form-data
- Streaming endpoints use Server-Sent Events (SSE)
- WebSocket support available for real-time updates

---

For interactive API exploration, visit the Swagger UI at `http://localhost:8080/swagger-ui` when the server is running.
