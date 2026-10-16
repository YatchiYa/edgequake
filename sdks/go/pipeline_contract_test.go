package edgequake_test

import (
	"context"
	"encoding/json"
	edgequake "github.com/edgequake/edgequake-go"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestPipelineServerResponseContracts(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.Path {
		case "/api/v1/pipeline/status":
			w.Write([]byte(`{"is_busy":true,"total_documents":10,"processed_documents":4,"pending_tasks":6,"processing_tasks":2,"completed_tasks":4,"failed_tasks":1,"latest_message":"extracting","history_messages":[{"timestamp":"2026-01-01T00:00:00Z","level":"info","message":"started"}]}`))
		case "/api/v1/pipeline/queue-metrics":
			w.Write([]byte(`{"pending_count":6,"processing_count":2,"active_workers":2,"max_workers":4,"worker_utilization":50,"avg_wait_time_seconds":1.5,"max_wait_time_seconds":3,"throughput_per_minute":8,"estimated_queue_time_seconds":45,"rate_limited":true,"timestamp":"2026-01-01T00:00:00Z","pressure":"elevated","pending_warn_threshold":5,"pending_critical_threshold":10,"operator_action":"wait"}`))
		default:
			t.Errorf("unexpected request: %s", r.URL)
			w.WriteHeader(404)
		}
	}))
	defer srv.Close()
	client := edgequake.NewClient(edgequake.WithBaseURL(srv.URL), edgequake.WithMaxRetries(0))
	t.Run("status", func(t *testing.T) {
		got, err := client.Pipeline.Status(context.Background())
		if err != nil {
			t.Fatal(err)
		}
		if got.ActiveTasks != 2 || got.QueuedTasks != 6 || got.CompletedTasks != 4 || got.FailedTasks != 1 {
			t.Fatalf("lost task counts: %+v", got)
		}
		data, err := json.Marshal(got)
		if err != nil {
			t.Fatal(err)
		}
		var fields map[string]any
		if err := json.Unmarshal(data, &fields); err != nil {
			t.Fatal(err)
		}
		if fields["is_busy"] != true || fields["processed_documents"] != float64(4) || fields["latest_message"] != "extracting" {
			t.Fatalf("lost progress: %s", data)
		}
		history, ok := fields["history_messages"].([]any)
		if !ok || len(history) != 1 {
			t.Fatalf("lost history: %s", data)
		}
	})
	t.Run("metrics", func(t *testing.T) {
		got, err := client.Pipeline.Metrics(context.Background())
		if err != nil {
			t.Fatal(err)
		}
		if got.QueueDepth != 6 || got.Processing != 2 {
			t.Fatalf("lost queue counts: %+v", got)
		}
		data, err := json.Marshal(got)
		if err != nil {
			t.Fatal(err)
		}
		var fields map[string]any
		if err := json.Unmarshal(data, &fields); err != nil {
			t.Fatal(err)
		}
		if fields["active_workers"] != float64(2) || fields["max_workers"] != float64(4) || fields["worker_utilization"] != float64(50) || fields["avg_wait_time_seconds"] != 1.5 || fields["pressure"] != "elevated" || fields["rate_limited"] != true {
			t.Fatalf("lost capacity: %s", data)
		}
	})
}

func TestPipelineLegacyAndCanonicalZero(t *testing.T) {
	var status edgequake.PipelineStatus
	if err := json.Unmarshal([]byte(`{"active_tasks":2,"queued_tasks":3}`), &status); err != nil {
		t.Fatal(err)
	}
	if status.ActiveTasks != 2 || status.QueuedTasks != 3 {
		t.Fatalf("legacy status lost: %+v", status)
	}
	if err := json.Unmarshal([]byte(`{"active_tasks":2,"queued_tasks":3,"processing_tasks":0,"pending_tasks":0}`), &status); err != nil {
		t.Fatal(err)
	}
	if status.ActiveTasks != 0 || status.QueuedTasks != 0 {
		t.Fatalf("canonical status zero ignored: %+v", status)
	}
	var metrics edgequake.QueueMetrics
	if err := json.Unmarshal([]byte(`{"queue_depth":3,"processing":2}`), &metrics); err != nil {
		t.Fatal(err)
	}
	if metrics.QueueDepth != 3 || metrics.Processing != 2 {
		t.Fatalf("legacy metrics lost: %+v", metrics)
	}
	if err := json.Unmarshal([]byte(`{"queue_depth":3,"processing":2,"pending_count":0,"processing_count":0}`), &metrics); err != nil {
		t.Fatal(err)
	}
	if metrics.QueueDepth != 0 || metrics.Processing != 0 {
		t.Fatalf("canonical metrics zero ignored: %+v", metrics)
	}
}
