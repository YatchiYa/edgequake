package edgequake

import "testing"

func TestParseEventsAndSocketURL(t *testing.T) {
	text := "data: {\"a\":1}\n\ndata: [DONE]\n\n: keep-alive\n\ndata: hi\n"
	if got := parseEvents(text); got != "{\"a\":1}\nhi" {
		t.Fatalf("events %q", got)
	}
	if got := socketURL("http://localhost:8080/", "/ws/pipeline/progress"); got != "ws://localhost:8080/ws/pipeline/progress" {
		t.Fatalf("socket %q", got)
	}
	if got := socketURL("https://api.example", "/ws/progress/t"); got != "wss://api.example/ws/progress/t" {
		t.Fatalf("socket %q", got)
	}
}
