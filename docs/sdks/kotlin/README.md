<<<<<<< HEAD
# Kotlin SDK

> **Product: v0.19.0** · Contract: [`openapi.snapshot.json`](../../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../../ingestion-cancel-and-fairness.md)
=======
---
title: "Kotlin SDK"
---

# Kotlin SDK

> **Product: v0.23.0** · Contract: [`openapi.snapshot.json`](../../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../../ingestion-cancel-and-fairness.md)
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

**Location:** `sdks/kotlin`  
**Build:** Maven only (`pom.xml`) — no Gradle wrapper in this repo.

## Maven dependency

```xml
<dependency>
    <groupId>io.edgequake</groupId>
    <artifactId>edgequake-sdk-kotlin</artifactId>
    <version>0.4.0</version>
</dependency>
```

Install to local Maven repo from source:

```bash
cd sdks/kotlin && mvn install -DskipTests
```

## Example

```kotlin
import io.edgequake.sdk.EdgeQuakeClient
import io.edgequake.sdk.EdgeQuakeConfig

fun main() {
    val client = EdgeQuakeClient(
        EdgeQuakeConfig(
            baseUrl = "http://localhost:8080",
            apiKey = System.getenv("EDGEQUAKE_API_KEY"),
            tenantId = System.getenv("EDGEQUAKE_TENANT_ID"),
            userId = System.getenv("EDGEQUAKE_USER_ID"),
            workspaceId = System.getenv("EDGEQUAKE_WORKSPACE_ID") ?: "default",
        )
    )

    val health = client.health.check()
    println(health.status)  // healthy

    val result = client.query.execute("What is EdgeQuake?")
    println(result.answer)
    result.sources.forEach { src ->
        println("${src.score} ${src.snippet?.take(80)}")
    }
}
```

Query responses expose **`answer`** and **`sources`** (not top-level chunks/entities).

## Build & test

```bash
cd sdks/kotlin && mvn test
```

E2E tests (requires running API):

```bash
cd sdks/kotlin && mvn test -Pe2e
```

## Lawful bulk delete

`client.conversations.bulkDelete(listOf("id-1", "id-2"))` posts `{"conversation_ids":[...]}` and reads `affected` from the JSON body.

<<<<<<< HEAD
=======
## v0.23 notes

- Document responses include `display_status` / `ui_phase` (SPEC-057 P4) — prefer them over raw `status`/`stage` for progress UI.
- **Stateless parse (SPEC-094):** no typed wrapper yet — raw HTTP `POST /api/v1/parse` (multipart; sync ≤ 15 pages / 20 MiB, async ≤ 1000 pages) + `GET /api/v1/parse/backends` + `GET /api/v1/parse/jobs/{id}`.
- **LLM cache (server-side):** `EDGEQUAKE_LLM_CACHE=1` default; `EDGEQUAKE_KEYWORD_CACHE` / `EDGEQUAKE_QUERY_ANSWER_CACHE` override — no client change.

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
## See also

- Full feature list: `sdks/kotlin/README.md`
- [SDK index](../README.md)
- [Custom Clients](../../integrations/custom-clients.md)
