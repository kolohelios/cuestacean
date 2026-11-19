package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func probe(t *testing.T, source string, args ...string) ([]byte, error) {
	t.Helper()
	path := filepath.Join(t.TempDir(), "raw.cue")
	if err := os.WriteFile(path, []byte(source), 0600); err != nil {
		t.Fatal(err)
	}
	return observe(append([]string{args[0], path}, args[1:]...))
}

func TestObservations(t *testing.T) {
	for _, tc := range []struct {
		source, mode, flags, status, value string
	}{
		{"a: *1|2", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":true", "concrete", "1"},
		{"a: 1|2", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":false", "incomplete", "null"},
		{"a: (*1|2)&(1|*2)", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":false", "incomplete", "null"},
		{"a: *1|*2|3", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":true", "incomplete", "null"},
		{"a: *int|1", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":true", "incomplete", "null"},
		{"a: 1&2", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":false", "conflict", "null"},
		{"a: null", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":false", "concrete", "null"},
		{"a: -9223372036854775808", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":false", "concrete", "-9223372036854775808"},
		{"a: 9223372036854775807", "inspect", "null,\"rightDefault\":null,\"defaultSelected\":false", "concrete", "9223372036854775807"},
		{"a: *1|2\nb: 1|*2", "unify", "null,\"rightDefault\":null,\"defaultSelected\":false", "incomplete", "null"},
		{"a: *1|2\nb: 1|*2", "default-unify", "true,\"rightDefault\":true,\"defaultSelected\":false", "conflict", "null"},
	} {
		args := []string{tc.mode, "a"}
		if tc.mode != "inspect" {
			args = append(args, "b")
		}
		got, err := probe(t, tc.source, args...)
		want := "{\"formatVersion\":1,\"mode\":\"" + tc.mode + "\",\"leftDefault\":" + tc.flags + ",\"status\":\"" + tc.status + "\",\"value\":" + tc.value + "}\n"
		if err != nil || string(got) != want {
			t.Errorf("%s %s: got %s, %v; want %s", tc.mode, tc.source, got, err, want)
		}
	}
}

// These expectations are handwritten public-API controls, not generated dumps.
// Columns preserve each direction/profile independently; acceptance is not an
// equivalence certificate, and false does not prove mathematical non-subsumption.
func TestRelationObservations(t *testing.T) {
	for _, tc := range []struct {
		name, left, right                    string
		ordinaryLR, ordinaryRL, rawLR, rawRL bool
	}{
		{"plain-marked", "1|2", "*1|2", true, true, false, true},
		{"opposite-defaults", "*1|2", "1|*2", true, true, false, false},
		{"flat-conflict", "(*1|2)&(1|*2)", "1|2", true, true, true, true},
		{"retained", "(*1|2)&1", "1", true, true, true, true},
		{"eliminated", "(*1|2)&2", "2", true, true, true, true},
		{"multiple", "1|2|3", "*1|*2|3", true, true, false, true},
		{"abstract", "*int|1", "int", true, true, true, true},
		{"top", "_", "int", true, false, true, false},
		{"minimum", "int", "-9223372036854775808", true, false, true, false},
		{"maximum", "int", "9223372036854775807", true, false, true, false},
		{"false", "false", "false", true, true, true, true},
		{"null", "null", "null", true, true, true, true},
		{"distinct", "1", "2", false, false, false, false},
		{"bottom", "int", "1&2", true, false, true, false},
		{"bottom-reflexive", "1&2", "1&2", true, true, true, true},
		{"union-reflexive", "1|2", "1|2", true, true, true, true},
		{"wrapper", "(*1|2)|*3", "1|*2", true, false, false, false},
		{"nested-wrapper", "*((*1|2)&(1|*2))|3", "*3|1|2", true, true, false, true},
		{"wrapper-union", "(*1|2)|(*2|1)", "*1|2", true, true, true, false},
		{"regression", "*((*1|2)|3)|(*4|5)", "1|4", true, true, true, false},
	} {
		t.Run(tc.name, func(t *testing.T) {
			source := "package relations\nnest: { left: (" + tc.left + "), right: (" + tc.right + ") }\nunrelated: 1&2\n"
			for _, call := range []struct {
				mode, super, sub string
				accepted         bool
			}{
				{"subsume", "nest.left", "nest.right", tc.ordinaryLR},
				{"subsume", "nest.right", "nest.left", tc.ordinaryRL},
				{"raw-subsume", "nest.left", "nest.right", tc.rawLR},
				{"raw-subsume", "nest.right", "nest.left", tc.rawRL},
			} {
				// probe creates a new file/build for every call and repetition.
				for range 2 {
					got, err := probe(t, source, call.mode, call.super, call.sub)
					accepted := "false"
					if call.accepted {
						accepted = "true"
					}
					want := "{\"formatVersion\":2,\"mode\":\"" + call.mode + "\",\"accepted\":" + accepted + "}\n"
					if err != nil || string(got) != want {
						t.Errorf("%s %s %s: got %s, %v; want %s", call.mode, call.super, call.sub, got, err, want)
					}
				}
			}
		})
	}
}

func TestRelationSetupAndAdmission(t *testing.T) {
	for _, mode := range []string{"subsume", "raw-subsume"} {
		t.Run(mode, func(t *testing.T) {
			for _, args := range [][]string{
				{mode}, {mode, "missing"}, {mode, "missing", "a"},
				{mode, "missing", "a", "b", "extra"}, {mode, "missing", "a", "b"},
			} {
				if got, err := observe(args); err == nil || got != nil {
					t.Errorf("admitted args %v: %s, %v", args, got, err)
				}
			}
			for _, selector := range []string{"", "a[", "missing", strings.Repeat("x", 257), string([]byte{0xff}), "nest"} {
				for _, paths := range [][2]string{{selector, "a"}, {"a", selector}} {
					if got, err := probe(t, "a: 1\nnest: {b: 1}", mode, paths[0], paths[1]); err == nil || got != nil {
						t.Errorf("admitted selectors %q: %s, %v", paths, got, err)
					}
				}
			}
			for _, source := range []string{
				"a: 1\nb: a", "a: 1\nunrelated: missing", "a: 1\nimport \"math\"",
				"a: 1\nnest: {int: 1}", "a: 1\nnest: {\"bool\": 1}",
				"a: 1\nb: [1]", "a: 1\nb: >=1", "a: (",
			} {
				if got, err := probe(t, source, mode, "a", "a"); err == nil || got != nil {
					t.Errorf("admitted %q: %s, %v", source, got, err)
				}
			}
			for name := range predeclared {
				for _, label := range []string{name, "\"" + name + "\""} {
					for _, source := range []string{label + ": 1\na: 1", "nest: {" + label + ": 1}\na: 1"} {
						if got, err := probe(t, source, mode, "a", "a"); err == nil || got != nil {
							t.Errorf("admitted shadow %q: %s, %v", source, got, err)
						}
					}
				}
			}
			name := strings.Repeat("x", 256)
			for _, paths := range [][2]string{{name, "a"}, {"a", name}} {
				if _, err := probe(t, name+": 1\na: 1", mode, paths[0], paths[1]); err != nil {
					t.Fatal(err)
				}
			}
			for _, size := range []int{sourceLimit, sourceLimit + 1} {
				got, err := probe(t, "a: 1"+strings.Repeat(" ", size-4), mode, "a", "a")
				if (err != nil) != (size > sourceLimit) || (err != nil && got != nil) {
					t.Errorf("source size %d: %s, %v", size, got, err)
				}
			}
		})
	}
}

func TestAdmission(t *testing.T) {
	for _, source := range []string{
		"import \"math\"\na: 1", "a: missing", "left: 1\nright: 2\na: left & right",
		"a: { b: 1 }\nz: a.b", "a: [1]", "a: len(\"x\")", "a: 1+2", "a: >=1", "a: _|_",
		"a?: 1", "a!: 1", "#A: 1", "_hidden: 1", "a: 1 @tag()", "a: {let x=1, b: x}",
		"a: {1}", "a: { [string]: 1 }", "a: { for x in [1] { b: x } }", "a: -(int)",
	} {
		if got, err := probe(t, source, "inspect", "a"); err == nil || got != nil {
			t.Errorf("admitted %q: %s, %v", source, got, err)
		}
	}
	for name := range predeclared {
		for _, label := range []string{name, "\"" + name + "\""} {
			for _, source := range []string{label + ": 1\na: 1", "nest: {" + label + ": 1}\na: 1"} {
				if _, err := probe(t, source, "inspect", "a"); err == nil {
					t.Errorf("admitted shadow: %s", source)
				}
			}
		}
	}
	for _, source := range []string{"package probe\na: 1\nb: 1&2", "a: 1\nb: 1&2", "a: { child: *1|2 }", "a: +1", "a: -(1.5)"} {
		selector := "a"
		if strings.Contains(source, "child") {
			selector = "a.child"
		}
		if _, err := probe(t, source, "inspect", selector); err != nil {
			t.Errorf("rejected %s: %v", source, err)
		}
	}
}

func TestSetupAndLimits(t *testing.T) {
	for _, args := range [][]string{nil, {"unknown"}, {"inspect"}, {"inspect", "missing", "a", "extra"}} {
		if got, err := observe(args); err == nil || got != nil {
			t.Errorf("admitted args %v", args)
		}
	}
	for _, selector := range []string{"", "a[", "missing", strings.Repeat("x", 257)} {
		if _, err := probe(t, "a: 1", "inspect", selector); err == nil {
			t.Errorf("admitted selector %q", selector)
		}
	}
	name := strings.Repeat("x", 256)
	if _, err := probe(t, name+": 1", "inspect", name); err != nil {
		t.Fatal(err)
	}
	for _, size := range []int{sourceLimit, sourceLimit + 1} {
		_, err := probe(t, "a: 1"+strings.Repeat(" ", size-4), "inspect", "a")
		if (err != nil) != (size > sourceLimit) {
			t.Errorf("source size %d: %v", size, err)
		}
	}
	if _, err := probe(t, "a: {b: 1}", "inspect", "a"); err == nil {
		t.Fatal("selected container")
	}
}
