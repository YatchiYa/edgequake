package edgequake_test

import (
	"context"
	"io"
	"net/http"
	"strings"
	"testing"

	edgequake "github.com/edgequake/edgequake-go"
)

type exportRetryTransport struct {
	t        *testing.T
	attempts int
	prior    *exportRetryBody
}

type exportRetryBody struct {
	io.Reader
	closed bool
}

func (b *exportRetryBody) Close() error { b.closed = true; return nil }
func (tr *exportRetryTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	tr.attempts++
	if tr.prior != nil && !tr.prior.closed {
		tr.t.Error("previous response body is still open when retry starts")
	}
	status, payload := http.StatusServiceUnavailable, `{"error":"busy"}`
	if tr.attempts == 2 {
		status, payload = http.StatusOK, "name\nALICE\n"
	}
	tr.prior = &exportRetryBody{Reader: strings.NewReader(payload)}
	return &http.Response{StatusCode: status, Header: make(http.Header), Body: tr.prior, Request: req}, nil
}

func TestLineageExportClosesResponseBeforeRetry(t *testing.T) {
	transport := &exportRetryTransport{t: t}
	client := edgequake.NewClient(edgequake.WithHTTPClient(&http.Client{Transport: transport}), edgequake.WithMaxRetries(1))
	data, err := client.Lineage.ExportLineage(context.Background(), "document", "csv")
	if err != nil || string(data) != "name\nALICE\n" {
		t.Fatalf("export = %q, %v", data, err)
	}
	if transport.attempts != 2 || !transport.prior.closed {
		t.Fatalf("attempts=%d, final body closed=%v", transport.attempts, transport.prior.closed)
	}
}

type exportReadFailure struct{}

func (exportReadFailure) Read([]byte) (int, error) { return 0, io.ErrUnexpectedEOF }

type exportOneResponseTransport struct {
	body   *exportRetryBody
	status int
}

func (tr exportOneResponseTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	return &http.Response{StatusCode: tr.status, Header: make(http.Header), Body: tr.body, Request: req}, nil
}
func TestLineageExportClosesFinalResponseOnFailure(t *testing.T) {
	for _, status := range []int{http.StatusOK, http.StatusBadRequest} {
		t.Run(http.StatusText(status), func(t *testing.T) {
			body := &exportRetryBody{Reader: exportReadFailure{}}
			client := edgequake.NewClient(edgequake.WithHTTPClient(&http.Client{Transport: exportOneResponseTransport{body: body, status: status}}), edgequake.WithMaxRetries(0))
			_, err := client.Lineage.ExportLineage(context.Background(), "document", "json")
			if err == nil {
				t.Fatal("expected failure")
			}
			if !body.closed {
				t.Fatal("failed response body was not closed")
			}
		})
	}
}
