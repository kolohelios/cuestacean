// Bounded diagnostic snapshots of pinned ADT state; not solver semantics.
package adt

import "fmt"

type CuestaceanTraceMode string

type CuestaceanTraceResult struct {
	Mode  CuestaceanTraceMode `json:"mode"`
	Value *int64              `json:"value"`
}

type CuestaceanTraceTrial struct {
	Mode         CuestaceanTraceMode     `json:"mode"`
	OriginalMode CuestaceanTraceMode     `json:"originalMode"`
	Value        *int64                  `json:"value"`
	Nested       []CuestaceanTraceResult `json:"nested"`
}

type CuestaceanTraceBatch struct {
	ID                uint32                  `json:"id"`
	Depth             uint32                  `json:"depth"`
	LeftDropsDefault  bool                    `json:"leftDropsDefault"`
	RightDropsDefault bool                    `json:"rightDropsDefault"`
	Trials            []CuestaceanTraceTrial  `json:"trials"`
	Results           []CuestaceanTraceResult `json:"results"`
}

// One fresh, single-command process owns this capture. No concurrent callers.
var cuestaceanCapture *cuestaceanTraceCapture

type cuestaceanTraceCapture struct {
	batches        []CuestaceanTraceBatch
	depth, records int
	stack          [128]int
	stages         [128]uint8
	err            error
}

func CuestaceanTraceStart() error {
	if cuestaceanCapture != nil {
		return fmt.Errorf("trace already active")
	}
	cuestaceanCapture = &cuestaceanTraceCapture{batches: []CuestaceanTraceBatch{}}
	return nil
}

func CuestaceanTraceStop() ([]CuestaceanTraceBatch, error) {
	c := cuestaceanCapture
	cuestaceanCapture = nil
	if c == nil {
		return nil, fmt.Errorf("trace not active")
	}
	if c.depth != 0 {
		c.fail("unbalanced trace hooks")
	}
	if c.err != nil {
		return nil, c.err
	}
	return c.batches, nil
}

func (c *cuestaceanTraceCapture) fail(message string) {
	if c.err == nil {
		c.err = fmt.Errorf("%s", message)
	}
}

func cuestaceanTraceEnter() int {
	c := cuestaceanCapture
	if c == nil {
		return -1
	}
	depth := c.depth
	c.depth++
	if c.err != nil {
		return -1
	}
	if len(c.batches) == 128 {
		c.fail("trace batch limit")
		return -1
	}
	id := len(c.batches)
	c.stack[depth] = id
	c.batches = append(c.batches, CuestaceanTraceBatch{
		ID: uint32(id), Depth: uint32(depth),
		Trials: []CuestaceanTraceTrial{}, Results: []CuestaceanTraceResult{},
	})
	return id
}

func (c *cuestaceanTraceCapture) current(id int, stage uint8) bool {
	if c.err != nil {
		return false
	}
	if c.depth < 1 || c.depth > 128 || id < 0 || id >= len(c.batches) ||
		c.stack[c.depth-1] != id || c.stages[id] != stage {
		c.fail("out-of-order trace hooks")
		return false
	}
	return true
}

func (c *cuestaceanTraceCapture) reserve(width, records int) bool {
	if width > 64 || records > 4096-c.records {
		c.fail("trace record/vector limit")
		return false
	}
	c.records += records
	return true
}

func (c *cuestaceanTraceCapture) mode(m defaultMode) CuestaceanTraceMode {
	switch m {
	case maybeDefault:
		return "maybe"
	case isDefault:
		return "default"
	case notDefault:
		return "not"
	default:
		c.fail("unknown ADT default mode")
		return ""
	}
}

func cuestaceanTraceValue(n *nodeContext) *int64 {
	if n == nil || n.node == nil {
		return nil
	}
	v, ok := n.node.BaseValue.(*Num)
	if !ok || v.K != IntKind {
		return nil
	}
	i, err := v.X.Int64()
	if err != nil {
		return nil
	}
	return &i
}

func (c *cuestaceanTraceCapture) result(n *nodeContext) CuestaceanTraceResult {
	return CuestaceanTraceResult{c.mode(n.defaultMode), cuestaceanTraceValue(n)}
}

func cuestaceanTraceTrials(id int, tmp []*nodeContext, left, right bool) {
	c := cuestaceanCapture
	if c == nil || !c.current(id, 0) {
		return
	}
	if !c.reserve(len(tmp), len(tmp)) {
		return
	}
	// Check all nested widths and cumulative records before allocating copies.
	for _, n := range tmp {
		if !c.reserve(len(n.disjuncts), len(n.disjuncts)) {
			return
		}
	}
	b := &c.batches[id]
	b.LeftDropsDefault, b.RightDropsDefault = left, right
	for _, n := range tmp {
		t := CuestaceanTraceTrial{c.mode(n.defaultMode), c.mode(n.origDefaultMode),
			cuestaceanTraceValue(n), []CuestaceanTraceResult{}}
		if c.err != nil {
			return
		}
		for _, child := range n.disjuncts {
			r := c.result(child)
			if c.err != nil {
				return
			}
			t.Nested = append(t.Nested, r)
		}
		b.Trials = append(b.Trials, t)
	}
	c.stages[id] = 1
}

func cuestaceanTraceResults(id int, dst []*nodeContext) {
	c := cuestaceanCapture
	if c == nil {
		return
	}
	defer func() { c.depth-- }()
	if !c.current(id, 1) || !c.reserve(len(dst), len(dst)) {
		return
	}
	b := &c.batches[id]
	for _, n := range dst {
		r := c.result(n)
		if c.err != nil {
			return
		}
		b.Results = append(b.Results, r)
	}
	c.stages[id] = 2
}
