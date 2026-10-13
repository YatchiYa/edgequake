package edgequake_test

import (
	"context"
	"encoding/json"
	edgequake "github.com/edgequake/edgequake-go"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestCostHistoryServerContract(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/v1/costs/history" || r.URL.Query().Get("start_date") != "2026-01-01" {
			t.Errorf("unexpected request: %s", r.URL)
		}
		w.Write([]byte(`[{"timestamp":"2026-01-02T00:00:00Z","total_cost":1.25,"total_tokens":123,"document_count":4}]`))
	}))
	defer srv.Close()
	client := edgequake.NewClient(edgequake.WithBaseURL(srv.URL), edgequake.WithMaxRetries(0))
	entries, err := client.Costs.History(context.Background(), "2026-01-01", "")
	if err != nil {
		t.Fatal(err)
	}
	if len(entries) != 1 {
		t.Fatalf("wrong entries: %+v", entries)
	}
	got := entries[0]
	if got.Date != "2026-01-02T00:00:00Z" || got.CostUSD != 1.25 || got.Tokens != 123 {
		t.Fatalf("lost history values: %+v", got)
	}
	data, err := json.Marshal(got)
	if err != nil {
		t.Fatal(err)
	}
	var fields map[string]any
	if err := json.Unmarshal(data, &fields); err != nil {
		t.Fatal(err)
	}
	if fields["document_count"] != float64(4) {
		t.Fatalf("lost document count: %s", data)
	}
}

func TestCostHistoryLegacyAndCanonicalZeroValues(t *testing.T) {
	var entry edgequake.CostEntry
	if err := json.Unmarshal([]byte(`{"date":"legacy","cost_usd":2,"tokens":12,"requests":3}`), &entry); err != nil {
		t.Fatal(err)
	}
	if entry.Date != "legacy" || entry.CostUSD != 2 || entry.Tokens != 12 || entry.Requests != 3 {
		t.Fatalf("legacy data lost: %+v", entry)
	}
	if err := json.Unmarshal([]byte(`{"date":"legacy","cost_usd":2,"tokens":12,"timestamp":"","total_cost":0,"total_tokens":0}`), &entry); err != nil {
		t.Fatal(err)
	}
	if entry.Date != "" || entry.CostUSD != 0 || entry.Tokens != 0 {
		t.Fatalf("canonical zero values ignored: %+v", entry)
	}
}
