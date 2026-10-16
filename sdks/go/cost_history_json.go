package edgequake

import "encoding/json"

// UnmarshalJSON reads the server's cost history names without breaking existing
// Date, CostUSD, and Tokens fields or legacy response fixtures.
func (e *CostEntry) UnmarshalJSON(data []byte) error {
	type plain CostEntry
	var decoded struct {
		plain
		Timestamp   *string  `json:"timestamp"`
		TotalCost   *float64 `json:"total_cost"`
		TotalTokens *int64   `json:"total_tokens"`
	}
	if err := json.Unmarshal(data, &decoded); err != nil {
		return err
	}
	if decoded.Timestamp != nil {
		decoded.plain.Date = *decoded.Timestamp
	}
	if decoded.TotalCost != nil {
		decoded.plain.CostUSD = *decoded.TotalCost
	}
	if decoded.TotalTokens != nil {
		decoded.plain.Tokens = *decoded.TotalTokens
	}
	*e = CostEntry(decoded.plain)
	return nil
}
