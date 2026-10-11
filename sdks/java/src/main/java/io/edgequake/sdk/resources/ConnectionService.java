package io.edgequake.sdk.resources;

import com.fasterxml.jackson.core.type.TypeReference;
import io.edgequake.sdk.internal.HttpHelper;
import io.edgequake.sdk.models.ConnectionModels.*;

import java.util.List;

/** Provider connections at /api/v1/connections and POST /api/v1/providers/test. */
public class ConnectionService {

    private final HttpHelper http;

    public ConnectionService(HttpHelper http) { this.http = http; }

    public List<Connection> list() {
        return http.get("/api/v1/connections", null, new TypeReference<List<Connection>>() {});
    }

    public Connection create(UpsertConnection body) {
        return http.post("/api/v1/connections", body, Connection.class);
    }

    public Connection update(String id, UpsertConnection body) {
        return http.put("/api/v1/connections/" + id, body, Connection.class);
    }

    public void delete(String id) {
        http.delete("/api/v1/connections/" + id);
    }

    public ProbeResult test(String id) {
        return http.post("/api/v1/connections/" + id + "/test", null, ProbeResult.class);
    }

    public ProbeResult testProvider(ProbeRequest body) {
        return http.post("/api/v1/providers/test", body, ProbeResult.class);
    }
}
