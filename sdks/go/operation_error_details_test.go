package edgequake_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	edgequake "github.com/edgequake/edgequake-go"
)

func TestGeneratedOperationPreservesStructuredAPIError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/ps" {
			t.Errorf("path = %s", r.URL.Path)
		}
		w.WriteHeader(http.StatusForbidden)
		_, _ = w.Write([]byte(`{"error":"quota_exhausted","message":"daily limit reached","details":"try tomorrow"}`))
	}))
	defer server.Close()
	client := edgequake.NewClient(edgequake.WithBaseURL(server.URL), edgequake.WithMaxRetries(0))
	_, err := client.Operations.OllamaPs(context.Background(), nil, nil)
	var apiError *edgequake.APIError
	if !errors.As(err, &apiError) {
		t.Fatalf("error = %v", err)
	}
	if apiError.ErrorCode != "quota_exhausted" || apiError.Message != "daily limit reached" || apiError.Details != "try tomorrow" {
		t.Fatalf("lost structured details: %+v", apiError)
	}
	if !errors.Is(err, edgequake.ErrForbidden) {
		t.Fatalf("missing status sentinel: %v", err)
	}
}

func TestGeneratedOperationPreservesUnstructuredErrorBody(t *testing.T) {
	for _, payload := range []string{"upstream unavailable", "{broken JSON", ""} {
		t.Run(payload, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
				w.WriteHeader(http.StatusBadGateway)
				_, _ = w.Write([]byte(payload))
			}))
			defer server.Close()
			client := edgequake.NewClient(edgequake.WithBaseURL(server.URL), edgequake.WithMaxRetries(0))
			_, err := client.Operations.OllamaPs(context.Background(), nil, nil)
			var apiError *edgequake.APIError
			if !errors.As(err, &apiError) || apiError.ErrorCode != "Bad Gateway" || apiError.Message != payload {
				t.Fatalf("error = %+v", apiError)
			}
		})
	}
}
