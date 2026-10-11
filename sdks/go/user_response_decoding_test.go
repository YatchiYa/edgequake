package edgequake_test

import (
	"context"
	"encoding/json"
	edgequake "github.com/edgequake/edgequake-go"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestUserResponseContracts(t *testing.T) {
	user := `{"user_id":"u1","username":"alice","email":"alice@example.test","role":"viewer"}`
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch {
		case r.URL.Path == "/api/v1/auth/me" || r.Method == http.MethodPost:
			w.Write([]byte(`{"user":` + user + `}`))
		case r.URL.Path == "/api/v1/users":
			w.Write([]byte(`{"users":[` + user + `],"total":1}`))
		default:
			w.Write([]byte(user))
		}
	}))
	defer srv.Close()
	client := edgequake.NewClient(edgequake.WithBaseURL(srv.URL), edgequake.WithMaxRetries(0))
	calls := map[string]func() (*edgequake.UserInfo, error){
		"me": func() (*edgequake.UserInfo, error) { return client.Auth.Me(context.Background()) },
		"create": func() (*edgequake.UserInfo, error) {
			return client.Users.Create(context.Background(), &edgequake.CreateUserParams{Username: "alice", Email: "alice@example.test", Password: "synthetic"})
		},
		"get": func() (*edgequake.UserInfo, error) { return client.Users.Get(context.Background(), "u1") },
	}
	for name, call := range calls {
		t.Run(name, func(t *testing.T) {
			got, err := call()
			if err != nil {
				t.Fatal(err)
			}
			if got.ID != "u1" || got.Username != "alice" || got.Email != "alice@example.test" || got.Role != "viewer" {
				t.Fatalf("lost user fields: %+v", got)
			}
		})
	}
	list, err := client.Users.List(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if len(list.Users) != 1 || list.Users[0].ID != "u1" {
		t.Fatalf("lost listed user: %+v", list)
	}
}

func TestUserResponseLegacyAndInvalidJSON(t *testing.T) {
	var user edgequake.UserInfo
	if err := json.Unmarshal([]byte(`{"id":"legacy","username":"old"}`), &user); err != nil {
		t.Fatal(err)
	}
	if user.ID != "legacy" || user.Username != "old" {
		t.Fatalf("legacy data lost: %+v", user)
	}
	if err := json.Unmarshal([]byte(`{"user":{"user_id":4}}`), &user); err == nil {
		t.Fatal("invalid user ID accepted")
	}
}
