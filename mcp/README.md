# EdgeQuake MCP Server

> **Model Context Protocol (MCP) Server for EdgeQuake**  
> Use EdgeQuake as persistent agent memory for AI agents and autonomous systems

This package provides a Model Context Protocol (MCP) server that integrates EdgeQuake's graph-based retrieval and generation capabilities with AI agents, enabling them to maintain structured, contextual memory across conversations.

## What is MCP?

The [Model Context Protocol](https://modelcontextprotocol.io) is an open standard that allows AI models to safely access data and tools in external systems. With this MCP server, AI agents can:

- **Store memories** as a knowledge graph in EdgeQuake
- **Query memories** using sophisticated graph traversal
- **Reason over relationships** between concepts, entities, and events
- **Maintain context** across multiple conversations with full traceability

## Features

### 🧠 Persistent Agent Memory

- **Structured Storage**: Entities, relationships, and communities are stored in EdgeQuake's knowledge graph
- **Multi-Hop Reasoning**: Query engine traverses graph relationships for complex reasoning
- **Entity Deduplication**: Automatic normalization prevents memory fragmentation
- **Typology Support**: 7 entity types (Person, Organization, Location, Concept, Event, Technology, Product)

### 🔗 MCP Resources

- **Memory Documents**: Store and retrieve memories as indexed documents
- **Entity Registry**: Access all extracted entities with metadata
- **Relationship Map**: Query relationships between concepts
- **Query History**: Track all previous queries and responses

### 🛠️ MCP Tools

- **store_memory**: Add new information to agent memory
- **query_memory**: Retrieve relevant information using hybrid graph/vector search
- **get_entity_details**: Retrieve information about specific entities
- **list_communities**: Discover semantic communities in memory
- **clear_memory**: Reset agent memory (for privacy/workspace isolation)

### 📊 Multiple Query Modes

The MCP server supports all 6 EdgeQuake query modes:

- **Naive**: Fast vector-only search
- **Local**: Entity-centric with neighborhood exploration
- **Global**: Community-based semantic search
- **Hybrid**: Combined local + global (default)
- **Mix**: Custom weighted combination
- **Bypass**: Direct LLM without graph results

## Installation

### Prerequisites

- **Node.js** 18.0.0 or later
- **EdgeQuake Backend**: Running on `http://localhost:8080` (or configure URL)
- **MCP Compatible Client**: Claude for Desktop, VS Code CopilotKit, or custom MCP client

### Via npm

```bash
npm install @edgequake/mcp-server
```

### From Source

```bash
git clone https://github.com/raphaelmansuy/edgequake.git
cd edgequake/mcp
npm install
npm run build
```

## Usage

### As an MCP Server

Run the server to make it available to MCP clients:

```bash
edgequake-mcp
```

The server starts on stdio and communicates via JSON-RPC 2.0. Configure your MCP client to connect to this server.

### Configuration

The server reads from environment variables:

```bash
# EdgeQuake backend URL
EDGEQUAKE_BASE_URL=http://localhost:8080

# Default Tenant
EDGEQUAKE_DEFAULT_TENANT=default

# Default Workspace
EDGEQUAKE_DEFAULT_WORKSPACE=default

# Optional: LLM model for entity extraction
EDGEQUAKE_MODEL=gpt-5-nano

# Optional: Enable debug logging
DEBUG=edgequake:*
```

### With Claude for Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "edgequake": {
      "command": "npx",
      "args": ["-y", "@edgequake/mcp-server"]
    }
  }
}
```

### With VS Code (GitHub Copilot Chat)

GitHub Copilot Chat supports MCP servers via your VS Code `settings.json`. Add the following configuration to enable EdgeQuake integration:

```json
{
  "github.copilot.chat.mcpServers": {
    "edgequake": {
      "command": "npx",
      "args": ["-y", "@edgequake/mcp-server"],
      "env": {
        "EDGEQUAKE_BASE_URL": "http://localhost:8080",
        "EDGEQUAKE_DEFAULT_TENANT": "default",
        "EDGEQUAKE_DEFAULT_WORKSPACE": "default"
      }
    }
  }
}
```

### With Cursor

Cursor supports MCP in its internal settings. Navigate to **Cursor Settings** > **Features** > **MCP Servers** and add:

- **Name**: `edgequake`
- **Type**: `command`
- **Command**: `npx -y @edgequake/mcp-server`

Then Claude can use commands like:

- "Remember this fact in my knowledge graph"
- "What do I know about X and how does it relate to Y?"
- "Show me all communities of related concepts"

### With CopilotKit

```typescript
import { CopilotKit } from "@copilotkit/react-core";
import { MCPProvider } from "@copilotkit/react-mcp";

export default function App() {
  return (
    <CopilotKit>
      <MCPProvider
        serverUrl="ws://localhost:3000/mcp"
        name="edgequake"
      >
        {/* Your app */}
      </MCPProvider>
    </CopilotKit>
  );
}
```

## Architecture

### How It Works

1. **Agent Issues Command**: AI agent sends request (e.g., "store this memory")
2. **MCP Handler**: Server receives JSON-RPC request
3. **EdgeQuake Integration**: Routes to EdgeQuake API for processing
4. **Graph Processing**: EdgeQuake extracts entities, relationships, communities
5. **Response**: Returns structured data to agent with next steps

### Storage Backend

The MCP server connects to EdgeQuake's backend, which supports:

- **PostgreSQL + pgvector**: Vector storage and similarity search
- **Apache AGE**: Property graph storage for relationships
- **LLM Integration**: OpenAI, Ollama, or custom providers

## Example Usage

### Storing Agent Memory

**Agent (Claude)**:

```
Remember: Sarah Chen founded TechCorp in 2020. It's a machine learning startup.
```

**MCP Server**:

1. Extracts entities: {Sarah Chen (Person), TechCorp (Organization), ML (Technology)}
2. Extracts relationships: {Sarah Chen --founded--> TechCorp, TechCorp --uses--> ML}
3. Stores in EdgeQuake knowledge graph

### Querying Agent Memory

**Agent**:

```
Who are the founders of companies in my memory that work on machine learning?
```

**MCP Server**:

1. Queries for entities matching query intent
2. Traverses relationships: Person --founded--> Organization --uses--> Technology
3. Returns relevant results with confidence scores

## Development

### Build

```bash
npm run build
```

### Watch Mode

```bash
npm run dev
```

### Test

```bash
npm test                 # Unit tests
npm run test:e2e        # Integration tests with live EdgeQuake instance
```

### Lint

```bash
npm run lint
```

## API Reference

### MCP Tools

#### `store_memory`

Store a fact or observation in agent memory.

**Parameters:**

- `content` (string): The fact to remember
- `metadata` (object, optional): Additional metadata

**Returns:**

- `document_id` (string): ID of stored memory
- `entities_extracted` (number): Count of entities found
- `relationships_extracted` (number): Count of relationships found

#### `query_memory`

Retrieve relevant information from agent memory.

**Parameters:**

- `query` (string): Question or search query
- `mode` (string, optional): "naive", "local", "global", "hybrid" (default)
- `limit` (number, optional): Max results to return (default: 10)

**Returns:**

- `results` (array): Relevant memories with relevance scores
- `total_results` (number): Total matches found

#### `get_entity_details`

Get detailed information about a specific entity.

**Parameters:**

- `entity_id` (string): ID of the entity
- `include_relationships` (boolean, optional): Include connected entities (default: true)

**Returns:**

- `entity` (object): Entity with all metadata
- `relationships` (array, optional): Connected entities and relationship types

## Troubleshooting

### "Connection refused" to EdgeQuake

**Problem**: MCP server can't connect to EdgeQuake backend

**Solution**:

```bash
# Check EdgeQuake is running
curl http://localhost:8080/health

# Set correct URL if running elsewhere
export EDGEQUAKE_API_URL=http://your-server:8080
edgequake-mcp
```

### "Not authorized" errors

**Problem**: MCP client not configured with proper permissions

**Solution**:

- Ensure your MCP client is listed in EdgeQuake's allowed clients
- Check `EDGEQUAKE_WORKSPACE_ID` matches your workspace
- Verify API key if using authentication

### Slow memory queries

**Problem**: Queries taking >1000ms

**Solution**:

- Use "naive" mode for simple queries (`mode: "naive"`)
- Ensure EdgeQuake has built indices (run after document upload)
- Check database connection and network latency

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) in the root repository.

## License

Apache License 2.0 - See [LICENSE](../LICENSE)

## Links

- **EdgeQuake Docs**: https://github.com/raphaelmansuy/edgequake/tree/edgequake-main/docs
- **MCP Spec**: https://spec.modelcontextprotocol.io
- **Report Issues**: https://github.com/raphaelmansuy/edgequake/issues

## Support

- **Questions?** Open an issue on [GitHub](https://github.com/raphaelmansuy/edgequake/)
- **Need Help?** Check the [FAQ](../docs/faq.md)
- **Want to Join?** See [Contributing Guide](../CONTRIBUTING.md)
