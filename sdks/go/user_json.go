package edgequake

import (
	"bytes"
	"encoding/json"
)

// UnmarshalJSON accepts the server user_id field and the user envelopes returned
// by auth/me and user creation, while retaining legacy bare id responses.
func (u *UserInfo) UnmarshalJSON(data []byte) error {
	var envelope struct {
		User json.RawMessage `json:"user"`
	}
	if err := json.Unmarshal(data, &envelope); err != nil {
		return err
	}
	if len(envelope.User) > 0 && !bytes.Equal(bytes.TrimSpace(envelope.User), []byte("null")) {
		data = envelope.User
	}
	type plain UserInfo
	var decoded struct {
		plain
		UserID string `json:"user_id"`
	}
	if err := json.Unmarshal(data, &decoded); err != nil {
		return err
	}
	if decoded.UserID != "" {
		decoded.plain.ID = decoded.UserID
	}
	*u = UserInfo(decoded.plain)
	return nil
}
