package io.edgequake.sdk

import com.fasterxml.jackson.databind.ObjectMapper
import org.junit.jupiter.api.Test
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.assertTrue

class OperationsManifestTest {
    @Test
    fun everyHttpOperationPathAndGeneratedVerb() {
        val root = findRepo()
        val ops = ObjectMapper().readTree(root.resolve("sdks/operation-manifest.json").toFile())
        val generated = Files.readString(root.resolve("sdks/kotlin/src/main/kotlin/io/edgequake/sdk/resources/OperationsGenerated.kt"))
        val blob = StringBuilder()
        Files.walk(root.resolve("sdks/kotlin/src/main")).use { walk ->
            walk.filter { it.toString().endsWith(".kt") }.forEach { blob.append(Files.readString(it)) }
        }
        for (op in ops) {
            if (op.path("kind").asText() == "websocket") {
                assertTrue(generated.contains(op.path("path").asText()), op.path("path").asText())
                continue
            }
            val path = op.path("path").asText()
            assertTrue(blob.contains(path), path)
            if (op.path("missing").path("kotlin").asBoolean()) {
                assertTrue(generated.contains(op.path("method").asText() + " " + path), path)
            }
        }
    }

    private fun findRepo(): Path {
        var dir: Path? = Path.of("").toAbsolutePath()
        repeat(8) {
            if (dir != null && Files.exists(dir!!.resolve("sdks/operation-manifest.json"))) return dir!!
            dir = dir?.parent
        }
        error("operation-manifest.json not found")
    }
}
