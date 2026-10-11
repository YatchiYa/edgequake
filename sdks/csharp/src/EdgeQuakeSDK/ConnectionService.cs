namespace EdgeQuakeSDK;

public class ConnectionView
{
    public string? Id { get; set; }
    public string? TenantId { get; set; }
    public string? Slug { get; set; }
    public string? DisplayName { get; set; }
    public string? ApiShape { get; set; }
    public string? Locality { get; set; }
    public string? BaseUrl { get; set; }
    public string? AuthScheme { get; set; }
    public string? KeyFingerprint { get; set; }
    public bool KeyConfigured { get; set; }
    public int TimeoutSecs { get; set; }
    public bool AllowPrivateNetwork { get; set; }
    public string? Source { get; set; }
    public bool? LastTestOk { get; set; }
    public string? LastTestError { get; set; }
}

public class ProbeResult
{
    public bool Ok { get; set; }
    public string? Kind { get; set; }
    public int LatencyMs { get; set; }
    public string? Message { get; set; }
    public List<string>? Models { get; set; }
    public int? EmbeddingDimension { get; set; }
    public bool ChatOk { get; set; }
    public bool EmbedOk { get; set; }
    public bool ListOk { get; set; }
}

public class ConnectionService(HttpHelper http)
{
    public Task<List<ConnectionView>> ListAsync() =>
        http.GetAsync<List<ConnectionView>>("/api/v1/connections");

    public Task<ConnectionView> CreateAsync(object body) =>
        http.PostAsync<ConnectionView>("/api/v1/connections", body);

    public Task<ConnectionView> UpdateAsync(string id, object body) =>
        http.PutAsync<ConnectionView>($"/api/v1/connections/{id}", body);

    public Task DeleteAsync(string id) =>
        http.DeleteNoContentAsync($"/api/v1/connections/{id}");

    public Task<ProbeResult> TestAsync(string id) =>
        http.PostAsync<ProbeResult>($"/api/v1/connections/{id}/test", null);

    public Task<ProbeResult> TestProviderAsync(object body) =>
        http.PostAsync<ProbeResult>("/api/v1/providers/test", body);
}
