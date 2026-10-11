package edgequake_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	edgequake "github.com/edgequake/edgequake-go"
)

func TestConnections_CreateAndProbe(t *testing.T) {
	mux := http.NewServeMux()
	mux.HandleFunc("/api/v1/connections", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			t.Fatalf("method %s", r.Method)
		}
		var body map[string]any
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatal(err)
		}
		if body["slug"] != "local" || body["api_shape"] != "openai" {
			t.Fatalf("body %+v", body)
		}
		_ = json.NewEncoder(w).Encode(edgequake.Connection{ID: "c1", Slug: "local"})
	})
	mux.HandleFunc("/api/v1/providers/test", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			t.Fatalf("method %s", r.Method)
		}
		var body map[string]any
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatal(err)
		}
		if body["shape"] != "ollama" || body["base_url"] != "http://localhost:11434" {
			t.Fatalf("body %+v", body)
		}
		_ = json.NewEncoder(w).Encode(edgequake.ProbeResult{OK: true, Kind: "ollama", Message: "ok"})
	})
	srv := httptest.NewServer(mux)
	defer srv.Close()

	c := edgequake.NewClient(edgequake.WithBaseURL(srv.URL))
	created, err := c.Connections.Create(context.Background(), edgequake.UpsertConnection{
		Slug: "local", DisplayName: "Local", APIShape: "openai", BaseURL: "http://localhost:11434",
	})
	if err != nil {
		t.Fatal(err)
	}
	if created.ID != "c1" {
		t.Fatalf("id %s", created.ID)
	}
	probe, err := c.Connections.TestProvider(context.Background(), edgequake.ProbeRequest{
		Shape: "ollama", BaseURL: "http://localhost:11434",
	})
	if err != nil {
		t.Fatal(err)
	}
	if !probe.OK || probe.Kind != "ollama" {
		t.Fatalf("%+v", probe)
	}
}
