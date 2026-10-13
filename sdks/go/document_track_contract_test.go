package edgequake_test

import (
	"context"
	"encoding/json"
	edgequake "github.com/edgequake/edgequake-go"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestDocumentTrackBatchContract(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/v1/documents/track/batch1" {
			t.Errorf("unexpected request: %s", r.URL)
		}
		w.Write([]byte(`{"track_id":"batch1","created_at":null,"documents":[{"id":"d1","title":"Report","status":"completed","chunk_count":2,"current_stage":"completed","stage_progress":1,"stage_message":"done"}],"total_count":1,"status_summary":{"pending":0,"processing":0,"completed":1,"partial_failure":0,"failed":0,"cancelled":0,"unknown":0},"is_complete":true,"expected_count":1,"registered_count":1,"latest_message":"finished"}`))
	}))
	defer srv.Close()
	client := edgequake.NewClient(edgequake.WithBaseURL(srv.URL), edgequake.WithMaxRetries(0))
	got, err := client.Documents.Track(context.Background(), "batch1")
	if err != nil {
		t.Fatal(err)
	}
	if got.TrackID != "batch1" {
		t.Fatalf("lost track ID: %+v", got)
	}
	data, err := json.Marshal(got)
	if err != nil {
		t.Fatal(err)
	}
	var fields map[string]any
	if err := json.Unmarshal(data, &fields); err != nil {
		t.Fatal(err)
	}
	if fields["total_count"] != float64(1) || fields["is_complete"] != true || fields["expected_count"] != float64(1) || fields["registered_count"] != float64(1) || fields["latest_message"] != "finished" {
		t.Fatalf("lost batch state: %s", data)
	}
	documents, ok := fields["documents"].([]any)
	if !ok || len(documents) != 1 {
		t.Fatalf("lost documents: %s", data)
	}
	document, ok := documents[0].(map[string]any)
	if !ok || document["id"] != "d1" || document["status"] != "completed" || document["current_stage"] != "completed" || document["stage_progress"] != float64(1) || document["stage_message"] != "done" {
		t.Fatalf("lost document state: %s", data)
	}
	counts, ok := fields["status_summary"].(map[string]any)
	if !ok || counts["completed"] != float64(1) || counts["pending"] != float64(0) {
		t.Fatalf("lost counts: %s", data)
	}
}

func TestDocumentTrackLegacyResponse(t *testing.T) {
	var got edgequake.TrackStatus
	if err := json.Unmarshal([]byte(`{"track_id":"old","status":"processing","progress":0.5,"message":"working","document_id":"d1"}`), &got); err != nil {
		t.Fatal(err)
	}
	if got.TrackID != "old" || got.Status != "processing" || got.Progress == nil || *got.Progress != 0.5 || got.Message != "working" || got.DocumentID != "d1" {
		t.Fatalf("legacy data lost: %+v", got)
	}
}
