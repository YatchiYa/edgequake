package edgequake

import "encoding/json"

// UnmarshalJSON maps current server task counts into existing public fields.
func (s *PipelineStatus) UnmarshalJSON(data []byte) error {
	type plain PipelineStatus
	var decoded struct {
		plain
		ProcessingTasks *int `json:"processing_tasks"`
		PendingTasks    *int `json:"pending_tasks"`
	}
	if err := json.Unmarshal(data, &decoded); err != nil {
		return err
	}
	if decoded.ProcessingTasks != nil {
		decoded.plain.ActiveTasks = *decoded.ProcessingTasks
	}
	if decoded.PendingTasks != nil {
		decoded.plain.QueuedTasks = *decoded.PendingTasks
	}
	*s = PipelineStatus(decoded.plain)
	return nil
}

// UnmarshalJSON preserves public queue fields and accepts legacy response names.
func (m *QueueMetrics) UnmarshalJSON(data []byte) error {
	type plain QueueMetrics
	var decoded struct {
		plain
		PendingCount    *int `json:"pending_count"`
		ProcessingCount *int `json:"processing_count"`
	}
	if err := json.Unmarshal(data, &decoded); err != nil {
		return err
	}
	if decoded.PendingCount != nil {
		decoded.plain.QueueDepth = *decoded.PendingCount
	}
	if decoded.ProcessingCount != nil {
		decoded.plain.Processing = *decoded.ProcessingCount
	}
	*m = QueueMetrics(decoded.plain)
	return nil
}
