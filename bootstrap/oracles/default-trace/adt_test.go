package adt

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestCuestaceanLifecycle(t *testing.T) {
	if cuestaceanTraceEnter() != -1 {
		t.Fatal("inactive hook")
	}
	cuestaceanTraceTrials(-1, nil, false, false)
	cuestaceanTraceResults(-1, nil)
	if _, err := CuestaceanTraceStop(); err == nil {
		t.Fatal("inactive stop")
	}
	if err := CuestaceanTraceStart(); err != nil {
		t.Fatal(err)
	}
	if err := CuestaceanTraceStart(); err == nil {
		t.Fatal("overlap")
	}
	a := cuestaceanTraceEnter()
	b := cuestaceanTraceEnter()
	cuestaceanTraceTrials(b, nil, true, true)
	cuestaceanTraceResults(b, nil)
	cuestaceanTraceTrials(a, nil, false, true)
	cuestaceanTraceResults(a, nil)
	batches, err := CuestaceanTraceStop()
	if err != nil || len(batches) != 2 || batches[0].ID != 0 || batches[1].ID != 1 || batches[1].Depth != 1 {
		t.Fatalf("batches: %+v %v", batches, err)
	}
	raw, err := json.Marshal(batches)
	if err != nil || strings.Contains(string(raw), "null") {
		t.Fatalf("empty arrays: %s %v", raw, err)
	}
}

func TestCuestaceanSnapshots(t *testing.T) {
	num := &Num{K: IntKind}
	num.X.SetInt64(-9223372036854775808)
	child := &nodeContext{node: &Vertex{BaseValue: num}}
	child.defaultMode = isDefault
	n := &nodeContext{node: &Vertex{BaseValue: &Vertex{BaseValue: num}}, disjuncts: []*nodeContext{child}}
	n.defaultMode, n.origDefaultMode = notDefault, maybeDefault
	if err := CuestaceanTraceStart(); err != nil {
		t.Fatal(err)
	}
	id := cuestaceanTraceEnter()
	cuestaceanTraceTrials(id, []*nodeContext{n}, false, true)
	child.defaultMode = notDefault
	num.X.SetInt64(7)
	cuestaceanTraceResults(id, []*nodeContext{child})
	b, err := CuestaceanTraceStop()
	if err != nil {
		t.Fatal(err)
	}
	trial := b[0].Trials[0]
	if trial.Mode != "not" || trial.OriginalMode != "maybe" || trial.Value != nil || trial.Nested[0].Mode != "default" || *trial.Nested[0].Value != -9223372036854775808 || *b[0].Results[0].Value != 7 {
		t.Fatalf("snapshot: %+v", b)
	}
	for _, k := range []Kind{FloatKind, IntKind} {
		num.K = k
		if _, _, err := num.X.SetString("9223372036854775808"); err != nil {
			t.Fatal(err)
		}
		if cuestaceanTraceValue(child) != nil {
			t.Fatal("unavailable integer")
		}
	}
}

func TestCuestaceanExactCaps(t *testing.T) {
	if err := CuestaceanTraceStart(); err != nil {
		t.Fatal(err)
	}
	for i := 0; i < 128; i++ {
		if id := cuestaceanTraceEnter(); id != i {
			t.Fatalf("entry %d: %d", i, id)
		}
	}
	for i := 127; i >= 0; i-- {
		cuestaceanTraceTrials(i, nil, false, false)
		cuestaceanTraceResults(i, nil)
	}
	b, err := CuestaceanTraceStop()
	if err != nil || len(b) != 128 || b[127].Depth != 127 {
		t.Fatalf("128 batches: %v %v", b, err)
	}

	n := &nodeContext{}
	vector := make([]*nodeContext, 64)
	for i := range vector {
		vector[i] = n
	}
	// Maximize all three vector types together without exceeding total records.
	trials := append([]*nodeContext{}, vector...)
	trials[0] = &nodeContext{disjuncts: vector}
	if err := CuestaceanTraceStart(); err != nil {
		t.Fatal(err)
	}
	id := cuestaceanTraceEnter()
	cuestaceanTraceTrials(id, trials, false, false)
	cuestaceanTraceResults(id, vector)
	b, err = CuestaceanTraceStop()
	if err != nil || len(b) != 1 || len(b[0].Trials) != 64 || len(b[0].Trials[0].Nested) != 64 || len(b[0].Results) != 64 {
		t.Fatalf("64 vectors: %v %v", b, err)
	}

	for _, overflow := range []bool{false, true} {
		if err := CuestaceanTraceStart(); err != nil {
			t.Fatal(err)
		}
		for i := 0; i < 32; i++ {
			id := cuestaceanTraceEnter()
			cuestaceanTraceTrials(id, vector, false, false)
			cuestaceanTraceResults(id, vector)
		}
		c := cuestaceanCapture
		if c.records != 4096 || c.err != nil || c.depth != 0 {
			t.Fatal("exact record limit", c)
		}
		if overflow {
			id := cuestaceanTraceEnter()
			cuestaceanTraceTrials(id, nil, false, false)
			cuestaceanTraceResults(id, vector[:1])
			sticky := c.err
			retained := len(c.batches)
			id = cuestaceanTraceEnter()
			cuestaceanTraceTrials(id, trials, false, false)
			cuestaceanTraceResults(id, vector)
			if sticky == nil || c.err != sticky || c.records != 4096 || len(c.batches) != retained || c.depth != 0 {
				t.Fatal("overflow not sticky/balanced", c)
			}
		}
		b, err := CuestaceanTraceStop()
		if overflow {
			if err == nil || b != nil {
				t.Fatal("partial overflow report")
			}
		} else if err != nil || len(b) != 32 {
			t.Fatal("4096-record capture", err)
		}
		if cuestaceanCapture != nil {
			t.Fatal("stop did not detach")
		}
	}
	if err := CuestaceanTraceStart(); err != nil {
		t.Fatal(err)
	}
	if b, err := CuestaceanTraceStop(); err != nil || b == nil || len(b) != 0 {
		t.Fatal("overflow recovery", err)
	}
}

func TestCuestaceanFailures(t *testing.T) {
	for _, kind := range []string{"unbalanced", "order", "unknown", "batch", "trials", "nested", "results", "records"} {
		t.Run(kind, func(t *testing.T) {
			if err := CuestaceanTraceStart(); err != nil {
				t.Fatal(err)
			}
			id := cuestaceanTraceEnter()
			n := &nodeContext{}
			vector := make([]*nodeContext, 65)
			for i := range vector {
				vector[i] = n
			}
			switch kind {
			case "unbalanced":
			case "order":
				cuestaceanTraceResults(id, nil)
			case "unknown":
				n.defaultMode = defaultMode(255)
				cuestaceanTraceTrials(id, vector[:1], false, false)
				cuestaceanTraceResults(id, nil)
			case "batch":
				for i := 0; i < 128; i++ {
					cuestaceanTraceEnter()
				}
				for i := 128; i >= 0; i-- {
					cuestaceanTraceResults(i, nil)
				}
			case "trials":
				cuestaceanTraceTrials(id, vector, false, false)
				cuestaceanTraceResults(id, nil)
			case "nested":
				n.disjuncts = vector
				cuestaceanTraceTrials(id, vector[:1], false, false)
				cuestaceanTraceResults(id, nil)
			case "results":
				cuestaceanTraceTrials(id, nil, false, false)
				cuestaceanTraceResults(id, vector)
			case "records":
				c := cuestaceanCapture
				if !c.reserve(64, 4096) || c.reserve(1, 1) {
					t.Fatal("record boundary")
				}
				cuestaceanTraceResults(id, nil)
			}
			b, err := CuestaceanTraceStop()
			if err == nil || b != nil || cuestaceanCapture != nil {
				t.Fatalf("failure leaked: %v %v", b, err)
			}
			if err := CuestaceanTraceStart(); err != nil {
				t.Fatal(err)
			}
			b, err = CuestaceanTraceStop()
			if err != nil || b == nil || len(b) != 0 {
				t.Fatal("recovery", err)
			}
		})
	}
}
