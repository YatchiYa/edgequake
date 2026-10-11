package io.edgequake.sdk;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import io.edgequake.sdk.internal.HttpHelper;
import io.edgequake.sdk.models.QueryModels;
import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class OperationsManifestTest {
    @Test
    void everyHttpOperationPathAndGeneratedVerb() throws Exception {
        Path root = findRepo();
        JsonNode ops = new ObjectMapper().readTree(root.resolve("sdks/operation-manifest.json").toFile());
        String generated = Files.readString(root.resolve("sdks/java/src/main/java/io/edgequake/sdk/resources/OperationsGenerated.java"));
        StringBuilder blob = new StringBuilder();
        try (var walk = Files.walk(root.resolve("sdks/java/src/main"))) {
            walk.filter(p -> p.toString().endsWith(".java")).forEach(p -> {
                try { blob.append(Files.readString(p)); } catch (Exception ignored) {}
            });
        }
        for (JsonNode op : ops) {
            if ("websocket".equals(op.path("kind").asText())) {
                assertTrue(generated.contains(op.path("path").asText()), op.path("path").asText());
                continue;
            }
            String path = op.path("path").asText();
            assertTrue(blob.toString().contains(path), path);
            if (op.path("missing").path("java").asBoolean()) {
                assertTrue(generated.contains(op.path("method").asText() + " " + path), path);
            }
        }
    }

    private static Path findRepo() {
        Path dir = Path.of("").toAbsolutePath();
        for (int i = 0; i < 8 && dir != null; i++) {
            if (Files.exists(dir.resolve("sdks/operation-manifest.json"))) return dir;
            dir = dir.getParent();
        }
        throw new IllegalStateException("operation-manifest.json not found");
    }

    @Test
    void nullQueryFieldsStayOffTheWire() throws Exception {
        var helper = new HttpHelper(new EdgeQuakeConfig.Builder().baseUrl("http://127.0.0.1:9").build());
        var json = helper.mapper().writeValueAsString(new QueryModels.QueryRequest("hello", null));
        assertFalse(json.contains("enable_rerank"), json);
        assertFalse(json.contains("\"mode\""), json);
        assertTrue(json.contains("hello"), json);
    }
}
