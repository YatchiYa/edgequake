# SPEC-049 Two-Pass LLM Judge Report

> Pass 1 (filter): `mistral-large-latest`  |  Pass 2 (specialize): `mistral-large-latest`
> Total crops: 63  |  Kept: **46** (73%)  |  Discarded: 17

## Summary

| Metric | Value |
|--------|-------|
| Proposed by L0/L1 geometry | 63 |
| Kept (real figures, Pass 1) | **46** (73%) |
| Discarded (noise, Pass 1)   | 17 |
| Pass-2 descriptions written | 46 |
| Errors | 0 |

## Figure Type Distribution (post-filter)

| Kind | Count |
|------|-------|
| ✗ text_block | 17 |
| ✓ architecture_diagram | 17 |
| ✓ bar_chart | 10 |
| ✓ line_chart | 6 |
| ✓ heatmap | 5 |
| ✓ diagram | 4 |
| ✓ scatter_plot | 2 |
| ✓ flowchart | 1 |
| ✓ radar_chart | 1 |

## Discarded Crops (noise eliminated by Pass 1)

| Doc | Asset | Kind | Confidence |
|-----|-------|------|------------|
| claude_code_2604.14228v1.pdf | `assets/p01-fig-01.png` | text_block | 0.99 |
| hierar_2607.02980v1.pdf | `assets/p05-fig-01.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p01-fig-01.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p16-fig-01.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p16-fig-02.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p16-fig-03.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p17-fig-01.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p17-fig-02.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p17-fig-03.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p18-fig-01.png` | text_block | 0.99 |
| ideas_2607.08758v1.pdf | `assets/p18-fig-02.png` | text_block | 0.99 |
| lighrad_2410.05779v3.pdf | `assets/p13-fig-01.png` | text_block | 0.99 |
| lighrad_2410.05779v3.pdf | `assets/p14-fig-01.png` | text_block | 0.99 |
| lighrad_2410.05779v3.pdf | `assets/p14-fig-02.png` | text_block | 0.99 |
| lighrad_2410.05779v3.pdf | `assets/p15-fig-01.png` | text_block | 0.99 |
| lighrad_2410.05779v3.pdf | `assets/p15-fig-02.png` | text_block | 0.99 |
| rem_2607.08716v1.pdf | `assets/p01-fig-01.png` | text_block | 0.99 |

## Per-Document Results

### claude_code_2604.14228v1.pdf

**8/9 real figures**

#### ✗ `assets/p01-fig-01.png` — text_block (p1)

*Discarded: text_block (confidence=0.99)*

#### ✓ `assets/p05-fig-01.png` — architecture_diagram (p5)

Here is the structured extraction from the provided architecture diagram:

---

### 1. **Top-Level Components**
- **User**
- **Interfaces**
- **Permission System**
- **Agent Loop**
- **Tools**
- **Execution Environment** (Files/Shell/Web/MCP)
- **State & Persistence**

---

### 2. **Data Flow**
| **Source**          | **Destination**      | **Data/Interaction**                     | **Direction**       |
|---------------------|----------------------|------------------------------------------|---------------------|
| User                | Interfaces           | Prompt, Command/Task                     | →                   |
| Interfaces          | Agent Loop           | Request                                  | →                   |
| Agent Loop          | Permission System    | Propose Action                           | →                   |
| Permission System   | Agent Loop           | Allow/Ask/Deny                           | →                   |
| Agent Loop          | Tools                | Tool Request                             | →                   |
| Tools               | Execution Environment| Tool Execution                           | →                   |
| Execution Environment| Tools               | Execution Result                         | →                   |
| Tools               | Agent Loop           | Tool Result                              | →                   |
| Agent Loop          | Interfaces           | Progress                                 | →                   |
| Interfaces          | User                 | Progress (implied feedback)              | →                   |
| Agent Loop          | State & Persistence  | Load/Persist                             | ↔                   |

---

### 3. **External Interfaces**
- **User**: Human input/output (e.g., CLI, GUI, or conversational interface).
- **Execution Environment**: External systems where actions are executed (e.g., Filesystem, Shell, Web APIs, MCP).
- **State & Persistence**: Database or storage system for maintaining agent state across sessions.

---

### 4. **Key Design Decisions**
1. **Modular Separation of Concerns**:
   - **Permission System** is decoupled from the **Agent Loop** to enforce security policies independently.
   - **Tools** and **Execution Environment** are separated to abstract tool usage from execution details.

2. **Agent-Centric Loop**:
   - The **Agent Loop** acts as the central orchestrator, handling requests, tool selection, and progress reporting.

3. **State Persistence**:
   - Explicit **State & Persistence** module to maintain context across interactions (e.g., session continuity).

4. **User Feedback Loop**:
   - Progress updates flow back to the **User** via **Interfaces**, ensuring transparency.

5. **Permission Workflow**:
   - Actions are proposed to the **Permission System** before execution, enforcing a "check-before-act" paradigm.

6. **Tool Abstraction**:
   - **Tools** serve as intermediaries between the **Agent Loop** and **Execution Environment**, enabling extensibility (e.g., adding new tools without modifying the agent).

7. **Execution Isolation**:
   - The

#### ✓ `assets/p07-fig-01.png` — flowchart (p7)

Here is the structured extraction from the provided flowchart:

---

### 1. **Start and End Conditions**
- **Start**: User provides a **User Prompt**.
- **End**: **Assistant Response** is generated and delivered to the user for reading and replying.

---

### 2. **Main Steps** (in order)
1. **Context Assembly** (gather settings, history, etc.).
2. **Iteration 1**:
   - Agent generates a **Tool Request**.
   - **Permission** check:
     - If **allowed**, the tool is **executed** (sync, subagent, or background) → **Result** is obtained.
     - If **denied**, proceed to next iteration or fallback.
3. **Iteration 2** (if needed):
   - Agent generates another **Tool Request** (due to compact context pressure or prior denial).
   - **Permission** check:
     - If **denied**, **Deny Feedback** is provided.
4. **Subsequent Iterations** (Iter 3 to N-1):
   - Repeat tool request and permission checks until either:
     - A tool is allowed and executed, or
     - No further tools are needed.
5. **Iteration N**:
   - Agent determines **No Tool Use** is required.
6. **Assistant Response** is generated and returned to the user.

---

### 3. **Decision Branches**
- **Tool Request → Permission**:
  - **Allowed** → Execute tool → Obtain result.
  - **Denied** → Proceed to next iteration or fallback.
- **Deny Feedback** → Triggers more iterations or refinement of requests.
- **No Tool Use** → Directly generate **Assistant Response**.

---

### 4. **Loops or Back-Edges**
- **Iterative Loop**: The process loops back from **Deny Feedback** or compact context pressure to generate new **Tool Requests** (Iter 2 to N).
- **Termination Condition**: Loop exits when either:
  - A tool is successfully executed, or
  - The agent decides **No Tool Use** is needed.

---

#### ✓ `assets/p08-fig-01.png` — architecture_diagram (p8)

Here is a structured extraction from the provided architecture diagram:

---

### 1. **Top-Level Components**
The diagram is organized into **five layers**, each containing key modules:

#### **Surface Layer**
- Interactive CLI
- IDE/Desktop/Browser
- Headless CLI
- Agent SDK
- UI/Render

#### **Core Layer**
- Agent Loop
- Compaction Pipeline

#### **Safety / Action Layer**
- **Permission System + Auto Classifier**
- **Hook Pipeline**
- **Built-in Tools**
- **MCP Tools**
- **Shell**
- **Subagent Spawning**
- **Extensibility (plugins & skills)**

#### **Backend Layer**
- Execution Backends
- External Resources

#### **State Layer**
- Context Assembly
- Runtime State
- Session Persistence
- CLAUDE.md + memory
- Sidechain Transcriptions

---

### 2. **Data Flow**
#### **Surface → Core Layer**
- **Interactive CLI / Headless CLI / Agent SDK → UI/Render**
  - Inputs: `submit/interrupt`, `headless request`, `programmatic query`
  - Outputs: `output progress`
- **UI/Render → Agent Loop**
  - Input: `tool request`
  - Output: `output progress`
- **Agent Loop → Compaction Pipeline**
  - Input: `pre-model shapers`
  - Output: `compacted context`

#### **Core → Safety / Action Layer**
- **Agent Loop → Permission System**
  - Input: `approval dialog` (for lifecycle events, modify decisions)
- **Permission System → Hook Pipeline**
  - Bidirectional: `Y/N` decisions, `lifecycle events`
- **Hook Pipeline → Built-in Tools / MCP Tools / Shell / Subagent Spawning**
  - Input: `tool request`
  - Output: `Y/N` (permission)
- **Built-in Tools / MCP Tools → Shell**
  - Input: `tool surface`, `shell commands`
- **Shell → Execution Backends**
  - Input: `sandboxed execution`
  - Output: `shell commands`
- **Subagent Spawning → External Resources**
  - Input: `subagent transcript`

#### **Safety / Action → Backend Layer**
- **Shell → Execution Backends**
  - Input: `sandbox commands`
  - Output: `local/cloud/remote` execution
- **Extensibility (plugins & skills) → Execution Backends**
  - Bidirectional: `plugins/skills` integration

#### **State Layer (Bidirectional Flows)**
- **Context Assembly ↔ Runtime State**
  - Input: `system prompt`
  - Output: `mutate state`
- **Runtime State ↔ Session Persistence**
  - Input: `transcript`
  - Output: `resume/fork`
- **Session Persistence ↔ CLAUDE.md + memory**

#### ✓ `assets/p13-fig-01.png` — architecture_diagram (p13)

```markdown
# Architecture Diagram Description

## 1. Top-level Components
- **Tools**
- **Policy Core**
  - Rules
  - Modes
  - Hooks
- **Execution Environment**
- **User/Auto Classifier**

## 2. Data Flow
- **Tool Use**: Flows from **Tools** to **Policy Core** (specifically into the **Modes** component).
- **Permission Decision**:
  - **Deny**: Flows from **Policy Core** to **Denied Result**.
  - **Allow**: Flows from **Policy Core** to **Execution Environment**.
  - **Ask (Allow/Deny)**: Flows bidirectionally between **Policy Core** and **User/Auto Classifier**.
- Output from **Execution Environment** is not explicitly shown but implied to proceed after permission.

## 3. External Interfaces
- **Tools**: External tools or systems interfacing with the architecture.
- **User/Auto Classifier**: Interface involving either a user decision or an automated classification system.
- **Denied Result**: Output interface indicating a denied operation.
- **Execution Environment**: Environment where allowed operations/tools are executed.

## 4. Key Design Decisions
- **Modular Policy Core**: The **Policy Core** is divided into subcomponents (**Rules**, **Modes**, **Hooks**) to manage different aspects of policy enforcement.
- **Permission Decision Workflow**: Clear separation of permission decisions (Deny, Allow, Ask) to ensure controlled access.
- **User/Auto Classifier Integration**: Incorporation of a feedback mechanism (either user or automated) for ambiguous or uncertain permission decisions.
- **Isolation of Execution Environment**: Ensures that only allowed operations are executed in a controlled environment.
```

#### ✓ `assets/p19-fig-01.png` — architecture_diagram (p19)

Here is a structured extraction from the provided architecture diagram:

---

### 1. **Top-level Components**
The diagram depicts the following named modules in the **context window** of the Claude model:

- **(1) System Layer** (startup)
  - System Prompt
  - Environment Info
  - Output Styles
  - Skill Description
  - MCP Tool Names

- **(2) Project Config** (startup / lazy)
  - CLAUDE.md Hierarchy (5 levels)
  - Path-scoped Rules (`./claude/rules/*`)
  - Hierarchy path: `Managed → OS → Project → .claude/ → local → directory-specific` (startup)

- **(3) Memory** (startup)
  - Auto Memory
  - Compact Summary (replaces long history)

- **(4) Conversation** (carry forward)
  - Conversation History
  - Subagent Summaries

- **(5) Runtime** (carry forward)
  - Read Files
  - Command Outputs
  - Tool Results

- **(6) On-Demand** (lazy)
  - Deferred Tool Definitions (full schemas loaded only when needed)

**Access Control Layers** (left side, ordered by mutability):
- Read-only
- Hot-reload
- Sys-write
- Append
- Model-trigger
- Lazy-load

---

### 2. **Data Flow**
#### **Directional Flow Between Components**
- **Context Window** (all modules 1–6) → **Claude Model**
  - The model reads the entire context window (dashed arrow labeled "Reads all").
- **Claude Model** → **Text Response** (output to user/tool interface).
- **Claude Model** → **Tool Calls** (triggers tools).
- **Tool Search** → **(6) On-Demand** (lazy loading of tool definitions).
- **Tool Results** → **(5) Runtime** (appended during execution).
- **(4) Conversation** accumulates per turn (carry forward).
- **(5) Runtime** data is added during execution.

#### **Load Timing**
- **(1), (2), (3)**: Loaded at startup.
- **(4), (5)**: Accumulate per turn.
- **(6)**: Loaded on-demand via Tool Search.

---

### 3. **External Interfaces**
- **User/API Input**:
  - Triggers the model to generate a **Text Response** or **Tool Calls**.
- **Tools**:
  - Invoked via **Tool Calls** (output from the model).
  - Results fed back into **(5) Runtime** as **Tool Results**.
  - **Tool Search** queries **(6) On-Demand** for deferred tool definitions.
- **File System**:
  - **(2) Project Config** reads `CLAUDE.md` and

#### ✓ `assets/p21-fig-01.png` — architecture_diagram (p21)

Here is a structured extraction of the architecture diagram:

---

### 1. **Top-Level Components**
- **Main Conversation**
  - Handles the primary interaction with the user or system.
- **Agent**
  - Central module that delegates tasks and manages subagents.
- **Explore**
  - Tool for exploration (e.g., searching or querying information).
- **Plan**
  - Tool for planning or strategizing actions.
- **General**
  - Built-in or legacy tool (marked with an "X," possibly deprecated or restricted).
- **Custom**
  - User-defined or customizable tool (highlighted in green).
- **Isolation Sandbox**
  - Isolated environment for executing subagents with restricted permissions.
- **Main Transcript**
  - Stores the history or log of the main conversation.
- **Subagent Transcript**
  - Stores the transcript of subagent interactions (`jsonl + meta.json`).
- **Subagent Report**
  - Stores the output or report generated by the subagent (`text + metadata`).

---

### 2. **Data Flow**
#### **Main Conversation → Agent**
- Delegates tasks to the **Agent** (e.g., task or legacy alias).
- **Read & Write**: The **Main Conversation** reads from and writes to the **Main Transcript**.

#### **Agent → Built-in Tools**
- The **Agent** invokes one of the following tools:
  - **Explore** (for exploration tasks).
  - **Plan** (for planning tasks).
  - **General** (legacy or built-in tool).
  - **Custom** (user-defined tool).
- Tools return results to the **Agent**.

#### **Agent → Isolation Sandbox**
- The **Agent** delegates tasks to subagents in the **Isolation Sandbox**.
- The sandbox enforces:
  1. **Rebuilt Permission Context & Tool Set** (restricted access).
  2. **Permission Mode Override** (optional).
  3. **Isolated Worktree** (isolated execution environment).

#### **Isolation Sandbox → Subagent Context**
- The **Isolation Sandbox** creates an **Isolated Subagent Context** for execution.
- Subagent interactions are logged in the **Subagent Transcript** (`jsonl + meta.json`).
- Subagent outputs are stored in the **Subagent Report** (`text + metadata`).

#### **Subagent → Main Transcript**
- Results from the **Subagent Report** are inserted into the **Main Transcript**.

#### **Other Built-in Tools**
- Tools like `statussline` and `verification` (dashed line) may interact with the **Agent** or **Main Conversation**, but their flow is not explicitly detailed.

---

### 3. **External Interfaces**
- **User Interaction**
  - The **Main Conversation** likely interacts with a user (not explicitly

#### ✓ `assets/p24-fig-01.png` — architecture_diagram (p24)

Here is the structured extraction from the provided architecture diagram:

---

### 1. **Top-level Components**
- **Session ID**
- **Conversation** (with a token limit `</>`)
- **Context Window**
- **Durable Storage**
- **Session Transcript** (outputs `.jsonl` files)
- **History / Subagent Logs**
- **Checkpoints**
- **Compaction Flow** (process, not a storage component)

---

### 2. **Data Flow**
#### **Primary Flows**
| **Source**          | **Destination**      | **Data/Action**                     | **Direction** |
|---------------------|----------------------|-------------------------------------|---------------|
| Conversation        | Context Window       | Conversation data                   | →             |
| Context Window      | Session Transcript   | Session data (streamed)             | →             |
| Session Transcript  | Durable Storage      | Persistent storage of transcripts   | ↓             |
| Session Transcript  | Checkpoints          | Serialized session state            | →             |
| Checkpoints         | Session Transcript   | Rewind (restore state)              | ←             |
| Old Tool Outputs    | Compaction Flow      | Remove (garbage collection)         | →             |
| Session Summary     | Compaction Flow      | Generate (summarization)            | →             |
| Compaction Flow     | Context Window       | Compacted boundary (marked data)    | →             |

#### **Secondary Flows**
- **Session Transcript** → **History / Subagent Logs**: Logs of session activity.
- **Checkpoints** → **External Interfaces**:
  - **Rewind** (restore to prior state)
  - **Resume** (continue from checkpoint)
  - **Fork** (branch from checkpoint).

---

### 3. **External Interfaces**
- **APIs/Users**:
  - **Rewind**: Restore a session from a checkpoint.
  - **Resume**: Continue a session from a checkpoint.
  - **Fork**: Create a new session branch from a checkpoint.
- **Storage**:
  - **Durable Storage**: Persistent storage for transcripts and logs.
  - **Checkpoints**: Serialized snapshots of session state (likely stored in a database or filesystem).

---

### 4. **Key Design Decisions**
1. **Context Window Management**:
   - The system enforces a **capacity limit** on the *Context Window* (e.g., token limit `</>`), implying a sliding window or truncation mechanism for efficiency.

2. **Compaction Flow**:
   - **Three-phase process** for managing long-running sessions:
     1. **Remove**: Discard old tool outputs to reduce clutter.
     2. **Generate**: Create session summaries to preserve key information.
     3. **Mark**: Define a *compact

#### ✓ `assets/p46-fig-01.png` — architecture_diagram (p46)

Here is a concise extraction of the requested information from the architecture diagram:

---

### 1. **Top-level Components**
The architecture is divided into the following named modules:

- **Entry & Startup**
- **UI Layer**
- **Core Loop**
- **Tools & Commands**
- **Safety & Permissions**
- **Extensibility**
- **Context & Memory**
- **Persistence**
- **Services & Integration**
- **Additional Infrastructure**

---

### 2. **Data Flow**
The diagram does not explicitly show arrows or directional flow, but the **runtime responsibilities** imply the following interactions:

#### **Entry & Startup → Other Components**
- Initializes the application, dispatches modes, and invokes CLI/SDK handlers.
- Triggers the **UI Layer** (REPL composition) and **Core Loop** (headless/SDK startup).

#### **Core Loop (query.ts, QueryEngine.ts) → Tools & Commands**
- The agentic query loop (in `queryLoop AsyncGenerator`) invokes **Tools** (42 implementations) and **Commands** (86 slash commands).
- Delegates tool execution to `services/tools/`.

#### **Core Loop ↔ Context & Memory**
- Assembles context via `getSystemContext` and `getUserContext` (in `context.ts`).
- Feeds context into the query loop and receives compacted memory (5-layer compaction in `services/compact/`).

#### **UI Layer ↔ Core Loop**
- Terminal UI (ink framework) renders outputs from the **Core Loop** (e.g., system prompts, query results).
- Forwards user input (e.g., slash commands) to the **Core Loop** or **Tools & Commands**.

#### **Tools & Commands ↔ Safety & Permissions**
- Tool execution is gated by permission checks (`useCanUseTool.tsx`).
- Permission dialogs (UI) interact with the **Safety & Permissions** module for user approval.

#### **Extensibility → Core Loop/Tools**
- Plugins/skills are loaded and registered, extending **Tools & Commands** or **Core Loop** functionality.
- Hooks (27 event types) dispatch lifecycle events to plugins/skills.

#### **Persistence ↔ Context & Memory**
- `history.ts` writes/reads global prompt history (`history.json`).
- `sessionStorage.ts` manages per-session transcripts and sidechains.

#### **Services & Integration → External Systems**
- MCP client (8+ transports) and API adapters communicate with external services (e.g., LSP, analytics).
- Remote execution backend and multi-agent coordination (`coordinator/`) interact with external agents.

#### **Additional Infrastructure → All Components**
- Provides foundational services like WebSocket communication, terminal rendering, and app initialization.

---

### 3. **External Interfaces**
#### **APIs/Protocols**
- **MCP Client**:


### deep_2604.26962v2.pdf

**8/8 real figures**

#### ✓ `assets/p02-fig-01.png` — architecture_diagram (p2)

Here is a structured extraction from the provided architecture diagram:

---

### 1. **Top-Level Components**
#### **(a) Agentic Problem Tutoring (Prior Work)**
- **Tutoring Question**
- **Tool Injection** (implicit, indicated by arrow label)
- **Step-base Verifier**
- **Response Generator**

#### **(b) Agentic Question Generation (Prior Work)**
- **Practice Objectives**
- **Reflection**
- **Critic**
- **Quiz Generation**

#### **(c) DeepTutor: Personalized Tutoring Loop (Proposed System)**
- **Citation-grounded Problem Tutoring**
- **Static Knowledge Grounding (SKG)**
- **Dynamic Personal Memory (DPM)**
- **Difficulty-calibrated Question Generation**

---

### 2. **Data Flow**
#### **(a) Agentic Problem Tutoring**
- **Tutoring Question** → (Tool Injection) → **Step-base Verifier** → **Response Generator**
  (Flow is left-to-right, with tool injection augmenting the input to the verifier.)

#### **(b) Agentic Question Generation**
- **Practice Objectives** → **Reflection** → **Quiz Generation**
- **Practice Objectives** → **Critic** → **Quiz Generation**
  (Reflection and Critic operate in parallel, both feeding into Quiz Generation.)

#### **(c) DeepTutor Personalized Loop**
- **Citation-grounded Problem Tutoring** → **Static Knowledge Grounding (SKG)**
- **Citation-grounded Problem Tutoring** → **Dynamic Personal Memory (DPM)**
  (Bidirectional arrows indicate iterative feedback between SKG/DPM and tutoring.)
- **SKG** ↔ **Difficulty-calibrated Question Generation**
- **DPM** ↔ **Difficulty-calibrated Question Generation**
  (Red arrows show feedback loops between SKG/DPM and question generation.)
- **Difficulty-calibrated Question Generation** → **Citation-grounded Problem Tutoring**
  (Closed loop: generated questions feed back into tutoring.)

---

### 3. **External Interfaces**
- **Implicit Inputs/Outputs**:
  - **Users**: Likely interact with the system via "Tutoring Question" (input) and "Response Generator" (output) in (a), and "Quiz Generation" (output) in (b).
  - **Databases/Knowledge Sources**:
    - **Static Knowledge Grounding (SKG)**: Presumably interfaces with a static knowledge base (e.g., textbooks, curated facts).
    - **Dynamic Personal Memory (DPM)**: Likely stores learner-specific data (e.g., past interactions, performance).
  - **Tools/APIs**:
    - **Tool Injection** in (a) suggests integration with external tools (e.g., calculators, simulators).

---

### 4. **Key Design Decisions**

#### ✓ `assets/p03-fig-01.png` — architecture_diagram (p3)

### 1. **Top-level Components**
The diagram illustrates a **Hybrid Personalization** architecture for a research or educational system. The named modules are:

- **Personalized Problem Tutoring**
- **Static Knowledge Grounding**
  - Multi-modal Parsing
  - KB (Knowledge Base) Construction
  - Mixed Retrieval
- **Personalized Question Generation**
- **Tool Agent** (with subcomponents for evidence extraction and scratchpad operations)
- **Dynamic Personal Trace Memory**
  - Solve Trace
  - Question Trace
  - Personal Profile (Session History, User Weakness, Self-Reflection)
- **Trace DB** (Database for storing traces)
- **TraceToolkit** (for trace generation and management)

---

### 2. **Data Flow**

#### **Personalized Problem Tutoring → Static Knowledge Grounding**
- **Initial Question** (Meta Question, Step Question) flows into **Static Knowledge Grounding** for grounding via RAG (Retrieval-Augmented Generation) tools and trace toolkits.
- **Scratchpad** (step-based) is created, read, and written iteratively, feeding into the **Tool Agent**.

#### **Static Knowledge Grounding**
- **Multi-modal Parsing**: Inputs (Text, Image, Formula, Table) are parsed into structured knowledge.
- **KB Construction**: Text KG (Knowledge Graph), Multi-modal KG, and Mixed KG are constructed.
- **Mixed Retrieval**: Uses Text VDB (Vector Database), Multi-modal VDB, and Mixed VDB to retrieve relevant information.
- **Search Query** is generated and sent to **Personalized Question Generation**.

#### **Personalized Question Generation**
- **Question Topic** (Exploration, Concentration, Idea Collection) is defined.
- **RAG Tool** generates sub-queries and traces.
- **Top-K Selector** filters questions based on clarity, relevance, diversity, validity, and consistency.
- **Question Templates** are loaded and filled with values to generate questions.
- **Idea Filter** refines questions before output.

#### **Tool Agent**
- Uses **RAG Tool**, **Code Tool**, **Web Tool**, and **Deep Reasoning** to extract **Evidences**.
- Evidences are written to the **Full Scratchpad** and used to draft answers.
- **Solve Trace** (Initial Question, Planned Steps, Solve Rounds) is recorded in **Dynamic Personal Trace Memory**.

#### **Dynamic Personal Trace Memory**
- **Solve Trace** and **Question Trace** are stored and evolve over time.
- Personal Profile components (**Session History**, **User Weakness**, **Self-Reflection**) are updated.
- Traces are stored in **Trace DB** and managed via **TraceToolkit**.

#### **Q-A Pair Generation**
- **RAG Tool** generates Q-A pairs, which are validated.
- If validation fails,

#### ✓ `assets/p06-fig-01.png` — architecture_diagram (p6)

### **1. Top-Level Components**
The diagram depicts the following named modules in the architecture:

- **KB Initialization** (Knowledge Base Initialization)
- **Profile Definition**
- **Personal Profile Construction**
- **Tutor Bench**

---

### **2. Data Flow**
The flow of data between components is as follows:

1. **KB Initialization → KB Indexing**
   - Input: **University-Level Textbooks** and **Research Papers**
   - Output: A structured **Domain Hierarchy** (indexed knowledge base)

2. **KB Indexing → Personal Profile Construction**
   - The indexed knowledge base is split into **chunks** (Chunk 1, Chunk 2, ..., Chunk k) for gap analysis.

3. **Profile Definition → Personal Profile Construction**
   - **Personality** and **Goal** (e.g., Beginner, Intermediate, Advanced) are passed as inputs.
   - Profiles define prior knowledge levels:
     - *Beginner*: Minimal prior knowledge
     - *Intermediate*: Growing prior knowledge
     - *Advanced*: Extensive prior knowledge

4. **Personal Profile Construction**
   - Takes **chunks** from the knowledge base and **profile definitions** to generate **knowledge gaps**:
     - **Misconception** (incorrect understanding)
     - **Incomplete** (partial understanding)
     - **Missing** (no understanding)
   - Generates **Task Candidates** from identified gaps.

5. **Task Candidates → Tutor Bench**
   - **Task Pool** is created from accepted candidates.
   - Tasks can be **Passed** or **Rejected** (filtered out).

6. **Tutor Bench → External Interfaces**
   - Outputs:
     - **Profile** (user knowledge state)
     - **Knowledge Gap** (identified deficiencies)
     - **Interactive Task** (exercises, assessments)
     - **Source Reference** (supporting materials from the knowledge base)

---

### **3. External Interfaces**
The system interacts with the following external entities:

- **Input Sources**
  - **University-Level Textbooks** (structured knowledge)
  - **Research Papers** (advanced knowledge)
  - **User Profiles** (personality and learning goals)

- **Output Consumers (Tutor Bench)**
  - **Learners/Tutors** (interactive tasks, feedback)
  - **Knowledge Gap Reports** (for adaptive learning)
  - **Source References** (supporting materials)

---

### **4. Key Design Decisions**
The diagram reveals the following architectural choices:

1. **Modular Knowledge Processing**
   - Separation of **KB Initialization** (static knowledge ingestion) from **Profile-Based Construction** (dynamic gap analysis).
   - Enables scalability and reusability of the knowledge base.

2. **Hierarchical Domain Structuring**
   - Knowledge is

#### ✓ `assets/p11-fig-01.png` — architecture_diagram (p11)

### 1. **Top-level Components**
- **Student Simulator**
  - Profile
  - Gap
  - Task
  - Relevant Materials
    - Source Reference
    - Relevant Origin KB
- **First-person Interactive Evaluation**
  - Student (Propose a Question)
  - Tutor (Respond with Solution)
  - Tutor (Customize Quiz for Student)
- **Traces Recorded** (Output storage)

---

### 2. **Data Flow**
#### **Student Simulator → First-person Interactive Evaluation**
- **First-person Beliefs** (derived from *Profile* and *Gap*)
- **Dynamic Solve List** (derived from *Task* and *Relevant Materials*)
  - Flows into the interactive loop as either:
    - **Solve List Unfinished** (triggers *Student* to *Propose a Question*)
    - **Solve List Finished** (triggers *Tutor* to *Customize Quiz*)

#### **First-person Interactive Evaluation (Internal Loop)**
1. **Student** → *Propose a Question* → **Tutor**
2. **Tutor** → *Respond with Solution* → **Student**
3. **Tutor** → *Customize Quiz* → **Student** (if *Solve List Finished*)

#### **Output**
- All interactions are recorded as **Traces Recorded**.

---

### 3. **External Interfaces**
- **Inputs to Student Simulator**:
  - *Profile* (e.g., student background)
  - *Gap* (e.g., knowledge gaps)
  - *Task* (e.g., problem set)
  - *Relevant Materials* (e.g., source references, knowledge base)
- **Human/User Roles**:
  - *Student* (interacts by proposing questions)
  - *Tutor* (interacts by responding and customizing quizzes)
- **Output**:
  - *Traces Recorded* (stored interaction logs, likely for analysis or evaluation).

---

### 4. **Key Design Decisions**
1. **Modular Separation of Concerns**:
   - *Student Simulator* generates personalized inputs (beliefs, solve lists) independently of the interactive evaluation.
   - *First-person Interactive Evaluation* focuses on real-time interaction between student and tutor.

2. **Dynamic Personalization**:
   - The *Dynamic Solve List* adapts based on *First-person Beliefs* and *Relevant Materials*, ensuring tailored problem-solving.
   - *Tutor* customizes quizzes based on student progress (*Solve List Finished*).

3. **Closed-Loop Feedback**:
   - The system records *Traces* of all interactions, enabling iterative improvement of the simulator or tutor responses.

4. **First-Person Perspective**:
   - The *Student* module simulates real student behavior (e.g., proposing questions

#### ✓ `assets/p12-fig-01.png` — bar_chart (p12)

1. **Chart title**
   The chart does not display an explicit overall title, but it is divided into two sections: **Tutoring Side** and **Practice Side**. The left side (blue bars) represents the Tutoring Side, and the right side (orange bars) represents the Practice Side.

2. **X-axis label and tick values**
   - **X-axis label**: Not explicitly labeled, but the tick values represent different categories or metrics.
   - **Tick values (categories)**:
     - Tutoring Side: SF, PER, APP, VID, LD
     - Practice Side: FIT, GND, DIV, ANS, CC

3. **Y-axis label and range**
   - **Y-axis label**: Score (1–5)
   - **Range**: 0.0 to 5.0

4. **Series/groups (legend entries)**
   - Humanities (dark blue)
   - Sciences (light blue)
   - Engineering (green)
   - Business (orange)
   - Research (pink)

5. **Key observations**
   - The **Tutoring Side** scores are generally higher than the **Practice Side** scores across all categories.
   - **VID** (Tutoring Side) and **ANS** (Practice Side) have the highest scores in their respective sections, both exceeding 4.5.
   - The **GND** category on the Practice Side has the lowest average score (~2.96).

6. **Data table**

| Category | Humanities | Sciences | Engineering | Business | Research | **Average Score** |
|----------|------------|----------|-------------|----------|----------|-------------------|
| SF       | 3.4        | 3.2      | 3.5         | 3.3      | 3.3      | **3.36**          |
| PER      | 4.6        | 4.6      | 4.6         | 4.5      | 4.7      | **4.59**          |
| APP      | 4.5        | 4.6      | 4.6         | 4.5      | 4.6      | **4.56**          |
| VID      | 4.8        | 4.8      | 4.8         | 4.8      | 4.9      | **4.81**          |
| LD       | 4.6        | 4.6      | 4.6         | 4.6      | 4.7      | **4.61**          |
| FIT      | 3.3        | 3.4      | 3.4         | 3.3      | 3.3      | **3

#### ✓ `assets/p13-fig-01.png` — bar_chart (p13)

1. **Chart title**
   The chart does not have a specific visible title, but it is divided into two sections labeled **"Tutoring Side"** and **"Practice Side"**.

2. **X-axis label** and tick values
   - **Label:** Not explicitly labeled, but represents different categories compared.
   - **Tick values:** `SF`, `PER`, `APP`, `VID`, `LD`, `FIT`, `GND`, `DIV`, `ANS`, `CC`
   - Each tick value has two conditions: `H` and `L`.

3. **Y-axis label** and range
   - **Label:** `Preference share (%)`
   - **Range:** 0% to 100%

4. **Series/groups (legend entries)**
   - **DeepTutor Win** (blue)
   - **Tie** (gray)
   - **Baseline Win** (orange)

5. **Key observations**
   - **DeepTutor** generally outperforms the baseline in most categories, especially in `PER`, `VID`, `LD`, and `ANS`, where its win share is significantly higher.
   - The **Tie** condition remains relatively consistent across categories, generally occupying a smaller share compared to wins.
   - The **Baseline Win** dominates in `APP` and `FIT`, where it achieves a higher preference share than DeepTutor.

6. **Data table**

| Category | DeepTutor Win (H) | DeepTutor Win (L) | Tie (H) | Tie (L) | Baseline Win (H) | Baseline Win (L) |
|----------|-------------------|-------------------|---------|---------|------------------|------------------|
| SF       | 55.6%             | 66.7%             | ~20%    | ~13%    | ~24%             | ~20%             |
| PER      | 73.3%             | 76.6%             | ~13%    | ~10%    | ~13%             | ~13%             |
| APP      | 40.0%             | 42.2%             | ~20%    | ~17%    | ~40%             | ~40%             |
| VID      | 71.1%             | 77.8%             | ~13%    | ~10%    | ~16%             | ~12%             |
| LD       | 55.6%             | 51.1%             | ~20%    | ~20%    | ~24%             | ~29%             |
| FIT      | 53.3%             | 48.9%

#### ✓ `assets/p13-fig-02.png` — line_chart (p13)

1. **Chart title**
   *Not visible in the provided crop.*

2. **X-axis**
   - **Label:** Outcome category (DeepTutor wins, Tie, Baseline wins)
   - **Range:** Three discrete categories (DeepTutor wins, Tie, Baseline wins)

3. **Y-axis**
   - **Label:** Preference rate
   - **Range:** 0 % – 80 %

4. **Series**
   - **Human** (blue line): rises to a peak at DeepTutor wins, dips at Tie, then rises again at Baseline wins
   - **LLM** (orange line): follows a similar pattern to the Human series

5. **Key observations**
   - Both humans and LLMs most frequently prefer DeepTutor over the baseline (highest preference at “DeepTutor wins”).
   - The “Tie” category has the lowest preference rate for both series.
   - At “Baseline wins,” preference rates increase again but remain below the “DeepTutor wins” peak.

6. **Approximate data**

   | Category          | Human (%) | LLM (%) |
   |-------------------|-----------|---------|
   | DeepTutor wins    | 54.7      | 59.1    |
   | Tie               | 15.1      | 12.0    |
   | Baseline wins     | 30.2      | 28.9    |

#### ✓ `assets/p14-fig-01.png` — bar_chart (p14)

### 1. Chart Title
- **(a)** Overall Quality (OQ) Drop (%)
- **(b)** Metric-level Variation (%)

---

### 2. X-axis Label and Tick Values
#### **(a) Overall Quality (OQ) Drop (%)**
- **X-axis label:** Drop percentage (%)
- **Tick values:** 0 to 6 (with specific values labeled on bars: -2.8, -4.6, -5.4)

#### **(b) Metric-level Variation (%)**
- **X-axis label:** Metrics (Solve Quality and Practice Quality)
- **Tick values (metrics):**
  - Solve Quality: SF, PER, APP, VID, LD
  - Practice Quality: FIT, GND, DIV, ANS, CC

---

### 3. Y-axis Label and Range
#### **(a) Overall Quality (OQ) Drop (%)**
- **Y-axis label:** Series/groups (experimental conditions)
- **Range:** Not numerically labeled on Y-axis; categorical (3 groups)

#### **(b) Metric-level Variation (%)**
- **Y-axis label:** Series/groups (experimental conditions)
- **Range:** Not numerically labeled on Y-axis; categorical (3 groups)
- **Color bar range:** 0 to -25% (Δ vs. Full)

---

### 4. Series/Groups (Legend Entries)
- **w/o DPM**
- **w/o SKG**
- **w/o DMP + SKG**

---

### 5. Key Observations
- **Overall Quality (OQ) Drop:**
  - The largest drop in overall quality occurs in the **w/o DMP + SKG** condition (-5.4%).
  - Removing **SKG** alone causes a larger drop (-4.6%) than removing **DPM** alone (-2.8%).

- **Metric-level Variation:**
  - The **GND** (Practice Quality) metric shows the largest negative variation, especially in the **w/o SKG** (-24.7%) and **w/o DMP + SKG** (-25.0%) conditions.
  - The **SF** (Solve Quality) metric is least affected or even slightly improved in some conditions (e.g., +0.9% in **w/o DMP + SKG**).

---

### 6. Data Table

#### **(a) Overall Quality (OQ) Drop (%)**

| Condition         | OQ Drop (%) |
|-------------------|-------------|
| w/o DPM           | -2.8        |
| w/o SKG           | -4.6        |
| w/o DMP + SKG     | -5.4        |

---

#### **(b) Metric-level Variation (%)**

| Condition     | SF    | PER   | APP   | VID   |


### hierar_2607.02980v1.pdf

**10/11 real figures**

#### ✓ `assets/p01-fig-01.png` — line_chart (p1)

1. **Chart title**
   RULER Long-Context Extrapolation

2. **X-axis** label and range
   Context length (8 K, 16 K, 32 K, 64 K, 128 K, 256 K, 512 K, 1 M)

3. **Y-axis** label and range
   RULER average exact match (%)
   0 % – 100 %

4. **Series** names and trend direction
   - Olmo3-CPT (YaRN): steep decline from near 100 % at 8 K to ~0 % at 1 M
   - Olmo3-HiLS-Attn: gradual decline from 99.0 % at 8 K to 81.7 % at 1 M

5. **Key observations**
   - Olmo3-HiLS-Attn maintains high exact-match performance well beyond its 8 K training length, only dropping below 90 % at 256 K.
   - Olmo3-CPT (YaRN) shows catastrophic degradation once context exceeds 16 K, falling to near zero by 128 K.
   - The gap between the two methods widens dramatically as context length increases.

6. **Approximate data**

   | Context length | Olmo3-CPT (YaRN) | Olmo3-HiLS-Attn |
   |----------------|-------------------|------------------|
   | 8 K            | 99.0              | 99.0             |
   | 16 K           | 98.7              | 98.7             |
   | 32 K           | 95.3              | 95.3             |
   | 64 K           | 97.3              | 94.7             |
   | 128 K          | 25.0              | 90.7             |
   | 256 K          | 10.0              | 89.3             |
   | 512 K          | 2.0               | 89.3             |
   | 1 M            | 0.0               | 81.7             |

#### ✓ `assets/p01-fig-02.png` — bar_chart (p1)

1. **Chart title**
   The chart does not show a specific title above the figure, but the subfigure label is **(d)**.

2. **X-axis label** and tick values
   - **X-axis label:** Task categories
   - **Tick values:** SDoc, MDoc, Summ., Few-shot, Synth., Code (grouped under LongBench-v1 < 8K and LongBench-v1 > 8K), and Overall

3. **Y-axis label** and range
   - **Y-axis label:** Score
   - **Range:** 0 to 60

4. **Series/groups** (legend entries)
   - Olmo3-CPT (gray bars)
   - Olmo3-HiLS-Attn (blue bars with diagonal stripes)

5. **Key observations**
   - Olmo3-HiLS-Attn consistently outperforms Olmo3-CPT across almost all tasks and categories.
   - The performance gap is most pronounced in the **Few-shot** and **Code** tasks for sequences longer than 8K.
   - Both models show relatively lower scores in **MDoc** and **Summ.** tasks for sequences shorter than 8K.

6. **Data table**

| Task Category       | Olmo3-CPT | Olmo3-HiLS-Attn |
|---------------------|-----------|-----------------|
| **LongBench-v1 < 8K** |           |                 |
| SDoc                | 27.6      | 37.8            |
| MDoc                | 2.7       | 22.5            |
| Summ.               | 22.5      | 25.4            |
| Few-shot            | 44.2      | 51.8            |
| **LongBench-v1 > 8K** |           |                 |
| Synth.              | 10.0      | 50.3            |
| Code                | 1.4       | 33.1            |
| SDoc                | 25.0      | 35.4            |
| MDoc                | 1.0       | 18.1            |
| Summ.               | 17.2      | 29.0            |
| Few-shot            | 12.7      | 38.7            |
| **Overall**         | 20.3      | 34.3            |

#### ✓ `assets/p01-fig-03.png` — line_chart (p1)

1. **Chart title**
   - The figure contains two subplots with individual titles:
     - (a) Prefill latency
     - (b) Decode latency

2. **X-axis** label and range
   - Label: **Context length**
   - Range: 8K to 512K

3. **Y-axis** label and range
   - **(a) Prefill latency**:
     - Label: **Latency (s)**
     - Range: 0 to 70 seconds
   - **(b) Decode latency**:
     - Label: **Latency (ms/token)**
     - Range: 0 to 80 ms/token

4. **Series names and trend direction**
   - **Full attention** (gray line):
     - Trend: Latency increases sharply with increasing context length, especially beyond 128K.
   - **HiLS-Attn** (blue line):
     - Trend: Latency increases very slowly with increasing context length, showing a near-flat trend.

5. **Key observations**
   - HiLS-Attn significantly reduces latency compared to Full attention for both prefill and decode phases.
   - The performance gap widens as context length increases, with HiLS-Attn being **13.5x faster** in prefill and **15.7x faster** in decode at 512K context length.
   - At smaller context lengths (8K–64K), the latency difference between Full attention and HiLS-Attn is minimal.

6. **Approximate data**

| Context length | Prefill latency (s) - Full | Prefill latency (s) - HiLS | Decode latency (ms/token) - Full | Decode latency (ms/token) - HiLS |
|----------------|----------------------------|----------------------------|----------------------------------|----------------------------------|
| 8K             | ~1                          | ~1                          | ~5                                | ~5                                |
| 16K            | ~2                          | ~1.5                        | ~7                                | ~5                                |
| 32K            | ~4                          | ~2                          | ~10                               | ~6                                |
| 64K            | ~8                          | ~3                          | ~15                               | ~7                                |
| 128K           | ~18                         | ~4                          | ~25                               | ~8                                |
| 256K           | ~40                         | ~5                          | ~50                               | ~10                               |
| 512K           | ~65                         | ~5                          | ~75                               | ~5                                |

#### ✓ `assets/p01-fig-04.png` — bar_chart (p1)

1. **Chart title**
   General / Math / Code Tasks

2. **X-axis label** and tick values
   - **Label:** Models
   - **Tick values:** MMLU, GPQA, HellaSwag, ARC-c, BoolQ, Swag, Race, C-Math, GSM8K, CRUX, HumanEval, MBPP+, Avg.

3. **Y-axis label** and range
   - **Label:** Score
   - **Range:** 0 to 80

4. **Series/groups**
   - General
   - Math
   - Code
   - Average (appears as a single bar on the right)

5. **Key observations**
   - The **BoolQ** model achieves the highest score (~74.9) in the **General** category.
   - **Math** and **Code** tasks generally have lower scores compared to **General** tasks, with **GSM8K** (~63.4) leading in Math and **HumanEval** (~32.5) in Code.
   - The **Average** score across all models and tasks is approximately **42.8**.

6. **Data table**

| Model       | Score (General) | Score (Math) | Score (Code) |
|-------------|-----------------|--------------|--------------|
| MMLU        | 60.1            | -            | -            |
| GPQA        | 33.3            | -            | -            |
| HellaSwag   | 43.9            | -            | -            |
| ARC-c       | 55.8            | -            | -            |
| BoolQ       | 74.9            | -            | -            |
| Swag        | 64.2            | -            | -            |
| Race        | 56.0            | -            | -            |
| C-Math      | -               | 40.1         | -            |
| GSM8K       | -               | 63.4         | -            |
| CRUX        | -               | 36.8         | -            |
| HumanEval   | -               | -            | 32.5         |
| MBPP+       | -               | -            | 28.5         |
| **Average** | **42.8**        |              |              |

#### ✓ `assets/p02-fig-01.png` — bar_chart (p2)

1. **Chart title**
   8K In-Domain RULER (345M, 8K Training)

2. **X-axis label** and tick values
   - Label: Task type
   - Tick values: single NIAH, multi-key multi-query, variable tracking

3. **Y-axis label** and range
   - Label: Exact match (%)
   - Range: 0 % – 100 %

4. **Series/groups** (legend entries)
   - NSA
   - Full-Attn
   - Dash-Attn
   - InfLLM v2
   - ours

5. **Key observations**
   - In the single NIAH and multi-key multi-query tasks, “ours” achieves near-perfect exact match (~95–100 %).
   - For variable tracking, “ours” still leads (~70 %) but with a larger gap to the next best method.
   - All other methods show much lower performance on the more complex multi-key and variable-tracking tasks.

6. **Data table**

| Task                  | NSA   | Full-Attn | Dash-Attn | InfLLM v2 | ours  |
|-----------------------|-------|-----------|-----------|-----------|-------|
| single NIAH           | 45    | 20        | 85        | 25        | 95    |
| multi-key multi-query | 20    | 10        | 60        | 30        | 95    |
| variable tracking     | 15    | 5         | 25        | 10        | 70    |

#### ✓ `assets/p04-fig-01.png` — architecture_diagram (p4)

Here is a structured extraction of the architecture diagram:

---

### 1. **Top-level Components**
1. **Naïve Block Sparse Attention** (Top section)
   - **Sliding window attention** mechanism over tokens grouped into chunks (`τ₁`, `τ₂`, `τ₃`, ..., `τ_c`).
   - **Softmax** normalization across tokens within a chunk.
   - **Tokens**:
     - Distant token (gray)
     - Adjacent token (light green)
     - Landmark token (blue)
     - Current token (dark green)

2. **HILS-Attention (Hierarchical Landmark Sparse Attention)** (Bottom section)
   - **Intra-chunk softmax**: Computes attention within a chunk (e.g., `qᵀk_c` for the current chunk).
   - **Inter-chunk softmax**: Computes attention between chunks using landmark tokens (`k'_c`).
   - **Chunk-mass surrogate (`Z'_c`)**:
     - Approximates the normalization term (`Z_c`) using landmark tokens.
   - **Weighted combination (`w_j`)**:
     - Combines intra-chunk and inter-chunk attention scores.

3. **Equivalence Condition** (Blue box, right)
   - States when `Z'_c = Z_c` (i.e., when the surrogate matches the exact normalization term).

---

### 2. **Data Flow**
#### **Naïve Block Sparse Attention**
- **Input**:
  - Query (`q`), keys (`k_j`), and values (`x_j`) for tokens in chunks.
- **Flow**:
  1. Tokens are grouped into chunks (`τ_c`).
  2. Attention scores are computed as `exp(qᵀk_j)` for tokens in the chunk.
  3. Softmax normalizes scores within the chunk to produce `Z_c`.
  4. Output: Weighted sum of values (`x_j`) using softmax scores.

#### **HILS-Attention**
- **Input**:
  - Query (`q`), keys (`k_j`, `k'_c`), and values (`x_j`) for tokens and landmark tokens.
- **Flow**:
  1. **Intra-chunk softmax**:
     - Computes attention scores (`qᵀk_j`) for tokens in the same chunk as `q`.
  2. **Inter-chunk softmax**:
     - Uses landmark tokens (`k'_c`) to compute surrogate chunk-mass (`Z'_c ∝ exp(qᵀk'_c)`).
  3. **Combination**:
     - Weights (`w_j`) are computed by combining intra-chunk scores and the surrogate (`Z'_c`).
  4. **Output**:
     - Weighted sum of values (`x_j`) using `w_j`.

#### **Direction

#### ✗ `assets/p05-fig-01.png` — text_block (p5)

*Discarded: text_block (confidence=0.99)*

#### ✓ `assets/p08-fig-01.png` — architecture_diagram (p8)

Here is a structured extraction of the architecture diagram:

---

### 1. **Top-level Components**
#### **(a) NSA Kernel**
- **Q (Query Tensor)**
  - Shape: `L × h × d`
- **Selected K (Key Tensor)**
  - Shape: `L × d`
- **S (Score/Attention Matrix)**
  - Intermediate computation result.
- **G (Tensor Core Compute Module)**
  - Computes `(d, d) · (d, 5)` on Tensor Core.
- **O (Output Tensor)**
  - Shape: `L × h × d`

#### **(b) HiLS-Attention Kernel**
- **Q (Query Tensor)**
  - Shape: `L × h × d`
- **Union of Selected K (Key Tensor)**
  - Shape: `L × d`
- **M (Masking/Selection Module)**
  - Filters or masks keys for computation.
- **G (Tensor Core Compute Module)**
  - Computes `(M×6, d) · (d, 5)` on Tensor Core.
- **O (Output Tensor)**
  - Shape: `L × h × d`
- **V (Value Tensor)**
  - Shape: `L × d` (used in the inner loop).

---

### 2. **Data Flow**
#### **(a) NSA Kernel**
1. **Input Flow**:
   - `Q` (`L × h × d`) and **selected `K`** (`L × d`) enter the **Inner Loop**.
2. **Inner Loop Computation**:
   - `Q` and `K` interact to produce `S` (score matrix).
   - `S` is passed to **G (Tensor Core)** for computation: `(d, d) · (d, 5)`.
   - Result is aggregated into `I_d` (intermediate tensor of shape `d`).
3. **Grid Loop**:
   - Iterates over `h` heads (parallel or sequential).
4. **Output Flow**:
   - Intermediate results are written to `O` (`L × h × d`).

#### **(b) HiLS-Attention Kernel**
1. **Input Flow**:
   - `Q` (`L × h × d`) and the **union of selected `K`** (`L × d`) enter the **Inner Loop**.
2. **Masking/Selection**:
   - `M` filters keys/queries for computation.
3. **Inner Loop Computation**:
   - `Q`, masked `K`, and `M` are passed to **G (Tensor Core)** for computation: `(M×6, d) · (d, 5)`.
   - Result is aggregated into `I_d` (intermediate tensor of shape `d`).
4. **Value (`V`) Interaction**:
   - `V` (`L × d`) is used in the inner loop

#### ✓ `assets/p14-fig-01.png` — line_chart (p14)

Here is the extracted structured information from the provided line charts:

---

### 1. **Chart Title**
- Not explicitly visible, but the figure appears to compare **perplexity** and **RULER average exact match** performance for **Full-Attention RoPE** and **HiLS-Attention HoPE** models.

---

### 2. **X-Axis**
- **Label:**
  - (a) Context Length (Log Scale)
  - (b) Context Length (Log Scale)
- **Range:**
  - (a) 64 to 512K
  - (b) 8K to 512K

---

### 3. **Y-Axis**
- **(a) Perplexity**
  - **Label:** Perplexity
  - **Range:** 0 to 45
- **(b) RULER Average Exact Match (%)**
  - **Label:** RULER average exact match (%)
  - **Range:** 0% to 100%

---

### 4. **Series Names and Trend Direction**
#### **Panel (a): Perplexity**
| Series Name (Steps) | Trend Direction (Full-Attention RoPE) | Trend Direction (HiLS-Attention HoPE) |
|---------------------|---------------------------------------|---------------------------------------|
| 20k                 | Decreases, then increases sharply     | Decreases steadily                    |
| 40k                 | Decreases, then increases sharply     | Decreases steadily                    |
| 60k                 | Decreases, then increases sharply     | Decreases steadily                    |
| 80k                 | Decreases, then increases sharply     | Decreases steadily                    |
| 100k                | Decreases, then increases sharply     | Decreases steadily                    |
| 120k                | Decreases, then increases sharply     | Decreases steadily                    |
| 140k / 143k         | Decreases, then increases sharply     | Decreases steadily                    |

#### **Panel (b): RULER Average Exact Match (%)**
| Series Name (Steps) | Trend Direction (Full-Attention RoPE) | Trend Direction (HiLS-Attention HoPE) |
|---------------------|---------------------------------------|---------------------------------------|
| 20k                 | Sharp decrease                        | Gradual decrease                      |
| 40k                 | Sharp decrease                        | Gradual decrease                      |
| 60k                 | Sharp decrease                        | Gradual decrease                      |
| 80k                 | Sharp decrease                        | Gradual decrease                      |
| 100k                | Sharp decrease                        | Gradual decrease                      |
| 120k                | Sharp decrease                        | Gradual decrease                      |
| 1

#### ✓ `assets/p16-fig-01.png` — bar_chart (p16)

Here is the extracted information from the bar chart:

---

1. **Chart title**
   - (a) Prefill latency
   - (b) Decode latency

2. **X-axis label** and tick values
   - **X-axis label**:
     - (a) Latency (ms, log scale)
     - (b) Latency (ms/token, log scale)
   - **Tick values**:
     - (a) 10, 10², 10³, 10⁴
     - (b) 1, 10, 10²

3. **Y-axis label** and range
   - **Y-axis label**: Context length
   - **Range**: 8K, 16K, 32K, 64K, 128K, 256K, 512K

4. **Series/groups** (legend entries)
   - Full attention (gray)
   - HiLS-Attn (blue)

5. **Key observations**
   - HiLS-Attn consistently shows lower latency compared to Full attention across all context lengths for both prefill and decode latencies.
   - The latency improvement of HiLS-Attn over Full attention increases with larger context lengths, reaching up to **15.7×** for decode latency at 512K context length.
   - For smaller context lengths (e.g., 8K), HiLS-Attn latency is actually *higher* than Full attention in decode latency (0.73× indicates HiLS-Attn is slower).

6. **Data table**

| Context Length | Prefill Latency (ms) - Full Attention | Prefill Latency (ms) - HiLS-Attn | Speedup (×) | Decode Latency (ms/token) - Full Attention | Decode Latency (ms/token) - HiLS-Attn | Speedup (×) |
|----------------|---------------------------------------|----------------------------------|-------------|-------------------------------------------|---------------------------------------|-------------|
| 8K             | ~10                                   | ~6.2                             | 0.62        | ~1.5                                      | ~2.1                                  | 0.73        |
| 16K            | ~30                                   | ~28                              | 1.1         | ~2.5                                      | ~2.3                                  | 1.1         |
| 32K            | ~120                                  | ~63                              | 1.9         | ~5                                        | ~3                                    | 1.7         |
| 64K            | ~500                                  | ~150                             | 3.3         | ~15                                       | ~5.2                                  |

#### ✓ `assets/p17-fig-01.png` — line_chart (p17)

### 1. Chart Title
- **(a)** Final-block KV loading
- **(b)** Chunk-id overlap

---

### 2. X-Axis
- **(a)** Label: Context length
  Range: 4K to 64K
- **(b)** Label: Grouped query tokens *M*
  Range: 2 to 64

---

### 3. Y-Axis
- **(a)** Label: Chunk ids (log scale)
  Range: 10^2 to 10^3
- **(b)** Label: Overlap / reuse (%)
  Range: 84% to 100%

---

### 4. Series Names and Trend Direction
- **(a)**
  - **Visible history chunks** (black, dashed line with circles): Increasing trend
  - **Loaded union chunks** (blue, solid line with crosses): Increasing trend, but at a slower rate than visible history chunks
- **(b)**
  - **Layer range** (light blue shaded area): Increasing trend
  - **Chunk overlap** (blue, solid line with circles): Increasing trend
  - **M=16 block reuse** (red dashed line): Constant at 92.8%

---

### 5. Key Observations
- **(a) Final-block KV loading**
  - The number of visible history chunks increases exponentially with context length.
  - The number of loaded union chunks also increases but remains significantly lower than visible history chunks, indicating efficiency in loading.
  - The gap between visible history chunks and loaded union chunks widens as context length increases.

- **(b) Chunk-id overlap**
  - Chunk overlap percentage increases with the number of grouped query tokens, reaching approximately 97% at 64 tokens.
  - The chunk overlap consistently outperforms the M=16 block reuse baseline (92.8%) for grouped query tokens greater than 4.

---

### 6. Approximate Data

#### (a) Final-block KV loading

| Context Length | Visible History Chunks | Loaded Union Chunks | Reduction (%) |
|----------------|------------------------|---------------------|---------------|
| 4K             | ~120                   | ~30                 | 76.4%         |
| 8K             | ~200                   | ~105                | 48.0%         |
| 16K            | ~350                   | ~240                | 31.9%         |
| 32K            | ~600                   | ~510                | 15.1%         |
| 64K            | ~1000                  | ~910                | 9.0%          |

#### (b) Chunk-id overlap

| Grouped Query Tokens (M


### hipo_2607.02303v1.pdf

**4/4 real figures**

#### ✓ `assets/p01-fig-01.png` — bar_chart (p1)

1. **Chart title**
   Written-in property Φ

2. **X-axis label** and tick values
   • Label: (not explicitly shown, but implies different sample conditions)
   • Tick values: W, A, A+, A++, X, X+, A•

3. **Y-axis label** and range
   • Label: (not explicitly shown, inferred as a property value)
   • Range: 0 to 30

4. **Series/groups**
   • Single series of bars with three highlighted in color (the last bar in green, the fifth in blue, the others in gray).

5. **Key observations**
   • The highest value (29.0) occurs for the first sample (W).
   • The lowest value (22.9) is observed for the last sample (A•).
   • A gradual decrease from W to A• is interrupted by a local peak at X+ (26.9).

6. **Data table**

| Sample | Value |
|--------|-------|
| W      | 29.0  |
| A      | 28.8  |
| A+     | 28.2  |
| A++    | 27.3  |
| X      | 26.9  |
| X+     | 26.2  |
| A•     | 22.9  |

#### ✓ `assets/p01-fig-02.png` — line_chart (p1)

1. **Chart title**
   *Not visible in the provided crop.*

2. **X-axis** label and range
   - Label: *Not explicitly labeled, but likely represents some measure of size or iteration count (e.g., number of samples, steps, or epochs).*
   - Range: **2k** to **32k**

3. **Y-axis** label and range
   - Label: *Not explicitly labeled, but likely represents an error metric (e.g., loss, classification error, or perplexity).*
   - Range: **1** to **5**

4. **Series** names and trend direction
   - **Green solid line with circles**: Initially stable, then a slight increase followed by a sharp decline.
   - **Red dashed line with triangles**: Steady decline throughout.
   - **Blue dotted line with squares**: Steep decline initially, then flattens out near the bottom.
   - **Gray dashed line with squares**: Steady decline, but less steep than the red series.

5. **Key observations**
   - The **blue series** achieves the lowest value earliest (around **4k**), then remains stable.
   - The **red series** shows a consistent downward trend, outperforming the green series at higher values (beyond **8k**).
   - The **green series** initially remains stable but eventually declines sharply after **16k**, though it does not reach the lowest error.

6. **Approximate data**
Since exact values are not fully readable, approximate trends are summarized below:

| X-axis | Green Series | Red Series | Blue Series | Gray Series |
|--------|--------------|------------|-------------|-------------|
| 2k     | ~3.5         | ~3.8       | ~4.5        | ~3.8        |
| 4k     | ~3.5         | ~3.3       | ~2.0        | ~3.2        |
| 8k     | ~3.5         | ~2.8       | ~2.0        | ~2.8        |
| 16k    | ~3.6         | ~2.3       | ~2.0        | ~2.4        |
| 32k    | ~2.5         | ~1.8       | ~2.0        | ~2.0        |

#### ✓ `assets/p04-fig-01.png` — diagram (p4)

1. **Main elements** and their roles:
   - **State memory (Sₜ)**: A component that stores the entire history of a process in a compressed form. The compression is lossy, meaning some information may be discarded to reduce the data size.
   - **Arrow labeled "tens"**: Indicates the progression of time or sequential input feeding into the state memory.

2. **Relationships and connections** between elements:
   - The arrow points into the **State memory (Sₜ)**, suggesting that sequential or temporal data (e.g., time steps or observations) is continuously incorporated into the state memory.
   - The state memory aggregates and compresses this incoming data to retain a summary of all past information.

3. **Directional flow** (if present):
   - The flow is unidirectional, moving from left (input or time progression) to right (state memory accumulation).

4. **Key takeaway** in one sentence:
   - The diagram illustrates how **state memory (Sₜ) compresses and retains all historical data in a lossy manner** as new inputs are processed over time.

#### ✓ `assets/p04-fig-02.png` — diagram (p4)

1. **Main elements** and their roles:
   - **β·||e|| (left side, green box)**: Likely represents a threshold or scaling factor related to an error or embedding norm, used to determine significance or surprise.
   - **Exact KV memory 𝒜ₜ (central text)**: The precise key-value (KV) memory at time *t*, which stores relevant information.
   - **top-w surprising KV pairs (central text)**: The subset of the most surprising or unexpected KV pairs selected from the exact memory, where *w* denotes the number of pairs.
   - **CA₃ hippocampus (bottom text, italicized)**: Refers to a region in the hippocampus, often associated with memory processing and retrieval in neuroscience-inspired models.

2. **Relationships and connections** between elements:
   - The **exact KV memory 𝒜ₜ** is filtered to extract the **top-w surprising KV pairs**, implying a selection mechanism based on a surprise or novelty metric.
   - The **β·||e||** term appears to act as a criterion or threshold for identifying the surprising KV pairs from the exact memory.

3. **Directional flow** (if present):
   - The flow is from the **exact KV memory 𝒜ₜ** to the **top-w surprising KV pairs**, indicating a filtering or selection process.
   - The green arrow suggests that the threshold **β·||e||** influences or gates this selection.

4. **Key takeaway** in one sentence:
   - The diagram illustrates a mechanism for selecting the most surprising key-value pairs from an exact memory store, guided by a threshold criterion, likely for efficient memory retrieval or update in a hippocampal-inspired model.


### ideas_2607.08758v1.pdf

**13/22 real figures**

#### ✗ `assets/p01-fig-01.png` — text_block (p1)

*Discarded: text_block (confidence=0.99)*

#### ✓ `assets/p02-fig-01.png` — diagram (p2)

### **1. Main Elements and Their Roles**

#### **Views:**
- **Paper-Centric View (Blue Box, Top Left):**
  Represents traditional research interpretation focusing on individual papers without structured evolutionary context.

- **Genome-Centric View (Red Box, Bottom Left):**
  Introduces the concept of an **Idea Genome** as a structured way to represent research evolution:
  1. **Idea Genome:** Core representation of research ideas as modular components.
  2. **Genome Diff:** Differences between idea genomes, showing evolutionary changes.
  3. **Population:** Collection of related idea genomes forming a research field.
  4. **Lineage:** Temporal progression and ancestry of ideas.

#### **Mechanisms of Evolution (Tree Metaphor):**
- **Mutation (1):**
  Small incremental changes in an idea (e.g., YOLO → YOLOv2).
- **Radiation (2):**
  Diversification of an idea into multiple variants (e.g., BERT → ViT, Audio Transformer, Mask R-CNN).
- **Hybridization (3):**
  Combination of distinct ideas to form a new hybrid (e.g., NeRF + Diffusion Model → NeRF-Diffusion Hybrid).
- **Speciation (4):**
  Divergence of an idea into distinct branches (e.g., CNN → Transformer).
- **Isolation (5):**
  Independent evolution of ideas without cross-pollination (e.g., CNN vs. LSTM).
- **Competition (6):**
  Comparative performance leading to selection (e.g., YOLO vs. Faster R-CNN).

#### **Idea Genome Framework (Center):**
- **Definition:**
  A modular structure comprising:
  - **Problem Genome**
  - **Mechanism Genome**
  - **Representation Genome**
  - **Objective Genome**
  - **Data Genome**
  - **Constraint Genome**
  - **Evaluation Genome**

---

### **2. Relationships and Connections Between Elements**

- The **tree metaphor** connects all evolutionary mechanisms (Mutation, Radiation, Hybridization, Speciation, Isolation, Competition) to the central **Idea Genome**.
- The **Genome-Centric View** (left) defines the lifecycle of an idea genome, which feeds into and is shaped by the evolutionary mechanisms on the tree.
- The **Paper-Centric View** is implicitly linked to the Genome-Centric View as the traditional, less structured precursor.
- **Hybridization** and **Competition** explicitly combine or contrast multiple idea genomes, showing interaction between branches.
- **Radiation** and **Speciation** show divergence from a common ancestor, while **Isolation** shows parallel but separate evolution.

---

### **3. Directional Flow**

- **Left Side (Genome-Centric View):**
  Linear progression from **Idea Genome

#### ✓ `assets/p07-fig-01.png` — architecture_diagram (p7)

Here is a structured extraction from the **IdeaGene-Bench** architecture diagram:

---

### 1. **Top-Level Components**
The pipeline consists of **6 core modules** (numbered in the top row) and **2 evaluation benchmarks** (bottom row):

| **Module**                     | **Description**                                                                                     |
|--------------------------------|-----------------------------------------------------------------------------------------------------|
| **1. Input Papers**            | Collection of research papers (likely PDFs or text) used as raw input.                              |
| **2. Idea Genome Extraction**  | Automated extraction of "idea genomes" (structured representations of ideas) from input papers.     |
| **3. GenomeDiff Alignment**    | Alignment of idea genomes to identify differences or similarities (e.g., via diff-like operations). |
| **4. Audited Lineage Graph**   | Construction of a graph representing the evolutionary lineage of ideas, with human auditing.        |
| **5. Evaluate Benchmarks**     | Two sub-benchmarks for evaluating models:                                                           |
|   - **IdeaGene-Exam**          | Objective tests for lineage understanding (4 task types: T1–T4).                                    |
|   - **IdeaGene-Arena**         | Subjective evaluation of lineage-grounded idea generation (via proposal and population evaluation).  |
| **6. Correlation Analysis**    | Measurement of correlation between understanding (Exam) and generation (Arena) performance.         |

---

### 2. **Data Flow**
**Direction**: Left to right, with feedback loops implied for auditing and evaluation.

| **Source**               | **Target**               | **Data Type**                                                                                     | **Notes**                                  |
|--------------------------|--------------------------|---------------------------------------------------------------------------------------------------|--------------------------------------------|
| Input Papers             | Idea Genome Extraction   | Raw text/papers                                                                                   |                                            |
| Idea Genome Extraction   | GenomeDiff Alignment     | Extracted "idea genomes" (structured representations)                                             |                                            |
| GenomeDiff Alignment     | Audited Lineage Graph    | Aligned genomes + diffs                                                                           |                                            |
| Audited Lineage Graph    | IdeaGene-Exam            | Lineage graph + task-specific inputs (e.g., pairs/chains of genomes)                              |                                            |
| Audited Lineage Graph    | IdeaGene-Arena           | Lineage graph + questions/libraries                                                               |                                            |
| IdeaGene-Exam            | Correlation Analysis     | Performance metrics (e.g., accuracy)                                                              |                                            |
| IdeaGene-Arena           | Correlation Analysis     | ELO scores (from population evaluation) + PES metrics (Heredity, Variation, Selection)            |                                            |
| Audited Lineage Graph    | IdeaGene-Arena           | Proposals (new ideas)                                                                             | Inserted into lineage for evaluation.      |
| IdeaGene-Arena           | Audited Lineage Graph    | Updated lineage graph                                                                             | Feedback

#### ✓ `assets/p10-fig-01.png` — scatter_plot (p10)

1. **Chart title**
   *Not visible in the provided crop.*

2. **X-axis and Y-axis labels**
   - X-axis: **PES (0–100)**
   - Y-axis: *Not explicitly labeled (appears to list different models/methods)*

3. **Groups/clusters visible (colour/shape coding)**
   - **Blue circles**: Question
   - **Green circles**: Library
   - **Orange circles**: Lineage

4. **Key observations**
   - **MiniMax** (Lineage) stands out as a clear outlier with the highest PES (~75) and a large positive offset (+9.9).
   - Models using **G5.5 + Codex** and **G5.5 + Claude** achieve the highest PES scores (~85–88) with moderate positive offsets.
   - Most models cluster between **PES 65–80**, with offsets generally ranging from **+1.8 to +6.9**, indicating incremental improvements across methods.

#### ✓ `assets/p10-fig-02.png` — heatmap (p10)

1. **Title**
   *Not explicitly visible in the crop.*

2. **Row labels** and **column labels**
   - **Row labels** (with sample sizes):
     • Mutation (n = 64)
     • Adaptive (n = 111)
     • Speciation (n = 15)
     • Hybrid (n = 224)
     • Library | Hybrid (n = 413)
     • Lineage | Hybrid (n = 407)
   - **Column labels**:
     • Heredity
     • Variation
     • Selection
     • PES

3. **Colour scale meaning**
   - Dark blue = lower scores (~60)
   - Light blue/white = higher scores (~85)

4. **Hotspot regions**
   - **Highest values**:
     • Library | Hybrid – Heredity (82.7)
     • Lineage | Hybrid – Heredity (84.2) and Variation (84.7)
   - **Lowest value**:
     • Mutation – PES (69.2)

5. **Markdown table**

```markdown
| Question \ Topic       | Heredity | Variation | Selection | PES  |
|------------------------|----------|-----------|-----------|------|
| Mutation (n=64)        | 61.9     | 82.7      | 79.7      | 69.2 |
| Adaptive (n=111)       | 74.2     | 83.2      | 80.3      | 78.8 |
| Speciation (n=15)      | 74.9     | 82.7      | 79.8      | 79.1 |
| Hybrid (n=224)         | 78.8     | 84.0      | 81.1      | 81.1 |
| Library | Hybrid (n=413) | 82.7     | 84.0      | 81.8      | 82.8 |
| Lineage | Hybrid (n=407) | 84.2     | 84.7      | 82.2      | 83.6 |
```

#### ✓ `assets/p11-fig-01.png` — scatter_plot (p11)

1. **Chart title**
   *(Not explicitly visible in the crop)*

2. **X-axis and Y-axis labels**
   - X-axis: IG-Exam overall exact accuracy (%)
   - Y-axis: IG-Arena Language heredity

3. **Groups/clusters visible**
   - Blue circles: Direct LLM
   - Green circles: Research agent
   - Orange circles: CLI harness

4. **Key observations**
   - Models using CLI harness (orange) tend to cluster toward the top-right, indicating both high exact accuracy and high language heredity.
   - The single green point (AI Sci) sits below the main cloud, showing lower language heredity for its accuracy compared to CLI-harness models.
   - Direct LLMs (blue) span a wide range of accuracy but generally lie below the CLI-harness points in language heredity.

#### ✓ `assets/p11-fig-02.png` — radar_chart (p11)

Here is the extracted information from the radar/spider chart:

### 1. **Axes (Dimensions)**
- Genome Abstraction
- Inheritance Tracing
- Evolutionary Reasoning
- Lineage Verification

### 2. **Series Compared (Legend Entries)**
- GPT-5.5
- GPT-5.5 + Col-Agent
- GPT-5.5 + AI Sci v2
- GPT-5.5 + Codex
- GPT-5.5 + ClaudeCode
- GPT-5.5 + Col-ClaudeCode (best)

### 3. **Notable Strengths/Weaknesses per Dimension**

| **Dimension**          | **Strengths**                                                                 | **Weaknesses**                          |
|------------------------|------------------------------------------------------------------------------|-----------------------------------------|
| **Genome Abstraction** | - GPT-5.5 + Col-ClaudeCode (best) performs exceptionally well (~85).         | - GPT-5.5 alone performs the weakest (~30). |
| **Inheritance Tracing**| - GPT-5.5 + Col-ClaudeCode (best) leads (~80).                              | - GPT-5.5 alone is the weakest (~30).   |
| **Evolutionary Reasoning** | - GPT-5.5 + Col-ClaudeCode (best) excels (~80).                          | - GPT-5.5 alone is the weakest (~30).   |
| **Lineage Verification** | - GPT-5.5 + Col-ClaudeCode (best) dominates (~85).                        | - GPT-5.5 alone is the weakest (~30).   |

### **Summary**
- **GPT-5.5 + Col-ClaudeCode (best)** consistently outperforms all other models across all dimensions.
- **GPT-5.5 alone** is the weakest performer in every category.
- The other variants (**Col-Agent, AI Sci v2, Codex, ClaudeCode**) show intermediate performance, with no single model dominating outside of the best-performing hybrid.

#### ✗ `assets/p16-fig-01.png` — text_block (p16)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p16-fig-02.png` — text_block (p16)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p16-fig-03.png` — text_block (p16)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p17-fig-01.png` — text_block (p17)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p17-fig-02.png` — text_block (p17)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p17-fig-03.png` — text_block (p17)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p18-fig-01.png` — text_block (p18)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p18-fig-02.png` — text_block (p18)

*Discarded: text_block (confidence=0.99)*

#### ✓ `assets/p19-fig-01.png` — heatmap (p19)

1. **Title**
   Lineage-setting PES by domain

2. **Row labels** (models)
   - GPT-5.5
   - + Claude Code
   - Opus 4.7 + Claude Code
   - Opus 4.7 + Codex
   - DeepSeek-V4-Pro
   - Claude Opus 4.7
   - Gemini 3.1 Pro
   - Kimi-K2-Thinking
   - GLM-5.1
   - AI Scientist v2
   - Col-Agent
   - + Codex
   - Qwen3.6-Max
   - MiniMax-M2.7

3. **Column labels** (domains)
   - Biology
   - Chemistry
   - Climate
   - CS
   - Energy
   - Materials
   - Math
   - Medicine
   - Neuro
   - Physics

4. **Colour scale meaning**
   - Dark blue (low) ≈ 66 PES
   - Light blue/white (high) ≈ 89 PES

5. **Hotspot regions**
   - GPT-5.5 in Energy (89)
   - + Claude Code in Energy (89) and Neuro (89)
   - Opus 4.7 + Claude Code in Energy (87)

6. **Markdown table**

```markdown
| Model                     | Biology | Chemistry | Climate | CS  | Energy | Materials | Math | Medicine | Neuro | Physics |
|---------------------------|---------|-----------|---------|-----|--------|-----------|------|----------|-------|---------|
| GPT-5.5                   | 87      | 87        | 88      | 87  | 89     | 87        | 87   | 88       | 87    | 87      |
| + Claude Code             | 88      | 87        | 88      | 87  | 89     | 88        | 87   | 89       | 87    | 89      |
| Opus 4.7 + Claude Code    | 86      | 84        | 87      | 85  | 87     | 86        | 87   | 86       | 84    | 71      |
| Opus 4.7 + Codex          | 84      | 84        | 86      | 86  | 86     | 85        | 82   | 84       | 83    | 83      |
| DeepSeek-V4-Pro           | 84      |

#### ✓ `assets/p20-fig-01.png` — heatmap (p20)

1. **Title**
   PES rubric decomposition in the Lineage setting

2. **Row labels** (models) and **column labels** (PES components)
   - Row labels:
     GPT-5.5
     + Claude Code
     Opus 4.7 + Claude Code
     Opus 4.7 + Codex
     DeepSeek-V4-Pro
     Claude Opus 4.7
     Gemini 3.1 Pro
     Kimi-K2-Thinking
     GLM-5.1
     AI Scientist v2
     Col-Agent
     + Codex
     Owen3.6-Max
     MiniMax-M2.7
   - Column labels:
     Heredity
     Variation
     Selection

3. **Colour scale meaning**
   - Dark blue → high subscore (~89)
   - Light blue → low subscore (~74)

4. **Hotspot regions**
   - Highest overall: GPT-5.5 in Variation (89.0)
   - Next highest: + Claude Code in Variation (89.7)
   - Lowest overall: MiniMax-M2.7 in Heredity (73.8)

5. **Markdown table**

```markdown
| Model                     | Heredity | Variation | Selection |
|---------------------------|----------|-----------|-----------|
| GPT-5.5                   | 87.7     | 89.0      | 85.8      |
| + Claude Code             | 88.0     | 89.7      | 86.2      |
| Opus 4.7 + Claude Code    | 85.7     | 86.4      | 83.4      |
| Opus 4.7 + Codex          | 85.4     | 85.0      | 82.6      |
| DeepSeek-V4-Pro           | 85.5     | 84.7      | 82.9      |
| Claude Opus 4.7           | 85.5     | 85.7      | 83.0      |
| Gemini 3.1 Pro            | 85.6     | 84.1      | 83.1      |
| Kimi-K2-Thinking          | 84.9     | 84.7      | 82.8      |
| GLM-5.1                   | 85.9     | 83.5      | 82.3      |
| AI Scientist v2           | 82.0     | 83.9      | 80.1      |
| Col-Agent                 | 80.5

#### ✓ `assets/p20-fig-02.png` — heatmap (p20)

1. **Title**
   PES by information setting

2. **Row labels** (models)
   - GPT-5.5
   - + Claude Code
   - Opus 4.7 + Claude Code
   - Opus 4.7 + Codex
   - DeepSeek-V4-0
   - Claude Opus 4.7
   - Gemini 3.1 Pro
   - Kimi-K2-Thinking
   - GLM-5.1
   - AI Scientist v2
   - Col-Agent
   - + Codex
   - Owen3.6-Max
   - MiniMax-MN.2-7B

3. **Column labels** (information settings)
   - Question
   - Library
   - Linesage

4. **Colour scale meaning**
   - Dark blue = high PES (better performance)
   - Light blue = low PES (worse performance)

5. **Hotspot regions**
   - GPT-5.5 across all three settings (highest values: 85.2, 86.9, 87.5)
   - + Claude Code and + Codex rows also show several top-scoring cells

6. **Markdown table**

```markdown
| Model                     | Question | Library | Linesage |
|---------------------------|----------|---------|----------|
| GPT-5.5                   | 85.2     | 86.9    | 87.5     |
| + Claude Code             | 82.9     | 87.4    | 88.0     |
| Opus 4.7 + Claude Code    | 81.4     | 84.3    | 84.2     |
| Opus 4.7 + Codex          | 79.9     | 83.9    | 84.4     |
| DeepSeek-V4-0             | 80.1     | 83.5    | 84.4     |
| Claude Opus 4.7           | 79.0     | 84.2    | 84.7     |
| Gemini 3.1 Pro            | 78.9     | 83.2    | 84.3     |
| Kimi-K2-Thinking          | 77.3     | 81.9    | 84.2     |
| GLM-5.1                   | 76.8     | 82.4    | 82.9     |
| AI Scientist v2           | 78.4     | 81.3    | 82.0     |
| Col-Agent                 | 79.4

#### ✓ `assets/p21-fig-01.png` — bar_chart (p21)

Here is the extracted information from the bar chart:

---

1. **Chart title**
   Generated dynamics distribution by information setting

2. **X-axis label** and tick values
   - Label: Information setting
   - Tick values: Question, Library, Lineage

3. **Y-axis label** and range
   - Label: Generated ideas (%)
   - Range: 0 to 100%

4. **Series/groups** (legend entries)
   - Mutation (dark blue)
   - Adaptive Radiation (teal)
   - Hybridization (orange)
   - Speciation (purple)
   - Niche Competition (light blue)
   - Other (grey)

5. **Key observations**
   - The **Question** setting generates a more diverse distribution of ideas, including significant contributions from Mutation, Adaptive Radiation, and Hybridization.
   - The **Library** and **Lineage** settings are dominated almost entirely by Hybridization, with minimal contributions from other dynamics.
   - Speciation and Niche Competition are only present in the **Question** setting, and even there, they represent a small fraction of the generated ideas.

6. **Data table**

| Information Setting | Mutation (%) | Adaptive Radiation (%) | Hybridization (%) | Speciation (%) | Niche Competition (%) | Other (%) |
|---------------------|--------------|------------------------|-------------------|----------------|-----------------------|-----------|
| Question            | 15           | 30                     | 40                | 5              | 5                     | 5         |
| Library             | 0            | 0                      | 95                | 0              | 0                     | 5         |
| Lineage             | 0            | 0                      | 95                | 0              | 0                     | 5         |

---

#### ✓ `assets/p21-fig-02.png` — bar_chart (p21)

1. **Chart title**
   *Not visible in the provided crop.*

2. **X-axis label** and tick values
   - **Label:** Model names
   - **Tick values:** GSS + Codex, GSS + Claude, GPT-3.5, Opus 4.7, DeepSeek, O4.7 + Codex, Gemini, O4.7 + Claude, Kimi K2, GLM-4, AI Sci, Col, Qwen3 6, MiniMax

3. **Y-axis label** and range
   - **Label:** Subscore
   - **Range:** 72.5 to 92.5

4. **Series/groups** (legend entries)
   - Heredity (blue)
   - Variation (orange)
   - Selection (green)

5. **Key observations**
   - GSS + Codex and GSS + Claude show the highest overall subscores, with Variation (orange) peaking above 90.
   - MiniMax has the lowest subscores across all three categories, especially in Selection (green) and Variation (orange).
   - Most models cluster between 80 and 87.5, with Variation often scoring higher than Heredity and Selection.

6. **Data table**

| Model           | Heredity | Variation | Selection |
|-----------------|----------|-----------|-----------|
| GSS + Codex     | 87.0     | 91.5      | 87.0      |
| GSS + Claude    | 88.0     | 89.5      | 87.5      |
| GPT-3.5         | 85.5     | 88.0      | 83.0      |
| Opus 4.7        | 85.0     | 84.5      | 82.0      |
| DeepSeek        | 84.0     | 86.0      | 81.5      |
| O4.7 + Codex    | 85.5     | 86.5      | 83.0      |
| Gemini          | 85.0     | 87.0      | 82.5      |
| O4.7 + Claude   | 86.0     | 83.5      | 82.0      |
| Kimi K2         | 81.5     | 83.5      | 80.5      |
| GLM-4           | 82.0     | 84.0      | 81.0      |
| AI Sci          | 80.5     | 82.5      | 79.5      |
|

#### ✓ `assets/p22-fig-01.png` — heatmap (p22)

1. **Title**
   *Not explicitly visible in the crop, but the figure appears to show "Exact accuracy (%) across capability axes for various models."*

2. **Row labels** (model names) and **column labels** (capability axes)
   - **Row labels** (top to bottom):
     • GPT-5.5 + Claude Code
     • GPT-5.5 + Codex
     • GPT-5.5
     • AI Scientist v2
     • Col-Agent
     • Opus 4.7 + Claude Code
     • Opus 4.7 + Codex
     • Owen3.6-Max
     • Kimi-K2-Thinking
     • Gemini 3.1 Pro
     • Claude Opus 4.7
     • DeepSeek-V2-Pro
     • GLM-4-1
     • MiniMax-M2.7
   - **Column labels** (left to right):
     • T1 Genome
     • T2 Evolve
     • T3 Trace
     • T4 Verify

3. **Colour scale meaning**
   - Dark blue = high exact accuracy (%)
   - Light blue/white = low exact accuracy (%)
   - Scale runs from 0 % (light) to 40 % (dark).

4. **Hotspot regions – standout cells**
   - **Highest values (> 30 %):**
     • GPT-5.5 + Claude Code – T1 (31.5 %)
     • GPT-5.5 + Codex – T1 (31.8 %)
     • Opus 4.7 + Codex – T1 (34.0 %)
     • Gemini 3.1 Pro – T1 (32.4 %)
   - **Lowest values (< 12 %):**
     • MiniMax-M2.7 – T2 (9.1 %)
     • DeepSeek-V2-Pro – T4 (8.5 %)
     • GLM-4-1 – T4 (8.5 %)

5. **Markdown table reconstruction**

| Model                     | T1 Genome | T2 Evolve | T3 Trace | T4 Verify | Axis mean |
|---------------------------|-----------|-----------|----------|-----------|-----------|
| GPT-5.5 + Claude Code     | 31.5      | 37.9      | 25.3     | 12.7      | 28.5      |
| GPT-5.5 + Codex           | 31.8      | 30.3      | 23.6     | 13.7      | –         |
| GPT-5.5                   | 27.5      | 25.7      | 23.

#### ✓ `assets/p22-fig-02.png` — diagram (p22)

### **1. Main Elements and Their Roles**

| **Element**               | **Role**                                                                                     | **Representation in Diagram**                     |
|---------------------------|---------------------------------------------------------------------------------------------|--------------------------------------------------|
| **Tiers (T1-T4)**         | Different levels or stages in the lineage failure analysis.                                 | Vertical bars on the left (T1 to T4).            |
| **Failed Field Family**   | Categories of failed fields grouped by their nature or domain.                              | Central stacked bars (e.g., dynamics, driver).   |
| **Error Classes (E1-E9)** | Specific types of errors corresponding to the failed fields.                               | Colored bars on the right (E1 to E9).            |
| **Legend (Types)**        | Defines the types of connections and verifications in the lineage failure flow.             | Top-right legend (e.g., failed field, verify).   |

- **Failed Field Families** (Central Bars):
  - **Dynamics** (Red)
  - **Driver** (Dark Red)
  - **Relation/Label** (Orange)
  - **Gene Fates/Mapping** (Light Orange)
  - **Order/Group** (Purple)
  - **Intruder** (Gray)

- **Connection Types** (Legend):
  - **Failed Field** (Black arrow)
  - **Verify Swapped Gene Role** (Blue arrow)
  - **Multi-Contribution Types** (Colored flows)

---

### **2. Relationships and Connections Between Elements**

- **Tiers (T1-T4) → Failed Field Families**:
  - Each tier contributes to failures in specific field families, represented by colored flows.
  - Example: T1 (bottom tier) has strong connections to **Dynamics** and **Driver** failures.

- **Failed Field Families → Error Classes (E1-E9)**:
  - Each failed field family maps to specific error classes via colored flows.
  - Example: **Dynamics** failures (red) primarily map to **E1** (red error class).

- **Cross-Tier and Cross-Error Connections**:
  - Flows often span multiple tiers and error classes, indicating complex failure propagation.
  - Example: Failures in **T4** (top tier) contribute to **E9** (gray) via the **Intruder** field.

---

### **3. Directional Flow**

- **Left to Right Flow**:
  - The diagram shows a **lineage failure flow** from **Tiers (T1-T4)** → **Failed Field Families** → **Error Classes (E1-E9)**.
  - Arrows indicate the direction of failure propagation.

- **Verification Flow**:
  - Blue arrows represent the verification of swapped gene roles, linking specific tiers to error classes.

- **Multi-Contribution Flows**:
  - Colored ribbons represent


### lighrad_2410.05779v3.pdf

**1/6 real figures**

#### ✓ `assets/p03-fig-01.png` — architecture_diagram (p3)

Here is a structured breakdown of the **LightRAG framework** architecture from the provided figure:

---

### 1. **Top-level Components**
The architecture consists of the following named modules/boxes:

- **Graph-based Text Indexing** (left side, yellow box):
  - **D(·)** (Data Indexer)
  - **P(·)** (Processing pipeline for deduplication and profiling)
  - **R(·)** (Entity and Relation Extraction)
- **Index Graph** (central graph structure):
  - Stores entities (e.g., *Beekeeper*, *Honey Bee*) and relationships (e.g., *Observe*, *Manage*).
- **Dual-level Retrieval Paradigm** (right side, blue box):
  - **Query + LLM** (combines user query with a large language model).
  - **Low-level Keys** (e.g., *Entities*, *Farmers*, *Hive*).
  - **High-level Keys** (e.g., *Agriculture*, *Production*, *Environmental Impact*).
- **Original Chunks** (metadata storage):
  - Contains *Entity Name*, *Type*, *Description*, and *Source* (e.g., original text chunks with IDs).

---

### 2. **Data Flow**
The flow of data between components is as follows:

1. **Input Text → Graph-based Text Indexing**:
   - Raw text (e.g., beekeeper practices) is fed into **R(·)** for **Entity & Relation Extraction**.
   - Extracted entities and relations are passed to **P(·)** for **Deduplication** (matching entities) and **LLM Profiling** (enriching descriptions).
   - Processed data is structured into **D(·)** (the indexed database).

2. **Indexed Data → Index Graph**:
   - Entities (e.g., *Beekeeper*) and relationships (e.g., *Observe*) are stored in the **Index Graph** for retrieval.

3. **Query → Dual-level Retrieval Paradigm**:
   - A user **Query** is combined with the **LLM** to generate **low-level keys** (e.g., *Beekeeper*) and **high-level keys** (e.g., *Agriculture*).
   - The **Query + LLM** module retrieves relevant **Entities** and **Relationships** from the **Index Graph**.

4. **Retrieval → Original Chunks**:
   - Retrieved entities/relationships are mapped back to **Original Chunks** (e.g., source text with IDs) for context.

5. **Output**:
   - The system returns **contextually relevant responses** (e.g., beekeeper practices) by leveraging retrieved information.

**Direction**: Left-to-right (text → indexing → retrieval → response) with feedback loops for LLM enrichment.

---

### 3. **

#### ✗ `assets/p13-fig-01.png` — text_block (p13)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p14-fig-01.png` — text_block (p14)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p14-fig-02.png` — text_block (p14)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p15-fig-01.png` — text_block (p15)

*Discarded: text_block (confidence=0.99)*

#### ✗ `assets/p15-fig-02.png` — text_block (p15)

*Discarded: text_block (confidence=0.99)*


### rem_2607.08716v1.pdf

**2/3 real figures**

#### ✗ `assets/p01-fig-01.png` — text_block (p1)

*Discarded: text_block (confidence=0.99)*

#### ✓ `assets/p04-fig-01.png` — architecture_diagram (p4)

### 1. **Top-Level Components**
The diagram consists of the following named boxes/modules:
- **Environment** (trinux + docker container)
- **Action Agent**
  - Capabilities:
    - Role, Act & Reason
    - Analysis
    - Planning
    - Command Generation
    - Execution
- **Agent Orchestration** (run one loop)
  - Steps within the loop:
    1. Accumulate recent context
    2. Trigger condition check (TES)
    3. Call Action Agent LLM
    4. Execute commands in trinux
- **Memory Agent**
  - Subcomponents:
    - LLM (Role: Remember & Retrieve)
    - Two Phases:
      1. Write (Update Memory): Run MemoryAgent to store important info
      2. Read (Randomize Memory): Retrieve relevant info as context
    - Memory Store:
      - Long-term memories
      - Summaries
      - Facts / Preferences
      - Patterns / Lessons

---

### 2. **Data Flow**
#### **Control and Data Flow Directions:**
- **Solid Black Arrows (Data/Control Flow):**
  - **Environment → Action Agent:** Observation (input from the environment).
  - **Action Agent → Environment:** Commands for execution, resulting in a new observation.
  - **Action Agent → Agent Orchestration:** Accumulated context and observations.
  - **Agent Orchestration → Action Agent:** System + base instructions, user input, and observations for analysis, planning, and command generation.
  - **Agent Orchestration → Environment:** Commands to execute in the trinux environment, producing the next observation.
  - **Agent Orchestration → Memory Agent:** Memory context frame (for writing/updating memory).
  - **Memory Agent → Agent Orchestration:** Relevant memory info as context (for reading).

- **Dashed Blue Arrows (Context / Memory Flow):**
  - **Memory Agent ↔ Agent Orchestration:** Bidirectional flow of memory-related data (e.g., context frames for storage and retrieval).

#### **Flow of Information in the Loop:**
1. **Observation** from the **Environment** flows into the **Action Agent**.
2. The **Agent Orchestration** accumulates recent context (last *N* steps).
3. The orchestration checks if the **Memory Agent** should be triggered (every *N* steps via TES).
4. If triggered, the **Memory Agent** is called to update or retrieve memory.
5. The **Action Agent LLM** is called with:
   - System + base instructions
   - User input (e.g., observation, goal)
   - Memory context (if any)
6. The **Action Agent** generates analysis, plans, and commands.
7. Commands are executed in the **Environment**, producing a new

#### ✓ `assets/p04-fig-02.png` — architecture_diagram (p4)

### 1. **Top-Level Components**
- **Input to Memory Agent**
- **Phase 1: Bank Management**
- **Memory Bank** (with subcomponents):
  - **Status**
  - **Knowledge**
  - **Procedural**
- **Phase 2: Context Decision**
- **Action Agent**

---

### 2. **Data Flow**

#### **Phase 1: Bank Management**
- **Input to Memory Agent** → **Phase 1: Bank Management**
  - Task description: recent trajectory (last *N* steps) + current memory bank.
  - System prompt: `PHASE1_SYSTEM`
  - Tools: `use_tools_knowledge`, `use_procedural`, `update_status`, `delete`
- **Phase 1: Bank Management** → **Memory Bank (subcomponents)**
  - Executes tool calls to update:
    - **Status** (current step ongoing status)
    - **Knowledge** (updated knowledge)
    - **Procedural** (updated procedures)
- **Memory Bank (subcomponents)** → **Phase 1: Bank Management**
  - Returns updated memory bank data.

#### **Phase 2: Context Decision**
- **Phase 1 Output** → **Phase 2: Context Decision**
  - Updated memory bank (`status`, `knowledge`, `procedural`) is passed as input.
  - System prompt: `PHASE2_SYSTEM`
  - Tools: `none`
- **Phase 2: Context Decision** → Decision Point
  - Decides whether to intervene based on:
    - `context_for_action` (specific, actionable note for the Action Agent)
    - `no_intervention` (stay silent)
- **Decision Point** → **Action Agent**
  - If `context_for_action`:
    - Sends `context_for_action` + `memory_context` to **Action Agent**.
  - If `no_intervention`:
    - No input is sent to the **Action Agent** (no-op).
- **Action Agent** → **Environment**
  - Acts every step based on input from **Phase 2** or continues without intervention.

---

### 3. **External Interfaces**
- **APIs/Tools**:
  - `use_tools_knowledge`, `use_procedural`, `update_status`, `delete` (used in **Phase 1**).
- **Databases/Memory Stores**:
  - **Memory Bank** (persistent storage for `status`, `knowledge`, and `procedural` data).
- **Users/Agents**:
  - **Input to Memory Agent**: Likely receives input from an external task or environment (e.g., user command or observation).
  - **Action Agent**: Interacts with the environment or downstream tasks based on decisions.

---

### 4. **Key Design Decisions**
1. **Two-Phase

