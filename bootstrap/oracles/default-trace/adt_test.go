package adt

import (
	"encoding/json"
	"strings"
	"testing"
)

func equalityBatch(t *testing.T, mode int) int {
	t.Helper()
	if cuestaceanCapture == nil {
		if err := CuestaceanTraceEqualityStart(); err != nil {
			t.Fatal(err)
		}
	}
	id := cuestaceanTraceEnter()
	cuestaceanTraceRunMode(id, mode)
	cuestaceanTraceTrials(id, nil, true, true)
	return id
}

func equalityStop(t *testing.T, id int, failed bool) []CuestaceanTraceBatch {
	t.Helper()
	cuestaceanTraceResults(id, nil)
	c := cuestaceanCapture
	if c.depth != 0 {
		t.Fatal("unbalanced equality capture", c.depth)
	}
	if failed {
		sticky, records := c.err, c.records
		if sticky == nil {
			t.Fatal("missing equality failure")
		}
		next := cuestaceanTraceEnter()
		cuestaceanTraceRunMode(next, 4)
		cuestaceanTraceAppend(nil)
		cuestaceanTraceResults(next, nil)
		if c.err != sticky || c.records != records || c.depth != 0 {
			t.Fatal("not sticky/balanced")
		}
	}
	b, err := CuestaceanTraceStop()
	if (err != nil) != failed || (failed && b != nil) || cuestaceanCapture != nil {
		t.Fatal(b, err)
	}
	if failed {
		id := equalityBatch(t, 4)
		cuestaceanTraceResults(id, nil)
		if _, err := CuestaceanTraceStop(); err != nil {
			t.Fatal("recovery", err)
		}
	}
	return b
}

func TestCuestaceanEqualityModes(t *testing.T) {
	for _, start := range []func() error{CuestaceanTraceStart, CuestaceanTraceOriginsStart, CuestaceanTraceHistoryStart} {
		if err := start(); err != nil {
			t.Fatal(err)
		}
		cuestaceanTraceRunMode(-1, 255)
		token := cuestaceanTraceAppend(nil)
		cuestaceanTraceCompare(token, nil, true)
		cuestaceanTraceIdentity(token, nil)
		cuestaceanTraceAccept(token)
		cuestaceanTraceInsert(token, -1)
		if cuestaceanTraceEqualityEnabled() || token != (cuestaceanTraceAppendToken{}) || cuestaceanCapture.records != 0 {
			t.Fatal("old capture inspected equality")
		}
		if _, err := CuestaceanTraceStop(); err != nil {
			t.Fatal(err)
		}
	}
	for _, mode := range []int{1, 2, 3, 4, 0, 255} {
		id := equalityBatch(t, mode)
		b := equalityStop(t, id, mode == 0 || mode == 255)
		if len(b) != 0 && b[0].Phase.RunMode != [...]string{"ignore", "attempt-only", "yield", "finalize"}[mode-1] {
			t.Fatal(b)
		}
	}
}

func TestCuestaceanEqualitySnapshots(t *testing.T) {
	id := equalityBatch(t, 4)
	num := &Num{K: IntKind}
	num.X.SetInt64(-9223372036854775808)
	x := &nodeContext{node: &Vertex{BaseValue: num}}
	x.defaultMode, x.origDefaultMode = isDefault, notDefault
	a := cuestaceanTraceAppend(x)
	x.defaultMode = notDefault
	num.X.SetInt64(9223372036854775807)
	cuestaceanTraceInsert(a, 0)
	// A direct indirection must stay null, even when its target is numeric.
	x.node = &Vertex{BaseValue: &Vertex{BaseValue: num}}
	for _, partial := range []bool{true, false} {
		for _, accept := range []bool{true, false} {
			a := cuestaceanTraceAppend(x)
			cuestaceanTraceCompare(a, x, partial)
			if accept {
				cuestaceanTraceAccept(a)
			} else {
				cuestaceanTraceInsert(a, 1)
			}
		}
	}
	a = cuestaceanTraceAppend(x)
	cuestaceanTraceCompare(a, x, true)
	cuestaceanTraceIdentity(a, x)
	b := equalityStop(t, id, false)
	p := b[0].Phase
	if *p.Appends[0].Candidate.Value != -9223372036854775808 || p.Appends[0].Candidate.Mode != "default" || *p.Appends[0].Candidate.OriginalMode != "not" {
		t.Fatal(p)
	}
	for i, want := range []string{"partial", "insert", "final", "insert"} {
		a := p.Appends[i+1]
		if a.Kind != want || a.Candidate.Value != nil || a.Comparisons[0].Existing.Value != nil || a.Comparisons[0].Accepted != (i%2 == 0) {
			t.Fatal(a)
		}
	}
	last := p.Appends[5]
	if last.Kind != "identity" || last.Index != 1 || last.Comparisons[0].Accepted || !last.Comparisons[1].Accepted || last.Comparisons[1].Index != 1 {
		t.Fatal(last)
	}
	if len(p.Appends[0].Comparisons) != 0 {
		t.Fatal("insert comparisons")
	}
}

func TestCuestaceanEqualityMixedRecords(t *testing.T) {
	if err := CuestaceanTraceEqualityStart(); err != nil {
		t.Fatal(err)
	}
	if err := CuestaceanTraceEqualityStart(); err == nil {
		t.Fatal("overlapping equality capture")
	}
	id := cuestaceanTraceEnter()
	cuestaceanTraceRunMode(id, 4)
	num := &Num{K: IntKind}
	num.X.SetInt64(9223372036854775807)
	x := &nodeContext{node: &Vertex{BaseValue: num}}
	x.defaultMode, x.origDefaultMode = notDefault, maybeDefault
	trial := &nodeContext{disjuncts: []*nodeContext{x, x}}
	cuestaceanTraceOrigin(id, x, 0)
	cuestaceanTraceTrials(id, []*nodeContext{trial}, false, false)
	a := cuestaceanTraceAppend(x)
	cuestaceanTraceCompare(a, x, true)
	cuestaceanTraceAccept(a)
	// Simulate the subsequent promotion/mutation, never an observer side effect.
	x.defaultMode = isDefault
	num.X.SetInt64(7)
	cuestaceanTraceResults(id, []*nodeContext{x})
	if cuestaceanCapture.records != 3+2*2+2+3+3 {
		t.Fatal("mixed v4 record formula", cuestaceanCapture.records)
	}
	b, err := CuestaceanTraceStop()
	if err != nil {
		t.Fatal(err)
	}
	r := b[0].Phase.Appends[0].Comparisons[0].Existing
	if r.Mode != "not" || *r.OriginalMode != "maybe" || *r.Value != 9223372036854775807 || b[0].Results[0].Mode != "default" || *b[0].Results[0].Value != 7 {
		t.Fatal("existing snapshot not copied", b)
	}
}

func TestCuestaceanEqualityInterleaving(t *testing.T) {
	id := equalityBatch(t, 2)
	x := &nodeContext{}
	parent := cuestaceanTraceAppend(x)
	child := cuestaceanTraceEnter()
	cuestaceanTraceRunMode(child, 2)
	if token := cuestaceanTraceAppend(nil); token != (cuestaceanTraceAppendToken{}) {
		t.Fatal("stage0 attributed to parent")
	}
	cuestaceanTraceTrials(child, nil, true, true)
	ct := cuestaceanTraceAppend(x)
	cuestaceanTraceInsert(ct, 0)
	cuestaceanTraceResults(child, nil)
	// Reentrant appends in one batch have independent LIFO state.
	nested := cuestaceanTraceAppend(x)
	cuestaceanTraceCompare(nested, x, false)
	cuestaceanTraceAccept(nested)
	cuestaceanTraceCompare(parent, x, true)
	cuestaceanTraceInsert(parent, 1)
	b := equalityStop(t, id, false)
	if len(b) != 2 || len(b[0].Phase.Appends) != 2 || len(b[1].Phase.Appends) != 1 || b[0].Phase.Appends[0].Kind != "insert" || b[0].Phase.Appends[1].Kind != "final" {
		t.Fatal(b)
	}
}

func TestCuestaceanEqualityFailures(t *testing.T) {
	for _, failure := range []string{"missingMode", "duplicateMode", "incomplete", "accept", "insert", "duplicate", "malformed", "stale", "lifo", "batch", "unknownCandidate", "unknownExisting"} {
		t.Run(failure, func(t *testing.T) {
			id := equalityBatch(t, 4)
			x := &nodeContext{}
			zero := cuestaceanTraceAppendToken{}
			cuestaceanTraceCompare(zero, nil, false)
			cuestaceanTraceIdentity(zero, nil)
			cuestaceanTraceAccept(zero)
			cuestaceanTraceInsert(zero, -1)
			a := cuestaceanTraceAppend(x)
			switch failure {
			case "missingMode":
				cuestaceanCapture.batches[id].Phase.RunMode = ""
			case "duplicateMode":
				// Duplicate is rejected even at the correct entry stage.
				cuestaceanCapture.stages[id] = 0
				cuestaceanTraceRunMode(id, 4)
			case "incomplete":
			case "accept":
				cuestaceanTraceAccept(a)
			case "insert":
				cuestaceanTraceInsert(a, 1)
			case "duplicate":
				cuestaceanTraceInsert(a, 0)
				cuestaceanTraceInsert(a, 0)
			case "malformed":
				a.event = -1
				cuestaceanTraceAccept(a)
			case "stale":
				cuestaceanTraceInsert(a, 0)
				equalityStop(t, id, false)
				id = equalityBatch(t, 4)
				cuestaceanTraceAppend(x) // Same batch/event numbers, different capture.
				cuestaceanTraceInsert(a, 0)
			case "lifo":
				cuestaceanTraceAppend(x)
				cuestaceanTraceInsert(a, 0)
			case "batch":
				child := equalityBatch(t, 4)
				cuestaceanTraceCompare(a, x, true)
				cuestaceanTraceResults(child, nil)
			case "unknownCandidate":
				x.origDefaultMode = 255
				cuestaceanTraceAppend(x)
			case "unknownExisting":
				x.defaultMode = 255
				cuestaceanTraceCompare(a, x, true)
			}
			equalityStop(t, id, true)
		})
	}
	// Actual missing run-mode hook, not only a cleared mandatory field.
	if err := CuestaceanTraceEqualityStart(); err != nil {
		t.Fatal(err)
	}
	id := cuestaceanTraceEnter()
	cuestaceanTraceTrials(id, nil, true, true)
	equalityStop(t, id, true)
}

func TestCuestaceanEqualityCaps(t *testing.T) {
	for _, vector := range []string{"appends", "comparisons"} {
		for _, width := range []int{64, 65} {
			id := equalityBatch(t, 4)
			x := &nodeContext{}
			a := cuestaceanTraceAppendToken{}
			if vector == "comparisons" {
				a = cuestaceanTraceAppend(x)
			}
			for i := 0; i < width; i++ {
				if vector == "appends" {
					a = cuestaceanTraceAppend(x)
					cuestaceanTraceInsert(a, 0)
				} else {
					cuestaceanTraceCompare(a, x, false)
				}
			}
			if vector == "comparisons" {
				cuestaceanTraceInsert(a, width)
			}
			if width == 64 {
				want := 3 * 64
				if vector == "comparisons" {
					want += 3
				}
				if cuestaceanCapture.records != want {
					t.Fatal("vector accounting", cuestaceanCapture.records)
				}
			}
			equalityStop(t, id, width == 65)
		}
	}
	for _, over := range []bool{false, true} {
		if err := CuestaceanTraceEqualityStart(); err != nil {
			t.Fatal(err)
		}
		x := &nodeContext{}
		v := make([]*nodeContext, 64)
		for i := range v {
			v[i] = x
		}
		for batch := 0; batch < 8; batch++ {
			id := cuestaceanTraceEnter()
			cuestaceanTraceRunMode(id, 4)
			for range v {
				cuestaceanTraceOrigin(id, x, 0)
			}
			cuestaceanTraceTrials(id, v, true, true)
			for range v {
				a := cuestaceanTraceAppend(x)
				cuestaceanTraceInsert(a, 0)
			}
			results := v
			if over && batch == 7 {
				results = v[:63]
			}
			cuestaceanTraceResults(id, results)
		}
		want := 4096
		if over {
			want = 4094
		}
		if cuestaceanCapture.records != want || cuestaceanCapture.err != nil {
			t.Fatal("record formula", cuestaceanCapture.records)
		}
		id := equalityBatch(t, 4)
		if over {
			cuestaceanTraceAppend(x)
		} // 4094 + 3 = 4097, exactly one over.
		equalityStop(t, id, over)
	}
}

func TestCuestaceanHistory(t *testing.T) {
	for _, start := range []func() error{CuestaceanTraceStart, CuestaceanTraceOriginsStart} {
		if err := start(); err != nil {
			t.Fatal(err)
		}
		r := &nodeContext{}
		r.origDefaultMode = defaultMode(255)
		id := cuestaceanTraceEnter()
		cuestaceanTraceTrials(id, nil, true, true)
		cuestaceanTraceResults(id, []*nodeContext{r})
		if b, err := CuestaceanTraceStop(); err != nil || b[0].Results[0].OriginalMode != nil {
			t.Fatal("old mode inspected result history", b, err)
		}
	}
	for _, failure := range []string{"", "records", "nested", "result"} {
		t.Run(failure, func(t *testing.T) {
			if err := CuestaceanTraceHistoryStart(); err != nil {
				t.Fatal(err)
			}
			if err := CuestaceanTraceHistoryStart(); err == nil {
				t.Fatal("overlap")
			}
			p := &nodeContext{}
			r := &nodeContext{}
			r.defaultMode, r.origDefaultMode = isDefault, notDefault
			vector := make([]*nodeContext, 64)
			for i := range vector {
				vector[i] = r
			}
			batch := func(trials, results int) {
				id := cuestaceanTraceEnter()
				for i := 0; i < trials; i++ {
					cuestaceanTraceOrigin(id, p, 0)
				}
				cuestaceanTraceTrials(id, vector[:trials], false, false)
				cuestaceanTraceResults(id, vector[:results])
			}
			if failure == "" || failure == "records" {
				for i := 0; i < 12; i++ {
					batch(64, 64) // 12 * (3*64 + 2*64) = 3840.
				}
				if failure == "records" {
					batch(49, 55) // Requested total 4097: exactly one over.
				} else {
					batch(48, 56) // 3840 + 144 + 112 = 4096.
					if cuestaceanCapture.records != 4096 {
						t.Fatal("exact history cap", cuestaceanCapture.records)
					}
				}
			} else {
				r.origDefaultMode = defaultMode(255)
				if failure == "nested" {
					vector[0] = &nodeContext{disjuncts: []*nodeContext{r, r}}
					batch(1, 1)
				} else {
					batch(0, 1) // Invalid origin only inspected by result capture.
				}
			}
			c := cuestaceanCapture
			if c.depth != 0 {
				t.Fatal("unbalanced history hooks")
			}
			if failure != "" {
				sticky, records := c.err, c.records
				if sticky == nil {
					t.Fatal("missing history failure")
				}
				batch(1, 1)
				if c.err != sticky || c.records != records || c.depth != 0 {
					t.Fatal("history failure not sticky/balanced")
				}
			}
			b, err := CuestaceanTraceStop()
			if failure == "" {
				if err != nil || len(b) != 13 || *b[0].Results[0].OriginalMode != "not" {
					t.Fatal(b, err)
				}
			} else if err == nil || b != nil {
				t.Fatal("partial history", b, err)
			}
			if cuestaceanCapture != nil {
				t.Fatal("not detached")
			}
			if err := CuestaceanTraceHistoryStart(); err != nil {
				t.Fatal(err)
			}
			r.origDefaultMode = notDefault
			vector[0] = &nodeContext{disjuncts: []*nodeContext{r, r}}
			batch(1, 1) // 3 + 2*2 + 2 = 9, including nested history.
			if cuestaceanCapture.records != 9 {
				t.Fatal("nested history accounting")
			}
			b, err = CuestaceanTraceStop()
			if err != nil || *b[0].Trials[0].Nested[0].OriginalMode != "not" {
				t.Fatal("history recovery", b, err)
			}
			raw, err := json.Marshal(b[0].Trials[0].Nested)
			if err != nil || string(raw) != `[{"mode":"default","originalMode":"not","value":null},{"mode":"default","originalMode":"not","value":null}]` {
				t.Fatal("nested history wire", string(raw), err)
			}
		})
	}
}

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
