package edgequake_test

import (
	"encoding/json"
	edgequake "github.com/edgequake/edgequake-go"
	"testing"
)

func TestUserResponseNullEnvelopeRetainsBareUser(t *testing.T) {
	for _, body := range []string{
		`{"user": null, "user_id": "canonical-user", "username": "rudy"}`,
		`{"user": null, "id": "canonical-user", "username": "rudy"}`,
	} {
		var got edgequake.UserInfo
		if err := json.Unmarshal([]byte(body), &got); err != nil {
			t.Fatal(err)
		}
		if got.ID != "canonical-user" || got.Username != "rudy" {
			t.Fatalf("null envelope discarded bare user: %+v", got)
		}
	}
}
