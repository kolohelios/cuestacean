// Test-only observations of the pinned CUE public API; not a native solver.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"
	"unicode/utf8"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/ast"
	"cuelang.org/go/cue/cuecontext"
	"cuelang.org/go/cue/parser"
	"cuelang.org/go/cue/token"
)

const sourceLimit = 65536

// Declaration labels may not shadow these expression identifiers at any depth.
var predeclared = map[string]bool{
	"_": true, "int": true, "bool": true, "string": true,
	"bytes": true, "number": true, "float": true,
}

func fields(decls []ast.Decl) error {
	for _, decl := range decls {
		f, ok := decl.(*ast.Field)
		if !ok || f.Alias != nil || f.Constraint != token.ILLEGAL || len(f.Attrs) != 0 {
			return fmt.Errorf("only ordinary fields are admitted")
		}
		switch label := f.Label.(type) {
		case *ast.Ident:
			if strings.HasPrefix(label.Name, "#") || strings.HasPrefix(label.Name, "_") {
				return fmt.Errorf("hidden/definition labels are outside the profile")
			}
		case *ast.BasicLit:
			if label.Kind != token.STRING {
				return fmt.Errorf("label must be an identifier or quoted string")
			}
		default:
			return fmt.Errorf("dynamic labels are outside the profile")
		}
		name, _, err := ast.LabelName(f.Label)
		if err != nil || predeclared[name] {
			return fmt.Errorf("invalid or predeclared-shadowing label")
		}
		if container, ok := f.Value.(*ast.StructLit); ok {
			if err := fields(container.Elts); err != nil {
				return err
			}
		} else if !scalar(f.Value) {
			return fmt.Errorf("expression outside reference-free scalar profile")
		}
	}
	return nil
}

func numericLiteral(e ast.Expr) bool {
	if p, ok := e.(*ast.ParenExpr); ok {
		return numericLiteral(p.X)
	}
	v, ok := e.(*ast.BasicLit)
	return ok && (v.Kind == token.INT || v.Kind == token.FLOAT)
}

func scalar(e ast.Expr) bool {
	switch v := e.(type) {
	case *ast.BasicLit:
		switch v.Kind {
		case token.INT, token.FLOAT, token.STRING, token.TRUE, token.FALSE, token.NULL:
			return true
		}
	case *ast.Ident:
		return predeclared[v.Name]
	case *ast.ParenExpr:
		return scalar(v.X)
	case *ast.BinaryExpr:
		return (v.Op == token.AND || v.Op == token.OR) && scalar(v.X) && scalar(v.Y)
	case *ast.UnaryExpr:
		return (v.Op == token.MUL && scalar(v.X)) ||
			((v.Op == token.ADD || v.Op == token.SUB) && numericLiteral(v.X))
	}
	return false
}

// Field order and explicit nulls are part of the wire contract.
type observation struct {
	FormatVersion   int             `json:"formatVersion"`
	Mode            string          `json:"mode"`
	LeftDefault     *bool           `json:"leftDefault"`
	RightDefault    *bool           `json:"rightDefault"`
	DefaultSelected bool            `json:"defaultSelected"`
	Status          string          `json:"status"`
	Value           json.RawMessage `json:"value"`
}

// This records public API acceptance, not a semantic non-subsumption proof.
// In particular, a nonnil Subsume result can be an inexact comparison.
type relationObservation struct {
	FormatVersion int    `json:"formatVersion"`
	Mode          string `json:"mode"`
	Accepted      bool   `json:"accepted"`
}

func observe(args []string) ([]byte, error) {
	if len(args) < 1 {
		return nil, fmt.Errorf("expected inspect, unify or default-unify")
	}
	mode, count := args[0], 4
	if mode == "inspect" {
		count = 3
	} else if mode != "unify" && mode != "default-unify" && mode != "subsume" && mode != "raw-subsume" {
		return nil, fmt.Errorf("unknown mode")
	}
	if len(args) != count {
		return nil, fmt.Errorf("incorrect argument count")
	}
	paths := make([]cue.Path, len(args)-2)
	for i, selector := range args[2:] {
		if len(selector) == 0 || len(selector) > 256 || !utf8.ValidString(selector) {
			return nil, fmt.Errorf("selector must contain 1..256 UTF-8 bytes")
		}
		paths[i] = cue.ParsePath(selector)
		if err := paths[i].Err(); err != nil {
			return nil, err
		}
	}
	file, err := os.Open(args[1])
	if err != nil {
		return nil, err
	}
	defer file.Close()
	source, err := io.ReadAll(io.LimitReader(file, sourceLimit+1))
	if err != nil {
		return nil, err
	}
	if len(source) > sourceLimit {
		return nil, fmt.Errorf("source exceeds 65536 bytes")
	}
	parsed, err := parser.ParseFile(args[1], source)
	if err != nil {
		return nil, err
	}
	if len(parsed.Imports) != 0 {
		return nil, fmt.Errorf("imports are outside the profile")
	}
	for _, decl := range parsed.Decls {
		// A package clause names this explicit file; it does not load a package.
		if _, ok := decl.(*ast.Package); ok {
			continue
		}
		if err := fields([]ast.Decl{decl}); err != nil {
			return nil, err
		}
	}
	// Never reject root.Err: an unrelated field may contain a semantic conflict.
	root := cuecontext.New().BuildFile(parsed)
	values := make([]cue.Value, len(paths))
	for i, path := range paths {
		values[i] = root.LookupPath(path)
		if !values[i].Exists() {
			return nil, fmt.Errorf("missing selector: %s", path)
		}
		if values[i].IncompleteKind() == cue.StructKind {
			return nil, fmt.Errorf("selector must name a scalar, not a container")
		}
	}
	if mode == "subsume" || mode == "raw-subsume" {
		// Both values come from this build. Do not project either operand with
		// Default or MarshalJSON, or classify its Err, before comparison.
		var err error
		if mode == "raw-subsume" {
			err = values[0].Subsume(values[1], cue.Raw())
		} else {
			err = values[0].Subsume(values[1])
		}
		encoded, encodeErr := json.Marshal(relationObservation{2, mode, err == nil})
		if encodeErr != nil {
			return nil, encodeErr
		}
		return append(encoded, '\n'), nil
	}
	out := observation{FormatVersion: 1, Mode: mode}
	if mode == "default-unify" {
		var left, right bool
		values[0], left = values[0].Default()
		values[1], right = values[1].Default()
		out.LeftDefault, out.RightDefault = &left, &right
	}
	selected := values[0]
	if mode != "inspect" {
		selected = selected.Unify(values[1])
	}
	selected, out.DefaultSelected = selected.Default()
	if selected.Err() != nil {
		out.Status = "conflict"
	} else if raw, err := selected.MarshalJSON(); err != nil {
		// MarshalJSON implicitly calls Default again; this is not ordinary projection.
		out.Status = "incomplete"
	} else {
		out.Status, out.Value = "concrete", raw
	}
	encoded, err := json.Marshal(out)
	if err != nil {
		return nil, err
	}
	return append(encoded, '\n'), nil
}

func main() {
	encoded, err := observe(os.Args[1:])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	if _, err := os.Stdout.Write(encoded); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
