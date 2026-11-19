package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"cuelang.org/go/cue/ast"
	"cuelang.org/go/cue/token"
)

func TestCuestaceanEqualityWire(t *testing.T) {
	path := filepath.Join(t.TempDir(), "input.cue")
	if err := os.WriteFile(path, []byte("a: *1|2"), 0600); err != nil {
		t.Fatal(err)
	}
	b, err := observe([]string{"trace-equality", path, "a"})
	want := `{"formatVersion":4,"mode":"trace-equality","defaultSelected":true,"status":"concrete","value":1,"batches":[{"id":0,"depth":0,"leftDropsDefault":true,"rightDropsDefault":false,"trials":[{"mode":"maybe","originalMode":"default","leftMode":"maybe","leftOriginalMode":"maybe","value":1,"nested":[]},{"mode":"maybe","originalMode":"not","leftMode":"maybe","leftOriginalMode":"maybe","value":2,"nested":[]}],"results":[{"mode":"default","originalMode":"default","value":1},{"mode":"not","originalMode":"not","value":2}],"phase":{"runMode":"finalize","appends":[{"candidate":{"mode":"default","originalMode":"default","value":1},"index":0,"kind":"insert","comparisons":[]},{"candidate":{"mode":"not","originalMode":"not","value":2},"index":1,"kind":"insert","comparisons":[{"index":0,"kind":"final","existing":{"mode":"default","originalMode":"default","value":1},"accepted":false}]}]}}]}` + "\n"
	if err != nil || string(b) != want {
		t.Fatalf("equality wire: %s %v", b, err)
	}
	var out observation
	if err := json.Unmarshal(b, &out); err != nil {
		t.Fatal(err)
	}
	for i := range out.Batches {
		out.Batches[i].Phase = nil
	}
	out.FormatVersion, out.Mode = 3, "trace-history"
	restored, err := encode(out)
	if err != nil {
		t.Fatal(err)
	}
	old, err := observe([]string{"trace-history", path, "a"})
	if err != nil || string(old) != string(restored) {
		t.Fatal("v3 subset changed", string(old), err)
	}
	again, err := observe([]string{"trace-equality", path, "a"})
	if err != nil || string(again) != want {
		t.Fatal("equality determinism", err)
	}
	if err := os.WriteFile(path, []byte("a: 1"), 0600); err != nil {
		t.Fatal(err)
	}
	b, err = observe([]string{"trace-equality", path, "a"})
	if err != nil || string(b) != `{"formatVersion":4,"mode":"trace-equality","defaultSelected":false,"status":"concrete","value":1,"batches":[]}`+"\n" {
		t.Fatal("empty batches", string(b), err)
	}
}

func TestCuestaceanEqualityEncodedBoundary(t *testing.T) {
	out := observation{FormatVersion: 4, Mode: "trace-equality", Status: "concrete", Value: json.RawMessage(`""`)}
	base, err := encode(out)
	if err != nil {
		t.Fatal(err)
	}
	for _, size := range []int{byteLimit, byteLimit + 1} {
		out.Value = json.RawMessage(`"` + strings.Repeat("x", size-len(base)) + `"`)
		b, err := encode(out)
		if size == byteLimit {
			if err != nil || len(b) != size || b[size-1] != '\n' {
				t.Fatal("v4 exact byte cap", err)
			}
		} else if err == nil || b != nil {
			t.Fatal("v4 one-over byte cap")
		}
	}
}

func TestCuestaceanHistoryWire(t *testing.T) {
	path := filepath.Join(t.TempDir(), "input.cue")
	if err := os.WriteFile(path, []byte("a: *1|2"), 0600); err != nil {
		t.Fatal(err)
	}
	b, err := observe([]string{"trace-history", path, "a"})
	want := `{"formatVersion":3,"mode":"trace-history","defaultSelected":true,"status":"concrete","value":1,"batches":[{"id":0,"depth":0,"leftDropsDefault":true,"rightDropsDefault":false,"trials":[{"mode":"maybe","originalMode":"default","leftMode":"maybe","leftOriginalMode":"maybe","value":1,"nested":[]},{"mode":"maybe","originalMode":"not","leftMode":"maybe","leftOriginalMode":"maybe","value":2,"nested":[]}],"results":[{"mode":"default","originalMode":"default","value":1},{"mode":"not","originalMode":"not","value":2}]}]}` + "\n"
	if err != nil || string(b) != want {
		t.Fatalf("history wire: %s %v", b, err)
	}
	for _, mode := range []string{"trace", "trace-origins"} {
		old, err := observe([]string{mode, path, "a"})
		var out observation
		if err != nil {
			t.Fatal(err)
		}
		if err := json.Unmarshal(old, &out); err != nil {
			t.Fatal(err)
		}
		for _, batch := range out.Batches {
			for _, result := range batch.Results {
				if result.OriginalMode != nil {
					t.Fatal("history leaked into old mode")
				}
			}
		}
	}
}

func TestCuestaceanOriginsWire(t *testing.T) {
	path := filepath.Join(t.TempDir(), "input.cue")
	if err := os.WriteFile(path, []byte("a: *1|2"), 0600); err != nil {
		t.Fatal(err)
	}
	v1, err := observe([]string{"trace", path, "a"})
	if err != nil {
		t.Fatal(err)
	}
	v2, err := observe([]string{"trace-origins", path, "a"})
	if err != nil {
		t.Fatal(err)
	}
	wantTrial := `{"mode":"maybe","originalMode":"default","leftMode":"maybe","leftOriginalMode":"maybe","value":1,"nested":[]}`
	if !strings.HasPrefix(string(v2), `{"formatVersion":2,"mode":"trace-origins",`) || !strings.Contains(string(v2), wantTrial) || strings.Contains(string(v1), "leftMode") || strings.Contains(string(v1), "leftOriginalMode") {
		t.Fatalf("wire: %s / %s", v1, v2)
	}
	var out observation
	if err := json.Unmarshal(v2, &out); err != nil {
		t.Fatal(err)
	}
	for i := range out.Batches {
		for j := range out.Batches[i].Trials {
			trial := &out.Batches[i].Trials[j]
			if trial.LeftMode == nil || trial.LeftOriginalMode == nil {
				t.Fatal("missing v2 fields")
			}
			trial.LeftMode, trial.LeftOriginalMode = nil, nil
		}
	}
	out.FormatVersion, out.Mode = 1, "trace"
	restored, err := encode(out)
	if err != nil || string(restored) != string(v1) {
		t.Fatalf("v1 changed: %s %v", restored, err)
	}
	again, err := observe([]string{"trace-origins", path, "a"})
	if err != nil || string(again) != string(v2) {
		t.Fatal("v2 determinism", err)
	}
}

func TestCuestaceanAdmission(t *testing.T) {
	for _, source := range []string{"a: 0", "package p\na: *-9223372036854775808|+9223372036854775807", "a: (*1|2)&(1|2)"} {
		if _, err := admit([]byte(source), "a"); err != nil {
			t.Fatalf("%s: %v", source, err)
		}
	}
	for _, source := range []string{
		"", "package p", "a: 1\nb: 2", "a: 1\na: 2", "a: int", "a: b", "a: _", "a: true", "a: null", "a: 1.0", "a: 0x1", "a: 1_000", "a: 1e2", "a: 1+2", "a: [1]", "a: {b: 1}", "a?: 1", "a!: 1", "#a: 1", "_a: 1", "\"a\": 1", "a: 1 @foo()", "let x = 1\na: 1", "import \"math\"\na: 1", "a: -(-1)", "a: -(1)", "a: --1", "a: +9223372036854775808", "a: -9223372036854775809", "a: 9223372036854775808",
		"a: " + strings.Repeat("(", 16) + "1" + strings.Repeat(")", 16),
	} {
		if _, err := admit([]byte(source), "a"); err == nil {
			t.Errorf("admitted %s", source)
		}
	}
	for _, label := range []string{"", "_", "int", "bool", "string", "bytes", "number", "float", "_x", "a.b", "é", strings.Repeat("a", 65)} {
		if labelOK(label) {
			t.Errorf("label %q", label)
		}
	}
	if !labelOK("A_1") || !labelOK(strings.Repeat("a", 64)) {
		t.Fatal("valid labels")
	}
	// Explicit grouping isolates node count from the depth guard.
	var tree func(int) string
	tree = func(d int) string {
		if d == 0 {
			return "1"
		}
		return "(" + tree(d-1) + "|" + tree(d-1) + ")"
	}
	// Five levels: 94 nodes, depth 11; six: 190 nodes, depth 13.
	if _, err := admit([]byte("a: "+tree(5)), "a"); err != nil {
		t.Fatal(err)
	}
	if _, err := admit([]byte("a: "+tree(6)), "a"); err == nil {
		t.Fatal("node cap")
	}
	var balanced func(int) ast.Expr
	balanced = func(d int) ast.Expr {
		if d == 0 {
			return &ast.BasicLit{Kind: token.INT, Value: "1"}
		}
		return &ast.BinaryExpr{Op: token.OR, X: balanced(d - 1), Y: balanced(d - 1)}
	}
	var e ast.Expr = balanced(6) // 127 nodes.
	for want := 127; want <= 129; want++ {
		nodes := 0
		if ok := expression(e, 1, &nodes); ok != (want <= 128) || nodes != want {
			t.Fatalf("node boundary %d: %d %v", want, nodes, ok)
		}
		e = &ast.ParenExpr{X: e}
	}
	if _, err := admit([]byte("a: "+strings.Repeat("(", 15)+"1"+strings.Repeat(")", 15)), "a"); err != nil {
		t.Fatal("depth boundary", err)
	}
}

func TestCuestaceanEncodedBoundary(t *testing.T) {
	// Exercise only the encoder's byte guard, not finite-source admission.
	out := observation{FormatVersion: 1, Mode: "trace", Status: "concrete", Value: json.RawMessage(`""`)}
	base, err := encode(out)
	if err != nil {
		t.Fatal(err)
	}
	for _, size := range []int{byteLimit, byteLimit + 1} {
		out.Value = json.RawMessage(`"` + strings.Repeat("x", size-len(base)) + `"`)
		b, err := encode(out)
		if size == byteLimit {
			if err != nil || len(b) != size || b[len(b)-1] != '\n' {
				t.Fatalf("exact encoded limit: %d %v", len(b), err)
			}
		} else if err == nil || b != nil {
			t.Fatal("one-over encoded limit")
		}
	}
	if b, err := encode(observation{Value: json.RawMessage(`1`)}); err != nil || len(b) == 0 {
		t.Fatal("encoder recovery", err)
	}
}

func TestCuestaceanObservation(t *testing.T) {
	path := filepath.Join(t.TempDir(), "input.cue")
	for _, row := range []struct {
		source, status, value string
		selected              bool
	}{
		{"a: *1|2", "concrete", "1", true},
		{"a: 1|2", "incomplete", "null", false},
		{"a: 1&2", "conflict", "null", false},
		{"a: -9223372036854775808", "concrete", "-9223372036854775808", false},
	} {
		if err := os.WriteFile(path, []byte(row.source), 0600); err != nil {
			t.Fatal(err)
		}
		b, err := observe([]string{"trace", path, "a"})
		if err != nil {
			t.Fatal(row.source, err)
		}
		var out observation
		if err := json.Unmarshal(b, &out); err != nil {
			t.Fatal(err)
		}
		if out.Status != row.status || string(out.Value) != row.value || out.DefaultSelected != row.selected || out.Batches == nil || b[len(b)-1] != '\n' {
			t.Fatalf("unexpected %s", b)
		}
		if row.source == "a: *1|2" && (len(out.Batches) != 1 || len(out.Batches[0].Trials) != 2 || len(out.Batches[0].Results) != 2) {
			t.Fatal("missing trace hooks")
		}
		again, err := observe([]string{"trace", path, "a"})
		if err != nil || string(b) != string(again) {
			t.Fatal("nondeterministic", err)
		}
	}
	for _, args := range [][]string{nil, {"inspect", path, "a"}, {"trace", path}, {"trace", path, "a", "extra"}, {"trace", path, "b"}, {"trace", path, "a.b"}, {"trace", path + "missing", "a"}} {
		if b, err := observe(args); err == nil || b != nil {
			t.Fatalf("setup leaked %q %v", b, err)
		}
	}
	if err := os.WriteFile(path, []byte(strings.Repeat(" ", byteLimit+1)), 0600); err != nil {
		t.Fatal(err)
	}
	if b, err := observe([]string{"trace", path, "a"}); err == nil || b != nil {
		t.Fatal("source cap")
	}
	if b, err := encode(observation{Value: json.RawMessage(`"` + strings.Repeat("x", byteLimit) + `"`)}); err == nil || b != nil {
		t.Fatal("encoded cap")
	}
	if b, err := encode(observation{Value: json.RawMessage(`invalid`)}); err == nil || b != nil {
		t.Fatal("encoding failure")
	}
}
