package io.edgequake.sdk.models;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.List;

/** Provider connection and probe models. */
public class ConnectionModels {

    public static class Connection {
        public String id;
        @JsonProperty("tenant_id") public String tenantId;
        public String slug;
        @JsonProperty("display_name") public String displayName;
        @JsonProperty("api_shape") public String apiShape;
        public String locality;
        @JsonProperty("base_url") public String baseUrl;
        @JsonProperty("auth_scheme") public String authScheme;
        @JsonProperty("key_fingerprint") public String keyFingerprint;
        @JsonProperty("key_configured") public boolean keyConfigured;
        @JsonProperty("timeout_secs") public int timeoutSecs;
        @JsonProperty("allow_private_network") public boolean allowPrivateNetwork;
        public String source;
        @JsonProperty("last_test_ok") public Boolean lastTestOk;
        @JsonProperty("last_test_error") public String lastTestError;
    }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public static class UpsertConnection {
        public String slug;
        @JsonProperty("display_name") public String displayName;
        @JsonProperty("api_shape") public String apiShape;
        @JsonProperty("base_url") public String baseUrl;
        public String locality;
        @JsonProperty("auth_scheme") public String authScheme;
        @JsonProperty("api_key") public String apiKey;
        @JsonProperty("timeout_secs") public Integer timeoutSecs;
        @JsonProperty("allow_private_network") public Boolean allowPrivateNetwork;
    }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public static class ProbeRequest {
        public String shape;
        @JsonProperty("base_url") public String baseUrl;
        public String model;
        @JsonProperty("embedding_model") public String embeddingModel;
        @JsonProperty("api_key") public String apiKey;
        @JsonProperty("auth_scheme") public String authScheme;
        @JsonProperty("allow_private_network") public Boolean allowPrivateNetwork;
        @JsonProperty("expected_dimension") public Integer expectedDimension;
    }

    public static class ProbeResult {
        public boolean ok;
        public String kind;
        @JsonProperty("latency_ms") public int latencyMs;
        public String message;
        public List<String> models;
        @JsonProperty("embedding_dimension") public Integer embeddingDimension;
        @JsonProperty("chat_ok") public boolean chatOk;
        @JsonProperty("embed_ok") public boolean embedOk;
        @JsonProperty("list_ok") public boolean listOk;
    }
}
