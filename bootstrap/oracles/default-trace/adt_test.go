package adt

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestCuestaceanOrigins(t *testing.T) {
	cuestaceanTraceOrigin(-1, nil, 1) // Inactive must not inspect p.
	if err := CuestaceanTraceStart(); err != nil {
		t.Fatal(err)
	}
	cuestaceanTraceOrigin(-1, nil, 1) // V1 must not validate or charge.
	if b, err := CuestaceanTraceStop(); err != nil || len(b) != 0 {
		t.Fatal(b, err)
	}
	for _, failure := range []string{"", "width", "records", "priority", "leftPriority", "originPriority", "missing", "extra", "order", "stage", "unknown"} {
		t.Run(failure, func(t *testing.T) {
			if err := CuestaceanTraceOriginsStart(); err != nil {
				t.Fatal(err)
			}
			if err := CuestaceanTraceStart(); err == nil {
				t.Fatal("overlap")
			}
			p := &nodeContext{}
			p.defaultMode, p.origDefaultMode = notDefault, isDefault
			r := &nodeContext{}
			r.defaultMode, r.origDefaultMode = maybeDefault, notDefault
			vector := make([]*nodeContext, 64)
			for i := range vector {
				vector[i] = r
			}
			id := cuestaceanTraceEnter()
			switch failure {
			case "", "records":
				// 16 batches * (64 origins*2 + 64 trials + 64 results).
				for batch := 0; batch < 16; batch++ {
					if batch != 0 {
						id = cuestaceanTraceEnter()
					}
					for range vector {
						cuestaceanTraceOrigin(id, p, 0)
					}
					cuestaceanTraceTrials(id, vector, false, false)
					cuestaceanTraceResults(id, vector)
				}
				if cuestaceanCapture.records != 4096 || cuestaceanCapture.err != nil {
					t.Fatal("exact v2 cap", cuestaceanCapture)
				}
				if failure == "records" {
					id = cuestaceanTraceEnter()
					cuestaceanTraceOrigin(id, p, 0)
				}
			case "width":
				for i := 0; i < 65; i++ {
					cuestaceanTraceOrigin(id, p, 0)
				}
			case "priority":
				cuestaceanTraceOrigin(id, p, 1)
			case "leftPriority":
				p.priority = 1
				cuestaceanTraceOrigin(id, p, 0)
			case "originPriority":
				p.origPriority = 1
				cuestaceanTraceOrigin(id, p, 0)
			case "missing":
				cuestaceanTraceTrials(id, vector[:1], false, false)
			case "extra":
				cuestaceanTraceOrigin(id, p, 0)
				cuestaceanTraceTrials(id, nil, false, false)
			case "order":
				cuestaceanTraceEnter()
				cuestaceanTraceOrigin(id, p, 0)
			case "stage":
				cuestaceanTraceTrials(id, nil, false, false)
				cuestaceanTraceOrigin(id, p, 0)
			case "unknown":
				p.origDefaultMode = defaultMode(255)
				cuestaceanTraceOrigin(id, p, 0)
			}
			c := cuestaceanCapture
			if failure != "" {
				sticky, records := c.err, c.records
				if sticky == nil {
					t.Fatal("missing failure")
				}
				cuestaceanTraceOrigin(id, nil, 0)
				cuestaceanTraceTrials(id, vector, false, false)
				if c.err != sticky || c.records != records {
					t.Fatal("not sticky")
				}
			}
			b, err := CuestaceanTraceStop()
			if failure == "" {
				if err != nil || len(b) != 16 || len(b[0].Trials) != 64 {
					t.Fatal(b, err)
				}
				trial := b[0].Trials[0]
				if trial.Mode != "maybe" || trial.OriginalMode != "not" || *trial.LeftMode != "not" || *trial.LeftOriginalMode != "default" {
					t.Fatal(trial)
				}
			} else if err == nil || b != nil {
				t.Fatal("partial report", b, err)
			}
			if cuestaceanCapture != nil {
				t.Fatal("not detached")
			}
			if err := CuestaceanTraceOriginsStart(); err != nil {
				t.Fatal(err)
			}
			if b, err := CuestaceanTraceStop(); err != nil || b == nil || len(b) != 0 {
				t.Fatal("recovery", b, err)
			}
		})
	}
}

func TestCuestaceanOriginInterleaving(t *testing.T) {
	if err := CuestaceanTraceOriginsStart(); err != nil {
		t.Fatal(err)
	}
	p := &nodeContext{}
	p.defaultMode, p.origDefaultMode = notDefault, isDefault
	p.disjuncts = []*nodeContext{{}, {}}
	a := cuestaceanTraceEnter()
	cuestaceanTraceOrigin(a, p, 0)
	p.defaultMode, p.origDefaultMode = isDefault, maybeDefault
	b := cuestaceanTraceEnter()
	cuestaceanTraceOrigin(b, p, 0)
	cuestaceanTraceTrials(b, []*nodeContext{p}, false, false)
	cuestaceanTraceResults(b, nil)
	cuestaceanTraceOrigin(a, p, 0)
	cuestaceanTraceTrials(a, []*nodeContext{p, p}, false, false)
	cuestaceanTraceResults(a, nil)
	if cuestaceanCapture.records != 15 { // 3*trials + 6 nested children.
		t.Fatal("origin records charged twice or nested records omitted")
	}
	batches, err := CuestaceanTraceStop()
	if err != nil {
		t.Fatal(err)
	}
	for i, want := range []CuestaceanTraceMode{"not", "default"} {
		if *batches[0].Trials[i].LeftMode != want {
			t.Fatal("uncopied/unordered origins", batches)
		}
	}
	if *batches[1].Trials[0].LeftOriginalMode != "maybe" {
		t.Fatal(batches)
	}
}

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
