package edgequake_test

import (
	"context"
	"encoding/json"
	edgequake "github.com/edgequake/edgequake-go"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestBudgetServerContract(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/v1/costs/budget" {
			t.Errorf("unexpected request: %s", r.URL)
		}
		w.Write([]byte(`{"monthly_budget_usd":100,"spent_usd":120,"remaining_usd":-20,"alert_threshold":80,"is_over_budget":true}`))
	}))
	defer srv.Close()
	client := edgequake.NewClient(edgequake.WithBaseURL(srv.URL), edgequake.WithMaxRetries(0))
	got, err := client.Costs.Budget(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if got.CurrentSpendUSD != 120 || got.MonthlyBudgetUSD == nil || *got.MonthlyBudgetUSD != 100 || got.RemainingUSD == nil || *got.RemainingUSD != -20 {
		t.Fatalf("lost budget values: %+v", got)
	}
	data, err := json.Marshal(got)
	if err != nil {
		t.Fatal(err)
	}
	var fields map[string]any
	if err := json.Unmarshal(data, &fields); err != nil {
		t.Fatal(err)
	}
	if fields["alert_threshold"] != float64(80) || fields["is_over_budget"] != true {
		t.Fatalf("lost alert state: %s", data)
	}
}

func TestBudgetLegacyAndCanonicalZero(t *testing.T) {
	var budget edgequake.BudgetInfo
	if err := json.Unmarshal([]byte(`{"current_spend_usd":9}`), &budget); err != nil {
		t.Fatal(err)
	}
	if budget.CurrentSpendUSD != 9 {
		t.Fatalf("legacy spend lost: %+v", budget)
	}
	if err := json.Unmarshal([]byte(`{"current_spend_usd":9,"spent_usd":0}`), &budget); err != nil {
		t.Fatal(err)
	}
	if budget.CurrentSpendUSD != 0 {
		t.Fatalf("canonical zero ignored: %+v", budget)
	}
}
