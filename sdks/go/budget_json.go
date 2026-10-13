package edgequake

import "encoding/json"

// UnmarshalJSON maps the server spent_usd field to the existing public spend
// field while accepting legacy current_spend_usd responses.
func (b *BudgetInfo) UnmarshalJSON(data []byte) error {
	type plain BudgetInfo
	var decoded struct {
		plain
		SpentUSD *float64 `json:"spent_usd"`
	}
	if err := json.Unmarshal(data, &decoded); err != nil {
		return err
	}
	if decoded.SpentUSD != nil {
		decoded.plain.CurrentSpendUSD = *decoded.SpentUSD
	}
	*b = BudgetInfo(decoded.plain)
	return nil
}
