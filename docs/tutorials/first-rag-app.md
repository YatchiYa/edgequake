---
title: "Tutorial: Building Your First RAG App"
---

<<<<<<< HEAD
> **Product: v0.19.0** · Contract: [`openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
=======
> **Product: v0.23.0** · Contract: [`openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

# Tutorial: Building Your First RAG App

> **End-to-End Guide: From Documents to Intelligent Q&A**

In this tutorial, you'll build a complete RAG application that can answer questions about your documents using EdgeQuake's graph-enhanced retrieval.

**Time**: ~30 minutes  
**Level**: Beginner  
**Prerequisites**: EdgeQuake running ([Quick Start](/docs/getting-started/quick-start/))

`make dev` sets `EDGEQUAKE_DEV_MODE=true` (open API). Production deployments require auth — see [Authentication headers](/docs/getting-started/quick-start/#authentication-headers).

---

## What You'll Build

```
┌─────────────────────────────────────────────────────────────────┐
│                   YOUR RAG APPLICATION                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │
│  │   Upload    │───▶│   Index     │───▶│   Query     │          │
│  │  Documents  │    │  & Extract  │    │  & Answer   │          │
│  └─────────────┘    └─────────────┘    └─────────────┘          │
│                                                                 │
│  Features:                                                      │
│  • Multi-document ingestion                                     │
│  • Knowledge graph extraction                                   │
│  • Multiple query modes                                         │
│  • Source citations                                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Step 1: Start EdgeQuake

First, make sure EdgeQuake is running:

```bash
# Option A: With Ollama (free, local)
make dev

# Option B: With OpenAI (requires API key)
export OPENAI_API_KEY="sk-your-key"
make dev
```

Verify it's running:

```bash
curl http://localhost:8080/health
```

Expected response:

```json
<<<<<<< HEAD
{ "status": "healthy", "version": "0.19.0", "storage_mode": "postgresql" }
=======
{ "status": "healthy", "version": "0.23.0", "storage_mode": "postgresql" }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
```

---

## Step 2: Create a Workspace

Workspaces organize your documents and provide isolation:

```bash
curl -X POST http://localhost:8080/api/v1/tenants/default/workspaces \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My First RAG App",
    "description": "Tutorial workspace for learning EdgeQuake"
  }'
```

**Response:**

```json
{
  "id": "ws_abc123",
  "name": "My First RAG App",
  "description": "Tutorial workspace for learning EdgeQuake",
  "created_at": "2024-01-15T10:00:00Z"
}
```

Save the workspace ID for later:

```bash
export WORKSPACE_ID="ws_abc123"
```

---

## Step 3: Prepare Sample Documents

Let's create some sample documents about a fictional company:

**doc1.txt** - Company Overview

```text
TechCorp Innovation Labs was founded in 2020 by Sarah Chen and Marcus Williams.
The company is headquartered in San Francisco, with research offices in Boston and Seattle.

Sarah Chen serves as CEO and leads the company's AI research initiatives.
She previously worked at Google DeepMind where she led the language model team.

Marcus Williams is the CTO and oversees all engineering operations.
He has a PhD in Computer Science from MIT and previously founded two startups.

TechCorp's flagship product is NeuralSearch, an enterprise search platform
that uses advanced AI to help companies find information in their documents.
```

**doc2.txt** - Recent News

```text
TechCorp Announces $50M Series B Funding

SAN FRANCISCO, January 2024 - TechCorp Innovation Labs announced today that
it has raised $50 million in Series B funding led by Venture Partners Capital.

"This funding will accelerate our mission to make enterprise knowledge
accessible to everyone," said Sarah Chen, CEO of TechCorp.

The company plans to use the funds to expand its engineering team and
open a new research office in London. NeuralSearch now serves over 200
enterprise customers including Fortune 500 companies.

Existing investors including Startup Capital and AI Ventures also
participated in the round.
```

**doc3.txt** - Product Features

```text
NeuralSearch Features and Capabilities

NeuralSearch is TechCorp's enterprise search platform that combines
traditional keyword search with AI-powered semantic understanding.

Key Features:
- Semantic Search: Understands the meaning behind queries, not just keywords
- Knowledge Graph: Automatically extracts entities and relationships from documents
- Multi-modal: Supports text, PDFs, images, and spreadsheets
- Enterprise Security: SOC 2 Type II certified with role-based access control
- Integrations: Works with Slack, Microsoft Teams, Google Workspace, and Salesforce

NeuralSearch was developed by Marcus Williams and his engineering team of 50+
engineers. The platform processes over 1 billion queries per month across
all customer deployments.
```

---

## Step 4: Upload Documents

Upload each document to your workspace (async — returns **202** with `track_id`):

```bash
# Upload doc1.txt (JSON text ingest)
curl -X POST "http://localhost:8080/api/v1/documents" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "content": "'"$(cat doc1.txt)"'",
    "title": "Company Overview"
  }'

# Or multipart file upload
curl -X POST "http://localhost:8080/api/v1/documents/upload" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -F "file=@doc2.txt" \
  -F "title=Series B Announcement"

curl -X POST "http://localhost:8080/api/v1/documents/upload" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -F "file=@doc3.txt" \
  -F "title=Product Features"
```

Each upload returns a document ID and triggers background processing:

```json
{
  "document_id": "doc_xyz789",
  "track_id": "f6fa9cad-bbff-4892-a855-3bd7d70da044",
  "title": "Company Overview",
  "status": "processing"
}
```

---

## Step 5: Monitor Processing

Check document processing status (prefer `display_status` over raw `status`):

```bash
curl "http://localhost:8080/api/v1/documents" \
  -H "X-Workspace-ID: $WORKSPACE_ID"
```

**Response:**

```json
{
  "documents": [
    {
      "id": "doc_xyz789",
      "title": "Company Overview",
      "display_status": "completed",
      "ui_phase": "terminal",
      "current_stage": "completed",
      "track_id": "f6fa9cad-bbff-4892-a855-3bd7d70da044",
      "chunk_count": 3,
      "entity_count": 8,
      "created_at": "2024-01-15T10:05:00Z"
    }
  ]
}
```

Wait until all documents show `display_status: "completed"`.

---

## Step 6: Explore the Knowledge Graph

See what entities were extracted:

```bash
curl "http://localhost:8080/api/v1/graph/entities?workspace_id=$WORKSPACE_ID"
```

**Response:**

```json
{
  "entities": [
    {
      "name": "SARAH_CHEN",
      "entity_type": "PERSON",
      "description": "CEO of TechCorp Innovation Labs, previously led language model team at Google DeepMind",
      "source_count": 3
    },
    {
      "name": "MARCUS_WILLIAMS",
      "entity_type": "PERSON",
      "description": "CTO of TechCorp, PhD from MIT, founded two startups",
      "source_count": 2
    },
    {
      "name": "TECHCORP_INNOVATION_LABS",
      "entity_type": "ORGANIZATION",
      "description": "AI company founded in 2020, headquartered in San Francisco",
      "source_count": 3
    },
    {
      "name": "NEURALSEARCH",
      "entity_type": "PRODUCT",
      "description": "Enterprise search platform with AI-powered semantic understanding",
      "source_count": 2
    }
  ]
}
```

See relationships between entities:

```bash
curl "http://localhost:8080/api/v1/graph/relationships?workspace_id=$WORKSPACE_ID"
```

**Response:**

```json
{
  "relationships": [
    {
      "source": "SARAH_CHEN",
      "target": "TECHCORP_INNOVATION_LABS",
      "relationship_type": "FOUNDED",
      "description": "Sarah Chen co-founded TechCorp Innovation Labs in 2020"
    },
    {
      "source": "SARAH_CHEN",
      "target": "TECHCORP_INNOVATION_LABS",
      "relationship_type": "LEADS",
      "description": "Sarah Chen serves as CEO"
    },
    {
      "source": "MARCUS_WILLIAMS",
      "target": "NEURALSEARCH",
      "relationship_type": "DEVELOPED",
      "description": "Marcus Williams and his engineering team developed NeuralSearch"
    }
  ]
}
```

---

## Step 7: Query Your Documents

Now the fun part! Ask questions about your documents:

### Simple Question

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "Who founded TechCorp?",
    "mode": "hybrid"
  }'
```

**Response** (`QueryResponse` — `answer` + `sources`, no top-level `chunks`):

```json
{
  "answer": "TechCorp Innovation Labs was founded in 2020 by Sarah Chen and Marcus Williams. Sarah Chen serves as CEO and leads the company's AI research initiatives, while Marcus Williams is the CTO overseeing all engineering operations.",
  "sources": [
    {
      "document_id": "doc_xyz789",
      "snippet": "TechCorp Innovation Labs was founded in 2020 by Sarah Chen and Marcus Williams...",
      "score": 0.92,
      "file_path": "Company Overview.txt"
    }
  ],
  "mode": "hybrid",
  "stats": {
    "total_time_ms": 2400
  }
}
```

### Relationship Question

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "What is the relationship between Sarah Chen and Google?",
    "mode": "local"
  }'
```

**Response:**

```json
{
  "answer": "Sarah Chen previously worked at Google DeepMind where she led the language model team before co-founding TechCorp Innovation Labs and becoming its CEO.",
  "sources": [...],
  "entities_used": ["SARAH_CHEN", "GOOGLE_DEEPMIND"]
}
```

### Overview Question

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "What are the main themes across these documents?",
    "mode": "global"
  }'
```

**Response:**

```json
{
  "answer": "The main themes across these documents are:\n\n1. **Company Leadership**: The documents describe TechCorp's founding team - Sarah Chen (CEO) and Marcus Williams (CTO) - their backgrounds and roles.\n\n2. **Product Innovation**: NeuralSearch is the company's flagship product, an AI-powered enterprise search platform.\n\n3. **Growth and Funding**: TechCorp recently raised $50M in Series B funding and is expanding internationally.\n\n4. **AI and Enterprise**: The company focuses on making enterprise knowledge accessible through AI technology.",
  "sources": [...],
  "communities_used": 2
}
```

---

## Step 8: Compare Query Modes

Try the same question with different modes:

```bash
# Naive mode (vector search only)
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{"query": "Tell me about NeuralSearch", "mode": "naive"}'

# Local mode (entity-focused)
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{"query": "Tell me about NeuralSearch", "mode": "local"}'

# Global mode (community summaries)
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{"query": "Tell me about NeuralSearch", "mode": "global"}'

# Hybrid mode (combined - default)
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{"query": "Tell me about NeuralSearch", "mode": "hybrid"}'
```

Notice how each mode provides slightly different perspectives based on its retrieval strategy.

---

## Step 9: Use the Web UI

Open the EdgeQuake Web UI for a visual experience:

1. Open <http://localhost:3000> in your browser when using local `make dev`
2. Select your workspace "My First RAG App"
3. Navigate to the **Documents** tab to see your uploads
4. Navigate to the **Graph** tab to visualize the knowledge graph
5. Navigate to the **Query** tab to ask questions interactively

```
┌─────────────────────────────────────────────────────────────────┐
│                   EDGEQUAKE WEB UI                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐             │
│  │Documents│  │  Graph  │  │  Query  │  │Settings │             │
│  └────┬────┘  └────┬────┘  └────┬────┘  └─────────┘             │
│       │            │            │                               │
│       ▼            ▼            ▼                               │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                          │
│  │ Upload  │  │ Visual  │  │ Chat    │                          │
│  │ List    │  │ Graph   │  │ Interface│                         │
│  │ Status  │  │ Explorer│  │ Modes   │                          │
│  └─────────┘  └─────────┘  └─────────┘                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Step 10: Clean Up (Optional)

Delete the workspace and all its data:

```bash
curl -X DELETE "http://localhost:8080/api/v1/workspaces/$WORKSPACE_ID"
```

---

## What You Learned

✅ Created a workspace for document organization  
✅ Uploaded multiple documents for processing  
✅ Monitored document indexing status  
✅ Explored the extracted knowledge graph  
✅ Queried with different modes (naive, local, global, hybrid)  
✅ Compared retrieval strategies  
✅ Used both API and Web UI

---

## Next Steps

| Tutorial                                                            | Description                    |
| ------------------------------------------------------------------- | ------------------------------ |
| [Document Ingestion Deep-Dive](/docs/tutorials/document-ingestion/) | Custom chunking and processing |
| [Query Optimization](/docs/tutorials/query-optimization/)           | Choosing the right mode        |
| [Multi-Tenant Setup](/docs/tutorials/multi-tenant/)                 | Building a SaaS app            |
| [Custom Entity Types](/docs/concepts/entity-extraction/)            | Domain-specific extraction     |

---

## Troubleshooting

### Documents stuck in "processing"

```bash
# Check worker status
curl http://localhost:8080/api/v1/tasks?status=pending

# View backend logs
docker compose logs -f edgequake
```

### Empty responses

1. Verify documents completed processing
2. Check workspace_id is correct
3. Try `naive` mode to verify basic retrieval works

### LLM errors

1. Check API key: `echo $OPENAI_API_KEY`
2. Verify Ollama is running: `curl http://localhost:11434/api/tags`
3. Check logs for rate limit errors

---

## Complete Code Example

Here's a Python script that does everything above:

```python
import requests
import time

BASE_URL = "http://localhost:8080/api/v1"

# Step 1: Create workspace
resp = requests.post(f"{BASE_URL}/tenants/default/workspaces", json={
    "name": "Python Tutorial",
    "description": "Created from Python script"
})
workspace = resp.json()
workspace_id = workspace["id"]
headers = {"X-Workspace-ID": workspace_id}
print(f"Created workspace: {workspace_id}")

# Step 2: Upload documents
documents = [
    ("Company Overview", "doc1.txt"),
    ("Series B Announcement", "doc2.txt"),
    ("Product Features", "doc3.txt"),
]

for title, filename in documents:
    with open(filename, "rb") as f:
        resp = requests.post(
            f"{BASE_URL}/documents/upload",
            files={"file": f},
            data={"title": title},
            headers=headers,
        )
        print(f"Uploaded: {title} -> {resp.json().get('document_id') or resp.json().get('id')}")

# Step 3: Wait for processing
print("Waiting for processing...")
while True:
    resp = requests.get(f"{BASE_URL}/documents", headers=headers)
    docs = resp.json()["documents"]
    if all(d.get("display_status") == "completed" for d in docs):
        break
    time.sleep(2)
print("All documents processed!")

# Step 4: Query
questions = [
    "Who founded TechCorp?",
    "What is NeuralSearch?",
    "How much funding did they raise?",
]

for question in questions:
    resp = requests.post(
        f"{BASE_URL}/query",
        json={"query": question, "mode": "hybrid"},
        headers=headers,
    )
    answer = resp.json()["answer"]
    sources = len(resp.json().get("sources", []))
    print(f"\nQ: {question}")
    print(f"A: {answer[:200]}... ({sources} sources)")
```

---

## See Also

- [Quick Start](/docs/getting-started/quick-start/) - Minimal setup guide
- [Query Modes](/docs/deep-dives/query-modes/) - Understanding retrieval strategies
- [REST API](/docs/api-reference/rest-api/) - Complete API reference
- [Architecture](/docs/architecture/overview/) - System design
