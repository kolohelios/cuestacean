// Bounded diagnostic snapshots of pinned ADT state; not solver semantics.
package adt

import "fmt"

type CuestaceanTraceMode string

type CuestaceanTraceResult struct {
	Mode         CuestaceanTraceMode  `json:"mode"`
	OriginalMode *CuestaceanTraceMode `json:"originalMode,omitempty"`
	Value        *int64               `json:"value"`
}

type CuestaceanTraceTrial struct {
	Mode             CuestaceanTraceMode     `json:"mode"`
	OriginalMode     CuestaceanTraceMode     `json:"originalMode"`
	LeftMode         *CuestaceanTraceMode    `json:"leftMode,omitempty"`
	LeftOriginalMode *CuestaceanTraceMode    `json:"leftOriginalMode,omitempty"`
	Value            *int64                  `json:"value"`
	Nested           []CuestaceanTraceResult `json:"nested"`
}

type CuestaceanTraceComparison struct {
	Index    uint32                `json:"index"`
	Kind     string                `json:"kind"`
	Existing CuestaceanTraceResult `json:"existing"`
	Accepted bool                  `json:"accepted"`
}

type CuestaceanTraceAppend struct {
	Candidate   CuestaceanTraceResult       `json:"candidate"`
	Index       uint32                      `json:"index"`
	Kind        string                      `json:"kind"`
	Comparisons []CuestaceanTraceComparison `json:"comparisons"`
}

type CuestaceanTracePhase struct {
	RunMode string                  `json:"runMode"`
	Appends []CuestaceanTraceAppend `json:"appends"`
}

type CuestaceanTraceBatch struct {
	ID                uint32                  `json:"id"`
	Depth             uint32                  `json:"depth"`
	LeftDropsDefault  bool                    `json:"leftDropsDefault"`
	RightDropsDefault bool                    `json:"rightDropsDefault"`
	Trials            []CuestaceanTraceTrial  `json:"trials"`
	Results           []CuestaceanTraceResult `json:"results"`
	Phase             *CuestaceanTracePhase   `json:"phase,omitempty"`
}

// One fresh, single-command process owns this capture. No concurrent callers.
var cuestaceanCapture *cuestaceanTraceCapture

type cuestaceanTraceCapture struct {
	batches         []CuestaceanTraceBatch
	depth, records  int
	stack           [128]int
	stages          [128]uint8
	err             error
	originsEnabled  bool
	historyEnabled  bool
	equalityEnabled bool
	pending         [128][]int
	origins         [128][][2]CuestaceanTraceMode
}

func CuestaceanTraceStart() error {
	if cuestaceanCapture != nil {
		return fmt.Errorf("trace already active")
	}
	cuestaceanCapture = &cuestaceanTraceCapture{batches: []CuestaceanTraceBatch{}}
	return nil
}

func CuestaceanTraceOriginsStart() error {
	if err := CuestaceanTraceStart(); err != nil {
		return err
	}
	cuestaceanCapture.originsEnabled = true
	return nil
}

func CuestaceanTraceHistoryStart() error {
	if err := CuestaceanTraceOriginsStart(); err != nil {
		return err
	}
	cuestaceanCapture.historyEnabled = true
	return nil
}

// Tokens retain diagnostic capture identity, never evaluator vertices.
type cuestaceanTraceAppendToken struct {
	capture      *cuestaceanTraceCapture
	batch, event int
}

func CuestaceanTraceEqualityStart() error {
	if err := CuestaceanTraceHistoryStart(); err != nil {
		return err
	}
	cuestaceanCapture.equalityEnabled = true
	return nil
}

func cuestaceanTraceEqualityEnabled() bool {
	c := cuestaceanCapture
	return c != nil && c.equalityEnabled && c.err == nil
}

func cuestaceanTraceRunMode(id, mode int) {
	if !cuestaceanTraceEqualityEnabled() {
		return
	}
	c := cuestaceanCapture
	if !c.current(id, 0) {
		return
	}
	if c.batches[id].Phase != nil || mode < int(ignore) || mode > int(finalize) {
		c.fail("invalid or duplicate trace run mode")
		return
	}
	c.batches[id].Phase = &CuestaceanTracePhase{
		RunMode: [...]string{"ignore", "attempt-only", "yield", "finalize"}[mode-int(ignore)],
		Appends: []CuestaceanTraceAppend{},
	}
}

func cuestaceanTraceAppend(x *nodeContext) cuestaceanTraceAppendToken {
	if !cuestaceanTraceEqualityEnabled() {
		return cuestaceanTraceAppendToken{}
	}
	c := cuestaceanCapture
	// Only the top batch owns this call; never search ancestors for stage1.
	if c.depth < 1 || c.depth > 128 {
		return cuestaceanTraceAppendToken{}
	}
	id := c.stack[c.depth-1]
	if c.stages[id] != 1 {
		return cuestaceanTraceAppendToken{}
	}
	p := c.batches[id].Phase
	if p == nil || x == nil {
		c.fail("missing trace equality phase/candidate")
		return cuestaceanTraceAppendToken{}
	}
	if !c.reserve(len(p.Appends)+1, 3) {
		return cuestaceanTraceAppendToken{}
	}
	r := c.result(x)
	if c.err != nil {
		return cuestaceanTraceAppendToken{}
	}
	i := len(p.Appends)
	p.Appends = append(p.Appends, CuestaceanTraceAppend{Candidate: r, Comparisons: []CuestaceanTraceComparison{}})
	c.pending[id] = append(c.pending[id], i)
	return cuestaceanTraceAppendToken{c, id, i}
}

func (t cuestaceanTraceAppendToken) current() (*cuestaceanTraceCapture, *CuestaceanTraceAppend) {
	if t == (cuestaceanTraceAppendToken{}) || !cuestaceanTraceEqualityEnabled() {
		return nil, nil
	}
	c := cuestaceanCapture
	if t.capture != c {
		c.fail("stale trace append capture")
		return nil, nil
	}
	if !c.current(t.batch, 1) {
		return nil, nil
	}
	p := c.batches[t.batch].Phase
	pending := c.pending[t.batch]
	if p == nil || t.event < 0 || t.event >= len(p.Appends) || len(pending) == 0 || pending[len(pending)-1] != t.event || p.Appends[t.event].Kind != "" {
		c.fail("invalid trace append token/order")
		return nil, nil
	}
	return c, &p.Appends[t.event]
}

func cuestaceanTraceCompare(t cuestaceanTraceAppendToken, existing *nodeContext, partial bool) {
	c, a := t.current()
	if c == nil {
		return
	}
	kind := "final"
	if partial {
		kind = "partial"
	}
	c.comparison(a, existing, kind)
}

func (c *cuestaceanTraceCapture) comparison(a *CuestaceanTraceAppend, existing *nodeContext, kind string) {
	if !c.reserve(len(a.Comparisons)+1, 3) {
		return
	}
	if existing == nil {
		c.fail("missing trace comparison vertex")
		return
	}
	r := c.result(existing)
	if c.err == nil {
		a.Comparisons = append(a.Comparisons, CuestaceanTraceComparison{
			Index: uint32(len(a.Comparisons)), Kind: kind, Existing: r,
		})
	}
}

func (c *cuestaceanTraceCapture) finish(t cuestaceanTraceAppendToken, a *CuestaceanTraceAppend, index int, kind string) {
	a.Index, a.Kind = uint32(index), kind
	c.pending[t.batch] = c.pending[t.batch][:len(c.pending[t.batch])-1]
}

func cuestaceanTraceIdentity(t cuestaceanTraceAppendToken, existing *nodeContext) {
	c, a := t.current()
	if c == nil {
		return
	}
	c.comparison(a, existing, "identity")
	if c.err == nil {
		i := len(a.Comparisons) - 1
		a.Comparisons[i].Accepted = true
		c.finish(t, a, i, "identity")
	}
}

func cuestaceanTraceAccept(t cuestaceanTraceAppendToken) {
	c, a := t.current()
	if c == nil {
		return
	}
	if len(a.Comparisons) == 0 {
		c.fail("trace acceptance without comparison")
		return
	}
	i := len(a.Comparisons) - 1
	a.Comparisons[i].Accepted = true
	c.finish(t, a, i, a.Comparisons[i].Kind)
}

func cuestaceanTraceInsert(t cuestaceanTraceAppendToken, index int) {
	c, a := t.current()
	if c == nil {
		return
	}
	if index != len(a.Comparisons) {
		c.fail("trace insertion/comparison count mismatch")
		return
	}
	c.finish(t, a, index, "insert")
}

// Copy predecessor facts at the successful append, after recursive evaluation.
func cuestaceanTraceOrigin(id int, p *nodeContext, priority int) {
	c := cuestaceanCapture
	if c == nil || !c.originsEnabled {
		return
	}
	if !c.current(id, 0) {
		return
	}
	if priority != 0 || p.priority != 0 || p.origPriority != 0 {
		c.fail("trace origins require priority zero")
		return
	}
	if !c.reserve(len(c.origins[id])+1, 2) {
		return
	}
	pair := [2]CuestaceanTraceMode{c.mode(p.defaultMode), c.mode(p.origDefaultMode)}
	if c.err == nil {
		c.origins[id] = append(c.origins[id], pair)
	}
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
	r := CuestaceanTraceResult{Mode: c.mode(n.defaultMode), Value: cuestaceanTraceValue(n)}
	if c.historyEnabled {
		m := c.mode(n.origDefaultMode)
		r.OriginalMode = &m
	}
	return r
}

func cuestaceanTraceTrials(id int, tmp []*nodeContext, left, right bool) {
	c := cuestaceanCapture
	if c == nil || !c.current(id, 0) {
		return
	}
	if c.originsEnabled && len(c.origins[id]) != len(tmp) {
		c.fail("trace origin/trial count mismatch")
		return
	}
	if !c.reserve(len(tmp), len(tmp)) {
		return
	}
	// Check all nested widths and cumulative records before allocating copies.
	for _, n := range tmp {
		if !c.reserve(len(n.disjuncts), c.resultRecords(len(n.disjuncts))) {
			return
		}
	}
	b := &c.batches[id]
	b.LeftDropsDefault, b.RightDropsDefault = left, right
	for i, n := range tmp {
		t := CuestaceanTraceTrial{Mode: c.mode(n.defaultMode), OriginalMode: c.mode(n.origDefaultMode),
			Value: cuestaceanTraceValue(n), Nested: []CuestaceanTraceResult{}}
		if c.originsEnabled {
			pair := c.origins[id][i]
			t.LeftMode, t.LeftOriginalMode = &pair[0], &pair[1]
		}
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

func (c *cuestaceanTraceCapture) resultRecords(width int) int {
	if c.historyEnabled {
		return 2 * width
	}
	return width
}

func cuestaceanTraceResults(id int, dst []*nodeContext) {
	c := cuestaceanCapture
	if c == nil {
		return
	}
	defer func() { c.depth-- }()
	if !c.current(id, 1) {
		return
	}
	if c.equalityEnabled && (c.batches[id].Phase == nil || c.batches[id].Phase.RunMode == "" || len(c.pending[id]) != 0) {
		c.fail("incomplete trace equality phase")
		return
	}
	if !c.reserve(len(dst), c.resultRecords(len(dst))) {
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
