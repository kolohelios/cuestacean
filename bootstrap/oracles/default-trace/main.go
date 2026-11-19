// Test-only pinned ADT observations; never a native preference certificate.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/ast"
	"cuelang.org/go/cue/cuecontext"
	"cuelang.org/go/cue/parser"
	"cuelang.org/go/cue/token"
	"cuelang.org/go/internal/core/adt"
)

const byteLimit = 65536

type observation struct {
	FormatVersion   uint32                     `json:"formatVersion"`
	Mode            string                     `json:"mode"`
	DefaultSelected bool                       `json:"defaultSelected"`
	Status          string                     `json:"status"`
	Value           json.RawMessage            `json:"value"`
	Batches         []adt.CuestaceanTraceBatch `json:"batches"`
}

func labelOK(s string) bool {
	if len(s) == 0 || len(s) > 64 {
		return false
	}
	switch s {
	case "_", "int", "bool", "string", "bytes", "number", "float":
		return false
	}
	for i, b := range []byte(s) {
		if (b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z') ||
			(i > 0 && (b == '_' || (b >= '0' && b <= '9'))) {
			continue
		}
		return false
	}
	return true
}

func decimal(s string) bool {
	if s == "" {
		return false
	}
	for _, b := range []byte(s) {
		if b < '0' || b > '9' {
			return false
		}
	}
	return true
}

// Count every expression AST node, including parentheses, signs and literals.
// Root depth is one. Signed literals validate the full spelling (not magnitude).
func expression(e ast.Expr, depth int, nodes *int) bool {
	*nodes++
	if depth > 16 || *nodes > 128 {
		return false
	}
	switch v := e.(type) {
	case *ast.BasicLit:
		_, err := strconv.ParseInt(v.Value, 10, 64)
		return v.Kind == token.INT && decimal(v.Value) && err == nil
	case *ast.ParenExpr:
		return expression(v.X, depth+1, nodes)
	case *ast.BinaryExpr:
		return (v.Op == token.AND || v.Op == token.OR) &&
			expression(v.X, depth+1, nodes) && expression(v.Y, depth+1, nodes)
	case *ast.UnaryExpr:
		if v.Op == token.MUL {
			return expression(v.X, depth+1, nodes)
		}
		lit, ok := v.X.(*ast.BasicLit)
		if !ok || (v.Op != token.ADD && v.Op != token.SUB) || lit.Kind != token.INT || !decimal(lit.Value) {
			return false
		}
		*nodes++
		_, err := strconv.ParseInt(v.Op.String()+lit.Value, 10, 64)
		return depth+1 <= 16 && *nodes <= 128 && err == nil
	}
	return false
}

func admit(source []byte, field string) (*ast.File, error) {
	parsed, err := parser.ParseFile("", source)
	if err != nil {
		return nil, err
	}
	if len(parsed.Imports) != 0 {
		return nil, fmt.Errorf("imports outside profile")
	}
	count := 0
	for _, decl := range parsed.Decls {
		if _, ok := decl.(*ast.Package); ok {
			continue
		}
		f, ok := decl.(*ast.Field)
		if !ok || f.Alias != nil || f.Constraint != token.ILLEGAL || len(f.Attrs) != 0 {
			return nil, fmt.Errorf("ordinary field required")
		}
		label, ok := f.Label.(*ast.Ident)
		if !ok || !labelOK(label.Name) || label.Name != field {
			return nil, fmt.Errorf("field label mismatch/outside profile")
		}
		nodes := 0
		if !expression(f.Value, 1, &nodes) {
			return nil, fmt.Errorf("expression outside bounded finite-i64 profile")
		}
		count++
	}
	if count != 1 {
		return nil, fmt.Errorf("exactly one field required")
	}
	return parsed, nil
}

func encode(out observation) ([]byte, error) {
	b, err := json.Marshal(out)
	if err != nil {
		return nil, err
	}
	if len(b)+1 > byteLimit {
		return nil, fmt.Errorf("trace output exceeds 65536 bytes")
	}
	return append(b, '\n'), nil
}

func observe(args []string) ([]byte, error) {
	if len(args) != 3 || (args[0] != "trace" && args[0] != "trace-origins" && args[0] != "trace-history") || !labelOK(args[2]) {
		return nil, fmt.Errorf("expected trace, trace-origins or trace-history FILE FIELD")
	}
	file, err := os.Open(args[1])
	if err != nil {
		return nil, err
	}
	defer file.Close()
	source, err := io.ReadAll(io.LimitReader(file, byteLimit+1))
	if err != nil {
		return nil, err
	}
	if len(source) > byteLimit {
		return nil, fmt.Errorf("source exceeds 65536 bytes")
	}
	parsed, err := admit(source, args[2])
	if err != nil {
		return nil, err
	}
	start := adt.CuestaceanTraceStart
	if args[0] == "trace-origins" {
		start = adt.CuestaceanTraceOriginsStart
	} else if args[0] == "trace-history" {
		start = adt.CuestaceanTraceHistoryStart
	}
	if err := start(); err != nil {
		return nil, err
	}
	// Also detach on an early error or panic; normal stop precedes encoding.
	active := true
	defer func() {
		if active {
			_, _ = adt.CuestaceanTraceStop()
		}
	}()
	root := cuecontext.New().BuildFile(parsed)
	selected := root.LookupPath(cue.MakePath(cue.Str(args[2])))
	if !selected.Exists() {
		return nil, fmt.Errorf("missing field")
	}
	out := observation{FormatVersion: 1, Mode: args[0]}
	if args[0] == "trace-origins" {
		out.FormatVersion = 2
	} else if args[0] == "trace-history" {
		out.FormatVersion = 3
	}
	selected, out.DefaultSelected = selected.Default()
	if selected.Err() != nil {
		out.Status = "conflict"
	} else if raw, err := selected.MarshalJSON(); err != nil {
		out.Status = "incomplete"
	} else {
		out.Status, out.Value = "concrete", raw
	}
	out.Batches, err = adt.CuestaceanTraceStop()
	active = false
	if err != nil {
		return nil, err
	}
	return encode(out)
}

func main() {
	encoded, err := observe(os.Args[1:])
	if err != nil {
		fmt.Fprintln(os.Stderr, strings.TrimSpace(err.Error()))
		os.Exit(1)
	}
	if _, err := os.Stdout.Write(encoded); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
