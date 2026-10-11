package edgequake_test

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestOperationsManifestPathAndVerb(t *testing.T) {
	root := filepath.Join("..", "..")
	raw, err := os.ReadFile(filepath.Join(root, "sdks", "operation-manifest.json"))
	if err != nil {
		t.Fatal(err)
	}
	var ops []map[string]any
	if err := json.Unmarshal(raw, &ops); err != nil {
		t.Fatal(err)
	}
	generated, err := os.ReadFile(filepath.Join("operations_generated.go"))
	if err != nil {
		t.Fatal(err)
	}
	var blob strings.Builder
	_ = filepath.WalkDir(".", func(path string, d os.DirEntry, err error) error {
		if err != nil || d.IsDir() || !strings.HasSuffix(path, ".go") {
			return nil
		}
		b, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		blob.Write(b)
		return nil
	})
	text := blob.String()
	gen := string(generated)
	for _, op := range ops {
		if op["kind"] == "websocket" {
			path := op["path"].(string)
			if !strings.Contains(gen, path) {
				t.Fatalf("generated websocket missing %s", path)
			}
			continue
		}
		path := op["path"].(string)
		if !strings.Contains(text, path) {
			t.Fatalf("missing path %s", path)
		}
		missing, _ := op["missing"].(map[string]any)
		if missing["go"] == true {
			needle := op["method"].(string) + " " + path
			if !strings.Contains(gen, needle) {
				t.Fatalf("generated method missing %s", needle)
			}
		}
	}
}
