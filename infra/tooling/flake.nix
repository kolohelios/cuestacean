{
  description = "Cuestacean development and validation tools";

  inputs = {
    kolohelios-nix.url = "https://flakehub.com/f/kolohelios/kolohelios-nix/*.tar.gz";
    nixpkgs.follows = "kolohelios-nix/nixpkgs";
    shaka.url = "https://flakehub.com/f/kolohelios/shaka/*.tar.gz";
  };

  outputs =
    {
      kolohelios-nix,
      shaka,
      ...
    }:
    let
      filePrattFixtures = [
        "file-primary"
        "file-pratt"
      ];
      filePrattNames = builtins.concatStringsSep " " filePrattFixtures;
      scannerFixtures = [
        "file-label"
        "file-colon"
        "file-boundary"
        "file-tail"
        "selector-open"
        "slice-separator"
        "selector-identifier"
        "record-open"
        "conjunction-open"
        "record-close"
        "record-label"
        "record-colon"
        "record-tail"
        "tests/valid/bound-inclusive-only"
        "index-close"
        "list-open"
        "list-close"
        "list-tail"
        "call-open"
        "whitespace"
        "after-operand"
        "after-word-operator"
        "group-open"
        "group-close"
        "negation-prefix"
        "not-prefix"
        "bound-prefix"
        "infix"
        "decimal"
        "integer-literal"
        "identifier"
        "string-literal"
        "tests/valid/string-no-unicode"
        "tests/valid/string-no-wide"
        "tests/valid/string-nested"
        "tests/valid/scanner-predicates"
        "tests/valid/scanner-repeat"
        "tests/valid/repeat-scopes"
        "tests/valid/repeat-zero"
        "tests/valid/repeat-fuel"
        "tests/valid/repeat-fuel-over"
        "tests/valid/string-unrolled"
        "tests/valid/scanner-horizontal"
        "tests/valid/scanner-stalled"
        "tests/valid/scanner-advance"
        "tests/valid/scanner-decimal-permissive"
        "tests/valid/scanner-reject"
        "tests/valid/scanner-state"
        "tests/valid/scanner-lookahead"
        "tests/valid/scanner-char-break"
      ];
      scannerNames = builtins.concatStringsSep " " scannerFixtures;
      builderFixtures = [
        "ast-program"
        "ast-atom"
        "ast-integer"
        "ast-text"
        "ast-binary"
        "ast-ternary"
        "ast-unary"
        "ast-record-field"
        "tests/valid/ast-program-v2"
        "tests/valid/ast-escaped"
        "tests/valid/ast-duplicate"
      ];
      builderNames = builtins.concatStringsSep " " builderFixtures;
      builderExportFixtures = builderFixtures ++ [ "tests/valid/ast-literal" ];
      builderExportNames = builtins.concatStringsSep " " builderExportFixtures;
      prattFixtures = [
        "slice-control"
        "tests/valid/slice-control-lower-one"
        "tests/valid/slice-control-tail"
        "conjunction-control"
        "tests/valid/conjunction-control-reverse"
        "tests/valid/conjunction-control-tail"
        "list-control"
        "tests/valid/list-control-homogeneous"
        "record-control"
        "tests/valid/record-control-homogeneous"
        "tests/valid/record-control-reverse"
        "tests/valid/record-control-scalars"
        "tests/valid/list-control-reverse"
        "tests/valid/list-control-scalars"
        "tests/valid/list-control-stalled"
        "tests/valid/list-control-unfinished"
        "primary"
        "record-primary"
        "conjunction-primary"
        "tests/valid/conjunction-primary-high-minimum"
        "tests/valid/primary-no-conjunction"
        "tests/valid/record-primary-high-minimum"
        "tests/valid/primary-no-records"
        "tests/valid/primary-nested"
        "tests/valid/primary-return"
        "tests/valid/primary-return-empty"
        "tests/valid/pratt-return-loop"
        "tests/valid/primary-bound-high-minimum"
        "tests/valid/primary-no-calls"
        "tests/valid/primary-call-high-minimum"
        "tests/valid/primary-no-lists"
        "tests/valid/primary-no-atoms"
        "tests/valid/primary-high-minimum"
        "tests/valid/primary-low-negation"
        "tests/valid/primary-cross-phase"
        "pratt"
        "tests/valid/pratt-no-selectors"
        "tests/valid/pratt-no-slices"
        "tests/valid/pratt-slice-high-minimum"
        "tests/valid/pratt-no-index"
        "tests/valid/pratt-index-high-minimum"
        "tests/valid/pratt-right"
        "tests/valid/pratt-stalled"
        "tests/valid/pratt-invalid-state"
        "tests/valid/pratt-overflow"
      ];
      prattNames = builtins.concatStringsSep " " prattFixtures;
      unifyFixtures = [
        "list-slice"
        "tests/valid/list-slice-identity"
        "index"
        "tests/valid/index-zero"
        "record-index"
        "record-length"
        "tests/valid/record-index-zero"
        "lower-exclusive-bound"
        "upper-exclusive-bound"
        "tests/valid/exclusive-reversed"
        "list-index"
        "tests/valid/list-index-zero"
        "length-control"
        "list-length-control"
        "tests/valid/list-length-zero"
        "string-length-control"
        "tests/valid/length-zero"
        "unify"
        "sum"
        "equal"
        "not-equal"
        "less"
        "less-equal"
        "greater"
        "greater-equal"
        "logical-and"
        "logical-or"
        "logical-not"
        "json-output"
        "integer-output"
        "tests/valid/output-zero"
        "lower-bound"
        "upper-bound"
        "tests/valid/bound-reversed"
        "tests/valid/unary-bound-endpoints"
        "negate-control"
        "difference-control"
        "product-control"
        "divide-control"
        "modulo-control"
        "quotient-control"
        "remainder-control"
        "tests/valid/not-true"
        "tests/valid/control-default"
        "tests/valid/unary-conditions"
        "tests/valid/integer-literals"
        "tests/valid/logic-false"
        "tests/valid/less-false"
        "tests/valid/equal-false"
        "tests/valid/sum-right"
        "tests/valid/unify-right"
        "tests/valid/unify-scoped"
      ];
      unifyNames = builtins.concatStringsSep " " unifyFixtures;
      loweringFixtures = [
        "lowering"
        "pratt-lowering"
        "unify-lowering"
        "ast-lowering"
        "scan-lowering"
        "meta-lowering"
        "tests/valid/lowering-bindings"
        "tests/valid/lowering-identifier"
        "tests/valid/meta-driver-keys"
        "tests/valid/lowering-numbers"
        "tests/valid/lowering-select"
        "tests/valid/lowering-left-twice"
        "tests/valid/lowering-escaped"
        "tests/valid/lowering-reverse"
        "tests/valid/lowering-module-escaped"
        "tests/valid/lowering-collections"
      ];
      loweringNames = builtins.concatStringsSep " " loweringFixtures;
      exportFixtures = [
        "literal"
        "addition"
        "subtraction"
        "negation"
        "multiplication"
        "division"
        "unification"
        "int-constraint"
        "bounds"
        "exclusive-bounds"
        "booleans"
        "strings"
        "string-length"
        "lists"
        "list-length"
        "list-expressions"
        "conjunction"
        "conjunction-conflict"
        "record-expressions"
        "mixed-collections"
        "mixed-unify"
        "mixed-conflict"
        "nested-records"
        "nested-records-unify"
        "nested-records-conflict"
        "nested-records-scalar-conflict"
        "nested-lists"
        "records"
        "records-unify"
        "records-index"
        "records-length"
        "records-equal"
        "records-logical"
        "records-conflict"
        "records-fields-conflict"
        "list-indexing"
        "list-slicing"
        "optional-slices"
        "optional-slices-domain"
        "optional-slices-range"
        "list-slicing-range"
        "list-slicing-domain"
        "collection-index"
        "selectors"
        "collection-index-record"
        "tests/valid/lists-unify"
        "tests/valid/lists-capacity"
        "tests/valid/lists-equal"
        "concatenation"
        "equality"
        "inequality"
        "ordering"
        "ordering-text"
        "logic-and"
        "logic-or"
        "not"
        "null"
        "top"
        "top-null"
        "tests/valid/equality-mixed"
        "tests/valid/strings-empty"
        "tests/valid/strings-equal"
        "tests/valid/strings-incomplete"
        "tests/valid/strings-conflict"
        "tests/valid/strings-mixed"
        "evaluation"
        "grammar"
        "tests/valid/grammar-no-brackets"
        "tests/valid/evaluation-left-twice"
        "tests/valid/pratt-lowering-no-increment"
        "tests/valid/unify-lowering-permissive"
        "tests/valid/integer-kernel-alternate"
        "tests/valid/division-kernel-alternate"
        "tests/valid/unify-equality-probe"
        "tests/valid/ast-lowering-forced"
        "tests/valid/scan-lowering-unicode"
        "tests/valid/meta-lowering-forced"
        "tests/valid/meta-driver-previous"
        "tests/valid/ast-literal"
      ]
      ++ filePrattFixtures
      ++ [ "file-pratt-lowering" ]
      ++ scannerFixtures
      ++ builderFixtures
      ++ prattFixtures
      ++ unifyFixtures
      ++ loweringFixtures;
      fixtureNames = builtins.concatStringsSep " " exportFixtures;
      # Verification of the real consumer functions, not a native input/seed.
      # The all-left context would conceal an absent provider without isolation.
      rightFieldConsumerProbe = ''
        printf '%s\n' 'package bootstrap' '#InstructionLowering & {' \
          'cases:{binaryRuntime:{parts:[_,_,_,_,{value:{value:{field:"left"}}},_]}}' \
          'cases:{sum:{args:[_,{value:{value:{field:"left"}}}]}}' \
          'cases:{less:{args:[_,{value:{value:{field:"left"}}}]}}' \
          'cases:{lessEqual:{args:[_,{value:{value:{field:"left"}}}]}}' \
          'cases:{greater:{args:[_,{value:{value:{field:"left"}}}]}}' \
          'cases:{greaterEqual:{args:[_,{value:{value:{field:"left"}}}]}}' \
          'cases:{notEqual:{args:[_,{value:{value:{field:"left"}}}]}}' \
          'cases:{remainder:{args:[_,{value:{value:{field:"left"}}}]}}' \
          'cases:{unify:{parts:[_,{args:[_,{value:{value:{field:"left"}}}]},_]}}' \
          '}' > right-backfill.cue
        savedRight=$right
        right=${../../bootstrap/instruction-right-field.cue}
        consume_right > right-backfill-positive.json
        test -s right-backfill-positive.json
        printf '%s\n' 'package bootstrap' '#InstructionRightField:{}' > right-missing.cue
        right=right-missing.cue
        if consume_right > right-stopped.json 2> right-stopped.err; then exit 1; fi
        test ! -s right-stopped.json && test -s right-stopped.err
        rightStatus=0
        consume_right > right-stopped-or.json 2> right-stopped-or.err || rightStatus=$?
        test "$rightStatus" -ne 0
        test ! -s right-stopped-or.json && test -s right-stopped-or.err
        right=$savedRight
      '';
      # The dependent operand must be complete before ANY policy/context is loaded.
      instructionOperandConsumer = ''
        group=${../../bootstrap/instruction-operand-cases.cue}
        group_gate() { cue export "$schema" "$bridge" "$right" "$operand" "$group" -e '#InstructionOperandCases' > /dev/null; }
        operand=${../../bootstrap/instruction-right-operand.cue}
        operand_gate() { cue export "$schema" "$bridge" "$right" "$operand" -e '#InstructionRightOperand' > /dev/null; }
        right_gate() {
          cue export "$schema" "$bridge" "$right" -e '#InstructionRightField' > /dev/null || return
          operand_gate || return
          group_gate
        }
        ${instructionRuntimeConsumer}
      '';
      # Only completeness/types: identity comes from closed selection or raw owners.
      instructionModuleConsumer = ''
        module=${../../bootstrap/instruction-module.cue}
        module_gate() {
          local helper
          for helper in InstructionModuleText InstructionModuleFirst InstructionModuleStep InstructionModule; do
            cue export ${../../bootstrap/instruction-module-schema.cue} "$module" "$@" \
              -e "#$helper" > /dev/null || return
          done
        }
      '';
      instructionModuleConsumerProbe = ''
        savedModule=$module; savedText=$text; savedFirst=$first; savedStep=$step
        printf '%s\n' 'package bootstrap' > absent.cue
        for absent in Text First Step Module; do
          module=$savedModule; text=$savedText; first=$savedFirst; step=$savedStep
          case "$absent" in Text) text=absent.cue ;; First) first=absent.cue ;; Step) step=absent.cue ;; Module) module=absent.cue ;; esac
          rm -f full-export-started
          set +e
          consume_module > "$absent-normal.json" 2> "$absent-normal.err"
          moduleStatus=$?
          set -e
          test "$moduleStatus" -eq 1
          moduleStatus=0
          (if consume_module; then exit 99; else exit 17; fi) > "$absent-if.json" 2> "$absent-if.err" || moduleStatus=$?
          test "$moduleStatus" -eq 17
          moduleStatus=0
          (consume_module || exit 17) > "$absent-or.json" 2> "$absent-or.err" || moduleStatus=$?
          test "$moduleStatus" -eq 17
          for mode in normal if or; do
            test ! -s "$absent-$mode.json" && test -s "$absent-$mode.err"
          done
          test ! -e full-export-started
        done
        module=$savedModule; text=$savedText; first=$savedFirst; step=$savedStep
        consume_module > module-recovered.json
        test -s module-recovered.json && test -e full-export-started
      '';
      instructionModuleInverse = ''
        restore_module() {
          awk -v owner=${../../bootstrap/instruction-module.cue} '
            BEGIN {
              while ((getline line < owner) > 0) {
                if (line ~ /^\tmodule:/) copying = 1
                if (copying) body = body line "\n"
                if (copying && line == "\t}]") break
              }
            }
            /^#InstructionModule: close\(\{module: _\}\)$/ { skip = 1; next }
            skip && /^$/ { skip = 0; next }
            /^\tmodule: #InstructionModule.module$/ { printf "%s", body; next }
            { print }
          ' "$1" > "$2"
        }
      '';
      instructionRuntimeConsumer = ''
        ${instructionModuleConsumer}
        runtime=${../../bootstrap/instruction-binary-runtime.cue}
        runtime_gate() { cue export "$schema" "$bridge" "$right" "$operand" "$group" "$1" "$runtime" -e '#InstructionBinaryRuntime' > /dev/null; }
      '';
      instructionRuntimeConsumerProbe = ''
        savedRuntime=$runtime
        printf '%s\n' 'package bootstrap' '#InstructionBinaryRuntime:{}' > runtime-missing.cue
        runtime=runtime-missing.cue
        rm -f full-export-started
        set +e
        consume_core > runtime-stopped-normal.json 2> runtime-stopped-normal.err
        runtimeStatus=$?
        set -e
        test "$runtimeStatus" -eq 1
        runtimeStatus=0
        (if consume_core; then exit 99; else exit 17; fi) > runtime-stopped-if.json 2> runtime-stopped-if.err || runtimeStatus=$?
        test "$runtimeStatus" -eq 17
        runtimeStatus=0
        (consume_core || exit 17) > runtime-stopped-or.json 2> runtime-stopped-or.err || runtimeStatus=$?
        test "$runtimeStatus" -eq 17
        for mode in normal if or; do
          test ! -s "runtime-stopped-$mode.json" && test -s "runtime-stopped-$mode.err"
        done
        test ! -e full-export-started
        runtime=$savedRuntime
        consume_core > runtime-recovery.json
        test -s runtime-recovery.json && test -e full-export-started
      '';
      instructionGroupConsumerProbe = ''
        savedGroup=$group
        printf '%s\n' 'package bootstrap' '#InstructionOperandCases:{}' > group-missing.cue
        group=group-missing.cue
        rm -f full-export-started
        set +e
        consume_operand > group-stopped-normal.json 2> group-stopped-normal.err
        groupStatus=$?
        set -e
        test "$groupStatus" -ne 0
        if consume_operand > group-stopped-if.json 2> group-stopped-if.err; then exit 1; fi
        groupStatus=0
        consume_operand > group-stopped-or.json 2> group-stopped-or.err || groupStatus=$?
        test "$groupStatus" -ne 0
        for mode in normal if or; do
          test ! -s "group-stopped-$mode.json" && test -s "group-stopped-$mode.err"
        done
        test ! -e full-export-started
        group=$savedGroup
      '';
      # Verification-only source inverse; never a constructor or answer provider.
      instructionGroupInverse = ''
        ${instructionModuleInverse}
        restore_runtime() {
          awk -v owner=${../../bootstrap/instruction-binary-runtime.cue} '
            BEGIN {
              while ((getline line < owner) > 0) {
                if (line ~ /^\tlet L = /) forward = 1
                if (line ~ /^\top: /) {
                  forward = 0; body = "\t\tbinaryRuntime: {" substr(line, 2) "\n"; continue
                }
                if (forward) declarations = declarations line "\n"
                if (line == "\t]") { body = body "\t\t]}\n"; break }
                if (body != "") body = body "\t" line "\n"
              }
            }
            /^#InstructionBinaryRuntimeShape:/ { shape = 1; next }
            shape && /^\/\/ Executable recursion/ { shape = 0 }
            shape { next }
            /^#InstructionLowering: / { print; printf "%s", declarations; next }
            /^\tcases: \{/ { cases = 1 }
            cases && /^\t\tbinaryRuntime:/ { printf "%s", body; next }
            cases && /^\t\t[^:]+:[[:space:]]*_$/ { sub(/:[[:space:]]*_$/, ": _") }
            { print }
          ' "$1" > "$2"
        }
        restore_group() {
          awk -v owner=${../../bootstrap/instruction-operand-cases.cue} '
            BEGIN {
              while ((getline line < owner) > 0) {
                if (line ~ /^\tlet L = /) left = line
                if (line == "\tcases: {") { cases = 1; continue }
                if (line == "\t}") cases = 0
                if (cases) {
                  if (line ~ /^\t\t[a-zA-Z]+:/) { key = line; sub(/:.*/, "", key); sub(/^\t\t/, "", key) }
                  body[key] = body[key] line "\n"
                }
              }
            }
            /^#InstructionOperandCases: close\(\{/ { shape = 1; next }
            shape && /^\/\/ Executable recursion/ { shape = 0 }
            shape { next }
            /^\tlet L = #InstructionOperandCases.leftOperand$/ { print left; next }
            /^\t\t#InstructionOperandCases.cases$/ { next }
            /^\t\t(sum|less|lessEqual|greater|greaterEqual|notEqual|remainder|unify):[[:space:]]*_$/ {
              key = $0; sub(/:.*/, "", key); sub(/^\t\t/, "", key); printf "%s", body[key]; next
            }
            { print }
          ' "$1" > "$2"
        }
      '';
      instructionOperandConsumerProbe = ''
        savedOperand=$operand
        printf '%s\n' 'package bootstrap' '#InstructionRightOperand:{}' > operand-missing.cue
        operand=operand-missing.cue
        rm -f full-export-started
        set +e
        consume_operand > operand-stopped-normal.json 2> operand-stopped-normal.err
        operandStatus=$?
        set -e
        test "$operandStatus" -ne 0
        if consume_operand > operand-stopped-if.json 2> operand-stopped-if.err; then exit 1; fi
        operandStatus=0
        consume_operand > operand-stopped-or.json 2> operand-stopped-or.err || operandStatus=$?
        test "$operandStatus" -ne 0
        for mode in normal if or; do
          test ! -s "operand-stopped-$mode.json" && test -s "operand-stopped-$mode.err"
        done
        test ! -e full-export-started
        operand=$savedOperand
        ${instructionGroupConsumerProbe}
      '';
      # Consumer-only validation: no policy schema/context may complete core DATA.
      instructionCoreConsumer = ''
        core=${../../bootstrap/instruction-core-cases.cue}
        coreShape=${../../bootstrap/instruction-core-cases-schema.cue}
        core_gate() { cue export "$coreShape" "$core" > /dev/null; }
      '';
      instructionCoreConsumerProbe = pkgs: ''
        savedCore=$core
        core=${nativeInstructionCoreCases pkgs}/instruction-core-cases.json
        consume_core > core-present.json
        test -s core-present.json
        ${pkgs.jq}/bin/jq 'del(.cases.text.op)' "$core" > core-missing.json
        core="$PWD/core-missing.json"
        set +e
        consume_core > core-stopped-normal.json 2> core-stopped-normal.err
        coreStatus=$?
        set -e
        test "$coreStatus" -ne 0
        if consume_core > core-stopped-if.json 2> core-stopped-if.err; then exit 1; fi
        coreStatus=0
        consume_core > core-stopped-or.json 2> core-stopped-or.err || coreStatus=$?
        test "$coreStatus" -ne 0
        for mode in normal if or; do
          test ! -s "core-stopped-$mode.json" && test -s "core-stopped-$mode.err"
        done
        core=$savedCore
      '';
      # Shared actual production boundary, also executed by the independent verifier.
      instructionCanonicalConsumer = pkgs: sources: ''
        policy_gates() {
          module_gate "''${moduleProviders[@]}" || return
          core_gate || return
          cue export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$rightProvider" -e '#InstructionRightField' > /dev/null || return
          cue export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$rightProvider" "$operand" -e '#InstructionRightOperand' > /dev/null || return
          cue export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$rightProvider" "$operand" "$group" -e '#InstructionOperandCases' > /dev/null || return
          cue export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$PWD/instruction-binary-tables.cue" -e '#InstructionBinaryTables' > /dev/null || return
          cue export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$rightProvider" "$operand" "$group" "$PWD/instruction-binary-tables.cue" "$runtime" -e '#InstructionBinaryRuntime' > /dev/null || return
        }
        export_instruction() {
          test "$#" -eq 0 || return 2
          local module="$PWD/instruction-module.cue"
          local core=${nativeInstructionCoreCases pkgs}/instruction-core-cases.json
          local coreShape=${../../bootstrap/instruction-core-cases-schema.cue}
          local rightProvider="$PWD/instruction-right-field.cue"
          local operand="$PWD/instruction-right-operand.cue"
          local group="$PWD/instruction-operand-cases.cue"
          local runtime="$PWD/instruction-binary-runtime.cue"
          local moduleProviders=("$PWD/instruction-module-text.cue" "$PWD/instruction-module-first.cue" "$PWD/instruction-module-step.cue")
          local inputs=(${sources}/schema.cue ${sources}/lowering-fragments.cue
            "$core" "$rightProvider" "$operand" "$group" "$runtime"
            "$PWD/instruction-binary-tables.cue" ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json
            "$module" "''${moduleProviders[@]}" ${sources}/lowering.cue)
          policy_gates || return
          touch full-export-started
          cue export "''${inputs[@]}"
        }
      '';
      # Verification-only: the conserved former publication probes run ONCE here.
      instructionCanonicalVerification = pkgs: sources: ''
        (
          mkdir -p canonical-go "$out/canonical-go"
          cd canonical-go
          out="$out/canonical-go"
          mkdir home
          argumentLog=setup-actual-arguments
          cue() {
            printf '%s\n' -- invocation "$@" >> "$argumentLog"
            env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"
          }
          jq() { env -i HOME="$PWD/home" PATH=/nonexistent "$queryTool" "$@"; }
          cue import -p bootstrap -l '#InstructionBinaryTables:' \
            -o "$PWD/instruction-binary-tables.cue" \
            ${nativeInstructionBinaryTables pkgs}/instruction-binary-tables.json
          cue import -p bootstrap -l '#InstructionRightField:' \
            -o "$PWD/instruction-right-field.cue" \
            ${nativeInstructionRightField pkgs}/right-field.json
          cue import -p bootstrap -l '#InstructionRightOperand:' \
            -o "$PWD/instruction-right-operand.cue" \
            ${nativeInstructionRightOperand pkgs}/right-operand.json
          operand="$PWD/instruction-right-operand.cue"
          cue import -p bootstrap -l '#InstructionOperandCases:' \
            -o "$PWD/instruction-operand-cases.cue" \
            ${nativeInstructionOperandCases pkgs}/operand-cases.json
          group="$PWD/instruction-operand-cases.cue"
          cue import -p bootstrap -l '#InstructionBinaryRuntime:' \
            -o "$PWD/instruction-binary-runtime.cue" \
            ${nativeInstructionBinaryRuntime pkgs}/binary-runtime.json
          runtime="$PWD/instruction-binary-runtime.cue"
          ${instructionCoreConsumer}
          core=${nativeInstructionCoreCases pkgs}/instruction-core-cases.json
          ${instructionModuleConsumer}
          ${instructionCanonicalConsumer pkgs sources}
          export_policy() {
            case "$fixture" in
              lowering|tests/valid/lowering-left-twice|tests/valid/lowering-reverse) policy_gates || return ;;
            esac
            touch full-export-started
            cue export "$@"
          }
          cue import -p bootstrap -l '#InstructionModule:' \
            -o "$PWD/instruction-module.cue" ${nativeInstructionModule pkgs}/module.json
          cue import -p bootstrap -l '#InstructionModuleText:' \
            -o "$PWD/instruction-module-text.cue" \
            ${nativeInstructionModuleText pkgs}/instruction-module-text.json
          cue import -p bootstrap -l '#InstructionModuleFirst:' \
            -o "$PWD/instruction-module-first.cue" ${nativeInstructionModuleControls pkgs}/first.json
          cue import -p bootstrap -l '#InstructionModuleStep:' \
            -o "$PWD/instruction-module-step.cue" ${nativeInstructionModuleControls pkgs}/step.json
          # Imported cohort members are owned, fixed snapshots of exact constructor inputs.
          chmod a-w instruction-module{,-text,-first,-step}.cue
          moduleProviders=("$PWD/instruction-module-text.cue" ${sources}/instruction-module-first.cue ${sources}/instruction-module-step.cue)
          # Check the actual raw wrapper, including conditional calls.
          right="$PWD/instruction-right-field.cue"
          fixture=lowering
          consume_right() {
            rightProvider=$right
            export_policy ${sources}/schema.cue ${sources}/lowering-fragments.cue \
              "$core" "$right" "$operand" "$group" "$runtime" "$PWD/instruction-binary-tables.cue" \
              ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json \
              "$module" "''${moduleProviders[@]}" right-backfill.cue
          }
          selectedGroup=$group; selectedRuntime=$runtime
          selectedModule="$PWD/instruction-module.cue"
          selectedModuleProviders=("$PWD/instruction-module-text.cue" "$PWD/instruction-module-first.cue" "$PWD/instruction-module-step.cue")
          runtime=${../../bootstrap/instruction-binary-runtime.cue}
          operand=${../../bootstrap/instruction-right-operand.cue}
          group=${../../bootstrap/instruction-operand-cases.cue}
          ${rightFieldConsumerProbe}
          consume_operand() { consume_right; }
          ${instructionOperandConsumerProbe}
          operand="$PWD/instruction-right-operand.cue"
          group=$selectedGroup; runtime=$selectedRuntime
          consume_core() {
            rightProvider=$right
            export_policy ${sources}/schema.cue ${sources}/lowering-fragments.cue \
              "$core" "$right" "$operand" "$group" "$runtime" "$PWD/instruction-binary-tables.cue" \
              ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json \
              "$module" "''${moduleProviders[@]}" ${sources}/lowering.cue
          }
          ${instructionCoreConsumerProbe pkgs}
          ${instructionRuntimeConsumerProbe}
          module=$selectedModule; moduleProviders=("''${selectedModuleProviders[@]}")
          rm -f full-export-started
          if export_instruction unexpected > rejected-arguments.json 2> rejected-arguments.err; then exit 1; fi
          test ! -s rejected-arguments.json && test ! -e full-export-started
          # Poison inherited selection: the zero-argument boundary must ignore it.
          module=missing.cue; moduleProviders=(missing.cue); rightProvider=missing.cue
          export_instruction | jq -S . > canonical-recovery.json
          module=$selectedModule; moduleProviders=("''${selectedModuleProviders[@]}")
          argumentLog=canonical-actual-arguments
          : > "$argumentLog"
          export_instruction | jq -S . > first.json
          export_instruction | jq -S . > second.json
          cmp first.json second.json
          cmp first.json canonical-recovery.json
          mkdir -p "$out"
          cp first.json "$out/lowering.json"
          cmp first.json ${jsonExports pkgs}/lowering.json
          expected_call() { printf '%s\n' -- invocation "$@" >> expected-arguments; }
          : > expected-arguments
          for repeat in first second; do
            for helper in InstructionModuleText InstructionModuleFirst InstructionModuleStep InstructionModule; do
              expected_call export ${../../bootstrap/instruction-module-schema.cue} "$PWD/instruction-module.cue" "$PWD/instruction-module-text.cue" "$PWD/instruction-module-first.cue" "$PWD/instruction-module-step.cue" -e "#$helper"
            done
            expected_call export ${../../bootstrap/instruction-core-cases-schema.cue} ${nativeInstructionCoreCases pkgs}/instruction-core-cases.json
            expected_call export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$PWD/instruction-right-field.cue" -e '#InstructionRightField'
            expected_call export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$PWD/instruction-right-field.cue" "$PWD/instruction-right-operand.cue" -e '#InstructionRightOperand'
            expected_call export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$PWD/instruction-right-field.cue" "$PWD/instruction-right-operand.cue" "$PWD/instruction-operand-cases.cue" -e '#InstructionOperandCases'
            expected_call export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$PWD/instruction-binary-tables.cue" -e '#InstructionBinaryTables'
            expected_call export ${sources}/schema.cue ${sources}/lowering-fragments.cue "$PWD/instruction-right-field.cue" "$PWD/instruction-right-operand.cue" "$PWD/instruction-operand-cases.cue" "$PWD/instruction-binary-tables.cue" "$PWD/instruction-binary-runtime.cue" -e '#InstructionBinaryRuntime'
            expected_call export ${sources}/schema.cue ${sources}/lowering-fragments.cue ${nativeInstructionCoreCases pkgs}/instruction-core-cases.json "$PWD/instruction-right-field.cue" "$PWD/instruction-right-operand.cue" "$PWD/instruction-operand-cases.cue" "$PWD/instruction-binary-runtime.cue" "$PWD/instruction-binary-tables.cue" ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json "$PWD/instruction-module.cue" "$PWD/instruction-module-text.cue" "$PWD/instruction-module-first.cue" "$PWD/instruction-module-step.cue" ${sources}/lowering.cue
          done
          cmp expected-arguments canonical-actual-arguments
          cp expected-arguments canonical-actual-arguments canonical-recovery.json "$out/"
        )
      '';
      makeJsonExports =
        pkgs:
        let
          cueSources = pkgs.lib.fileset.toSource {
            root = ../../bootstrap;
            fileset = pkgs.lib.fileset.unions (
              [
                ../../bootstrap/schema.cue
                ../../bootstrap/scanner-schema.cue
                ../../bootstrap/ast-schema.cue
                ../../bootstrap/scanner-fragments.cue
                ../../bootstrap/file-scanner-fragments.cue
                ../../bootstrap/decimal-fragments.cue
                ../../bootstrap/primary-fragments.cue
                ../../bootstrap/primary-bound-minimum-input.cue
                ../../bootstrap/primary-call-minimum-input.cue
                ../../bootstrap/primary-list-minimum-input.cue
                ../../bootstrap/pratt-fragments.cue
                ../../bootstrap/pratt-index-minimum-input.cue
                ../../bootstrap/pratt-slice-minimum-input.cue
                ../../bootstrap/list-control-fragments.cue
                ../../bootstrap/record-control-fragments.cue
                ../../bootstrap/conjunction-control-fragments.cue
                ../../bootstrap/slice-control-fragments.cue
                ../../bootstrap/slice-lower-default.cue
                ../../bootstrap/record-primary-fragments.cue
                ../../bootstrap/conjunction-primary-fragments.cue
                ../../bootstrap/record-minimum-input.cue
                ../../bootstrap/conjunction-minimum-input.cue
                ../../bootstrap/string-literal-fragments.cue
                ../../bootstrap/equality-fragments.cue
                ../../bootstrap/ordering-fragments.cue
                ../../bootstrap/logic-fragments.cue
                ../../bootstrap/integer-control-fragments.cue
                ../../bootstrap/division-control-fragments.cue
                ../../bootstrap/output-control-fragments.cue
                ../../bootstrap/bound-control-fragments.cue
                ../../bootstrap/exclusive-bound-fragments.cue
                ../../bootstrap/lowering-fragments.cue
                ../../bootstrap/instruction-module.cue
                ../../bootstrap/instruction-operand-cases.cue
                ../../bootstrap/instruction-binary-runtime.cue
                ../../bootstrap/instruction-core-cases.cue
                ../../bootstrap/instruction-core-cases-schema.cue
                ../../bootstrap/instruction-right-field.cue
                ../../bootstrap/instruction-right-operand.cue
                ../../bootstrap/instruction-module-first.cue
                ../../bootstrap/instruction-module-step.cue
                ../../bootstrap/pratt-lowering-fragments.cue
                ../../bootstrap/pratt-lowering-body.cue
                ../../bootstrap/pratt-add-policy.cue
                ../../bootstrap/pratt-lowering-tables.cue
                ../../bootstrap/unify-lowering-fragments.cue
                ../../bootstrap/unify-lowering-helpers.cue
                ../../bootstrap/unify-lowering-values.cue
                ../../bootstrap/unify-lowering-control.cue
                ../../bootstrap/unify-difference-policy.cue
                ../../bootstrap/unify-divide-policy.cue
                ../../bootstrap/unify-same-policy.cue
                ../../bootstrap/ast-lowering-fragments.cue
                ../../bootstrap/ast-lowering-body.cue
                ../../bootstrap/ast-integer-policy.cue
                ../../bootstrap/scan-lowering-fragments.cue
                ../../bootstrap/scan-lowering-helpers.cue
                ../../bootstrap/scan-lowering-flow.cue
                ../../bootstrap/scan-lowering-predicates.cue
                ../../bootstrap/scan-advance-policy.cue
                ../../bootstrap/meta-lowering-fragments.cue
                ../../bootstrap/meta-lowering-body.cue
                ../../bootstrap/meta-text-alias.cue
                ../../bootstrap/meta-next-rule-alias.cue
                ../../bootstrap/tests/invalid
              ]
              ++ map (fixture: ../../bootstrap + "/${fixture}.cue") exportFixtures
            );
          };
        in
        pkgs.runCommand "cuestacean-bootstrap-json"
          {
            nativeBuildInputs = [
              pkgs.cue
              pkgs.jq
            ];
          }
          ''
            set -euo pipefail
            export LC_ALL=C TZ=UTC
            # Import concrete native DATA into the existing Go-backed consumer,
            # never into the independent native constructor.
            cue import -p bootstrap -l '#InstructionBinaryTables:' \
              -o "$PWD/instruction-binary-tables.cue" \
              ${nativeInstructionBinaryTables pkgs}/instruction-binary-tables.json
            ${instructionCoreConsumer}
            ${instructionModuleConsumer}
            ${instructionCanonicalConsumer pkgs cueSources}
            export_policy() {
              case "$fixture" in
                lowering|tests/valid/lowering-left-twice|tests/valid/lowering-reverse) policy_gates || return ;;
              esac
              touch full-export-started
              cue export "$@"
            }
            cue import -p bootstrap -l '#InstructionModuleText:' \
              -o "$PWD/instruction-module-text.cue" \
              ${nativeInstructionModuleText pkgs}/instruction-module-text.json
            cue() { ${pkgs.cue}/bin/cue "$@"; }
            jq() { ${pkgs.jq}/bin/jq "$@"; }
            source ${./slice-lower-provider.sh}
            source ${./primary-minimum-provider.sh}
            source ${./primary-control-minima-provider.sh}
            source ${./pratt-minima-provider.sh}
            for fixture in ${fixtureNames}; do
              echo "exporting $fixture"
              case "$fixture" in
                lowering)
                  mkdir -p "$out"
                  cp ${nativeInstructionLoweringPolicy pkgs}/lowering.json "$out/lowering.json"
                  continue
                  ;;
                pratt-lowering)
                  mkdir -p "$out"
                  cp ${nativePrattLoweringPolicy pkgs}/pratt-lowering.json "$out/pratt-lowering.json"
                  continue
                  ;;
                unify-lowering)
                  mkdir -p "$out"
                  cp ${nativeUnifyLoweringPolicy pkgs}/unify-lowering.json "$out/unify-lowering.json"
                  continue
                  ;;
                scan-lowering)
                  mkdir -p "$out"
                  cp ${nativeScanLoweringPolicy pkgs}/scan-lowering.json "$out/scan-lowering.json"
                  continue
                  ;;
                meta-lowering)
                  mkdir -p "$out"
                  cp ${nativeMetaLoweringPolicy pkgs}/meta-lowering.json "$out/meta-lowering.json"
                  continue
                  ;;
                ast-lowering)
                  mkdir -p "$out"
                  cp ${nativeAstLoweringPolicy pkgs}/ast-lowering.json "$out/ast-lowering.json"
                  continue
                  ;;
                ${builtins.concatStringsSep "|" builderExportFixtures})
                  mkdir -p "$(dirname "$out/$fixture.json")"
                  cp ${nativeBuilders pkgs}/"$fixture.json" "$out/$fixture.json"
                  continue
                  ;;
              esac
              # Load only the fixture's definition closure. Loading every compiler
              # for a literal can exhaust the CI runner while checking unused CUE.
              schema=schema
              case "$fixture" in
                ${builtins.concatStringsSep "|" scannerFixtures}) schema=scanner-schema ;;
              esac
              fragments=()
              case "$fixture" in
                file-pratt-lowering)
                  ${pkgs.cue}/bin/cue export "${cueSources}/schema.cue" \
                    "${cueSources}/file-pratt-lowering.cue" \
                    ${nativePrattLoweringPolicy pkgs}/pratt-lowering.json > "$out/$fixture.json"
                  continue ;;
                file-pratt) fragments=(pratt-fragments) ;;
                file-label|file-colon|file-boundary|file-tail) fragments=(scanner-fragments file-scanner-fragments) ;;
                pratt|tests/valid/pratt-no-slices|tests/valid/pratt-slice-high-minimum|tests/valid/pratt-no-selectors|tests/valid/pratt-no-index|tests/valid/pratt-index-high-minimum|tests/valid/pratt-right) fragments=(pratt-fragments) ;;
                list-control|tests/valid/list-control-reverse|tests/valid/list-control-scalars|tests/valid/list-control-homogeneous) fragments=(list-control-fragments) ;;
                slice-control|tests/valid/slice-control-lower-one|tests/valid/slice-control-tail) fragments=(slice-control-fragments) ;;
                conjunction-control|tests/valid/conjunction-control-reverse|tests/valid/conjunction-control-tail) fragments=(conjunction-control-fragments) ;;
                record-control|tests/valid/record-control-reverse|tests/valid/record-control-scalars|tests/valid/record-control-homogeneous) fragments=(record-control-fragments) ;;
                conjunction-primary|tests/valid/conjunction-primary-high-minimum) fragments=(conjunction-primary-fragments) ;;
                record-primary|tests/valid/record-primary-high-minimum) fragments=(record-primary-fragments) ;;
                primary|tests/valid/primary-no-conjunction|tests/valid/primary-bound-high-minimum|tests/valid/primary-call-high-minimum|tests/valid/primary-no-lists|tests/valid/primary-no-records) fragments=(primary-fragments) ;;
                decimal|integer-literal) fragments=(decimal-fragments) ;;
                string-literal|tests/valid/string-no-unicode|tests/valid/string-no-wide|tests/valid/string-nested|tests/valid/string-unrolled) fragments=(string-literal-fragments) ;;
                whitespace|after-operand|after-word-operator|group-close|call-open|index-close)
                  fragments=(scanner-fragments) ;;
                lowering|tests/valid/lowering-left-twice|tests/valid/lowering-reverse)
                  fragments=(lowering-fragments) ;;
                pratt-lowering|tests/valid/pratt-lowering-*) fragments=(pratt-lowering-fragments) ;;
                unify-lowering|tests/valid/unify-lowering-*|tests/valid/integer-kernel-alternate|tests/valid/division-kernel-alternate) fragments=(unify-lowering-fragments) ;;
                ast-lowering|tests/valid/ast-lowering-*) fragments=(ast-lowering-fragments) ;;
                scan-lowering|tests/valid/scan-lowering-*) fragments=(scan-lowering-fragments) ;;
                meta-lowering|tests/valid/meta-lowering-*|tests/valid/meta-driver-previous)
                  fragments=(meta-lowering-fragments) ;;
                equal|not-equal) fragments=(equality-fragments) ;;
                less|less-equal|greater|greater-equal) fragments=(ordering-fragments) ;;
                logical-and|logical-or) fragments=(equality-fragments logic-fragments) ;;
                json-output|integer-output) fragments=(output-control-fragments) ;;
                lower-exclusive-bound|upper-exclusive-bound|tests/valid/exclusive-reversed) fragments=(exclusive-bound-fragments) ;;
                lower-bound|upper-bound|tests/valid/bound-reversed) fragments=(bound-control-fragments) ;;
                difference-control|product-control) fragments=(integer-control-fragments) ;;
                divide-control|modulo-control|quotient-control|remainder-control) fragments=(division-control-fragments) ;;
              esac
              # Every open/context path chooses RAW Module and the complete provider
              # tuple BEFORE constructing any array, never by patching it afterwards.
              module=${cueSources}/instruction-module.cue
              moduleProviders=("$PWD/instruction-module-text.cue" ${cueSources}/instruction-module-first.cue ${cueSources}/instruction-module-step.cue)
              slice_lower_provider "$fixture" ${nativeSliceLower pkgs}/slice-lower.json \
                ${cueSources}/slice-lower-default.cue
              primary_minimum_provider "$fixture" \
                ${nativePrimaryMinimum pkgs "record"}/record-minimum.json \
                ${nativePrimaryMinimum pkgs "conjunction"}/conjunction-minimum.json \
                ${cueSources}/record-minimum-input.cue ${cueSources}/conjunction-minimum-input.cue
              primary_control_minima_provider "$fixture" \
                ${nativePrimaryMinimum pkgs "primary-bound"}/primary-bound-minimum.json \
                ${nativePrimaryMinimum pkgs "primary-call"}/primary-call-minimum.json \
                ${nativePrimaryMinimum pkgs "primary-list"}/primary-list-minimum.json \
                ${cueSources}/primary-bound-minimum-input.cue \
                ${cueSources}/primary-call-minimum-input.cue \
                ${cueSources}/primary-list-minimum-input.cue
              pratt_minima_provider "$fixture" \
                ${nativePrimaryMinimum pkgs "pratt-index"}/pratt-index-minimum.json \
                ${nativePrimaryMinimum pkgs "pratt-slice"}/pratt-slice-minimum.json \
                ${cueSources}/pratt-index-minimum-input.cue \
                ${cueSources}/pratt-slice-minimum-input.cue
              inputs=("${cueSources}/$schema.cue" "''${primary_control_providers[@]}" "''${pratt_minimum_providers[@]}")
              if [ -n "$slice_provider" ]; then inputs+=("$slice_provider"); fi
              if [ -n "$min_provider" ]; then inputs+=("$min_provider"); fi
              for fragment in "''${fragments[@]}"; do inputs+=("${cueSources}/$fragment.cue"); done
              case "$fixture" in
                lowering|tests/valid/lowering-left-twice|tests/valid/lowering-reverse)
                  inputs+=("${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json"
                    "$module" "''${moduleProviders[@]}" "$PWD/instruction-binary-tables.cue")
                  core=${nativeInstructionCoreCases pkgs}/instruction-core-cases.json
                  case "$fixture" in
                    lowering)
                      group="$PWD/instruction-operand-cases.cue"
                      runtime="$PWD/instruction-binary-runtime.cue" ;;
                    *)
                      core="${cueSources}/instruction-core-cases.cue"
                      group="${cueSources}/instruction-operand-cases.cue"
                      runtime="${cueSources}/instruction-binary-runtime.cue" ;;
                  esac
                  inputs+=("$core" "$group" "$runtime") ;;
              esac
              case "$fixture" in
                lowering)
                  rightProvider="$PWD/instruction-right-field.cue"
                  operand="$PWD/instruction-right-operand.cue"
                  inputs+=("$operand" "$rightProvider") ;;
                tests/valid/lowering-left-twice|tests/valid/lowering-reverse)
                  rightProvider="${cueSources}/instruction-right-field.cue"
                  operand="${cueSources}/instruction-right-operand.cue"
                  inputs+=("$operand" "$rightProvider") ;;
              esac
              inputs+=("${cueSources}/$fixture.cue")
              # Choose the provider before execution: overrides need the raw
              # preference, never the already materialized default artifact.
              case "$fixture" in
                tests/valid/pratt-lowering-*)
                  inputs+=("${cueSources}/pratt-lowering-body.cue" "${cueSources}/pratt-add-policy.cue"
                    "${cueSources}/pratt-lowering-tables.cue") ;;
                tests/valid/unify-lowering-*|tests/valid/integer-kernel-alternate|tests/valid/division-kernel-alternate)
                  inputs+=("${cueSources}/unify-lowering-helpers.cue" "${cueSources}/unify-lowering-values.cue"
                    "${cueSources}/unify-lowering-control.cue" "${cueSources}/unify-difference-policy.cue"
                    "${cueSources}/unify-divide-policy.cue" "${cueSources}/unify-same-policy.cue") ;;
                tests/valid/ast-lowering-*)
                  inputs+=("${cueSources}/ast-lowering-body.cue" "${cueSources}/ast-integer-policy.cue") ;;
                tests/valid/scan-lowering-*)
                  inputs+=("${cueSources}/scan-lowering-helpers.cue" "${cueSources}/scan-lowering-flow.cue"
                    "${cueSources}/scan-lowering-predicates.cue" "${cueSources}/scan-advance-policy.cue") ;;
                tests/valid/meta-lowering-*|tests/valid/meta-driver-previous)
                  inputs+=("${cueSources}/meta-lowering-body.cue" "${cueSources}/meta-text-alias.cue" "${cueSources}/meta-next-rule-alias.cue") ;;
              esac
              export_policy "''${inputs[@]}" --out json | jq -S . > first.json
              export_policy "''${inputs[@]}" --out json | jq -S . > second.json
              cmp first.json second.json
              mkdir -p "$(dirname "$out/$fixture.json")"
              cp first.json "$out/$fixture.json"
            done
            for fixture in ${cueSources}/tests/invalid/*.cue; do
              if cue export ${cueSources}/schema.cue ${cueSources}/scanner-schema.cue ${cueSources}/ast-schema.cue "$fixture" --out json > invalid.log 2>&1; then
                echo "Expected rejection: $fixture" >&2
                exit 1
              fi
              if grep -Eq 'reference .*not found|undefined' invalid.log; then
                echo "Missing definition in invalid fixture: $fixture" >&2
                exit 1
              fi
            done
          '';

      rustPackage =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "bootstrap-executor";
          meta.mainProgram = "bootstrap-executor";
          version = "0.1.0";
          # Documentation is not an input to either Rust package build.
          src = pkgs.lib.fileset.toSource {
            root = ../../bootstrap;
            fileset = pkgs.lib.fileset.fileFilter (file: !file.hasExt "md") ../../bootstrap;
          };
          postUnpack = ''
            sourceRoot="$sourceRoot/executor"
          '';
          cargoLock.lockFile = ../../bootstrap/executor/Cargo.lock;
          # Test-only provider; seed overrides inherit the native artifact, not its
          # Go-CUE-dependent verifier. No policy JSON is embedded in seed binaries.
          CUESTACEAN_DEMAND_POLICIES = nativeDemandPolicies pkgs;
          nativeCheckInputs = [
            pkgs.rustfmt
            pkgs.clippy
            # Architecture tests run mandatory pinned upstream probes.
            pkgs.cue
          ];
          preCheck = ''
            cargo fmt --check
            cargo clippy --offline --all-targets -- -D warnings
            (
              mkdir lowering-profile-save
              mv ../generated/lowering*.rs ../generated/pratt-lowering.rs ../generated/unify-lowering.rs ../generated/ast-lowering.rs ../generated/scan-lowering.rs ../generated/meta-lowering.rs ../generated/meta-driver-*.rs lowering-profile-save/
              trap 'mv lowering-profile-save/*.rs ../generated/; rmdir lowering-profile-save' EXIT
              cargo check --offline --no-default-features --features semantics --bins
            )
            cargo check --offline --no-default-features --features compiled-lowering --bins
            cargo test --offline --no-default-features --lib
            cargo test --offline --no-default-features --test solver_spike
            cargo test --offline --no-default-features --test default_provenance
            cargo test --offline --no-default-features --test default_readiness
            cargo test --offline --no-default-features --test demand_policy
            cargo test --offline --no-default-features --test preference_projection
            cargo test --offline --no-default-features --test transparent_preferences
            cargo test --offline --no-default-features --test context_preferences
          '';
        };

      # Test-only command built inside the exact pinned CUE module/vendor tree.
      # Do not put this provider in rustPackage: codegenSeed inherits that base.
      defaultObserver =
        pkgs:
        pkgs.runCommand "cuestacean-default-observer"
          {
            nativeBuildInputs = [ pkgs.go ];
            meta.mainProgram = "cuestacean-default-observer";
          }
          ''
            cp -R ${pkgs.cue.src} source
            chmod -R u+w source
            cd source
            ln -s ${pkgs.cue.goModules} vendor
            mkdir -p cmd/cuestacean-default-observer
            cp ${../../bootstrap/oracles/default-observer/main.go} cmd/cuestacean-default-observer/main.go
            cp ${../../bootstrap/oracles/default-observer/main_test.go} cmd/cuestacean-default-observer/main_test.go
            export CGO_ENABLED=0 GOTOOLCHAIN=local GOPROXY=off GOSUMDB=off
            export HOME=$TMPDIR GOCACHE=$TMPDIR/go-cache
            go test -mod=vendor ./cmd/cuestacean-default-observer
            mkdir -p "$out/bin"
            go build -mod=vendor -trimpath -o "$out/bin/cuestacean-default-observer" ./cmd/cuestacean-default-observer
          '';
      # Separate instrumented oracle: the pinned evaluator is changed only by
      # ten diagnostic-only hook lines; old modes stay byte-stable. No seed wiring.
      defaultTrace =
        pkgs:
        pkgs.runCommand "cuestacean-default-trace"
          {
            nativeBuildInputs = [
              pkgs.go
              pkgs.perl
            ];
            meta.mainProgram = "cuestacean-default-trace";
          }
          ''
            cp -R ${pkgs.cue.src} source
            chmod -R u+w source
            cd source
            ln -s ${pkgs.cue.goModules} vendor
            mkdir -p cmd/cuestacean-default-trace "$out/bin"
            cp ${../../bootstrap/oracles/default-trace/adt.go} internal/core/adt/cuestacean_trace.go
            cp ${../../bootstrap/oracles/default-trace/adt_test.go} internal/core/adt/cuestacean_trace_test.go
            cp ${../../bootstrap/oracles/default-trace/main.go} cmd/cuestacean-default-trace/main.go
            cp ${../../bootstrap/oracles/default-trace/main_test.go} cmd/cuestacean-default-trace/main_test.go
            perl -0pi -e '
              my $n = s~(\Qfunc (n *nodeContext) crossProduct(dst, cross []*nodeContext, dn *envDisjunct, mode runMode) []*nodeContext {\E\n)~$1\ttraceID := cuestaceanTraceEnter()\n~g;
              die "trace entry hook drift" unless $n == 1;
              $n = s~(\ttraceID := cuestaceanTraceEnter\(\)\n)~$1\tif cuestaceanTraceEqualityEnabled() { cuestaceanTraceRunMode(traceID, int(mode)) }\n~g;
              die "trace run mode hook drift" unless $n == 1;
              $n = s~(\t\t\ttmp = append\(tmp, r\)\n)~\t\t\tcuestaceanTraceOrigin(traceID, p, int(priority))\n$1~g;
              die "trace origin hook drift" unless $n == 1;
              $n = s~(\thasNonMaybe := false\n)~\tcuestaceanTraceTrials(traceID, tmp, leftDropsDefault, rightDropsDefault)\n$1~g;
              die "trace trials hook drift" unless $n == 1;
              $n = s~(\treturn dst\n\}\n\nfunc combineDefault2)~\tcuestaceanTraceResults(traceID, dst)\n$1~g;
              die "trace results hook drift" unless $n == 1;
              $n = s~(\tnv := x.node.DerefValue\(\)\n)~\ttraceAppend := cuestaceanTraceAppend(x)\n$1~g;
              die "trace append entry hook drift" unless $n == 1;
              $n = s~(\t\tif xn == x \{\n)~$1\t\t\tcuestaceanTraceIdentity(traceAppend, xn)\n~g;
              die "trace identity hook drift" unless $n == 1;
              $n = s~(\t\txv := xn.node.DerefValue\(\)\n)~$1\t\tif cuestaceanTraceEqualityEnabled() { cuestaceanTraceCompare(traceAppend, xn, xv.status != finalized || nv.status != finalized) }\n~g;
              die "trace comparison hook drift" unless $n == 1;
              $n = s~(\t\t// free vertex\n)~\t\tcuestaceanTraceAccept(traceAppend)\n$1~g;
              die "trace accept hook drift" unless $n == 1;
              $n = s~(\treturn append\(a, x\)\n)~\tcuestaceanTraceInsert(traceAppend, len(a))\n$1~g;
              die "trace insert hook drift" unless $n == 1;
            ' internal/core/adt/disjunct2.go
            # Do not gofmt the injected upstream file: guards are single lines.
            # Removing exactly ten diagnostic lines must restore pinned bytes.
            perl -ne 'if (/^\s*(?:traceID := cuestaceanTraceEnter\(\)|traceAppend := cuestaceanTraceAppend\(x\)|cuestaceanTrace(?:Origin|Trials|Results|Identity|Accept|Insert)\(.*\)|if cuestaceanTraceEqualityEnabled\(\) \{ cuestaceanTrace(?:RunMode|Compare)\(.*\) \})\n$/) { $n++ } else { print } END { die "trace restoration hook count" unless $n == 10 }' \
              internal/core/adt/disjunct2.go > restored.go
            cmp restored.go ${pkgs.cue.src}/internal/core/adt/disjunct2.go
            diff -u --label pinned/disjunct2.go --label instrumented/disjunct2.go ${pkgs.cue.src}/internal/core/adt/disjunct2.go internal/core/adt/disjunct2.go > "$out/instrumentation.diff" || test "$?" = 1
            export CGO_ENABLED=0 GOTOOLCHAIN=local GOPROXY=off GOSUMDB=off
            export HOME=$TMPDIR GOCACHE=$TMPDIR/go-cache
            go test -mod=vendor ./internal/core/adt -run '^TestCuestacean'
            go test -mod=vendor ./cmd/cuestacean-default-trace
            go build -mod=vendor -trimpath -o "$out/bin/cuestacean-default-trace" ./cmd/cuestacean-default-trace
          '';
      defaultObserverVerification =
        pkgs:
        let
          # This one seed builds all six fresh compilers. .drv roots include
          # recursive build-time dependencies, unlike runtime output references.
          seedClosure = pkgs.closureInfo { rootPaths = [ (codegenSeed pkgs).drvPath ]; };
        in
        pkgs.runCommand "cuestacean-default-observer-verification" { nativeBuildInputs = [ pkgs.jq ]; } ''
          for forbidden in ${(defaultObserver pkgs).drvPath} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${seedClosure}/store-paths; then
              echo "observer leaked into compiler construction graph" >&2
              exit 1
            fi
          done
          mkdir -p "$out"
          cp ${seedClosure}/store-paths "$out/seed-build-paths"
          cp ${seedClosure}/registration "$out/seed-build-registration"
          printf '%s\n' ${(codegenSeed pkgs).drvPath} > "$out/seed-build-roots"
          printf '%s\n' ${(defaultObserver pkgs).drvPath} > "$out/forbidden-observer-drv"
          printf 'a: *1|2\n' > probe.cue
          env -i PATH=/nonexistent ${defaultObserver pkgs}/bin/cuestacean-default-observer inspect probe.cue a > "$out/observation.json"
          printf '%s\n' '{"formatVersion":1,"mode":"inspect","leftDefault":null,"rightDefault":null,"defaultSelected":true,"status":"concrete","value":1}' > expected.json
          cmp expected.json "$out/observation.json"
          printf '%s\n' ${(defaultTrace pkgs).drvPath} > "$out/forbidden-trace-drv"
          env -i PATH=/nonexistent ${defaultTrace pkgs}/bin/cuestacean-default-trace trace probe.cue a > "$out/trace.json"
          jq -c '{formatVersion,mode,defaultSelected,status,value}' "$out/trace.json" > trace-summary.json
          printf '%s\n' '{"formatVersion":1,"mode":"trace","defaultSelected":true,"status":"concrete","value":1}' > trace-expected.json
          cmp trace-expected.json trace-summary.json
          jq -e '.batches | length > 0' "$out/trace.json" > /dev/null
          env -i PATH=/nonexistent ${defaultTrace pkgs}/bin/cuestacean-default-trace trace-origins probe.cue a > "$out/trace-origins.json"
          jq -e '.formatVersion == 2 and .mode == "trace-origins" and .value == 1 and all(.batches[].trials[]; .leftMode == "maybe" and .leftOriginalMode == "maybe")' "$out/trace-origins.json" > /dev/null
          env -i PATH=/nonexistent ${defaultTrace pkgs}/bin/cuestacean-default-trace trace-history probe.cue a > "$out/trace-history.json"
          jq -e '.formatVersion == 3 and .mode == "trace-history" and .value == 1 and .batches[0].results == [{mode:"default",originalMode:"default",value:1},{mode:"not",originalMode:"not",value:2}]' "$out/trace-history.json" > /dev/null
          mkdir -p "$TMPDIR/equality-home"
          env -i PATH=/nonexistent HOME="$TMPDIR/equality-home" ${defaultTrace pkgs}/bin/cuestacean-default-trace trace-equality probe.cue a > "$out/trace-equality.json"
          jq -e '.formatVersion == 4 and .mode == "trace-equality" and .value == 1 and .batches[0].phase == {runMode:"finalize",appends:[{candidate:{mode:"default",originalMode:"default",value:1},index:0,kind:"insert",comparisons:[]},{candidate:{mode:"not",originalMode:"not",value:2},index:1,kind:"insert",comparisons:[{index:0,kind:"final",existing:{mode:"default",originalMode:"default",value:1},accepted:false}]}]}' "$out/trace-equality.json" > /dev/null
        '';

      # Delete only staged build copies, never the working tree. Neither stage
      # may consume leftover working-tree JSON or committed Rust intermediates.
      clearArtifacts = ''
        for fixture in ${fixtureNames}; do
          rm -f "../$fixture.json"
        done
        rm -f ../generated/*.rs
      '';

      # Exact production cut: no executor library, parser, generated answers or tests.
      nativeCoreSources =
        pkgs: includeTests:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [
              ../../bootstrap/native-codegen-core/Cargo.toml
              ../../bootstrap/native-codegen-core/Cargo.lock
              ../../bootstrap/native-codegen-core/src/lib.rs
              ../../bootstrap/executor/src/ast.rs
              ../../bootstrap/executor/src/lowering.rs
              ../../bootstrap/executor/src/module.rs
              ../../bootstrap/executor/src/bin/bootstrap-ast-codegen.rs
              ../../bootstrap/executor/src/bin/bootstrap-lower-codegen.rs
            ]
            ++ pkgs.lib.optionals includeTests [ ../../bootstrap/native-codegen-core/tests/compiler.rs ]
          );
        };
      nativeCorePolicies =
        pkgs:
        pkgs.runCommand "cuestacean-native-core-policies" { } ''
          mkdir -p "$out"
          cp ${nativeAstLoweringPolicy pkgs}/ast-lowering.json "$out/ast-lowering.json"
          cp ${nativeMetaLoweringPolicy pkgs}/meta-lowering.json "$out/meta-lowering.json"
        '';
      nativeCodegenCore =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "cuestacean-native-codegen-core";
          version = "0.1.0";
          src = nativeCoreSources pkgs false;
          postUnpack = ''
            sourceRoot="$sourceRoot/native-codegen-core"
          '';
          cargoLock.lockFile = ../../bootstrap/native-codegen-core/Cargo.lock;
          CUESTACEAN_BOOTSTRAP_JSON = nativeCorePolicies pkgs;
          cargoBuildFlags = [ "--no-default-features" ];
          # Construction never executes the shared defensive tests or expected code.
          doCheck = false;
          meta.license = pkgs.lib.licenses.asl20;
        };
      nativeCoreTests =
        pkgs:
        (nativeCodegenCore pkgs).overrideAttrs (_: {
          pname = "cuestacean-native-codegen-core-tests";
          src = nativeCoreSources pkgs true;
          nativeCheckInputs = [ pkgs.clippy ];
          doCheck = true;
          postPatch = ''
            (
              set -euo pipefail
              mkdir -p ../generated "$TMPDIR/core-home"
              for tool in ${nativeCodegenCore pkgs}/bin/bootstrap-ast-codegen ${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen; do
                case "$tool" in /*) ;; *) echo "nonabsolute compiler provider: $tool" >&2; exit 1 ;; esac
                test -r "$tool" && test -x "$tool"
              done
              for name in ast-program ast-atom ast-integer ast-text ast-unary ast-record-field ast-binary ast-ternary; do
                env -i PATH=/nonexistent HOME="$TMPDIR/core-home" \
                  ${nativeCodegenCore pkgs}/bin/bootstrap-ast-codegen \
                  ${nativeBuilders pkgs}/"$name.json" > "../generated/$name.rs"
              done
              for name in ast-lowering meta-lowering; do
                env -i PATH=/nonexistent HOME="$TMPDIR/core-home" \
                  ${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen \
                  ${nativeCorePolicies pkgs}/"$name.json" > "../generated/$name.rs"
              done
            )
          '';
          checkPhase = ''
            runHook preCheck
            for profile in debug release; do
              releaseFlag=
              if [ "$profile" = release ]; then releaseFlag=--release; fi
              for features in "" source compiled-lowering source,compiled-lowering; do
                cargo test --offline --locked --all-targets --no-default-features \
                  $releaseFlag --features "$features"
                cargo clippy --offline --locked --all-targets --no-default-features \
                  $releaseFlag --features "$features" -- -D warnings
              done
            done
            runHook postCheck
          '';
        });
      nativeScannerSources =
        pkgs: includeTests:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [
              ../../bootstrap/native-scan-codegen/Cargo.toml
              ../../bootstrap/native-scan-codegen/Cargo.lock
              ../../bootstrap/native-scan-codegen/src/lib.rs
              ../../bootstrap/executor/src/scanner.rs
              ../../bootstrap/executor/src/lowering.rs
              ../../bootstrap/executor/src/module.rs
              ../../bootstrap/executor/src/bin/bootstrap-scan-codegen.rs
            ]
            ++ pkgs.lib.optionals includeTests [ ../../bootstrap/native-scan-codegen/tests/compiler.rs ]
          );
        };
      nativeScannerPolicies =
        pkgs:
        pkgs.runCommand "cuestacean-native-scanner-policies" { } ''
          mkdir -p "$out"
          cp ${nativeScanLoweringPolicy pkgs}/scan-lowering.json "$out/scan-lowering.json"
          cp ${nativeMetaLoweringPolicy pkgs}/meta-lowering.json "$out/meta-lowering.json"
        '';
      nativeScanCodegen =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "cuestacean-native-scan-codegen";
          version = "0.1.0";
          src = nativeScannerSources pkgs false;
          postUnpack = ''
            sourceRoot="$sourceRoot/native-scan-codegen"
          '';
          cargoLock.lockFile = ../../bootstrap/native-scan-codegen/Cargo.lock;
          CUESTACEAN_BOOTSTRAP_JSON = nativeScannerPolicies pkgs;
          cargoBuildFlags = [ "--no-default-features" ];
          # Construction never executes shared defensive tests or expected code.
          doCheck = false;
          meta.license = pkgs.lib.licenses.asl20;
        };
      nativeScannerTests =
        pkgs:
        (nativeScanCodegen pkgs).overrideAttrs (_: {
          pname = "cuestacean-native-scan-codegen-tests";
          src = nativeScannerSources pkgs true;
          nativeCheckInputs = [ pkgs.clippy ];
          doCheck = true;
          postPatch = ''
            (
              set -euo pipefail
              mkdir -p ../generated "$TMPDIR/scanner-home"
              for tool in ${nativeScanCodegen pkgs}/bin/bootstrap-scan-codegen ${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen; do
                case "$tool" in /*) ;; *) echo "nonabsolute compiler provider: $tool" >&2; exit 1 ;; esac
                test -r "$tool" && test -x "$tool"
              done
              # PROGRAM specifications are Go-exported: this derivative is a VERIFIER.
              # Only the 27 production wrappers, never copied committed Rust.
              for name in ${
                builtins.concatStringsSep " " (
                  builtins.filter (name: !(pkgs.lib.hasPrefix "tests/" name)) scannerFixtures
                )
              }; do
                env -i PATH=/nonexistent HOME="$TMPDIR/scanner-home" \
                  ${nativeScanCodegen pkgs}/bin/bootstrap-scan-codegen \
                  ${jsonExports pkgs}/"$name.json" > "../generated/$name.rs"
              done
              for name in scan-lowering meta-lowering; do
                env -i PATH=/nonexistent HOME="$TMPDIR/scanner-home" \
                  ${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen \
                  ${nativeScannerPolicies pkgs}/"$name.json" > "../generated/$name.rs"
              done
            )
          '';
          checkPhase = ''
            runHook preCheck
            for profile in debug release; do
              releaseFlag=
              if [ "$profile" = release ]; then releaseFlag=--release; fi
              for features in "" source compiled-lowering source,compiled-lowering; do
                cargo test --offline --locked --all-targets --no-default-features \
                  $releaseFlag --features "$features"
                cargo clippy --offline --locked --all-targets --no-default-features \
                  $releaseFlag --features "$features" -- -D warnings
              done
            done
            runHook postCheck
          '';
        });
      nativeScannerVerification =
        pkgs:
        let
          scanner = nativeScanCodegen pkgs;
          closure = pkgs.closureInfo { rootPaths = [ scanner.drvPath ]; };
        in
        pkgs.runCommand "cuestacean-native-scan-codegen-verification"
          {
            nativeBuildInputs = [ pkgs.rustfmt ];
            featureTests = nativeScannerTests pkgs;
            # Retain all prior source-effect/typed gates and real compiled Unicode control.
            policyVerification = nativeScanLoweringPolicyVerification pkgs;
            fullExecutor = executor pkgs;
          }
          ''
            set -euo pipefail
            mkdir -p "$out" "$TMPDIR/scanner-home"
            for tool in ${scanner}/bin/bootstrap-scan-codegen ${pkgs.rustfmt}/bin/rustfmt; do
              case "$tool" in /*) ;; *) echo "nonabsolute verifier tool: $tool" >&2; exit 1 ;; esac
              test -r "$tool" && test -x "$tool"
            done
            test -d "$featureTests" && test -d "$policyVerification" && test -d "$fullExecutor"
            test "$(find ${nativeScannerSources pkgs false} -type f | wc -l)" -eq 7
            test "$(find ${nativeScannerSources pkgs true} -type f | wc -l)" -eq 8
            printf '%s\n' meta-lowering.json scan-lowering.json > expected-policies
            find ${nativeScannerPolicies pkgs} -type f -printf '%f\n' | sort > policies
            cmp expected-policies policies
            for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
              ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
              ${executor pkgs} ${(executor pkgs).drvPath} \
              ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
              ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
              ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
              ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
              ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${closure}/store-paths; then
                echo "forbidden native scanner compiler dependency: $forbidden" >&2; exit 1
              fi
            done
            test -x ${codegenSeed pkgs}/bin/bootstrap-scan-codegen
            generate() {
              env -i PATH=/nonexistent HOME="$TMPDIR/scanner-home" \
                ${scanner}/bin/bootstrap-scan-codegen "$@" | \
                env -i PATH=/nonexistent HOME="$TMPDIR/scanner-home" ${pkgs.rustfmt}/bin/rustfmt --edition 2024
            }
            for machine in ${scannerNames}; do
              generate ${jsonExports pkgs}/"$machine.json" > first.rs
              generate ${jsonExports pkgs}/"$machine.json" > second.rs
              cmp first.rs second.rs
              cmp first.rs ${../../bootstrap/generated}/"$(basename "$machine").rs"
              cp first.rs "$out/$(basename "$machine").rs"
            done
            generate ${jsonExports pkgs}/tests/valid/scanner-advance.json \
              ${jsonExports pkgs}/tests/valid/scan-lowering-unicode.json > "$out/unicode.rs"
            cmp "$out/unicode.rs" ${../../bootstrap/generated/scan-unicode-advance.rs}
            cp ${closure}/store-paths "$out/native-build-paths"
            cp ${closure}/registration "$out/native-build-registration"
            printf '%s\n' ${scanner.drvPath} > "$out/native-build-roots"
          '';
      nativeUnifierSources =
        pkgs: includeTests:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [
              ../../bootstrap/native-unify-codegen/Cargo.toml
              ../../bootstrap/native-unify-codegen/Cargo.lock
              ../../bootstrap/native-unify-codegen/src/lib.rs
              ../../bootstrap/executor/src/unification/compiler.rs
              ../../bootstrap/executor/src/unification/kind.rs
              ../../bootstrap/executor/src/lowering.rs
              ../../bootstrap/executor/src/module.rs
              ../../bootstrap/executor/src/bin/bootstrap-unify-codegen.rs
            ]
            ++ pkgs.lib.optionals includeTests [ ../../bootstrap/native-unify-codegen/tests/compiler.rs ]
          );
        };
      nativeUnifierPolicies =
        pkgs:
        pkgs.runCommand "cuestacean-native-unifier-policies" { } ''
          mkdir -p "$out"
          cp ${nativeUnifyLoweringPolicy pkgs}/unify-lowering.json "$out/unify-lowering.json"
          cp ${nativeMetaLoweringPolicy pkgs}/meta-lowering.json "$out/meta-lowering.json"
        '';
      nativeUnifyCodegen =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "cuestacean-native-unify-codegen";
          version = "0.1.0";
          src = nativeUnifierSources pkgs false;
          postUnpack = ''
            sourceRoot="$sourceRoot/native-unify-codegen"
          '';
          cargoLock.lockFile = ../../bootstrap/native-unify-codegen/Cargo.lock;
          CUESTACEAN_BOOTSTRAP_JSON = nativeUnifierPolicies pkgs;
          cargoBuildFlags = [ "--no-default-features" ];
          # Construction never executes shared defensive tests or expected code.
          doCheck = false;
          meta.license = pkgs.lib.licenses.asl20;
        };
      nativeUnifierTests =
        pkgs:
        (nativeUnifyCodegen pkgs).overrideAttrs (_: {
          pname = "cuestacean-native-unify-codegen-tests";
          src = nativeUnifierSources pkgs true;
          nativeCheckInputs = [ pkgs.clippy ];
          doCheck = true;
          postPatch = ''
            (
              set -euo pipefail
              mkdir -p ../generated "$TMPDIR/unifier-home"
              tool=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
              case "$tool" in /*) ;; *) echo "nonabsolute compiler provider: $tool" >&2; exit 1 ;; esac
              test -r "$tool" && test -x "$tool"
              for name in unify-lowering meta-lowering; do
                env -i PATH=/nonexistent HOME="$TMPDIR/unifier-home" \
                  "$tool" ${nativeUnifierPolicies pkgs}/"$name.json" > "../generated/$name.rs"
              done
            )
          '';
          checkPhase = ''
            (
              set -euo pipefail
              runHook preCheck
              for profile in debug release; do
                releaseFlag=
                if [ "$profile" = release ]; then releaseFlag=--release; fi
                for features in "" compiled-lowering; do
                  cargo test --offline --locked --all-targets --no-default-features \
                    $releaseFlag --features "$features"
                  cargo clippy --offline --locked --all-targets --no-default-features \
                    $releaseFlag --features "$features" -- -D warnings
                done
              done
              runHook postCheck
            )
          '';
        });
      nativeUnifierVerification =
        pkgs:
        let
          unifier = nativeUnifyCodegen pkgs;
          tests = nativeUnifierTests pkgs;
          closure = pkgs.closureInfo { rootPaths = [ unifier.drvPath ]; };
          featureClosure = pkgs.closureInfo { rootPaths = [ tests.drvPath ]; };
        in
        pkgs.runCommand "cuestacean-native-unify-codegen-verification"
          {
            nativeBuildInputs = [ pkgs.rustfmt ];
            featureTests = tests;
            policyVerification = nativeUnifyLoweringPolicyVerification pkgs;
            fullExecutor = executor pkgs;
          }
          ''
            set -euo pipefail
            mkdir -p "$out" "$TMPDIR/unifier-home"
            for tool in ${unifier}/bin/bootstrap-unify-codegen ${pkgs.rustfmt}/bin/rustfmt; do
              case "$tool" in /*) ;; *) echo "nonabsolute verifier tool: $tool" >&2; exit 1 ;; esac
              test -r "$tool" && test -x "$tool"
            done
            test -d "$featureTests" && test -d "$policyVerification" && test -d "$fullExecutor"
            find ${nativeUnifierSources pkgs false} -type f | sort > "$out/constructor-source-paths"
            find ${nativeUnifierSources pkgs true} -type f | sort > "$out/test-source-paths"
            test "$(wc -l < "$out/constructor-source-paths")" -eq 8
            test "$(wc -l < "$out/test-source-paths")" -eq 9
            printf '%s\n' meta-lowering.json unify-lowering.json > expected-policies
            find ${nativeUnifierPolicies pkgs} -type f -printf '%f\n' | sort > "$out/policy-files"
            cmp expected-policies "$out/policy-files"
            for paths in ${closure}/store-paths ${featureClosure}/store-paths; do
              for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
                ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
                ${executor pkgs} ${(executor pkgs).drvPath} \
                ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
                ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
                ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
                ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
                ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
                if grep -Fx "$forbidden" "$paths"; then
                  echo "forbidden native unifier construction dependency: $forbidden" >&2; exit 1
                fi
              done
            done
            # The interpreted constructor consumes policy DATA, not sibling compilers.
            # Feature generation legitimately uses the native core lower compiler.
            for forbidden in ${nativeCodegenCore pkgs} ${(nativeCodegenCore pkgs).drvPath} \
              ${nativeScanCodegen pkgs} ${(nativeScanCodegen pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${closure}/store-paths; then
                echo "sibling compiler in native unifier constructor: $forbidden" >&2; exit 1
              fi
            done
            test -x ${codegenSeed pkgs}/bin/bootstrap-unify-codegen
            generate() {
              env -i PATH=/nonexistent HOME="$TMPDIR/unifier-home" \
                ${unifier}/bin/bootstrap-unify-codegen "$@" | \
                env -i PATH=/nonexistent HOME="$TMPDIR/unifier-home" ${pkgs.rustfmt}/bin/rustfmt --edition 2024
            }
            for control in ${unifyNames}; do
              generate ${jsonExports pkgs}/"$control.json" > first.rs
              generate ${jsonExports pkgs}/"$control.json" > second.rs
              cmp first.rs second.rs
              cmp first.rs ${../../bootstrap/generated}/"$(basename "$control").rs"
              cp first.rs "$out/$(basename "$control").rs"
            done
            for pair in 'unify-lowering-permissive tests/valid/unify-equality-probe unify-permissive-bool' \
              'integer-kernel-alternate difference-control difference-as-product' \
              'division-kernel-alternate divide-control divide-as-quotient'; do
              read -r policy control golden <<< "$pair"
              generate ${jsonExports pkgs}/"$control.json" \
                ${jsonExports pkgs}/tests/valid/"$policy.json" > first.rs
              generate ${jsonExports pkgs}/"$control.json" \
                ${jsonExports pkgs}/tests/valid/"$policy.json" > second.rs
              cmp first.rs second.rs
              cmp first.rs ${../../bootstrap/generated}/"$golden.rs"
              cp first.rs "$out/$golden.rs"
            done
            # Independently predicted whole-source effects, verification inputs ONLY.
            for index in 0 1 2 3 4 5; do
              generate "$policyVerification/tiny.json" "$policyVerification/owner-$index.json" > first.rs
              generate "$policyVerification/tiny.json" "$policyVerification/owner-$index.json" > second.rs
              cmp first.rs second.rs
              cmp first.rs "$policyVerification/$index-expected.rs"
              cp first.rs "$out/owner-$index.rs"
            done
            cp ${closure}/store-paths "$out/native-build-paths"
            cp ${closure}/registration "$out/native-build-registration"
            printf '%s\n' ${unifier.drvPath} > "$out/native-build-roots"
            cp ${featureClosure}/store-paths "$out/feature-build-paths"
            cp ${featureClosure}/registration "$out/feature-build-registration"
            printf '%s\n' ${tests.drvPath} > "$out/feature-build-roots"
          '';
      nativePrattSources =
        pkgs: includeTests:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [
              ../../bootstrap/native-pratt-codegen/Cargo.toml
              ../../bootstrap/native-pratt-codegen/Cargo.lock
              ../../bootstrap/native-pratt-codegen/src/lib.rs
              ../../bootstrap/executor/src/pratt/compiler.rs
              ../../bootstrap/executor/src/unification/kind.rs
              ../../bootstrap/executor/src/lowering.rs
              ../../bootstrap/executor/src/module.rs
              ../../bootstrap/executor/src/bin/bootstrap-pratt-codegen.rs
            ]
            ++ pkgs.lib.optionals includeTests [ ../../bootstrap/native-pratt-codegen/tests/compiler.rs ]
          );
        };
      nativePrattPolicies =
        pkgs:
        pkgs.runCommand "cuestacean-native-pratt-policies" { } ''
          mkdir -p "$out"
          cp ${nativePrattLoweringPolicy pkgs}/pratt-lowering.json "$out/pratt-lowering.json"
          cp ${nativeMetaLoweringPolicy pkgs}/meta-lowering.json "$out/meta-lowering.json"
        '';
      nativePrattCodegen =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "cuestacean-native-pratt-codegen";
          version = "0.1.0";
          src = nativePrattSources pkgs false;
          postUnpack = ''
            sourceRoot="$sourceRoot/native-pratt-codegen"
          '';
          cargoLock.lockFile = ../../bootstrap/native-pratt-codegen/Cargo.lock;
          CUESTACEAN_BOOTSTRAP_JSON = nativePrattPolicies pkgs;
          cargoBuildFlags = [ "--no-default-features" ];
          # Construction never executes shared defensive tests or expected code.
          doCheck = false;
          meta.license = pkgs.lib.licenses.asl20;
        };
      nativePrattTests =
        pkgs:
        (nativePrattCodegen pkgs).overrideAttrs (_: {
          pname = "cuestacean-native-pratt-codegen-tests";
          src = nativePrattSources pkgs true;
          nativeCheckInputs = [ pkgs.clippy ];
          doCheck = true;
          postPatch = ''
            (
              set -euo pipefail
              mkdir -p ../generated "$TMPDIR/pratt-home"
              tool=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
              case "$tool" in /*) ;; *) echo "nonabsolute compiler provider: $tool" >&2; exit 1 ;; esac
              test -f "$tool" && test -r "$tool" && test -x "$tool"
              for name in pratt-lowering meta-lowering; do
                env -i PATH=/nonexistent HOME="$TMPDIR/pratt-home" \
                  "$tool" ${nativePrattPolicies pkgs}/"$name.json" > "../generated/$name.rs"
              done
            )
          '';
          checkPhase = ''
            (
              set -euo pipefail
              runHook preCheck
              for profile in debug release; do
                releaseFlag=
                if [ "$profile" = release ]; then releaseFlag=--release; fi
                for features in "" compiled-lowering; do
                  cargo test --offline --locked --all-targets --no-default-features \
                    $releaseFlag --features "$features"
                  cargo clippy --offline --locked --all-targets --no-default-features \
                    $releaseFlag --features "$features" -- -D warnings
                done
              done
              runHook postCheck
            )
          '';
        });
      nativePrattVerification =
        pkgs:
        let
          pratt = nativePrattCodegen pkgs;
          tests = nativePrattTests pkgs;
          closure = pkgs.closureInfo { rootPaths = [ pratt.drvPath ]; };
          featureClosure = pkgs.closureInfo { rootPaths = [ tests.drvPath ]; };
        in
        pkgs.runCommand "cuestacean-native-pratt-codegen-verification"
          {
            nativeBuildInputs = [ pkgs.rustfmt ];
            featureTests = tests;
            policyVerification = nativePrattLoweringPolicyVerification pkgs;
            fullExecutor = executor pkgs;
          }
          ''
            set -euo pipefail
            mkdir -p "$out" "$TMPDIR/pratt-home"
            for tool in ${pratt}/bin/bootstrap-pratt-codegen ${pkgs.rustfmt}/bin/rustfmt; do
              case "$tool" in /*) ;; *) echo "nonabsolute verifier tool: $tool" >&2; exit 1 ;; esac
              test -f "$tool" && test -r "$tool" && test -x "$tool"
            done
            test -d "$featureTests" && test -d "$policyVerification" && test -d "$fullExecutor"
            printf '%s\n' executor/src/bin/bootstrap-pratt-codegen.rs executor/src/lowering.rs \
              executor/src/module.rs executor/src/pratt/compiler.rs executor/src/unification/kind.rs \
              native-pratt-codegen/Cargo.lock native-pratt-codegen/Cargo.toml \
              native-pratt-codegen/src/lib.rs > expected-sources
            (cd ${nativePrattSources pkgs false}; find . -type f -printf '%P\n' | sort) > "$out/constructor-source-paths"
            cmp expected-sources "$out/constructor-source-paths"
            printf '%s\n' native-pratt-codegen/tests/compiler.rs >> expected-sources
            (cd ${nativePrattSources pkgs true}; find . -type f -printf '%P\n' | sort) > "$out/test-source-paths"
            cmp expected-sources "$out/test-source-paths"
            printf '%s\n' meta-lowering.json pratt-lowering.json > expected-policies
            find ${nativePrattPolicies pkgs} -type f -printf '%P\n' | sort > "$out/policy-files"
            cmp expected-policies "$out/policy-files"
            for paths in ${closure}/store-paths ${featureClosure}/store-paths; do
              for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
                ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
                ${executor pkgs} ${(executor pkgs).drvPath} \
                ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
                ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
                ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
                ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
                ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
                if grep -Fx "$forbidden" "$paths"; then
                  echo "forbidden native Pratt construction dependency: $forbidden" >&2; exit 1
                fi
              done
            done
            # Feature generation legitimately uses the native core lower compiler.
            for forbidden in ${nativeCodegenCore pkgs} ${(nativeCodegenCore pkgs).drvPath} \
              ${nativeScanCodegen pkgs} ${(nativeScanCodegen pkgs).drvPath} \
              ${nativeUnifyCodegen pkgs} ${(nativeUnifyCodegen pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${closure}/store-paths; then
                echo "sibling compiler in native Pratt constructor: $forbidden" >&2; exit 1
              fi
            done
            test -x ${codegenSeed pkgs}/bin/bootstrap-pratt-codegen
            generate() {
              env -i PATH=/nonexistent HOME="$TMPDIR/pratt-home" \
                ${pratt}/bin/bootstrap-pratt-codegen "$@" | \
                env -i PATH=/nonexistent HOME="$TMPDIR/pratt-home" ${pkgs.rustfmt}/bin/rustfmt --edition 2024
            }
            for control in ${prattNames}; do
              generate ${jsonExports pkgs}/"$control.json" > first.rs
              generate ${jsonExports pkgs}/"$control.json" > second.rs
              cmp first.rs second.rs
              cmp first.rs ${../../bootstrap/generated}/"$(basename "$control").rs"
              cp first.rs "$out/$(basename "$control").rs"
            done
            # File-only policy is verification input, never a constructor input.
            for control in ${filePrattNames}; do
              generate ${jsonExports pkgs}/"$control.json" ${jsonExports pkgs}/file-pratt-lowering.json > first.rs
              generate ${jsonExports pkgs}/"$control.json" ${jsonExports pkgs}/file-pratt-lowering.json > second.rs
              cmp first.rs second.rs
              cmp first.rs ${../../bootstrap/generated}/"$control.rs"
              cp first.rs "$out/$control.rs"
            done
            for repeat in 1 2; do
              generate ${jsonExports pkgs}/pratt.json \
                ${jsonExports pkgs}/tests/valid/pratt-lowering-no-increment.json > "$repeat.rs"
            done
            cmp 1.rs 2.rs
            cmp 1.rs ${../../bootstrap/generated/pratt-no-increment.rs}
            cp 1.rs "$out/pratt-no-increment.rs"
            # All three raw source owners have independently predicted whole-source effects.
            for owner in body add tables; do
              generate "$policyVerification/tiny.json" "$policyVerification/owner-$owner-policy.json" > first.rs
              generate "$policyVerification/tiny.json" "$policyVerification/owner-$owner-policy.json" > second.rs
              cmp first.rs second.rs
              cmp first.rs "$policyVerification/$owner-expected.rs"
              cp first.rs "$out/owner-$owner.rs"
            done
            for invalid in opcode shape unbound; do
              status=0
              env -i PATH=/nonexistent HOME="$TMPDIR/pratt-home" \
                ${pratt}/bin/bootstrap-pratt-codegen "$policyVerification/tiny.json" \
                "$policyVerification/malformed-$invalid-policy.json" > invalid.rs 2> invalid.err || status=$?
              test "$status" -eq 2
              test ! -s invalid.rs
              cmp invalid.err "$policyVerification/$invalid-typed.err"
              cp invalid.err "$out/$invalid-typed.err"
            done
            cp ${closure}/store-paths "$out/native-build-paths"
            cp ${closure}/registration "$out/native-build-registration"
            printf '%s\n' ${pratt.drvPath} > "$out/native-build-roots"
            cp ${featureClosure}/store-paths "$out/feature-build-paths"
            cp ${featureClosure}/registration "$out/feature-build-registration"
            printf '%s\n' ${tests.drvPath} > "$out/feature-build-roots"
          '';
      nativeInstructionSources =
        pkgs: includeTests:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [
              ../../bootstrap/native-instruction-codegen/Cargo.toml
              ../../bootstrap/native-instruction-codegen/Cargo.lock
              ../../bootstrap/native-instruction-codegen/src/lib.rs
              ../../bootstrap/executor/src/codegen/compiler.rs
              ../../bootstrap/executor/src/lowering.rs
              ../../bootstrap/executor/src/module.rs
              ../../bootstrap/executor/src/bin/bootstrap-codegen.rs
            ]
            ++ pkgs.lib.optionals includeTests [ ../../bootstrap/native-instruction-codegen/tests/compiler.rs ]
          );
        };
      nativeInstructionPolicies =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-policies" { } ''
          mkdir -p "$out"
          cp ${nativeInstructionLoweringPolicy pkgs}/lowering.json "$out/lowering.json"
          cp ${nativeMetaLoweringPolicy pkgs}/meta-lowering.json "$out/meta-lowering.json"
        '';
      nativeInstructionCodegen =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "cuestacean-native-instruction-codegen";
          version = "0.1.0";
          src = nativeInstructionSources pkgs false;
          postUnpack = ''
            sourceRoot="$sourceRoot/native-instruction-codegen"
          '';
          cargoLock.lockFile = ../../bootstrap/native-instruction-codegen/Cargo.lock;
          CUESTACEAN_BOOTSTRAP_JSON = nativeInstructionPolicies pkgs;
          cargoBuildFlags = [ "--no-default-features" ];
          # Expected source and feature execution belong only to verification.
          doCheck = false;
          meta.license = pkgs.lib.licenses.asl20;
        };
      nativeInstructionTests =
        pkgs:
        (nativeInstructionCodegen pkgs).overrideAttrs (_: {
          pname = "cuestacean-native-instruction-codegen-tests";
          src = nativeInstructionSources pkgs true;
          nativeCheckInputs = [ pkgs.clippy ];
          doCheck = true;
          postPatch = ''
            (
              set -euo pipefail
              mkdir -p ../generated "$TMPDIR/instruction-home"
              tool=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
              case "$tool" in /*) ;; *) echo "nonabsolute compiler provider: $tool" >&2; exit 1 ;; esac
              test -f "$tool"; test -r "$tool"; test -x "$tool"
              for name in lowering meta-lowering; do
                env -i PATH=/nonexistent HOME="$TMPDIR/instruction-home" \
                  "$tool" ${nativeInstructionPolicies pkgs}/"$name.json" > "../generated/$name.rs"
              done
            )
          '';
          checkPhase = ''
            (
              set -euo pipefail
              runHook preCheck
              for profile in debug release; do
                releaseFlag=
                if [ "$profile" = release ]; then releaseFlag=--release; fi
                for features in "" compiled-lowering; do
                  cargo test --offline --locked --all-targets --no-default-features \
                    $releaseFlag --features "$features"
                  cargo clippy --offline --locked --all-targets --no-default-features \
                    $releaseFlag --features "$features" -- -D warnings
                done
              done
              runHook postCheck
            )
          '';
        });
      nativeInstructionVerification =
        pkgs:
        let
          compiler = nativeInstructionCodegen pkgs;
          tests = nativeInstructionTests pkgs;
          closure = pkgs.closureInfo { rootPaths = [ compiler.drvPath ]; };
          featureClosure = pkgs.closureInfo { rootPaths = [ tests.drvPath ]; };
        in
        pkgs.runCommand "cuestacean-native-instruction-codegen-verification"
          {
            nativeBuildInputs = [ pkgs.rustfmt ];
            featureTests = tests;
            policyVerification = nativeInstructionLoweringPolicyVerification pkgs;
          }
          ''
            set -euo pipefail
            mkdir -p "$out" "$TMPDIR/instruction-home"
            native=${compiler}/bin/bootstrap-codegen
            legacy=${legacyCodegenSeed pkgs}/bin/bootstrap-codegen
            formatter=${pkgs.rustfmt}/bin/rustfmt
            for tool in "$native" "$legacy" "$formatter"; do
              case "$tool" in /*) ;; *) echo "nonabsolute verifier tool: $tool" >&2; exit 1 ;; esac
              test -f "$tool"; test -r "$tool"; test -x "$tool"
            done
            test -d "$featureTests"
            test -d "$policyVerification"
            printf '%s\n' executor/src/bin/bootstrap-codegen.rs executor/src/codegen/compiler.rs \
              executor/src/lowering.rs executor/src/module.rs native-instruction-codegen/Cargo.lock \
              native-instruction-codegen/Cargo.toml native-instruction-codegen/src/lib.rs > expected-sources
            printf '%s\n' executor executor/src executor/src/bin executor/src/codegen \
              native-instruction-codegen native-instruction-codegen/src > expected-directories
            for variant in constructor test; do
              sources=${nativeInstructionSources pkgs false}
              if [ "$variant" = test ]; then
                sources=${nativeInstructionSources pkgs true}
                printf '%s\n' native-instruction-codegen/tests/compiler.rs >> expected-sources
                printf '%s\n' native-instruction-codegen/tests >> expected-directories
                cmp "$sources/native-instruction-codegen/tests/compiler.rs" ${../../bootstrap/native-instruction-codegen/tests/compiler.rs}
              fi
              (cd "$sources"; find . -type f -printf '%P\n' | sort) > "$out/$variant-source-paths"
              (cd "$sources"; find . -mindepth 1 -type d -printf '%P\n' | sort) > "$out/$variant-directories"
              cmp expected-sources "$out/$variant-source-paths"
              cmp expected-directories "$out/$variant-directories"
              test -z "$(find "$sources" ! -type f ! -type d -print)"
              for shared in bin/bootstrap-codegen.rs codegen/compiler.rs lowering.rs module.rs; do
                cmp "$sources/executor/src/$shared" ${../../bootstrap/executor/src}/"$shared"
              done
              for facade in Cargo.toml Cargo.lock src/lib.rs; do
                cmp "$sources/native-instruction-codegen/$facade" ${../../bootstrap/native-instruction-codegen}/"$facade"
              done
            done
            substitute ${../../bootstrap/native-pratt-codegen/Cargo.lock} expected-lock \
              --replace-fail cuestacean-native-pratt-codegen cuestacean-native-instruction-codegen
            cmp expected-lock ${../../bootstrap/native-instruction-codegen/Cargo.lock}
            printf '%s\n' lowering.json meta-lowering.json > expected-policies
            find ${nativeInstructionPolicies pkgs} -type f -printf '%P\n' | sort > "$out/policy-files"
            cmp expected-policies "$out/policy-files"
            test -z "$(find ${nativeInstructionPolicies pkgs} -mindepth 1 ! -type f -print)"
            cmp ${nativeInstructionPolicies pkgs}/lowering.json ${nativeInstructionLoweringPolicy pkgs}/lowering.json
            cmp ${nativeInstructionPolicies pkgs}/meta-lowering.json ${nativeMetaLoweringPolicy pkgs}/meta-lowering.json
            for paths in ${closure}/store-paths ${featureClosure}/store-paths; do
              for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
                ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
                ${executor pkgs} ${(executor pkgs).drvPath} \
                ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
                ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
                ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
                ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath} \
                ${nativeScanCodegen pkgs} ${(nativeScanCodegen pkgs).drvPath} \
                ${nativeUnifyCodegen pkgs} ${(nativeUnifyCodegen pkgs).drvPath} \
                ${nativePrattCodegen pkgs} ${(nativePrattCodegen pkgs).drvPath}; do
                if grep -Fx "$forbidden" "$paths"; then
                  echo "forbidden native instruction construction dependency: $forbidden" >&2; exit 1
                fi
              done
            done
            # Only feature generation may use the native core lower compiler.
            for forbidden in ${nativeCodegenCore pkgs} ${(nativeCodegenCore pkgs).drvPath} \
              ${tests} ${tests.drvPath}; do
              if grep -Fx "$forbidden" ${closure}/store-paths; then
                echo "feature or sibling compiler in instruction constructor: $forbidden" >&2; exit 1
              fi
            done
            invoke() { env -i PATH=/nonexistent HOME="$TMPDIR/instruction-home" "$@"; }
            for mode in default left-twice reverse; do
              args=(${jsonExports pkgs}/evaluation.json)
              golden=evaluation
              case "$mode" in
                left-twice) args+=(${jsonExports pkgs}/tests/valid/lowering-left-twice.json); golden=evaluation-left-twice ;;
                reverse) args+=(${jsonExports pkgs}/tests/valid/lowering-reverse.json); golden=evaluation-reverse ;;
              esac
              invoke "$native" "''${args[@]}" > "$mode.rs"
              invoke "$native" "''${args[@]}" > repeat.rs
              invoke "$legacy" "''${args[@]}" > legacy.rs
              cmp "$mode.rs" repeat.rs
              cmp "$mode.rs" legacy.rs
              invoke "$formatter" --edition 2024 < "$mode.rs" > formatted.rs
              cmp formatted.rs ${../../bootstrap/generated}/"$golden.rs"
              cp "$mode.rs" "$out/$mode.rs"
            done
            for variant in canonical core unary binary-method binary-keys right r group runtime text footer-source first step module separator body-order reverse; do
              fixture="$policyVerification/source-$variant"
              for route in native raw; do
                invoke "$native" "$fixture/tiny.json" "$fixture/$route-policy.json" > actual.rs
                cmp actual.rs "$fixture/expected.rs"
                cp actual.rs "$out/$variant-$route.rs"
              done
              if [ "$variant" = canonical ]; then
                invoke "$native" "$fixture/tiny.json" > actual.rs
                cmp actual.rs "$fixture/expected.rs"
                cp actual.rs "$out/canonical-default.rs"
              fi
            done
            fixture="$policyVerification/source-canonical"
            for negative in oversize-module cursor-protocol; do
              status=0
              invoke "$native" "$fixture/tiny.json" "$policyVerification/$negative.json" > invalid.rs 2> invalid.err || status=$?
              test "$status" -eq 2
              test ! -s invalid.rs
              case "$negative" in
                oversize-module) printf '%s\n' 'lowering: module text exceeds 64 KiB' > expected.err ;;
                cursor-protocol) printf '%s\n' 'lowering: no current rule' > expected.err ;;
              esac
              cmp expected.err invalid.err
              cp invalid.err "$out/$negative.err"
              invoke "$native" "$fixture/tiny.json" "$fixture/native-policy.json" > recovery.rs
              cmp recovery.rs "$fixture/expected.rs"
              cp recovery.rs "$out/$negative-recovery.rs"
            done
            test "$(readlink -f ${codegenSeed pkgs}/bin/bootstrap-codegen)" = "$(readlink -f "$native")"
            cp ${closure}/store-paths "$out/native-build-paths"
            cp ${closure}/registration "$out/native-build-registration"
            printf '%s\n' ${compiler.drvPath} > "$out/native-build-roots"
            cp ${featureClosure}/store-paths "$out/feature-build-paths"
            cp ${featureClosure}/registration "$out/feature-build-registration"
            printf '%s\n' ${tests.drvPath} > "$out/feature-build-roots"
          '';
      makeLegacyCodegenSeed =
        pkgs:
        (rustPackage pkgs).overrideAttrs (old: {
          pname = "bootstrap-legacy-codegen-seed";
          # This build selects only bootstrap-codegen. The full executor still
          # builds and checks the public CLI from the unfiltered Rust sources.
          src = pkgs.lib.fileset.toSource {
            root = ../../bootstrap;
            fileset =
              pkgs.lib.fileset.difference (pkgs.lib.fileset.fileFilter (file: !file.hasExt "md") ../../bootstrap)
                (
                  pkgs.lib.fileset.unions [
                    ../../bootstrap/executor/src/bin/cuestacean.rs
                    ../../bootstrap/executor/src/bin/cuestacean
                  ]
                );
          };
          # Policy is fresh upstream CUE data, not a generated Rust seed dependency.
          CUESTACEAN_BOOTSTRAP_JSON = jsonExports pkgs;
          meta = old.meta // {
            mainProgram = "bootstrap-codegen";
          };
          cargoBuildFlags = [
            "--no-default-features"
            "--bin"
            "bootstrap-codegen"
          ];
          postPatch = clearArtifacts;
          # The tests require generated modules. Break that bootstrap cycle here;
          # the final package below runs the complete test suite, including codegen.
          doCheck = false;
        });

      makeCodegenSeed =
        pkgs:
        pkgs.symlinkJoin {
          name = "bootstrap-codegen-seed";
          paths = [
            (nativeCodegenCore pkgs)
            (nativeScanCodegen pkgs)
            (nativeUnifyCodegen pkgs)
            (nativePrattCodegen pkgs)
            (nativeInstructionCodegen pkgs)
          ];
          postBuild = ''
            # Assert both membership and actual compiler provenance, not PATH order.
            printf '%s\n' bootstrap-ast-codegen bootstrap-codegen bootstrap-lower-codegen \
              bootstrap-pratt-codegen bootstrap-scan-codegen bootstrap-unify-codegen > expected
            find "$out/bin" -mindepth 1 -maxdepth 1 -printf '%f\n' | sort > actual
            cmp expected actual
            for name in bootstrap-ast-codegen bootstrap-lower-codegen; do
              test "$(readlink -f "$out/bin/$name")" = "$(readlink -f ${nativeCodegenCore pkgs}/bin/"$name")"
            done
            test "$(readlink -f "$out/bin/bootstrap-scan-codegen")" = "$(readlink -f ${nativeScanCodegen pkgs}/bin/bootstrap-scan-codegen)"
            test "$(readlink -f "$out/bin/bootstrap-unify-codegen")" = "$(readlink -f ${nativeUnifyCodegen pkgs}/bin/bootstrap-unify-codegen)"
            test "$(readlink -f "$out/bin/bootstrap-pratt-codegen")" = "$(readlink -f ${nativePrattCodegen pkgs}/bin/bootstrap-pratt-codegen)"
            test "$(readlink -f "$out/bin/bootstrap-codegen")" = "$(readlink -f ${nativeInstructionCodegen pkgs}/bin/bootstrap-codegen)"
          '';
        };
      nativeCoreVerification =
        pkgs:
        let
          core = nativeCodegenCore pkgs;
          closure = pkgs.closureInfo { rootPaths = [ core.drvPath ]; };
        in
        pkgs.runCommand "cuestacean-native-codegen-core-verification"
          {
            nativeBuildInputs = [ pkgs.rustfmt ];
            # Force real feature execution; never feed this back into construction.
            featureTests = nativeCoreTests pkgs;
          }
          ''
            set -euo pipefail
            mkdir -p "$out" "$TMPDIR/core-home"
            for tool in ${core}/bin/bootstrap-ast-codegen ${core}/bin/bootstrap-lower-codegen; do
              case "$tool" in /*) ;; *) echo "nonabsolute compiler provider: $tool" >&2; exit 1 ;; esac
              test -r "$tool" && test -x "$tool"
            done
            test -d "$featureTests"
            test "$(find ${nativeCoreSources pkgs false} -type f | wc -l)" -eq 8
            test "$(find ${nativeCoreSources pkgs true} -type f | wc -l)" -eq 9
            printf '%s\n' ast-lowering.json meta-lowering.json > expected-policies
            find ${nativeCorePolicies pkgs} -type f -printf '%f\n' | sort > policies
            cmp expected-policies policies
            for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
              ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
              ${executor pkgs} ${(executor pkgs).drvPath} \
              ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
              ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
              ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
              ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
              ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${closure}/store-paths; then
                echo "forbidden native compiler dependency: $forbidden" >&2; exit 1
              fi
            done
            # The joined constructor performs its own exact-six/provenance assertions.
            test -x ${codegenSeed pkgs}/bin/bootstrap-ast-codegen
            for fixture in ${builderExportNames}; do
              name=$(basename "$fixture")
              for repeat in 1 2; do
                env -i PATH=/nonexistent HOME="$TMPDIR/core-home" \
                  ${core}/bin/bootstrap-ast-codegen ${nativeBuilders pkgs}/"$fixture.json" > "$repeat.rs"
              done
              cmp 1.rs 2.rs
              rustfmt --edition 2024 < 1.rs > actual.rs
              if [ "$fixture" = tests/valid/ast-literal ]; then
                printf '%s\n' '// Generated by bootstrap-ast-codegen. Do not edit by hand.' \
                  'use serde_json::Value;' 'pub fn build() -> Value {' '    Value::from(7i64)' '}' > expected.rs
                cmp expected.rs actual.rs
              else
                cmp actual.rs ${../../bootstrap/generated}/"$name.rs"
              fi
              cp actual.rs "$out/$name.rs"
            done
            for name in lowering pratt-lowering unify-lowering ast-lowering scan-lowering meta-lowering; do
              for repeat in 1 2; do
                env -i PATH=/nonexistent HOME="$TMPDIR/core-home" \
                  ${core}/bin/bootstrap-lower-codegen ${jsonExports pkgs}/"$name.json" > "$repeat.rs"
              done
              cmp 1.rs 2.rs
              rustfmt --edition 2024 < 1.rs > "$out/$name.rs"
              cmp "$out/$name.rs" ${../../bootstrap/generated}/"$name.rs"
            done
            env -i PATH=/nonexistent HOME="$TMPDIR/core-home" \
              ${core}/bin/bootstrap-ast-codegen ${nativeBuilders pkgs}/tests/valid/ast-literal.json \
              ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json | rustfmt --edition 2024 > forced.rs
            cmp forced.rs ${../../bootstrap/generated/ast-forced-integer.rs}
            for pair in 'lowering-numbers meta-lowering-forced lowering-meta-forced' \
              'meta-driver-keys meta-driver-previous meta-driver-previous'; do
              read -r input policy golden <<< "$pair"
              env -i PATH=/nonexistent HOME="$TMPDIR/core-home" \
                ${core}/bin/bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/"$input.json" \
                ${jsonExports pkgs}/tests/valid/"$policy.json" | rustfmt --edition 2024 > actual.rs
              cmp actual.rs ${../../bootstrap/generated}/"$golden.rs"
            done
            cp ${closure}/store-paths "$out/native-build-paths"
            cp ${closure}/registration "$out/native-build-registration"
            printf '%s\n' ${core.drvPath} > "$out/native-build-roots"
          '';
      makeExecutor =
        pkgs:
        (rustPackage pkgs).overrideAttrs (old: {
          nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
            pkgs.rustfmt
            (codegenSeed pkgs)
          ];
          CUESTACEAN_BOOTSTRAP_JSON = jsonExports pkgs;
          CUESTACEAN_DEFAULT_OBSERVER = "${defaultObserver pkgs}/bin/cuestacean-default-observer";
          CUESTACEAN_DEFAULT_TRACE = "${defaultTrace pkgs}/bin/cuestacean-default-trace";
          preCheck = old.preCheck + ''
            cargo test --offline --no-default-features --test default_observer
            cargo test --offline --no-default-features --test default_trace
            cargo test --offline --no-default-features --test batch_defaults
            cargo test --offline --no-default-features --test flat_default_sources
            cargo test --offline --no-default-features --test recursive_default_scheduling
            cargo test --offline --no-default-features --test default_equality_trace
            cargo test --offline --no-default-features --test native_coalescing_audit
          '';
          postPatch = clearArtifacts + ''
            # Do not leak strict shell options into later stdenv hooks.
            (
              set -euo pipefail
              for policy in ${loweringNames}; do
                name=$(basename "$policy")
                bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$policy.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              for rules in ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/evaluation-left-twice.json tests/fixtures/codegen-escaped-rules.json; do
                name=$(basename "$rules" .json)
                bootstrap-codegen "$rules" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-codegen "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-left-twice.json" | rustfmt --edition 2024 > policy-generated.rs
              cmp policy-generated.rs ../generated/evaluation-left-twice.rs
              mv policy-generated.rs ../generated/evaluation-left-twice.rs
              bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-numbers.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-lowering-forced.json" | rustfmt --edition 2024 > ../generated/lowering-meta-forced.rs
              bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-driver-keys.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-driver-previous.json" | rustfmt --edition 2024 > ../generated/meta-driver-previous.rs
              bootstrap-codegen "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-reverse.json" | rustfmt --edition 2024 > ../generated/evaluation-reverse.rs
              for machine in ${scannerNames}; do
                name=$(basename "$machine")
                bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$machine.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/scanner-advance.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/scan-lowering-unicode.json" | rustfmt --edition 2024 > ../generated/scan-unicode-advance.rs
              for builder in ${builderNames}; do
                name=$(basename "$builder")
                bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$builder.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/ast-literal.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/ast-lowering-forced.json" | rustfmt --edition 2024 > ../generated/ast-forced-integer.rs
              for control in ${prattNames}; do
                name=$(basename "$control")
                bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              for control in ${filePrattNames}; do
                bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" "$CUESTACEAN_BOOTSTRAP_JSON/file-pratt-lowering.json" | rustfmt --edition 2024 > "../generated/$control.rs"
              done
              bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/pratt.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/pratt-lowering-no-increment.json" | rustfmt --edition 2024 > ../generated/pratt-no-increment.rs
              for control in ${unifyNames}; do
                name=$(basename "$control")
                bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/unify-equality-probe.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/unify-lowering-permissive.json" | rustfmt --edition 2024 > ../generated/unify-permissive-bool.rs
              bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/difference-control.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/integer-kernel-alternate.json" | rustfmt --edition 2024 > ../generated/difference-as-product.rs
              bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/divide-control.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/division-kernel-alternate.json" | rustfmt --edition 2024 > ../generated/divide-as-quotient.rs
            )
          '';
        });
      specSources =
        pkgs: tests:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [
              ../../bootstrap/spec-export/src/lib.rs
              ../../bootstrap/spec-export/src/main.rs
              ../../bootstrap/spec-export/src/model.rs
              ../../bootstrap/spec-export/src/parser.rs
              ../../bootstrap/spec-export/src/eval.rs
              ../../bootstrap/spec-export/src/bindings.rs
              ../../bootstrap/spec-export/src/comprehension.rs
              ../../bootstrap/spec-export/src/projection.rs
              ../../bootstrap/spec-export/Cargo.toml
              ../../bootstrap/spec-export/Cargo.lock
            ]
            ++ pkgs.lib.optionals tests (
              [
                ../../bootstrap/spec-export/tests
                ../../bootstrap/instruction-policy-group.cue
                ../../bootstrap/instruction-policy-shell.cue
                ../../bootstrap/instruction-module.cue
                ../../bootstrap/instruction-module-export.cue
                ../../bootstrap/instruction-module-schema.cue
                ../../bootstrap/instruction-core-cases.cue
                ../../bootstrap/instruction-core-cases-schema.cue
                ../../bootstrap/instruction-right-field.cue
                ../../bootstrap/instruction-right-field-export.cue
                ../../bootstrap/instruction-right-operand.cue
                ../../bootstrap/instruction-right-operand-export.cue
                ../../bootstrap/instruction-operand-cases.cue
                ../../bootstrap/instruction-operand-cases-export.cue
                ../../bootstrap/instruction-binary-runtime-export.cue
                ../../bootstrap/instruction-binary-runtime.cue
                ../../bootstrap/instruction-unary-tables.cue
                ../../bootstrap/instruction-binary-tables.cue
                ../../bootstrap/instruction-module-text.cue
                ../../bootstrap/instruction-module-text-export.cue
                ../../bootstrap/instruction-module-first.cue
                ../../bootstrap/instruction-module-first-export.cue
                ../../bootstrap/instruction-module-step.cue
                ../../bootstrap/instruction-module-step-export.cue
                ../../bootstrap/lowering-fragments.cue
                ../../bootstrap/lowering.cue
                ../../bootstrap/tests/valid/lowering-left-twice.cue
                ../../bootstrap/tests/valid/lowering-reverse.cue
                ../../bootstrap/pratt-lowering-tables.cue
                ../../bootstrap/pratt-lowering-body.cue
                ../../bootstrap/pratt-add-policy.cue
                ../../bootstrap/pratt-lowering-fragments.cue
                ../../bootstrap/pratt-lowering.cue
                ../../bootstrap/tests/valid/pratt-lowering-no-increment.cue
                ../../bootstrap/ast-integer-policy.cue
                ../../bootstrap/native-policy-package.cue
                ../../bootstrap/schema.cue
                ../../bootstrap/ast-lowering.cue
                ../../bootstrap/tests/valid/ast-lowering-forced.cue
                ../../bootstrap/ast-lowering-fragments.cue
                ../../bootstrap/ast-lowering-body.cue
                ../../bootstrap/compose-policy.jq
                ../../bootstrap/meta-lowering-body.cue
                ../../bootstrap/meta-text-alias.cue
                ../../bootstrap/meta-next-rule-alias.cue
                ../../bootstrap/meta-lowering-fragments.cue
                ../../bootstrap/meta-lowering.cue
                ../../bootstrap/tests/valid/meta-lowering-forced.cue
                ../../bootstrap/tests/valid/meta-driver-previous.cue
                ../../bootstrap/scan-lowering.cue
                ../../bootstrap/scan-lowering-fragments.cue
                ../../bootstrap/scan-lowering-helpers.cue
                ../../bootstrap/scan-lowering-flow.cue
                ../../bootstrap/scan-lowering-predicates.cue
                ../../bootstrap/scan-advance-policy.cue
                ../../bootstrap/tests/valid/scan-lowering-unicode.cue
                ../../bootstrap/unify-lowering-helpers.cue
                ../../bootstrap/unify-lowering-values.cue
                ../../bootstrap/unify-lowering-control.cue
                ../../bootstrap/unify-difference-policy.cue
                ../../bootstrap/unify-divide-policy.cue
                ../../bootstrap/unify-same-policy.cue
                ../../bootstrap/unify-lowering-fragments.cue
                ../../bootstrap/unify-lowering.cue
                ../../bootstrap/tests/valid/integer-kernel-alternate.cue
                ../../bootstrap/tests/valid/division-kernel-alternate.cue
                ../../bootstrap/tests/valid/unify-lowering-permissive.cue
                ../../bootstrap/scanner-schema.cue
                ../../bootstrap/group-open.cue
                ../../bootstrap/ast-schema.cue
                ../../bootstrap/tests/invalid/ast-operation.cue
                ../../bootstrap/spikes/demand-policy-schema.cue
                ../../bootstrap/spikes/demand-policy.cue
                ../../bootstrap/spikes/demand-policy-defer.cue
                ../../bootstrap/spikes/preference-projection-schema.cue
                ../../bootstrap/spikes/preference-projection.cue
                ../../bootstrap/spikes/preference-projection-defer.cue
              ]
              ++ map (fixture: ../../bootstrap + "/${fixture}.cue") builderExportFixtures
            )
          );
        };
      specExporter =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "cuestacean-spec-export";
          version = "0.1.0";
          src = specSources pkgs false;
          postUnpack = ''sourceRoot="$sourceRoot/spec-export"'';
          cargoLock.lockFile = "${specSources pkgs false}/spec-export/Cargo.lock";
          cargoBuildFlags = [
            "--bin"
            "cuestacean-spec-export"
          ];
          # Expected answers/oracles belong only to the separate check derivation.
          doCheck = false;
        };
      specTests =
        pkgs:
        (specExporter pkgs).overrideAttrs (_: {
          pname = "cuestacean-spec-export-tests";
          src = specSources pkgs true;
          doCheck = true;
          nativeCheckInputs = [
            pkgs.rustfmt
            pkgs.clippy
            pkgs.cue
            pkgs.jq
          ];
          checkPhase = ''
            cargo fmt --check
            cargo clippy --offline --all-targets -- -D warnings
            cargo test --offline --all-targets
            cargo test --offline --all-targets --release
          '';
        });
      demandPolicyPairs = builtins.concatStringsSep " " [
        "demand-policy-schema:demand-policy"
        "demand-policy-schema:demand-policy-defer"
        "preference-projection-schema:preference-projection"
        "preference-projection-schema:preference-projection-defer"
      ];
      demandPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/spikes/demand-policy-schema.cue
            ../../bootstrap/spikes/demand-policy.cue
            ../../bootstrap/spikes/demand-policy-defer.cue
            ../../bootstrap/spikes/preference-projection-schema.cue
            ../../bootstrap/spikes/preference-projection.cue
            ../../bootstrap/spikes/preference-projection-defer.cue
          ];
        };
      nativeDemandPolicies =
        pkgs:
        pkgs.runCommand "cuestacean-native-demand-policies" { } ''
          set -euo pipefail
          mkdir -p "$out"
          for pair in ${demandPolicyPairs}; do
            schema="''${pair%%:*}"
            program="''${pair#*:}"
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${demandPolicySources pkgs}/spikes/"$schema.cue" \
              ${demandPolicySources pkgs}/spikes/"$program.cue" > first.json
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${demandPolicySources pkgs}/spikes/"$schema.cue" \
              ${demandPolicySources pkgs}/spikes/"$program.cue" > second.json
            cmp first.json second.json
            cp first.json "$out/$program.json"
          done
        '';
      nativeDemandPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          # Derivation roots attest build inputs, not merely runtime references.
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeDemandPolicies pkgs).drvPath
          ];
        };
      nativeDemandPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-demand-policies-verification"
          {
            nativeBuildInputs = [
              pkgs.cue
              pkgs.jq
            ];
          }
          ''
            set -euo pipefail
            # These identities are forbidden only in the native construction
            # graph. This separate oracle/check package is not Go-CUE-free.
            for forbidden in ${pkgs.cue.drvPath} ${pkgs.cue.src} ${(jsonExports pkgs).drvPath} ${(executor pkgs).drvPath} ${(defaultObserver pkgs).drvPath} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeDemandPolicyBuildClosure pkgs}/store-paths; then
                echo "forbidden bootstrap dependency: $forbidden" >&2; exit 1
              fi
            done
            for pair in ${demandPolicyPairs}; do
              schema="''${pair%%:*}"
              program="''${pair#*:}"
              cue export ${demandPolicySources pkgs}/spikes/"$schema.cue" \
                ${demandPolicySources pkgs}/spikes/"$program.cue" --out json | jq -S . > oracle.json
              cmp oracle.json ${nativeDemandPolicies pkgs}/"$program.json"
            done
            mkdir -p "$out"
            cp ${nativeDemandPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeDemandPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeDemandPolicies pkgs).drvPath} > "$out/native-build-roots"
          '';
      astIntegerPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/ast-integer-policy.cue
            ../../bootstrap/native-policy-package.cue
          ];
        };
      nativeAstIntegerPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-integer-policy" { } ''
          set -euo pipefail
          mkdir -p "$out" "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -r "$exporter" && test -x "$exporter"
          for run in first second; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
              ${astIntegerPolicySources pkgs}/native-policy-package.cue \
              ${astIntegerPolicySources pkgs}/ast-integer-policy.cue > "$run.json"
          done
          cmp first.json second.json
          cp first.json "$out/ast-integer-policy.json"
        '';
      nativeAstIntegerPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeAstIntegerPolicy pkgs).drvPath
          ];
        };
      nativeAstIntegerPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-integer-policy-verification"
          {
            nativeBuildInputs = [
              pkgs.jq
              pkgs.rustfmt
            ];
          }
          ''
            set -euo pipefail
            # This check may use Go CUE and the executor; construction may not.
            for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
              ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
              ${executor pkgs} ${(executor pkgs).drvPath} \
              ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
              ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeAstIntegerPolicyBuildClosure pkgs}/store-paths; then
                echo "forbidden integer-policy construction dependency: $forbidden" >&2
                exit 1
              fi
            done
            mkdir -p "$out" "$PWD/probe/home"
            cd probe
            exporter=${specExporter pkgs}/bin/cuestacean-spec-export
            oracle=${pkgs.cue}/bin/cue
            codegen=${executor pkgs}/bin/bootstrap-ast-codegen
            for binary in "$exporter" "$oracle" "$codegen"; do
              test -r "$binary" && test -x "$binary"
              case "$binary" in /*) ;; *) exit 1 ;; esac
            done
            native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
            cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
            generate() {
              env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | rustfmt --edition 2024
            }
            no_export() {
              if cue export "$@" > failed.json 2> failed.err; then
                echo "unexpected concrete policy" >&2; exit 1
              fi
              test ! -s failed.json && test -s failed.err
            }
            cp ${astIntegerPolicySources pkgs}/native-policy-package.cue renamed-package.cue
            cp ${astIntegerPolicySources pkgs}/ast-integer-policy.cue renamed-owner.cue
            # Complete independent answers; neither is a construction input.
            answer='{"cases":{"integer":{"op":"concat","parts":[{"op":"text","value":"Value::from("},{"op":"signed","field":"value"},{"op":"text","value":"i64)"}]}}}'
            printf '%s\n' "$answer" | jq -S . > expected.json
            native renamed-package.cue renamed-owner.cue > default.json
            native renamed-owner.cue renamed-package.cue > reversed.json
            cmp default.json reversed.json
            cmp default.json expected.json
            cmp default.json ${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json
            cue export renamed-package.cue renamed-owner.cue | jq -S . > oracle.json
            cmp default.json oracle.json

            # Both forced paths still observe the RAW preference, in two files.
            printf 'package bootstrap\n{cases:{integer:{op:"text",value:"Value::from(99i64)"}}}\n' > forced.cue
            native renamed-owner.cue forced.cue > forced.json
            printf '%s\n' '{"cases":{"integer":{"op":"text","value":"Value::from(99i64)"}}}' | jq -S . > expected-forced.json
            cmp forced.json expected-forced.json
            cue export renamed-owner.cue forced.cue | jq -S . > oracle-forced.json
            cmp forced.json oracle-forced.json
            printf 'package bootstrap\n{cases:{integer:{op:"text"}}}\n' > incomplete.cue
            if native renamed-owner.cue incomplete.cue > incomplete.json 2> incomplete.err; then exit 1; fi
            test ! -s incomplete.json
            grep -F 'incomplete export demand' incomplete.err
            no_export renamed-owner.cue incomplete.cue
            cue eval renamed-owner.cue incomplete.cue > /dev/null

            schema=${../../bootstrap/schema.cue}
            fragments=${../../bootstrap/ast-lowering-fragments.cue}
            body=${../../bootstrap/ast-lowering-body.cue}
            selector=${../../bootstrap/ast-lowering.cue}
            # Whole policies, under the existing canonicalization, not subtrees.
            cue export "$schema" "$fragments" "$body" "$selector" renamed-owner.cue | jq -S . > raw-policy.json
            cue export "$schema" "$fragments" "$body" "$selector" \
              ${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json | jq -S . > assembled-policy.json
            cmp raw-policy.json assembled-policy.json
            cmp assembled-policy.json ${jsonExports pkgs}/ast-lowering.json
            no_export "$schema" "$fragments" "$body" "$selector"
            cp failed.err missing-provider.err
            grep -F 'incomplete value' missing-provider.err
            no_export "$schema" "$fragments" "$body" "$selector" renamed-owner.cue incomplete.cue
            cue eval "$schema" "$fragments" "$body" "$selector" renamed-owner.cue incomplete.cue > /dev/null
            no_export "$schema" "$fragments" "$body" "$selector" \
              ${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json forced.cue
            cue export "$schema" "$fragments" "$body" renamed-owner.cue \
              ${../../bootstrap/tests/valid/ast-lowering-forced.cue} | jq -S . > forced-policy.json
            cmp forced-policy.json ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json
            generate ${jsonExports pkgs}/tests/valid/ast-literal.json forced-policy.json > forced.rs
            cmp forced.rs ${../../bootstrap/generated/ast-forced-integer.rs}
            for builder in ${builderNames}; do
              name=$(basename "$builder")
              generate ${jsonExports pkgs}/"$builder.json" assembled-policy.json > "$name.rs"
              cmp "$name.rs" ${../../bootstrap/generated}/"$name.rs"
            done

            # Verifier-only source mutation. BOTH alternatives remain in place;
            # preferred source data must affect native JSON, full policy and Rust.
            substitute renamed-owner.cue variant.cue \
              --replace-fail 'Value::from(' 'Value::from(1 + '
            native renamed-package.cue variant.cue > variant.json
            printf '%s\n' '{"cases":{"integer":{"op":"concat","parts":[{"op":"text","value":"Value::from(1 + "},{"op":"signed","field":"value"},{"op":"text","value":"i64)"}]}}}' | jq -S . > expected-variant.json
            cmp variant.json expected-variant.json
            if cmp -s default.json variant.json; then exit 1; fi
            cue export renamed-package.cue variant.cue | jq -S . > oracle-variant.json
            cmp variant.json oracle-variant.json
            cue export "$schema" "$fragments" "$body" "$selector" variant.json | jq -S . > variant-policy.json
            cue export "$schema" "$fragments" "$body" "$selector" variant.cue | jq -S . > raw-variant-policy.json
            cmp variant-policy.json raw-variant-policy.json
            if cmp -s assembled-policy.json variant-policy.json; then exit 1; fi
            generate ${jsonExports pkgs}/tests/valid/ast-literal.json variant-policy.json > variant.rs
            printf '%s\n' \
              '// Generated by bootstrap-ast-codegen. Do not edit by hand.' \
              'use serde_json::Value;' \
              'pub fn build() -> Value {' \
              '    Value::from(1 + 7i64)' \
              '}' > expected-variant.rs
            cmp variant.rs expected-variant.rs
            cp default.json forced.json incomplete.err missing-provider.err \
              raw-policy.json assembled-policy.json variant.json variant-policy.json variant.rs "$out/"
            cp ${nativeAstIntegerPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeAstIntegerPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeAstIntegerPolicy pkgs).drvPath} > "$out/native-build-roots"
          '';
      unifyLoweringPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/unify-lowering-helpers.cue
            ../../bootstrap/unify-lowering-values.cue
            ../../bootstrap/unify-lowering-control.cue
            ../../bootstrap/unify-difference-policy.cue
            ../../bootstrap/unify-divide-policy.cue
            ../../bootstrap/unify-same-policy.cue
          ];
        };
      unifyPolicyComposition = pkgs: ''
        compose() {
          test "$1" != "$3" && test "$2" != "$3"
          test "$(wc -c < "$1")" -le 65536
          test "$(wc -c < "$2")" -le 65536
          env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
            -f ${../../bootstrap/compose-policy.jq} "$1" "$2" > "$3"
          test "$(wc -c < "$3")" -le 65536
        }
        assemble() {
          compose "$1" "$2" assemble-1.json
          compose assemble-1.json "$3" assemble-2.json
          compose assemble-2.json "$4" assemble-3.json
          compose assemble-3.json "$5" "$6"
        }
      '';
      nativeUnifyLoweringPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-unify-lowering-policy" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          jq=${pkgs.jq}/bin/jq
          for binary in "$exporter" "$jq"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          ${unifyPolicyComposition pkgs}
          sources=${unifyLoweringPolicySources pkgs}
          for source in "$sources"/*.cue; do test "$(wc -c < "$source")" -le 65536; done
          for run in first second; do
            for module in values control difference divide same; do
              companion=native-policy-package; source=unify-$module-policy
              case "$module" in values|control) companion=unify-lowering-helpers; source=unify-lowering-$module ;; esac
              env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
                "$sources/$companion.cue" "$sources/$source.cue" > "$run-$module.json"
              test "$(wc -c < "$run-$module.json")" -le 65536
            done
            assemble "$run-values.json" "$run-control.json" "$run-difference.json" "$run-divide.json" "$run-same.json" "$run.json"
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -nr '
              def permutations: if length == 0 then [] else .[] as $x | [.[] | select(. != $x)] | permutations | [$x] + . end;
              ["values","control","difference","divide","same"] | permutations | join(" ")' > orders
            test "$(wc -l < orders)" -eq 120
            while read -r a b c d e; do
              assemble "$run-$a.json" "$run-$b.json" "$run-$c.json" "$run-$d.json" "$run-$e.json" left.json
              compose "$run-$d.json" "$run-$e.json" right-1.json
              compose "$run-$c.json" right-1.json right-2.json
              compose "$run-$b.json" right-2.json right-3.json
              compose "$run-$a.json" right-3.json right.json
              cmp "$run.json" left.json; cmp "$run.json" right.json
            done < orders
          done
          for module in values control difference divide same; do cmp "first-$module.json" "second-$module.json"; done
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/unify-lowering.json"
        '';
      nativeUnifyLoweringPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeUnifyLoweringPolicy pkgs).drvPath
          ];
        };
      unifyOverrideFixtures =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap/tests/valid;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/tests/valid/unify-lowering-permissive.cue
            ../../bootstrap/tests/valid/integer-kernel-alternate.cue
            ../../bootstrap/tests/valid/division-kernel-alternate.cue
          ];
        };
      nativeUnifyLoweringPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-unify-lowering-policy-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeUnifyLoweringPolicyBuildClosure pkgs}/store-paths; then
              echo "forbidden unification construction dependency: $forbidden" >&2; exit 1
            fi
          done
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-unify-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$jq" "$rustfmt"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          ${unifyPolicyComposition pkgs}
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/unify-lowering-fragments.cue}
          selector=${../../bootstrap/unify-lowering.cue}
          cp ${unifyLoweringPolicySources pkgs}/*.cue .
          owners=(unify-lowering-helpers.cue unify-lowering-values.cue unify-lowering-control.cue
            unify-difference-policy.cue unify-divide-policy.cue unify-same-policy.cue)
          raw() { cue export "$schema" "$bridge" "$selector" "''${owners[@]}" "$@"; }
          raw | json > raw.json
          cmp raw.json ${nativeUnifyLoweringPolicy pkgs}/unify-lowering.json
          cmp raw.json ${jsonExports pkgs}/unify-lowering.json
          cue export "$schema" "$bridge" "$selector" raw.json | json > validated.json
          cmp raw.json validated.json
          for control in ${unifyNames}; do
            generate ${jsonExports pkgs}/"$control.json" raw.json > control.rs
            cmp control.rs ${../../bootstrap/generated}/"$(basename "$control").rs"
          done
          for pair in 'unify-lowering-permissive unify-equality-probe unify-permissive-bool' \
            'integer-kernel-alternate difference-control difference-as-product' \
            'division-kernel-alternate divide-control divide-as-quotient'; do
            read -r policy control golden <<< "$pair"
            raw ${unifyOverrideFixtures pkgs}/"$policy.cue" | json > override.json
            cmp override.json ${jsonExports pkgs}/tests/valid/"$policy.json"
            if test "$control" = unify-equality-probe; then control=tests/valid/$control; fi
            generate ${jsonExports pkgs}/"$control.json" override.json > override.rs
            cmp override.rs ${../../bootstrap/generated}/"$golden.rs"
          done
          no_export() {
            if cue export "$@" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          }
          for missing in 0 1 2 3 4 5; do
            providers=()
            for index in 0 1 2 3 4 5; do
              if test "$index" != "$missing"; then providers+=("''${owners[$index]}"); fi
            done
            no_export "$schema" "$bridge" "$selector" "''${providers[@]}"
            if test "$missing" = 0; then grep -E 'reference .*not found|undefined' failed.err
            else
              grep -F 'incomplete value' failed.err
              cue eval "$schema" "$bridge" "$selector" "''${providers[@]}" > /dev/null
            fi
            cp failed.err "$missing-missing.err"
          done
          # Test each required slot independently; the bridge must not supply it.
          env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -c '
            ([.cases | keys[] | ["cases", .]] +
             [.cases.integerPrimitive.cases | keys[] | ["cases","integerPrimitive","cases",.]] +
             [["aliases","choose"]])[]' raw.json > slots
          test "$(wc -l < slots)" -eq 41
          while read -r slot; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" --argjson slot "$slot" 'delpaths([$slot])' raw.json > missing.json
            no_export "$schema" "$bridge" "$selector" missing.json
            grep -F 'incomplete value' failed.err
            cue eval "$schema" "$bridge" "$selector" missing.json > /dev/null
          done < slots
          for policy in unify-lowering-permissive integer-kernel-alternate division-kernel-alternate; do
            no_export "$schema" "$bridge" "$selector" raw.json ${unifyOverrideFixtures pkgs}/"$policy.cue"
          done
          materialize() {
            native "''${owners[0]}" "''${owners[1]}" > values.json
            native "''${owners[0]}" "''${owners[2]}" > control.json
            native native-policy-package.cue "''${owners[3]}" > difference.json
            native native-policy-package.cue "''${owners[4]}" > divide.json
            native native-policy-package.cue "''${owners[5]}" > same.json
            assemble values.json control.json difference.json divide.json same.json variant.json
          }
          materialize; cmp variant.json raw.json
          printf '%s\n' '{"formatVersion":1,"result":{"op":"if","condition":{"op":"same"},"then":{"op":"integerPrimitive","operation":"difference"},"else":{"op":"integerPrimitive","operation":"divide"}}}' > tiny.json
          # Independently predicted COMPLETE source for each of the six owners.
          for index in 0 1 2 3 4 5; do
            source="''${owners[$index]}"
            banner='// Generated by bootstrap-unify-codegen. Do not edit by hand.'
            prefix='if ('; condition='Ok::<_, Error>(_left == _right)'
            difference='crate::runtime::integer_difference(&_left, &_right)'
            divide='crate::runtime::integer_divide(&_left, &_right)'
            case "$index" in
              0) substitute "$source" variant.cue --replace-fail 'if (' 'if /* Helper */ ('; prefix='if /* Helper */ (' ;;
              1) substitute "$source" variant.cue --replace-fail '// Generated by bootstrap-unify-codegen.' '// Values control.\n// Generated by bootstrap-unify-codegen.'; banner=$'// Values control.\n'"$banner" ;;
              2) substitute "$source" variant.cue --replace-fail '"if": #UnifyLoweringConditional' '"if": {op: "concat", parts: [{op: "text", value: "/* Control */ "}, #UnifyLoweringConditional]}'; prefix='/* Control */ if (' ;;
              3) substitute "$source" variant.cue --replace-fail 'crate::runtime::integer_difference' '/* Difference */ crate::runtime::integer_difference'; difference="/* Difference */ $difference" ;;
              4) substitute "$source" variant.cue --replace-fail 'crate::runtime::integer_divide' '/* Divide */ crate::runtime::integer_divide'; divide="/* Divide */ $divide" ;;
              5) substitute "$source" variant.cue --replace-fail 'Ok::<_, Error>(_left == _right)' 'Ok::<_, Error>(/* Same */ _left == _right)'; condition='Ok::<_, Error>(/* Same */ _left == _right)' ;;
            esac
            owners[$index]=variant.cue
            materialize
            raw | json > variant-raw.json
            cmp variant.json variant-raw.json
            if cmp -s raw.json variant.json; then exit 1; fi
            generate tiny.json variant.json > "$index.rs"
            format > "$index-expected.rs" <<RUST
          $banner
          use crate::{Error, runtime::Value};
          pub fn unify(_left: Value, _right: Value) -> Result<Value, Error> {
              $prefix$condition)? { $difference } else { $divide }
          }
          RUST
            cmp "$index.rs" "$index-expected.rs"
            cp variant.json "$out/owner-$index.json"
            owners[$index]="$source"
          done
          for invalid in opcode shape unbound; do
            case "$invalid" in
              opcode) substitute "''${owners[1]}" invalid.cue --replace-fail 'op: "concat"' 'op: "unknownNativeControl"' ;;
              shape) substitute "''${owners[1]}" invalid.cue --replace-fail 'formatVersion: 1' 'formatVersion: "wrong"' ;;
              unbound) substitute "''${owners[1]}" invalid.cue --replace-fail '{op: "lower", field: "result"}' '{op: "get", name: "unbound"}' ;;
            esac
            source="''${owners[1]}"; owners[1]=invalid.cue
            materialize
            if test "$invalid" = unbound; then
              raw | json > valid-shape.json; cmp valid-shape.json variant.json
            else
              no_export "$schema" "$bridge" "$selector" "''${owners[@]}"
              cp failed.err "$invalid-schema.err"
            fi
            if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" tiny.json variant.json > invalid.rs 2> invalid.err; then exit 1; fi
            test ! -s invalid.rs && test -s invalid.err
            if test "$invalid" = unbound; then grep -F 'unknown binding' invalid.err; fi
            cp invalid.err "$invalid-typed.err"
            owners[1]="$source"
          done
          cp raw.json "$out/unify-lowering.json"
          cp tiny.json "$out/tiny.json"
          cp *-missing.err *-schema.err *-typed.err [0-5].rs *-expected.rs "$out/"
          cp ${nativeUnifyLoweringPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeUnifyLoweringPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeUnifyLoweringPolicy pkgs).drvPath} > "$out/native-build-roots"
        '';
      prattLoweringPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/pratt-lowering-body.cue
            ../../bootstrap/pratt-add-policy.cue
          ];
        };
      prattPolicyComposition = pkgs: ''
        compose() {
          env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
            -f ${../../bootstrap/compose-policy.jq} "$1" "$2" > "$3"
          test "$(wc -c < "$3")" -le 65536
        }
        assemble() {
          compose "$1" "$2" assembly.json
          compose assembly.json "$3" "$4"
        }
      '';
      nativePrattLoweringPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-pratt-lowering-policy" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          jq=${pkgs.jq}/bin/jq
          for binary in "$exporter" "$jq"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          ${prattPolicyComposition pkgs}
          sources=${prattLoweringPolicySources pkgs}
          for source in "$sources"/*.cue ${prattTableSources pkgs}/*.cue; do
            test "$(wc -c < "$source")" -le 65536
          done
          cp ${nativePrattTables pkgs}/pratt-lowering-tables.json tables.json
          test "$(wc -c < tables.json)" -le 65536
          for run in first second; do
            for module in body add; do
              source=pratt-lowering-body
              if test "$module" = add; then source=pratt-add-policy; fi
              env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
                "$sources/native-policy-package.cue" "$sources/$source.cue" > "$run-$module.json"
              test "$(wc -c < "$run-$module.json")" -le 65536
            done
            cp tables.json "$run-tables.json"
            assemble "$run-body.json" "$run-add.json" "$run-tables.json" "$run.json"
            for order in 'body add tables' 'body tables add' 'add body tables' \
              'add tables body' 'tables body add' 'tables add body'; do
              read -r a b c <<< "$order"
              assemble "$run-$a.json" "$run-$b.json" "$run-$c.json" left.json
              compose "$run-$b.json" "$run-$c.json" right-1.json
              compose "$run-$a.json" right-1.json right.json
              cmp "$run.json" left.json; cmp "$run.json" right.json
            done
          done
          for module in body add; do cmp "first-$module.json" "second-$module.json"; done
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/pratt-lowering.json"
        '';
      nativePrattLoweringPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativePrattLoweringPolicy pkgs).drvPath
          ];
        };
      nativePrattLoweringPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-pratt-lowering-policy-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativePrattLoweringPolicyBuildClosure pkgs}/store-paths; then
              echo "forbidden Pratt policy construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${prattLoweringPolicySources pkgs} -type f | wc -l)" -eq 3
          test "$(find ${prattTableSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-pratt-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$jq" "$rustfmt"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" native-policy-package.cue "$1"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          ${prattPolicyComposition pkgs}
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/pratt-lowering-fragments.cue}
          selector=${../../bootstrap/pratt-lowering.cue}
          cp ${prattLoweringPolicySources pkgs}/*.cue .
          cp ${prattTableSources pkgs}/pratt-lowering-tables.cue .
          owners=(pratt-lowering-body.cue pratt-add-policy.cue pratt-lowering-tables.cue)
          raw() { cue export "$schema" "$bridge" "$selector" "''${owners[@]}" "$@"; }
          complete() { cue export "$schema" "$bridge" "$selector" "$@"; }
          native pratt-lowering-body.cue > body.json
          native pratt-add-policy.cue > add.json
          cp ${nativePrattTables pkgs}/pratt-lowering-tables.json tables.json
          assemble body.json add.json tables.json policy.json
          cmp policy.json ${nativePrattLoweringPolicy pkgs}/pratt-lowering.json
          cmp policy.json ${jsonExports pkgs}/pratt-lowering.json
          test "$(wc -c < policy.json)" -eq 11423
          raw | json > raw.json
          complete policy.json | json > validated.json
          cmp policy.json raw.json; cmp policy.json validated.json
          for control in ${prattNames}; do
            generate ${jsonExports pkgs}/"$control.json" policy.json > control.rs
            cmp control.rs ${../../bootstrap/generated}/"$(basename "$control").rs"
          done
          override=${../../bootstrap/tests/valid/pratt-lowering-no-increment.cue}
          raw "$override" | json > override.json
          cmp override.json ${jsonExports pkgs}/tests/valid/pratt-lowering-no-increment.json
          generate ${jsonExports pkgs}/pratt.json override.json > override.rs
          cmp override.rs ${../../bootstrap/generated/pratt-no-increment.rs}
          if complete policy.json "$override" > failed.json 2> failed.err; then exit 1; fi
          test ! -s failed.json; grep -F 'conflicting values' failed.err
          no_export() {
            if complete "$@" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json; grep -F 'incomplete value' failed.err
            cue eval "$schema" "$bridge" "$selector" "$@" > /dev/null
          }
          no_export
          env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -c \
            '([.aliases | keys[] | ["aliases", .]] + [.cases | keys[] | ["cases", .]])[]' policy.json > slots
          test "$(wc -l < slots)" -eq 47
          while read -r slot; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" --argjson slot "$slot" 'delpaths([$slot])' policy.json > missing.json
            no_export missing.json
          done < slots
          for other in pratt-add-policy.cue pratt-lowering-tables.cue; do
            if env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" pratt-lowering-body.cue "$other" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json; grep -F 'unsupported:' failed.err
          done
          printf '%s\n' '{"formatVersion":1,"body":[{"op":"setBindingPower","value":{"op":"add","left":{"op":"integer","value":1},"right":{"op":"integer","value":2}}},{"op":"scanSelector"},{"op":"parseGroup","minimum":{"op":"integer","value":0}}]}' > tiny.json
          # Independent COMPLETE predicted source, one authoritative owner at a time.
          for variant in body add tables; do
            power=set_binding_power; addition=binding_add; group=parse_group
            case "$variant" in
              body) substitute pratt-lowering-body.cue variant.cue --replace-fail '_frame.set_binding_power(' '_frame.set_binding_power_changed('; power=set_binding_power_changed ;;
              add) substitute pratt-add-policy.cue variant.cue --replace-fail '_frame.binding_add(' '_frame.binding_changed('; addition=binding_changed ;;
              tables) substitute pratt-lowering-tables.cue variant.cue --replace-fail '"parse_group"' '"parse_group_changed"'; group=parse_group_changed ;;
            esac
            native variant.cue > variant.json
            b=body.json; a=add.json; t=tables.json
            changed=("''${owners[@]}")
            case "$variant" in body) b=variant.json; changed[0]=variant.cue ;; add) a=variant.json; changed[1]=variant.cue ;; tables) t=variant.json; changed[2]=variant.cue ;; esac
            assemble "$b" "$a" "$t" variant-policy.json
            cp variant-policy.json "$out/owner-$variant-policy.json"
            if cmp -s variant-policy.json policy.json; then exit 1; fi
            cue export "$schema" "$bridge" "$selector" "''${changed[@]}" | json > variant-raw.json
            cmp variant-policy.json variant-raw.json
            generate tiny.json variant-policy.json > "$variant.rs"
            format > "$variant-expected.rs" <<RUST
          // Generated by bootstrap-pratt-codegen. Do not edit by hand.
          use crate::parser::{Error, Frame};
          pub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {
              _frame.step()?;
              _frame.$power(_frame.$addition(1u8, 2u8)?)?;
              _frame.step()?;
              _frame.scan_selector()?;
              _frame.step()?;
              _frame.$group(0u8)?;
              Ok(())
          }
          RUST
            cmp "$variant.rs" "$variant-expected.rs"
          done
          for invalid in opcode shape unbound; do
            case "$invalid" in
              opcode) replacement='{op: "unknownNativePrattOp"}' ;;
              shape) replacement='{op: "text", value: true}' ;;
              unbound) replacement='{op: "get", name: "missingNativeBinding"}' ;;
            esac
            substitute pratt-lowering-body.cue invalid.cue --replace-fail '{op: "text", value: "_frame.minimum()"}' "$replacement"
            native invalid.cue > invalid.json
            assemble invalid.json add.json tables.json malformed.json
            cp malformed.json "$out/malformed-$invalid-policy.json"
            if test "$invalid" = unbound; then complete malformed.json > schema-valid.json
            else
              if complete malformed.json > invalid-schema.json 2> invalid-schema.err; then exit 1; fi
              test ! -s invalid-schema.json && test -s invalid-schema.err
              cp invalid-schema.err "$out/$invalid-schema.err"
            fi
            if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" tiny.json malformed.json > invalid.rs 2> invalid.err; then exit 1; fi
            test ! -s invalid.rs && test -s invalid.err
            case "$invalid" in
              opcode) grep -F 'unknown variant `unknownNativePrattOp`' invalid.err ;;
              shape) grep -F 'invalid type: boolean `true`, expected a string' invalid.err ;;
              unbound) grep -F 'unknown binding' invalid.err ;;
            esac
            cp invalid.err "$out/$invalid-typed.err"
          done
          cp tiny.json policy.json override.json body.rs add.rs tables.rs *-expected.rs "$out/"
          cp ${nativePrattLoweringPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativePrattLoweringPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativePrattLoweringPolicy pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionBinarySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/instruction-binary-tables.cue
          ];
        };
      nativeInstructionBinaryTables =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-binary-tables" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          package=${instructionBinarySources pkgs}/native-policy-package.cue
          tables=${instructionBinarySources pkgs}/instruction-binary-tables.cue
          for run in first second reversed; do
            inputs=("$package" "$tables")
            if test "$run" = reversed; then inputs=("$tables" "$package"); fi
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "''${inputs[@]}" > "$run.json"
            test "$(wc -c < "$run.json")" -le 65536
            cmp first.json "$run.json"
          done
          mkdir -p "$out"
          cp first.json "$out/instruction-binary-tables.json"
        '';
      nativeInstructionBinaryTablesBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionBinaryTables pkgs).drvPath
          ];
        };
      instructionOverrideFixtures =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap/tests/valid;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/tests/valid/lowering-left-twice.cue
            ../../bootstrap/tests/valid/lowering-reverse.cue
          ];
        };
      nativeInstructionBinaryTablesVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-binary-tables-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionBinaryTablesBuildClosure pkgs}/store-paths; then
              echo "forbidden binary table construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionBinarySources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$driver" "$jq" "$rustfmt"; do
            test -f "$binary" && test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          query() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$@"; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" native-policy-package.cue "$1"; }
          cp ${instructionBinarySources pkgs}/*.cue .
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          unary=${../../bootstrap/instruction-unary-tables.cue}
          text=${../../bootstrap/instruction-module-text.cue}
          first=${../../bootstrap/instruction-module-first.cue}
          step=${../../bootstrap/instruction-module-step.cue}
          selector=${../../bootstrap/lowering.cue}
          place() { cue import -f -p bootstrap -l '#InstructionBinaryTables:' -o "$2" "$1"; }
          gate() { cue export "$schema" "$bridge" "$1" -e '#InstructionBinaryTables' > /dev/null; }
          ${instructionCoreConsumer}
          right=${../../bootstrap/instruction-right-field.cue}
          ${instructionOperandConsumer}
          policy() {
            provider=$1; shift
            core_gate || return
            right_gate || return
            gate "$provider" || return
            runtime_gate "$provider" || return
            module_gate "$text" "$first" "$step" || return
            touch full-export-started
            cue export "$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$runtime" "$unary" "$text" "$first" "$step" "$provider" "$@"
          }
          # Every raw oracle exports the SAME original default-free table source
          # independently, then imports and gates it; no old inline owner assists.
          raw() {
            cue export native-policy-package.cue "$1" > raw-tables.json || return
            shift
            place raw-tables.json raw-placed.cue || return
            policy raw-placed.cue "$@"
          }
          native instruction-binary-tables.cue > tables.json
          cmp tables.json ${nativeInstructionBinaryTables pkgs}/instruction-binary-tables.json
          test "$(wc -c < tables.json)" -eq 1429
          # Independent complete contract tree, not projected from any producer.
          query -n '
            [["index","index"],["difference","difference"],["product","product"],
             ["divide","divide"],["modulo","modulo"],["quotient","quotient"],
             ["recordIndex","record_index"],["listIndex","list_index"],
             ["logicalAnd","logical_and"],["logicalOr","logical_or"],["equal","equal"]]
            | {aliases:(map({key:.[0],value:"binaryRuntime"})|from_entries),
               select:{op:"select",field:"op",cases:(map({key:.[0],
                 value:{op:"text",value:("runtime::"+.[1])}})|from_entries)}}' | json > expected.json
          cmp tables.json expected.json
          place tables.json placed.cue
          consume_right() { policy placed.cue right-backfill.cue; }
          ${rightFieldConsumerProbe}
          consume_operand() { consume_right; }
          ${instructionOperandConsumerProbe}
          consume_core() { policy placed.cue "$selector"; }
          ${instructionCoreConsumerProbe pkgs}
          ${instructionRuntimeConsumerProbe}
          for context in lowering tests/valid/lowering-left-twice tests/valid/lowering-reverse; do
            case "$context" in lowering) source="$selector" ;; *) source=${instructionOverrideFixtures pkgs}/"$(basename "$context").cue" ;; esac
            name=$(basename "$context")
            policy placed.cue "$source" | json > "$name.json"
            raw instruction-binary-tables.cue "$source" | json > "$name-raw.json"
            cmp "$name.json" "$name-raw.json"
            cmp "$name.json" ${jsonExports pkgs}/"$context.json"
            core_gate
            gate placed.cue
            right_gate
            runtime_gate placed.cue
            module_gate "$text" "$first" "$step"
            cue export "$module" "$source" placed.cue "$step" "$first" "$text" "$unary" "$runtime" "$group" "$operand" "$right" "$core" "$bridge" "$schema" | json > reversed.json
            cmp "$name.json" reversed.json
          done
          # The reused R occurrences remain independently right; only sum refines.
          query -e '.cases.sum.args[1].value.value.field == "left" and
            .cases.binaryRuntime.parts[4].value.value.field == "right" and
            ([.cases.less,.cases.lessEqual,.cases.greater,.cases.greaterEqual,
              .cases.notEqual,.cases.remainder,.cases.unify.parts[1]] |
             all(.args[1].value.value.field == "right"))' lowering-left-twice.json > /dev/null
          for state in last-next first-previous; do
            case "$state" in last-next) initial=lastRule; advance=nextRule ;; first-previous) initial=firstRule; advance=previousRule ;; esac
            printf '%s\n' 'package bootstrap' \
              "#InstructionLowering & {module: [_, {op: \"$initial\"}, {body: [_, _, _, _, {op: \"$advance\"}]}, _]}" > context.cue
            policy placed.cue context.cue | json > "$state.json"
            raw instruction-binary-tables.cue context.cue | json > "$state-raw.json"
            cmp "$state.json" "$state-raw.json"
          done
          # Every scalar slot must be independently complete BEFORE policy schema.
          query -c 'paths(scalars)' tables.json > slots
          test "$(wc -l < slots)" -eq 35
          no_helper() {
            if cue export "$schema" "$bridge" "$1" -e '#InstructionBinaryTables' > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          }
          no_policy() {
            if policy "$1" "$selector" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          }
          printf '%s\n' 'package bootstrap' '#InstructionBinaryTables: {}' > missing.cue
          no_helper missing.cue
          cue eval "$schema" "$bridge" missing.cue -e '#InstructionBinaryTables' > /dev/null
          while read -r slot; do
            query --argjson slot "$slot" 'delpaths([$slot])' tables.json > missing.json
            place missing.json missing.cue
            cue eval "$schema" "$bridge" missing.cue -e '#InstructionBinaryTables' > /dev/null
            no_helper missing.cue
            grep -F 'incomplete value' failed.err
            no_policy missing.cue
            query --argjson slot "$slot" 'setpath($slot;42)' tables.json > invalid.json
            place invalid.json invalid.cue
            no_helper invalid.cue
            no_policy invalid.cue
          done < slots
          for extra in '.extra=true' '.aliases.extra="x"' '.select.extra="x"' \
            '.select.cases.extra={op:"text",value:"x"}' '.select.cases.index.extra="x"'; do
            query "$extra" tables.json > invalid.json
            place invalid.json invalid.cue
            no_helper invalid.cue
            no_policy invalid.cue
          done
          query 'del(.select.op)' tables.json > missing.json
          place missing.json missing.cue
          no_helper missing.cue
          # Deliberate counterfactual: bypassing Binary isolation supplies the missing op.
          # Right completeness remains independent; do not bypass its gate.
          core_gate
          right_gate
          module_gate "$text" "$first" "$step"
          cue export "$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$runtime" "$unary" "$text" "$first" "$step" missing.cue "$selector" > bypass.json
          query -e '.cases.binaryRuntime.parts[0].op=="select"' bypass.json > /dev/null
          no_policy missing.cue
          printf '%s\n' '{"formatVersion":1,"rules":{"a":{"result":{"op":"index","left":{"op":"field","field":"left"},"right":{"op":"field","field":"right"}}},"b":{"result":{"op":"difference","left":{"op":"field","field":"left"},"right":{"op":"field","field":"right"}}}}}' > tiny.json
          for invalid in opcode child empty-field alias selector-limit; do
            case "$invalid" in
              opcode) mutation='.select.op="unknownBinaryOp"' ;;
              child) mutation='.select.cases.index.op="get"' ;;
              empty-field) mutation='.select.field=""' ;;
              alias) mutation='.aliases.index="noSuchCase"' ;;
              selector-limit) mutation='.select.field=("x"*129)' ;;
            esac
            query "$mutation" tables.json > invalid.json
            place invalid.json invalid.cue
            gate invalid.cue
            case "$invalid" in
              opcode|child|empty-field) no_policy invalid.cue ;;
              alias|selector-limit)
                policy invalid.cue "$selector" > invalid-policy.json
                status=0
                env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" tiny.json invalid-policy.json > invalid.rs 2> invalid.err || status=$?
                test "$status" -eq 2 && test ! -s invalid.rs && test -s invalid.err
                case "$invalid" in
                  alias) grep -F 'alias must not collide and must target a canonical case' invalid.err ;;
                  selector-limit) grep -F 'selector must contain 1..=128 bytes' invalid.err ;;
                esac ;;
            esac
          done
          # Source-only mutations reach the COMPLETE independently predicted Rust.
          for variant in key method affix; do
            firstMethod=index; secondMethod=difference
            case "$variant" in
              key) substitute instruction-binary-tables.cue variant.cue --replace-fail 'index:' 'temporaryKey:' --replace-fail 'difference:' 'index:' --replace-fail 'temporaryKey:' 'difference:'; firstMethod=difference; secondMethod=index ;;
              method) substitute instruction-binary-tables.cue variant.cue --replace-fail '"index"' '"index_changed"'; firstMethod=index_changed ;;
              affix) substitute instruction-binary-tables.cue variant.cue --replace-fail 'runtime::\(method)' 'runtime::changed_\(method)_suffix'; firstMethod=changed_index_suffix; secondMethod=changed_difference_suffix ;;
            esac
            native variant.cue > variant.json
            if cmp -s variant.json tables.json; then exit 1; fi
            place variant.json variant-placed.cue
            policy variant-placed.cue "$selector" | json > variant-policy.json
            raw variant.cue "$selector" | json > variant-raw.json
            cmp variant-policy.json variant-raw.json
            generate tiny.json variant-policy.json > "$variant.rs"
            format > "$variant-expected.rs" <<RUST
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              match runtime::kind(node)? {
                  "a" => runtime::$firstMethod((runtime::integer(node, "left"))?, (runtime::integer(node, "right"))?),
                  "b" => runtime::$secondMethod((runtime::integer(node, "left"))?, (runtime::integer(node, "right"))?),
                  kind => Err(Error::MissingRule { kind: kind.to_string() }),
              }
          }
          RUST
            cmp "$variant.rs" "$variant-expected.rs"
            cp variant.cue "$variant-source.cue"
            cp variant.json "$variant-tables.json"
            cp variant-policy.json "$variant-policy.json"
            cp variant-raw.json "$variant-raw-policy.json"
          done
          for control in lowering lowering-left-twice lowering-reverse; do
            generate ${jsonExports pkgs}/evaluation.json "$control.json" > "$control.rs"
            case "$control" in lowering) golden=evaluation ;; *) golden=evaluation-''${control#lowering-} ;; esac
            cmp "$control.rs" ${../../bootstrap/generated}/"$golden.rs"
          done
          generate ${../../bootstrap/executor/tests/fixtures/codegen-escaped-rules.json} lowering.json > escaped.rs
          cmp escaped.rs ${../../bootstrap/generated/codegen-escaped-rules.rs}
          for policyName in ${loweringNames}; do
            name=$(basename "$policyName")
            env -i HOME="$PWD/home" PATH=/nonexistent "$driver" ${jsonExports pkgs}/"$policyName.json" | format > "$name.rs"
            cmp "$name.rs" ${../../bootstrap/generated}/"$name.rs"
          done
          cp tables.json expected.json lowering*.json last-next*.json first-previous*.json \
            key-* method-* affix-* key.rs method.rs affix.rs "$out/"
          cp ${nativeInstructionBinaryTablesBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionBinaryTablesBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionBinaryTables pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionUnarySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/instruction-unary-tables.cue
          ];
        };
      nativeInstructionUnaryTables =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-unary-tables" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          for run in first second; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
              ${instructionUnarySources pkgs}/native-policy-package.cue \
              ${instructionUnarySources pkgs}/instruction-unary-tables.cue > "$run.json"
            test "$(wc -c < "$run.json")" -le 65536
          done
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/instruction-unary-tables.json"
        '';
      nativeInstructionUnaryTablesBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionUnaryTables pkgs).drvPath
          ];
        };
      nativeInstructionUnaryTablesVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-unary-tables-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionUnaryTablesBuildClosure pkgs}/store-paths; then
              echo "forbidden unary table construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionUnarySources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$driver" "$jq" "$rustfmt"; do
            test -f "$binary" && test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" native-policy-package.cue "$1"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          selector=${../../bootstrap/lowering.cue}
          cp ${instructionUnarySources pkgs}/*.cue .
          # Raw text remains an oracle input, not a unary-constructor dependency.
          moduleText=${../../bootstrap/instruction-module-text.cue}
          firstOwner=${../../bootstrap/instruction-module-first.cue}
          stepOwner=${../../bootstrap/instruction-module-step.cue}
          # Independent Go-only table provider, never an input to native construction.
          cue export ${../../bootstrap/native-policy-package.cue} ${../../bootstrap/instruction-binary-tables.cue} > binary.json
          cue import -p bootstrap -l '#InstructionBinaryTables:' -o binary-placed.cue binary.json
          binary_gate() { cue export "$schema" "$bridge" binary-placed.cue -e '#InstructionBinaryTables' > /dev/null; }
          right=${../../bootstrap/instruction-right-field.cue}
          ${instructionOperandConsumer}
          ${instructionCoreConsumer}
          binary_export() { core_gate || return; right_gate || return; binary_gate || return; runtime_gate binary-placed.cue || return; module_gate "$moduleText" "$firstOwner" "$stepOwner" || return; touch full-export-started; cue export "$module" "$core" "$right" "$operand" "$group" "$runtime" "$@" binary-placed.cue; }
          raw() { binary_export "$schema" "$bridge" "$moduleText" "$firstOwner" "$stepOwner" instruction-unary-tables.cue "$@"; }
          complete() { binary_export "$schema" "$bridge" "$moduleText" "$firstOwner" "$stepOwner" "$@"; }
          native instruction-unary-tables.cue > tables.json
          cmp tables.json ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json
          test "$(wc -c < tables.json)" -eq 2107
          cue export native-policy-package.cue instruction-unary-tables.cue | json > oracle-tables.json
          cmp tables.json oracle-tables.json
          raw "$selector" | json > raw.json
          consume_right() { complete tables.json right-backfill.cue; }
          ${rightFieldConsumerProbe}
          consume_operand() { consume_right; }
          ${instructionOperandConsumerProbe}
          consume_core() { complete tables.json "$selector"; }
          ${instructionCoreConsumerProbe pkgs}
          ${instructionRuntimeConsumerProbe}
          complete tables.json "$selector" | json > complete.json
          cmp raw.json complete.json
          cmp complete.json ${jsonExports pkgs}/lowering.json
          binary_export "$selector" tables.json "$stepOwner" "$firstOwner" "$moduleText" "$bridge" "$schema" | json > reversed.json
          cmp complete.json reversed.json
          for control in left-twice reverse; do
            context=${../../bootstrap/tests/valid}/lowering-"$control.cue"
            raw "$context" | json > "raw-$control.json"
            complete tables.json "$context" | json > "$control.json"
            cmp "raw-$control.json" "$control.json"
            cmp "$control.json" ${jsonExports pkgs}/tests/valid/lowering-"$control.json"
            generate ${jsonExports pkgs}/evaluation.json "$control.json" > "evaluation-$control.rs"
            cmp "evaluation-$control.rs" ${../../bootstrap/generated}/evaluation-"$control.rs"
          done
          generate ${jsonExports pkgs}/evaluation.json complete.json > evaluation.rs
          cmp evaluation.rs ${../../bootstrap/generated/evaluation.rs}
          generate ${jsonExports pkgs}/tests/valid/evaluation-left-twice.json complete.json > rules-left-twice.rs
          cmp rules-left-twice.rs ${../../bootstrap/generated/evaluation-left-twice.rs}
          generate ${../../bootstrap/executor/tests/fixtures/codegen-escaped-rules.json} complete.json > escaped.rs
          cmp escaped.rs ${../../bootstrap/generated/codegen-escaped-rules.rs}
          # All six driver policies plus the retained compiled adversarial controls.
          for policy in ${loweringNames}; do
            name=$(basename "$policy")
            env -i HOME="$PWD/home" PATH=/nonexistent "$driver" ${jsonExports pkgs}/"$policy.json" | format > "$name.rs"
            cmp "$name.rs" ${../../bootstrap/generated}/"$name.rs"
          done
          no_export() {
            if complete "$@" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
            grep -F 'incomplete value' failed.err
            core_gate || return
            binary_gate
            right_gate
            runtime_gate binary-placed.cue || return
            cue eval "$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$runtime" "$moduleText" "$firstOwner" "$stepOwner" binary-placed.cue "$@" > /dev/null
          }
          no_export "$selector"
          env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -c \
            '([.aliases | keys[] | ["aliases", .]] + [.cases | keys[] | ["cases", .]])[]' tables.json > slots
          test "$(wc -l < slots)" -eq 11
          while read -r slot; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" --argjson slot "$slot" 'delpaths([$slot])' tables.json > missing.json
            no_export missing.json "$selector"
          done < slots
          printf '%s\n' '{"formatVersion":1,"rules":{"a":{"result":{"op":"negate","value":{"op":"field","field":"value"}}},"b":{"result":{"op":"length","value":{"op":"field","field":"value"}}}}}' > tiny.json
          # Predict the COMPLETE source independently, not by editing native output.
          for variant in key method affix; do
            first=negate; second=length
            case "$variant" in
              key) substitute instruction-unary-tables.cue variant.cue --replace-fail 'negate:' 'temporaryKey:' --replace-fail 'length:' 'negate:' --replace-fail 'temporaryKey:' 'length:'; first=length; second=negate ;;
              method) substitute instruction-unary-tables.cue variant.cue --replace-fail '"negate"' '"negate_changed"'; first=negate_changed ;;
              affix) substitute instruction-unary-tables.cue variant.cue --replace-fail 'runtime::\(method)' 'runtime::changed_\(method)'; first=changed_negate; second=changed_length ;;
            esac
            native variant.cue > variant.json
            if cmp -s variant.json tables.json; then exit 1; fi
            complete variant.json "$selector" | json > variant-policy.json
            complete variant.cue "$selector" | json > variant-raw.json
            cmp variant-policy.json variant-raw.json
            generate tiny.json variant-policy.json > "$variant.rs"
            format > "$variant-expected.rs" <<RUST
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              match runtime::kind(node)? {
                  "a" => runtime::$first((runtime::integer(node, "value"))?),
                  "b" => runtime::$second((runtime::integer(node, "value"))?),
                  kind => Err(Error::MissingRule { kind: kind.to_string() }),
              }
          }
          RUST
            cmp "$variant.rs" "$variant-expected.rs"
          done
          substitute instruction-unary-tables.cue invalid.cue --replace-fail 'value: "runtime::\(method)"' 'value: int'
          if native invalid.cue > invalid.json 2> invalid.err; then exit 1; fi
          test ! -s invalid.json && test -s invalid.err
          for invalid in opcode shape unbound; do
            case "$invalid" in
              opcode) substitute instruction-unary-tables.cue invalid.cue --replace-fail 'op: "text", value: "runtime::\(method)"' 'op: "unknownUnaryOp", value: "runtime::\(method)"' ;;
              shape) substitute instruction-unary-tables.cue invalid.cue --replace-fail 'value: "runtime::\(method)"' 'value: true' ;;
              unbound) substitute instruction-unary-tables.cue invalid.cue --replace-fail 'op: "text", value: "runtime::\(method)"' 'op: "get", "name": method' ;;
            esac
            native invalid.cue > invalid.json
            if test "$invalid" = unbound; then
              complete invalid.json "$selector" > schema-valid.json
            else
              if complete invalid.json "$selector" > failed.json 2> failed.err; then exit 1; fi
              test ! -s failed.json && test -s failed.err
              cp failed.err "$out/$invalid-schema.err"
            fi
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" --slurpfile m invalid.json '.cases.unaryRuntime=$m[0].cases.unaryRuntime' complete.json > malformed.json
            if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" tiny.json malformed.json > invalid.rs 2> invalid.err; then exit 1; fi
            test ! -s invalid.rs && test -s invalid.err
            case "$invalid" in
              opcode) grep -F 'unknown variant `unknownUnaryOp`' invalid.err ;;
              shape) grep -F 'invalid type: boolean `true`, expected a string' invalid.err ;;
              unbound) grep -F 'unknown binding' invalid.err ;;
            esac
            cp invalid.err "$out/$invalid-typed.err"
          done
          cp tables.json complete.json left-twice.json reverse.json *-expected.rs key.rs method.rs affix.rs "$out/"
          cp ${nativeInstructionUnaryTablesBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionUnaryTablesBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionUnaryTables pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/instruction-policy-group.cue
            ../../bootstrap/instruction-policy-shell.cue
          ];
        };
      instructionPolicyTools = ''
        bounded() {
          local file
          for file in "$@"; do
            test -f "$file" && test -r "$file" || return
            test "$(wc -c < "$file")" -le 65536 || return
          done
        }
        native_pair() {
          bounded "$1" "$2" || return
          test "$(($(wc -c < "$1") + $(wc -c < "$2")))" -le 65536 || return
          local order repeat
          local inputs
          for order in forward reverse; do
            inputs=("$1" "$2")
            if test "$order" = reverse; then inputs=("$2" "$1"); fi
            for repeat in first second; do
              env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "''${inputs[@]}" > "$3-$order-$repeat.json" || return
              bounded "$3-$order-$repeat.json" || return
              cmp "$3-forward-first.json" "$3-$order-$repeat.json" || return
            done
          done
        }
        compose() {
          test "$#" -eq 3 || return 2
          bounded "$1" "$2" || return
          test "$1" != "$3" && test "$2" != "$3" || return 2
          if test -e "$3"; then
            test ! "$1" -ef "$3" && test ! "$2" -ef "$3" || return 2
          fi
          local temporary status
          temporary=$(mktemp "$3.partial.XXXXXXXX") || return
          if env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
            -f ${../../bootstrap/compose-policy.jq} "$1" "$2" > "$temporary"; then
            status=0
            bounded "$temporary" || status=$?
            if test "$status" -eq 0; then mv "$temporary" "$3" || return; else
              rm -f "$temporary"
              return "$status"
            fi
          else
            status=$?
            rm -f "$temporary"
            return "$status"
          fi
        }
        placements() {
          bounded "$groupData" "$binaryData" "$runtimeData" || return
          cp "$package" group-adapter.cue || return
          chmod u+w group-adapter.cue || return
          printf '\n#InstructionOperandCases: ' >> group-adapter.cue || return
          cat "$groupData" >> group-adapter.cue || return
          cp "$package" shell-adapter.cue || return
          chmod u+w shell-adapter.cue || return
          printf '\n#InstructionBinaryTables: ' >> shell-adapter.cue || return
          cat "$binaryData" >> shell-adapter.cue || return
          printf '\n#InstructionBinaryRuntime: ' >> shell-adapter.cue || return
          cat "$runtimeData" >> shell-adapter.cue || return
          native_pair "$groupOwner" group-adapter.cue group || return
          native_pair "$shellOwner" shell-adapter.cue shell || return
        }
        assemble_policy() {
          compose "$coreData" "$unaryData" join-1.json || return
          compose join-1.json group-forward-first.json join-2.json || return
          compose join-2.json shell-forward-first.json join-3.json || return
          compose join-3.json "$moduleData" "$1"
        }
      '';
      nativeInstructionLoweringPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-lowering-policy" { } ''
          set -euo pipefail
          mkdir -p home "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          jq=${pkgs.jq}/bin/jq
          for tool in "$exporter" "$jq"; do
            test -f "$tool" && test -r "$tool" && test -x "$tool"
            case "$tool" in /*) ;; *) exit 1 ;; esac
          done
          ${instructionPolicyTools}
          package=${instructionPolicySources pkgs}/native-policy-package.cue
          groupOwner=${instructionPolicySources pkgs}/instruction-policy-group.cue
          shellOwner=${instructionPolicySources pkgs}/instruction-policy-shell.cue
          groupData=${nativeInstructionOperandCases pkgs}/operand-cases.json
          binaryData=${nativeInstructionBinaryTables pkgs}/instruction-binary-tables.json
          runtimeData=${nativeInstructionBinaryRuntime pkgs}/binary-runtime.json
          coreData=${nativeInstructionCoreCases pkgs}/instruction-core-cases.json
          unaryData=${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json
          moduleData=${nativeInstructionModule pkgs}/module.json
          bounded "$package" "$groupOwner" "$shellOwner" "$coreData" "$unaryData" "$moduleData"
          placements
          assemble_policy first.json
          assemble_policy second.json
          cmp first.json second.json
          cp first.json "$out/lowering.json"
        '';
      nativeInstructionLoweringPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionLoweringPolicy pkgs).drvPath
          ];
        };
      nativeInstructionLoweringPolicyVerification =
        pkgs:
        let
          canonicalConsumer = instructionCanonicalVerification pkgs (specSources pkgs true);
        in
        pkgs.runCommand "cuestacean-native-instruction-lowering-policy-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${../../bootstrap/schema.cue} ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionLoweringPolicyBuildClosure pkgs}/store-paths; then
              echo "forbidden instruction policy construction dependency: $forbidden" >&2; exit 1
            fi
          done
          for dependency in ${(nativeInstructionOperandCases pkgs).drvPath} \
            ${(nativeInstructionBinaryTables pkgs).drvPath} ${(nativeInstructionBinaryRuntime pkgs).drvPath} \
            ${(nativeInstructionCoreCases pkgs).drvPath} ${(nativeInstructionUnaryTables pkgs).drvPath} \
            ${(nativeInstructionModule pkgs).drvPath}; do
            grep -Fx "$dependency" ${nativeInstructionLoweringPolicyBuildClosure pkgs}/store-paths
          done
          # Bind the actual SAME whole providers at both construction use sites.
          for artifact in ${nativeInstructionBinaryTables pkgs}/instruction-binary-tables.json \
            ${nativeInstructionOperandCases pkgs}/operand-cases.json; do
            grep -F "$artifact" ${(nativeInstructionBinaryRuntime pkgs).drvPath}
            grep -F "$artifact" ${(nativeInstructionLoweringPolicy pkgs).drvPath}
          done
          grep -F '${nativeInstructionLoweringPolicy pkgs}/lowering.json' ${(jsonExports pkgs).drvPath}
          test "$(find ${instructionPolicySources pkgs} -type f | wc -l)" -eq 3
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" probe/home
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          compiler=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          queryTool=$jq
          for tool in "$exporter" "$oracle" "$compiler" "$driver" "$jq"; do
            test -f "$tool" && test -r "$tool" && test -x "$tool"
            case "$tool" in /*) ;; *) exit 1 ;; esac
          done
          ${instructionPolicyTools}
          query() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$@"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          ${canonicalConsumer}
          printf '%s\n' '${builtins.hashString "sha256" canonicalConsumer}' > "$out/executed-canonical-consumer-sha256"
          printf '%s\n' ${specSources pkgs true} > "$out/executed-canonical-source"
          # The following independent proof never enters the policy constructor.
          ${instructionPolicyEvidence pkgs}
          cp ${nativeInstructionLoweringPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionLoweringPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionLoweringPolicy pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionPolicyEvidence = pkgs: ''
        cat > header <<'HEADER'
        // Generated by bootstrap-codegen. Do not edit by hand.
        use bootstrap_executor::{Error, runtime};
        use serde_json::Value;

        pub fn execute(input: &str) -> Result<i64, Error> {
        let mut _context = runtime::EvaluationContext::default();
        runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
        }

        pub fn execute_json(input: &str) -> Result<Value, Error> {
        let mut _context = runtime::EvaluationContext::default();
        runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
        }

        fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
        _context.visit(|_context| evaluate_node(node, _context))
        }

        fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
        match runtime::kind(node)? {
        HEADER
        printf 'kind => Err(Error::MissingRule { kind: kind.to_string() }),\n}\n}\n' > footer
        # This tree is authored from the instruction contract, NEVER producer JSON.
        cat > expected.jq <<'EXPECTED'
        def path($s): {op:"path",value:$s};
        def text($s): {op:"text",value:$s};
        def string($s): {op:"string",field:$s};
        def call($s;$a): {op:"call",path:$s,args:$a};
        def wrapped($s): {op:"try",value:{op:"group",value:{op:"lower",field:$s}}};
        def constant($s): {op:"ok",value:path("runtime::Value::"+$s)};
        def unary: {
          lowerExclusiveBound:"lower_exclusive_bound",upperExclusiveBound:"upper_exclusive_bound",
          lowerBound:"lower_bound",upperBound:"upper_bound",negate:$unaryMethod,
          logicalNot:"logical_not",length:"length",stringLength:"string_length",
          listLength:"list_length",recordLength:"record_length"};
        def binary: {
          index:"index",difference:$difference,product:$product,divide:"divide",modulo:"modulo",
          quotient:"quotient",recordIndex:"record_index",listIndex:"list_index",
          logicalAnd:"logical_and",logicalOr:"logical_or",equal:"equal"};
        def select($m): {op:"select",field:"op",cases:($m|with_entries(.value=text("runtime::"+.value)))};
        def closure($x): {op:"concat",parts:[text("(|_context: &mut runtime::EvaluationContext| "),$x,text(")(_context)")]};
        path("node") as $n | string("field") as $f | path("_context") as $c | path("evaluate") as $e |
        {
          formatVersion:1,
          aliases: ((unary|with_entries(.value="unaryRuntime"))+(binary|with_entries(.value="binaryRuntime"))),
          cases:{
            "null":constant("Null"),top:constant("Top"),int:constant($integer),bool:constant("Bool"),stringType:constant("String"),
            text:call("runtime::text";[$n,$f]),list:call("runtime::list";[$n,$f]),record:call("runtime::record";[$n,$f]),
            recordExpression:call("runtime::record_expression";[$n,$f,$c,$e]),
            listExpression:call("runtime::list_expression";[$n,$f,$c,$e]),
            boolean:{op:"ok",value:call("runtime::Value::Boolean";[{op:"boolean",field:"value"}])},
            field:call("runtime::integer";[$n,$f]),
            evaluate:call("evaluate";[{op:"try",value:call("runtime::field";[$n,$f])},$c]),
            conjunction:call("runtime::conjunction";[$n,$f,$c,$e]),
            slice:call("runtime::slice_fields";[$n,string("operandField"),string("lowerField"),string("upperField"),$c,$e]),
            listSlice:closure(call("runtime::list_slice";[wrapped("value"),wrapped("lower"),wrapped("upper")])),
            sum:call("runtime::"+$sum;[wrapped($left),wrapped($right)]),
            less:call("runtime::less";[wrapped($left),wrapped($right)]),
            lessEqual:call("runtime::less_equal";[wrapped($left),wrapped($right)]),
            greater:call("runtime::greater";[wrapped($left),wrapped($right)]),
            greaterEqual:call("runtime::greater_equal";[wrapped($left),wrapped($right)]),
            notEqual:call("runtime::not_equal";[wrapped($left),wrapped($right)]),
            remainder:call("runtime::remainder";[wrapped($left),wrapped($right)]),
            unify:closure(call("runtime::unify";[wrapped($left),wrapped($right)])),
            unaryRuntime:{op:"concat",parts:[select(unary),text("("),wrapped("value"),text(")")]},
            binaryRuntime:{op:"concat",parts:[select(binary),text($opening),wrapped($left),text(", "),wrapped($right),text(")")]}
          },
          module:[text($h),{op:$initial},{op:"whileRules",body:[
            {op:(if $bodyOrder=="ordinary" then "kind" else "expression" end)},text($separator),
            {op:(if $bodyOrder=="ordinary" then "expression" else "kind" end)},text($terminator),{op:$advance}]},text($ftr)]
        }
        EXPECTED
        expected_policy() {
          query -n -S --rawfile h header --rawfile ftr footer \
            --arg integer "$integer" --arg unaryMethod "$unaryMethod" \
            --arg difference "$difference" --arg product "$product" --arg sum "$sum" \
            --arg left "$left" --arg right "$right" --arg opening "$opening" \
            --arg initial "$initial" --arg advance "$advance" --arg separator "$separator" \
            --arg bodyOrder "$bodyOrder" --arg terminator "$terminator" -f expected.jq
        }
        # Complete unformatted source: int/bool children distinguish operand choice;
        # distinct keys distinguish all four cursor states. No emitted-source input.
        cat > tiny.json <<'RULES'
        {"formatVersion":1,"rules":{"a":{"result":{"op":"sum","left":{"op":"int"},"right":{"op":"bool"}}},"b":{"result":{"op":"difference","left":{"op":"int"},"right":{"op":"bool"}}},"c":{"result":{"op":"negate","value":{"op":"int"}}}}}
        RULES
        # Typed IR recursively lowers child instructions, not runtime AST lookups.
        expected_rust() {
          cat header
          local key l r expression
          for key in $keys; do
            l="Ok::<_, Error>(runtime::Value::$integer)"
            r='Ok::<_, Error>(runtime::Value::Bool)'
            if test "$left" = right; then l=$r; fi
            if test "$right" = left; then r="Ok::<_, Error>(runtime::Value::$integer)"; fi
            case "$key" in
              a) expression="runtime::$sum(($l)?, ($r)?)" ;;
              b) expression="runtime::$difference$opening($l)?, ($r)?)" ;;
              c) expression="runtime::$unaryMethod((Ok::<_, Error>(runtime::Value::$integer))?)" ;;
            esac
            if test "$bodyOrder" = ordinary; then
              printf '"%s"%s%s%s' "$key" "$separator" "$expression" "$terminator"
            else
              printf '%s%s"%s"%s' "$expression" "$separator" "$key" "$terminator"
            fi
          done
          cat footer
        }
        sources=${specSources pkgs true}
        ${instructionCoreConsumer}
        ${instructionModuleConsumer}
        ${instructionCanonicalConsumer pkgs (specSources pkgs true)}
        rebuild() {
          native_pair raw/instruction-unary-tables.cue raw/native-policy-package.cue unary
          native_pair raw/instruction-core-cases.cue raw/native-policy-package.cue core
          native_pair raw/instruction-binary-tables.cue raw/native-policy-package.cue binary
          native_pair raw/instruction-right-field.cue raw/instruction-right-field-export.cue right
          cp raw/instruction-right-operand-export.cue r-adapter.cue; chmod u+w r-adapter.cue
          printf '\n#InstructionRightField: ' >> r-adapter.cue; cat right-forward-first.json >> r-adapter.cue
          native_pair raw/instruction-right-operand.cue r-adapter.cue r
          cp raw/instruction-operand-cases-export.cue g-adapter.cue; chmod u+w g-adapter.cue
          printf '\n#InstructionRightOperand: ' >> g-adapter.cue; cat r-forward-first.json >> g-adapter.cue
          native_pair raw/instruction-operand-cases.cue g-adapter.cue operand-cases
          cp raw/instruction-binary-runtime-export.cue runtime-adapter.cue; chmod u+w runtime-adapter.cue
          printf '\n#InstructionBinaryTables: ' >> runtime-adapter.cue; cat binary-forward-first.json >> runtime-adapter.cue
          printf '\n#InstructionOperandCases: ' >> runtime-adapter.cue; cat operand-cases-forward-first.json >> runtime-adapter.cue
          printf '\n#InstructionRightOperand: ' >> runtime-adapter.cue; cat r-forward-first.json >> runtime-adapter.cue
          native_pair raw/instruction-binary-runtime.cue runtime-adapter.cue runtime
          for provider in text first step; do
            native_pair "raw/instruction-module-$provider.cue" "raw/instruction-module-$provider-export.cue" "$provider"
          done
          cp raw/instruction-module-export.cue module-adapter.cue; chmod u+w module-adapter.cue
          printf '\n#InstructionModuleText: ' >> module-adapter.cue; cat text-forward-first.json >> module-adapter.cue
          printf '\n#InstructionModuleFirst: ' >> module-adapter.cue; cat first-forward-first.json >> module-adapter.cue
          printf '\n#InstructionModuleStep: ' >> module-adapter.cue; cat step-forward-first.json >> module-adapter.cue
          native_pair raw/instruction-module.cue module-adapter.cue module
          package=raw/native-policy-package.cue
          groupOwner=raw/instruction-policy-group.cue; shellOwner=raw/instruction-policy-shell.cue
          groupData=operand-cases-forward-first.json; binaryData=binary-forward-first.json; runtimeData=runtime-forward-first.json
          coreData=core-forward-first.json; unaryData=unary-forward-first.json; moduleData=module-forward-first.json
          placements
          assemble_policy native-policy.json
        }
        raw_policy() {
          cue export raw/native-policy-package.cue raw/instruction-binary-tables.cue > raw-binary.json || return
          cue import -f -p bootstrap -l '#InstructionBinaryTables:' -o instruction-binary-tables.cue raw-binary.json || return
          core=raw/instruction-core-cases.cue
          rightProvider=raw/instruction-right-field.cue; operand=raw/instruction-right-operand.cue
          group=raw/instruction-operand-cases.cue; runtime=raw/instruction-binary-runtime.cue
          module=raw/instruction-module.cue
          moduleProviders=(raw/instruction-module-text.cue raw/instruction-module-first.cue raw/instruction-module-step.cue)
          policy_gates || return
          cue export "$sources/schema.cue" "$sources/lowering-fragments.cue" "$core" \
            "$rightProvider" "$operand" "$group" "$runtime" instruction-binary-tables.cue \
            raw/instruction-unary-tables.cue "$module" "''${moduleProviders[@]}" "$sources/lowering.cue"
        }
        for variant in canonical core unary binary-method binary-keys right r group runtime text footer-source first step module separator body-order reverse; do
          mkdir -p "$variant/raw" "$variant/home"
          cp "$sources"/instruction-*.cue "$variant/raw/"
          cp "$sources/native-policy-package.cue" "$variant/raw/"
          cp header footer tiny.json expected.jq "$variant/"
          (
            cd "$variant"
            integer=Int; unaryMethod=negate; difference=difference; product=product; sum=sum
            left=left; right=right; opening='('; initial=firstRule; advance=nextRule
            separator=' => '; bodyOrder=ordinary; terminator=$',\n'; keys='a b c'
            case "$variant" in
              core) substituteInPlace raw/instruction-core-cases.cue --replace-fail 'runtime::Value::Int' 'runtime::Value::Top'; integer=Top ;;
              unary) substituteInPlace raw/instruction-unary-tables.cue --replace-fail 'negate:  "negate"' 'negate:  "logical_not"'; unaryMethod=logical_not ;;
              binary-method) substituteInPlace raw/instruction-binary-tables.cue --replace-fail '"difference"' '"sum"'; difference=sum ;;
              binary-keys)
                substituteInPlace raw/instruction-binary-tables.cue --replace-fail 'difference:' 'temporary:' --replace-fail 'product:' 'difference:' --replace-fail 'temporary:' 'product:'
                difference=product; product=difference ;;
              right) substituteInPlace raw/instruction-right-field.cue --replace-fail '*"right" | "left"' '"right" | *"left"'; right=left ;;
              r) substituteInPlace raw/instruction-right-operand.cue --replace-fail '#InstructionRightField.field' '"left"'; right=left ;;
              group) substituteInPlace raw/instruction-operand-cases.cue --replace-fail 'field: "left"' 'field: "right"'; left=right ;;
              runtime) substituteInPlace raw/instruction-binary-runtime.cue --replace-fail 'value: "("' 'value: "( "'; opening='( ' ;;
              text) substituteInPlace raw/instruction-module-text.cue --replace-fail '// Generated by bootstrap-codegen. Do not edit by hand.' '// Independent complete-policy header.'
                substituteInPlace header --replace-fail '// Generated by bootstrap-codegen. Do not edit by hand.' '// Independent complete-policy header.' ;;
              footer-source)
                substituteInPlace raw/instruction-module-text.cue --replace-fail 'footer: "kind' 'footer: "// Independent complete-policy footer.\nkind'
                printf '// Independent complete-policy footer.\n' > footer
                cat ../footer >> footer ;;
              separator)
                substituteInPlace raw/instruction-module.cue --replace-fail 'value: " => "' 'value: " =>  "'
                separator=' =>  ' ;;
              body-order)
                substituteInPlace raw/instruction-module.cue --replace-fail '{op: "kind"}' '{op: "temporaryKind"}' \
                  --replace-fail '{op: "expression"}' '{op: "kind"}' --replace-fail '{op: "temporaryKind"}' '{op: "expression"}'
                bodyOrder=swapped ;;
              first|reverse) substituteInPlace raw/instruction-module-first.cue --replace-fail '*"firstRule" | "lastRule"' '"firstRule" | *"lastRule"'; initial=lastRule; keys=c ;;
              step) substituteInPlace raw/instruction-module-step.cue --replace-fail '*"nextRule" | "previousRule"' '"nextRule" | *"previousRule"'; advance=previousRule; keys=a ;;
              module) substituteInPlace raw/instruction-module.cue --replace-fail 'value: ",\n"' 'value: ", \n"'; terminator=$', \n' ;;
            esac
            if test "$variant" = reverse; then
              substituteInPlace raw/instruction-module-step.cue --replace-fail '*"nextRule" | "previousRule"' '"nextRule" | *"previousRule"'
              advance=previousRule; keys='c b a'
            fi
            rebuild
            expected_policy > expected-policy.json
            cmp expected-policy.json native-policy.json
            raw_policy | query -S . > raw-policy.json
            cmp expected-policy.json raw-policy.json
            expected_rust > expected.rs
            for mode in raw native; do
              env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json "$mode-policy.json" > "$mode.rs"
              cmp expected.rs "$mode.rs"
            done
            if test "$variant" != canonical; then
              if cmp -s expected.rs ../canonical/expected.rs; then echo "unobservable source effect: $variant" >&2; exit 1; fi
            fi
          )
          cp -r "$variant" "$out/source-$variant"
          rm -rf "$out/source-$variant/home"
        done
        cmp canonical/native-policy.json ${nativeInstructionLoweringPolicy pkgs}/lowering.json
        cmp canonical/native-policy.json ${jsonExports pkgs}/lowering.json
        substitute canonical/raw/instruction-policy-group.cue unrelated-owner.cue \
          --replace-fail '#InstructionOperandCases' '#Whole'
        substitute canonical/group-adapter.cue unrelated-data.cue \
          --replace-fail '#InstructionOperandCases' '#Whole'
        native_pair unrelated-owner.cue unrelated-data.cue renamed-group
        cmp renamed-group-forward-first.json canonical/group-forward-first.json
        substitute canonical/raw/instruction-policy-shell.cue arbitrary-owner.cue \
          --replace-fail '#InstructionBinaryTables' '#Tables' --replace-fail '#InstructionBinaryRuntime' '#Run'
        substitute canonical/shell-adapter.cue arbitrary-data.cue \
          --replace-fail '#InstructionBinaryTables' '#Tables' --replace-fail '#InstructionBinaryRuntime' '#Run'
        native_pair arbitrary-owner.cue arbitrary-data.cue renamed-shell
        cmp renamed-shell-forward-first.json canonical/shell-forward-first.json
        # Original raw refinement remains live and unequal selected DATA conflicts.
        cue export "$sources/schema.cue" "$sources/lowering-fragments.cue" \
          canonical/raw/instruction-{unary-tables,core-cases,right-field,right-operand,operand-cases,binary-runtime,module-text,module-first,module-step,module}.cue \
          canonical/instruction-binary-tables.cue "$sources/tests/valid/lowering-left-twice.cue" | query -S . > twice.json
        query '.cases.sum.args[1].value.value.field="left"' canonical/expected-policy.json > twice-expected.json
        cmp twice.json twice-expected.json
        printf '%s\n' 'package bootstrap' 'cases:{sum:{args:[_,{value:{value:{field:"left"}}}]}}' > late-context.cue
        if cue export ${nativeInstructionLoweringPolicy pkgs}/lowering.json late-context.cue > late.json 2> late.err; then exit 1; fi
        test ! -s late.json && test -s late.err
        complete() {
          query -e --slurpfile expected canonical/expected-policy.json '
            keys==($expected[0]|keys) and .formatVersion==1 and
            (.cases|keys)==($expected[0].cases|keys) and
            (.aliases|keys)==($expected[0].aliases|keys) and has("module")' "$1" > /dev/null
        }
        complete canonical/native-policy.json
        test "$(query '[..]|length' canonical/native-policy.json)" -eq 512
        # Required slots come ONLY from the independently authored inventory.
        query -nr --slurpfile p canonical/expected-policy.json '
          ($p[0].cases|keys[]|"cases "+.),($p[0].aliases|keys[]|"aliases "+.)' > required-slots
        test "$(wc -l < required-slots)" -eq 47
        while read -r namespace key; do
          query --arg n "$namespace" --arg k "$key" 'del(.[$n][$k])' canonical/native-policy.json > missing-slot.json
          if complete missing-slot.json; then exit 1; fi
        done < required-slots
        for root in module formatVersion; do
          query --arg k "$root" 'del(.[$k])' canonical/native-policy.json > "missing-$root.json"
          if complete "missing-$root.json"; then exit 1; fi
        done
        schema_policy() {
          cue import -f -p bootstrap -o candidate.cue "$1" || return
          printf '%s\n' 'package bootstrap' '#LoweringPolicy' > candidate-root.cue
          cue export "$sources/schema.cue" candidate.cue candidate-root.cue
        }
        typed_failure() {
          local status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json "$1" > typed.out 2> typed.err || status=$?
          test "$status" -eq 2 && test ! -s typed.out && grep -F "$2" typed.err
          env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json canonical/native-policy.json > typed-recovery.rs
          cmp typed-recovery.rs canonical/expected.rs
        }
        # Go singleton backfill is explicitly NOT a complete native DATA version.
        schema_policy missing-formatVersion.json | query -S . > version-backfilled.json
        cmp version-backfilled.json canonical/expected-policy.json
        typed_failure missing-formatVersion.json 'missing field `formatVersion`'
        # VERIFIER ONLY: do not feed an oversize text control into publication.
        query '.module[0].value=("x"*65537)' canonical/expected-policy.json > oversize-module.json
        schema_policy oversize-module.json > oversize-schema.json
        status=0
        env -i HOME="$PWD/home" PATH=/nonexistent "$driver" oversize-module.json > oversized.rs 2> oversized.err || status=$?
        test "$status" -eq 2 && test ! -s oversized.rs
        grep -F 'module text exceeds 64 KiB' oversized.err
        # Go/typed-valid cursor actions fail ONLY during real module assembly.
        query '.module[1].op="nextRule"' canonical/expected-policy.json > cursor-protocol.json
        schema_policy cursor-protocol.json > cursor-schema.json
        env -i HOME="$PWD/home" PATH=/nonexistent "$driver" cursor-protocol.json > cursor-driver.rs
        test -s cursor-driver.rs
        status=0
        env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json cursor-protocol.json > cursor.rs 2> cursor.err || status=$?
        test "$status" -eq 2 && test ! -s cursor.rs
        grep -F 'no current rule' cursor.err
        env -i HOME="$PWD/home" PATH=/nonexistent "$driver" canonical/expected-policy.json > valid-driver.rs
        env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json canonical/expected-policy.json > module-recovery.rs
        cmp module-recovery.rs canonical/expected.rs
        cp oversize-module.json oversize-schema.json oversized.err cursor-protocol.json cursor-schema.json cursor-driver.rs cursor.err valid-driver.rs module-recovery.rs "$out/"
        for variant in unknown-op wrong-type selector group-wiring shell-version; do
          mkdir -p "negative-$variant/home"
          cp -r canonical/raw "negative-$variant/raw"
          cp tiny.json canonical/expected-policy.json "negative-$variant/"
          (
            cd "negative-$variant"
            case "$variant" in
              unknown-op) substituteInPlace raw/instruction-core-cases.cue --replace-fail 'int: {op: "ok"' 'int: {op: "bogus"' ;;
              wrong-type) substituteInPlace raw/instruction-core-cases.cue --replace-fail 'value: "runtime::Value::Int"' 'value: false' ;;
              selector)
                long=$(printf '%0129d' 0)
                substituteInPlace raw/instruction-right-field.cue --replace-fail '"right"' "\"$long\"" ;;
              group-wiring) substituteInPlace raw/instruction-policy-group.cue --replace-fail '.cases' '.leftOperand' ;;
              shell-version) substituteInPlace raw/instruction-policy-shell.cue --replace-fail 'formatVersion: 1' 'formatVersion: 2' ;;
            esac
            rebuild
            # Mutate ONLY the independently authored tree, never observed output.
            case "$variant" in
              unknown-op) query '.cases.int.op="bogus"' expected-policy.json > expected-bad.json ;;
              wrong-type) query '.cases.int.value.value=false' expected-policy.json > expected-bad.json ;;
              selector)
                query --arg field "$long" '
                  .cases.binaryRuntime.parts[4].value.value.field=$field |
                  .cases.unify.parts[1].args[1].value.value.field=$field |
                  reduce ["sum","less","lessEqual","greater","greaterEqual","notEqual","remainder"][] as $name
                    (.;.cases[$name].args[1].value.value.field=$field)' expected-policy.json > expected-bad.json ;;
              group-wiring)
                query -S 'del(.cases.sum,.cases.less,.cases.lessEqual,.cases.greater,.cases.greaterEqual,
                  .cases.notEqual,.cases.remainder,.cases.unify) |
                  .cases.op="try" | .cases.value={op:"group",value:{op:"lower",field:"left"}}' expected-policy.json > expected-bad.json ;;
              shell-version) query '.formatVersion=2' expected-policy.json > expected-bad.json ;;
            esac
            cmp expected-bad.json native-policy.json
            # Native literal DATA construction succeeds; validation is a NEW phase.
            status=0
            if test "$variant" = selector; then
              schema_policy native-policy.json > schema.json
              diagnostic='selector must contain 1..=128 bytes'
            else
              schema_policy native-policy.json > schema.json 2> schema.err || status=$?
              test "$status" -eq 1 && test ! -s schema.json && test -s schema.err
              case "$variant" in
                unknown-op) diagnostic='unknown variant `bogus`' ;;
                wrong-type) diagnostic='invalid type: boolean' ;;
                group-wiring) diagnostic='invalid type: string' ;;
                shell-version) diagnostic='formatVersion 1 required' ;;
              esac
            fi
            status=0
            env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json native-policy.json > typed.out 2> typed.err || status=$?
            test "$status" -eq 2 && test ! -s typed.out && grep -F "$diagnostic" typed.err
            printf 'native=0 schema=%s typed=%s\n' "$(if test "$variant" = selector; then echo 0; else echo 1; fi)" "$status" > phase-receipt
          )
          env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json canonical/native-policy.json > negative-recovery.rs
          cmp negative-recovery.rs canonical/expected.rs
        done
        # An actual whole-provider omission fails native lookup, not schema backfill.
        cp ${instructionPolicySources pkgs}/native-policy-package.cue missing-provider.cue
        status=0
        env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
          ${instructionPolicySources pkgs}/instruction-policy-group.cue missing-provider.cue > missing-provider.json 2> missing-provider.err || status=$?
        test "$status" -eq 2 && test ! -s missing-provider.json
        grep -F '#InstructionOperandCases' missing-provider.err
        for missing in BinaryTables BinaryRuntime; do
          cp ${instructionPolicySources pkgs}/native-policy-package.cue missing-shell.cue
          chmod u+w missing-shell.cue
          if test "$missing" = BinaryTables; then
            printf '\n#InstructionBinaryRuntime: ' >> missing-shell.cue
            cat canonical/runtime-forward-first.json >> missing-shell.cue
          else
            printf '\n#InstructionBinaryTables: ' >> missing-shell.cue
            cat canonical/binary-forward-first.json >> missing-shell.cue
          fi
          status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
            ${instructionPolicySources pkgs}/instruction-policy-shell.cue missing-shell.cue > missing-shell.json 2> missing-shell.err || status=$?
          test "$status" -eq 2 && test ! -s missing-shell.json
          grep -F "#Instruction$missing" missing-shell.err
        done
        # Enumeration is independent of the composer/producer. ALL14 Catalan trees.
        query -nr '
          def orders: if length==0 then [] else .[] as $x | [.[]|select(.!=$x)] | orders | [$x]+. end;
          ["core","unary","group","shell","module"] | orders | join(" ")' > orders
        query -nr '
          def trees($n;$offset):
            if $n==1 then [($offset|tostring)] else
              [range(1;$n) as $k | trees($k;$offset)[] as $a |
               trees($n-$k;$offset+$k)[] | $a+" "+.+" +"] end;
          trees(5;0)[]' > trees
        test "$(wc -l < orders)" -eq 120 && test "$(sort -u orders | wc -l)" -eq 120
        test "$(wc -l < trees)" -eq 14 && test "$(sort -u trees | wc -l)" -eq 14
        cp canonical/core-forward-first.json part-core.json
        cp canonical/unary-forward-first.json part-unary.json
        cp canonical/group-forward-first.json part-group.json
        cp canonical/shell-forward-first.json part-shell.json
        cp canonical/module-forward-first.json part-module.json
        calls=0; results=0
        : > composition-coverage
        for repeat in first second; do
          while read -r a b c d e; do
            parts=("part-$a.json" "part-$b.json" "part-$c.json" "part-$d.json" "part-$e.json")
            while read -r tree; do
              stack=(); index=0
              for token in $tree; do
                if test "$token" = +; then
                  size=''${#stack[@]}; test "$size" -ge 2
                  leftFile=''${stack[size-2]}; rightFile=''${stack[size-1]}
                  stack=("''${stack[@]:0:size-2}")
                  result="tree-$index.json"; index=$((index+1))
                  compose "$leftFile" "$rightFile" "$result"
                  calls=$((calls+1)); stack+=("$result")
                else stack+=("''${parts[token]}"); fi
              done
              test "$index" -eq 4 && test "''${#stack[@]}" -eq 1
              cmp "''${stack[0]}" canonical/expected-policy.json
              printf '%s|%s %s %s %s %s|%s\n' "$repeat" "$a" "$b" "$c" "$d" "$e" "$tree" >> composition-coverage
              results=$((results+1))
            done < trees
          done < orders
        done
        test "$calls" -eq 13440 && test "$results" -eq 3360
        test "$(sort -u composition-coverage | wc -l)" -eq 3360
        printf 'calls=%s results=%s\n' "$calls" "$results" > composition-complete
        # Strict binary wrapper failure propagation, including conditional callers.
        printf '{}\n' > empty.json; printf '[]\n' > array.json
        printf '{"x":1}\n' > overlap.json; printf '{"x":[]}\n' > atomic.json
        query -S . overlap.json > overlap-sorted.json
        for pair in 'array.json empty.json' 'overlap.json overlap.json' 'atomic.json atomic.json'; do
          read -r a b <<< "$pair"
          for mode in normal if or; do
            rm -f rejected.json
            status=0
            case "$mode" in
              normal) set +e; compose "$a" "$b" rejected.json > failure.out 2> failure.err; status=$?; set -e ;;
              if) if compose "$a" "$b" rejected.json > failure.out 2> failure.err; then exit 1; else status=$?; fi ;;
              or) compose "$a" "$b" rejected.json > failure.out 2> failure.err || status=$? ;;
            esac
            test "$status" -ne 0 && test ! -e rejected.json
            compose empty.json overlap.json recovery.json
            cmp recovery.json overlap-sorted.json
          done
        done
        query -n '{a:("x"*40000)}' > large-a.json
        query -n '{b:("x"*40000)}' > large-b.json
        if compose large-a.json large-b.json oversized.json > large.out 2> large.err; then exit 1; fi
        test ! -e oversized.json
        if compose overlap.json empty.json overlap.json; then exit 1; fi
        ln overlap.json linked.json
        if compose linked.json empty.json overlap.json; then exit 1; fi
        cp -r canonical "$out/"
        cp orders trees composition-coverage composition-complete twice.json twice-expected.json required-slots "$out/"
        cp -r negative-* "$out/"
      '';
      instructionModuleSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/instruction-module.cue
            ../../bootstrap/instruction-module-export.cue
          ];
        };
      nativeInstructionModule =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-module" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          owner=${instructionModuleSources pkgs}/instruction-module.cue
          root=${instructionModuleSources pkgs}/instruction-module-export.cue
          text=${nativeInstructionModuleText pkgs}/instruction-module-text.json
          first=${nativeInstructionModuleControls pkgs}/first.json
          step=${nativeInstructionModuleControls pkgs}/step.json
          for input in "$owner" "$root" "$text" "$first" "$step"; do
            test -f "$input" && test -r "$input"
            test "$(wc -c < "$input")" -le 65536
          done
          cp "$root" adapter.cue
          chmod u+w adapter.cue
          for provider in Text First Step; do
            case "$provider" in Text) input=$text ;; First) input=$first ;; Step) input=$step ;; esac
            printf '\n#InstructionModule%s: ' "$provider" >> adapter.cue
            cat "$input" >> adapter.cue
          done
          test "$(wc -c < adapter.cue)" -le 65536
          test "$(($(wc -c < "$owner") + $(wc -c < adapter.cue)))" -le 65536
          for order in forward reverse; do
            for repeat in first second; do
              inputs=("$owner" adapter.cue)
              if test "$order" = reverse; then inputs=(adapter.cue "$owner"); fi
              env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "''${inputs[@]}" > "$order-$repeat.json"
              test "$(wc -c < "$order-$repeat.json")" -le 65536
              cmp forward-first.json "$order-$repeat.json"
            done
          done
          cp forward-first.json "$out/module.json"
        '';
      nativeInstructionModuleBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionModule pkgs).drvPath
          ];
        };
      nativeInstructionModuleVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-module-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${../../bootstrap/schema.cue} ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionModuleBuildClosure pkgs}/store-paths; then
              echo "forbidden Module construction dependency: $forbidden" >&2; exit 1
            fi
          done
          for dependency in ${(nativeInstructionModuleText pkgs).drvPath} ${(nativeInstructionModuleControls pkgs).drvPath}; do
            grep -Fx "$dependency" ${nativeInstructionModuleBuildClosure pkgs}/store-paths
          done
          # Inspect actual scripts/selected paths, not invented cohort metadata.
          for artifact in ${nativeInstructionModuleText pkgs}/instruction-module-text.json \
            ${nativeInstructionModuleControls pkgs}/first.json ${nativeInstructionModuleControls pkgs}/step.json; do
            grep -F "$artifact" ${(nativeInstructionModule pkgs).drvPath}
            grep -F "$artifact" ${(nativeInstructionLoweringPolicyVerification pkgs).drvPath}
          done
          grep -F '${nativeInstructionModule pkgs}/module.json' ${(nativeInstructionLoweringPolicyVerification pkgs).drvPath}
          test "$(find ${instructionModuleSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          compiler=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          for tool in "$exporter" "$oracle" "$compiler" "$driver" "$jq"; do
            test -f "$tool" && test -r "$tool" && test -x "$tool"
            case "$tool" in /*) ;; *) exit 1 ;; esac
          done
          cue() { printf '%s\n' -- invocation "$@" >> actual-arguments; env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          query() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$@"; }
          json() { query -S .; }
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
          place() { cue import -f -p bootstrap -l "$1:" -o "$3" "$2"; }
          bounded() { for file in "$@"; do test "$(wc -c < "$file")" -le 65536; done; }
          pair() {
            bounded "$1" "$2"
            test "$(($(wc -c < "$1") + $(wc -c < "$2")))" -le 65536
            for order in forward reverse; do
              for repeat in first second; do
                inputs=("$1" "$2")
                if test "$order" = reverse; then inputs=("$2" "$1"); fi
                native "''${inputs[@]}" > "$3-$order-$repeat.json"
                bounded "$3-$order-$repeat.json"
                cmp "$3-forward-first.json" "$3-$order-$repeat.json"
              done
            done
          }
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          shape=${../../bootstrap/instruction-module-schema.cue}
          root=${instructionModuleSources pkgs}/instruction-module-export.cue
          ${instructionCoreConsumer}
          ${instructionModuleConsumer}
          ${instructionModuleInverse}
          restore_module "$bridge" original-bridge.cue
          test "$(sha256sum original-bridge.cue | cut -d ' ' -f 1)" = 4ee28528b98d8a47d9c39566f18457d4b3ef77b22428a180df84764071db6582
          ${instructionCanonicalConsumer pkgs (specSources pkgs true)}
          # The same actual raw consumer constructs arrays only AFTER selection.
          policy() {
            local moduleProviders=("$text" "$first" "$step")
            local rightProvider="$PWD/instruction-right-field.cue"
            local operand="$PWD/instruction-right-operand.cue"
            local group="$PWD/instruction-operand-cases.cue"
            local runtime="$PWD/instruction-binary-runtime.cue"
            policy_gates || return
            local inputs=("$schema" "$bridge" "$core" "$rightProvider" "$operand" "$group" "$runtime"
              "$PWD/instruction-binary-tables.cue" ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json
              "$module" "''${moduleProviders[@]}" "$context")
            if test "$1" = reverse; then
              local reversed=(); local input
              for input in "''${inputs[@]}"; do reversed=("$input" "''${reversed[@]}"); done
              inputs=("''${reversed[@]}")
            fi
            touch full-export-started
            cue export "''${inputs[@]}"
          }
          for provider in RightField RightOperand OperandCases BinaryRuntime BinaryTables; do
            case "$provider" in
              RightField) data=${nativeInstructionRightField pkgs}/right-field.json; name=right-field ;;
              RightOperand) data=${nativeInstructionRightOperand pkgs}/right-operand.json; name=right-operand ;;
              OperandCases) data=${nativeInstructionOperandCases pkgs}/operand-cases.json; name=operand-cases ;;
              BinaryRuntime) data=${nativeInstructionBinaryRuntime pkgs}/binary-runtime.json; name=binary-runtime ;;
              BinaryTables) data=${nativeInstructionBinaryTables pkgs}/instruction-binary-tables.json; name=binary-tables ;;
            esac
            place "#Instruction$provider" "$data" "instruction-$name.cue"
          done
          place '#InstructionModule' ${nativeInstructionModule pkgs}/module.json instruction-module.cue
          place '#InstructionModuleText' ${nativeInstructionModuleText pkgs}/instruction-module-text.json instruction-module-text.cue
          place '#InstructionModuleFirst' ${nativeInstructionModuleControls pkgs}/first.json instruction-module-first.cue
          place '#InstructionModuleStep' ${nativeInstructionModuleControls pkgs}/step.json instruction-module-step.cue
          # Complete payload prediction, authored independently of producer outputs.
          cat > expected-header <<'HEADER'
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
          let mut _context = runtime::EvaluationContext::default();
          runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
          let mut _context = runtime::EvaluationContext::default();
          runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
          _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
          match runtime::kind(node)? {
          HEADER
          printf 'kind => Err(Error::MissingRule { kind: kind.to_string() }),\n}\n}\n' > expected-footer
          expected_module() {
            query -n --rawfile h "$variant/header" --rawfile f "$variant/footer" \
              --arg first "$initial" --arg step "$advance" --arg separator "$separator" --arg terminator "$terminator" --arg order "$bodyOrder" '
              def text($s):{op:"text",value:$s};
              {module:[text($h),{op:$first},{op:"whileRules",body:[
                {op:(if $order=="ordinary" then "kind" else "expression" end)},text($separator),
                {op:(if $order=="ordinary" then "expression" else "kind" end)},text($terminator),{op:$step}]},text($f)]}' | json
          }
          expected_rust() {
            cat "$variant/header"
            for key in $keys; do
              case "$key" in a) expression='Ok::<_, Error>(runtime::Value::Int)' ;; b) expression='Ok::<_, Error>(runtime::Value::Bool)' ;; esac
              if test "$bodyOrder" = ordinary; then printf '"%s"%s%s%s' "$key" "$separator" "$expression" "$terminator"
              else printf '%s%s"%s"%s' "$expression" "$separator" "$key" "$terminator"; fi
            done
            cat "$variant/footer"
          }
          materialize() {
            pair "$variant/text.cue" ${../../bootstrap/instruction-module-text-export.cue} "$variant/text"
            pair "$variant/first.cue" ${../../bootstrap/instruction-module-first-export.cue} "$variant/first"
            pair "$variant/step.cue" ${../../bootstrap/instruction-module-step-export.cue} "$variant/step"
            cp "$root" "$variant/adapter.cue"
            chmod u+w "$variant/adapter.cue"
            for provider in Text First Step; do
              case "$provider" in Text) data=text ;; First) data=first ;; Step) data=step ;; esac
              bounded "$variant/$data-forward-first.json"
              printf '\n#InstructionModule%s: ' "$provider" >> "$variant/adapter.cue"
              cat "$variant/$data-forward-first.json" >> "$variant/adapter.cue"
            done
            pair "$variant/owner.cue" "$variant/adapter.cue" "$variant/module"
            expected_module > "$variant/expected-module.json"
            json < "$variant/module-forward-first.json" > "$variant/sorted-module.json"
            cmp "$variant/expected-module.json" "$variant/sorted-module.json"
            for order in forward reverse; do
              inputs=("$variant/owner.cue" "$variant/adapter.cue")
              if test "$order" = reverse; then inputs=("$variant/adapter.cue" "$variant/owner.cue"); fi
              cue export "''${inputs[@]}" | json > "$variant/go-$order.json"
              cmp "$variant/expected-module.json" "$variant/go-$order.json"
            done
          }
          printf '%s\n' '{"formatVersion":1,"rules":{"b":{"result":{"op":"bool"}},"a":{"result":{"op":"int"}}}}' > tiny.json
          for variant in canonical header footer separator terminator body-order first-priority first-literal step-priority reverse; do
            mkdir "$variant"
            cp ${../../bootstrap/instruction-module-text.cue} "$variant/text.cue"
            cp ${../../bootstrap/instruction-module-first.cue} "$variant/first.cue"
            cp ${../../bootstrap/instruction-module-step.cue} "$variant/step.cue"
            cp ${instructionModuleSources pkgs}/instruction-module.cue "$variant/owner.cue"
            cp expected-header "$variant/header"; cp expected-footer "$variant/footer"
            initial=firstRule; advance=nextRule; separator=' => '; terminator=$',\n'; bodyOrder=ordinary; keys='a b'
            case "$variant" in
              header)
                substituteInPlace "$variant/text.cue" --replace-fail '// Generated by bootstrap-codegen. Do not edit by hand.' '// Independent header effect.'
                substituteInPlace "$variant/header" --replace-fail '// Generated by bootstrap-codegen. Do not edit by hand.' '// Independent header effect.' ;;
              footer)
                substituteInPlace "$variant/text.cue" --replace-fail 'footer: "kind' 'footer: "// Independent footer effect.\nkind'
                printf '// Independent footer effect.\n' > "$variant/footer"
                cat expected-footer >> "$variant/footer" ;;
              separator)
                substituteInPlace "$variant/owner.cue" --replace-fail 'value: " => "' 'value: " =>  "'; separator=' =>  ' ;;
              terminator)
                substituteInPlace "$variant/owner.cue" --replace-fail 'value: ",\n"' 'value: ", \n"'; terminator=$', \n' ;;
              body-order)
                substituteInPlace "$variant/owner.cue" --replace-fail '{op: "kind"}' '{op: "temporaryKind"}' --replace-fail '{op: "expression"}' '{op: "kind"}' --replace-fail '{op: "temporaryKind"}' '{op: "expression"}'
                test "$(grep -o 'op: "kind"' "$variant/owner.cue" | wc -l)" -eq 1
                test "$(grep -o 'op: "expression"' "$variant/owner.cue" | wc -l)" -eq 1
                bodyOrder=swapped ;;
              first-priority|reverse)
                substituteInPlace "$variant/first.cue" --replace-fail '*"firstRule" | "lastRule"' '"firstRule" | *"lastRule"'
                initial=lastRule; keys=b ;;
              first-literal)
                substituteInPlace "$variant/first.cue" --replace-fail '*"firstRule" | "lastRule"' '*"lastRule" | "lastRule"'
                initial=lastRule; keys=b ;;
              step-priority)
                substituteInPlace "$variant/step.cue" --replace-fail '*"nextRule" | "previousRule"' '"nextRule" | *"previousRule"'
                advance=previousRule; keys=a ;;
            esac
            if test "$variant" = reverse; then
              substituteInPlace "$variant/step.cue" --replace-fail '*"nextRule" | "previousRule"' '"nextRule" | *"previousRule"'
              advance=previousRule; keys='b a'
            fi
            materialize
            # Raw source and whole independently native cohort are chosen BEFORE arrays.
            module="$variant/owner.cue"; text="$variant/text.cue"; first="$variant/first.cue"; step="$variant/step.cue"
            context=${../../bootstrap/lowering.cue}
            policy forward | json > "$variant/raw-policy.json"
            for provider in Module ModuleText ModuleFirst ModuleStep; do
              case "$provider" in Module) data=module ;; ModuleText) data=text ;; ModuleFirst) data=first ;; ModuleStep) data=step ;; esac
              place "#Instruction$provider" "$variant/$data-forward-first.json" "$variant/placed-$data.cue"
            done
            module="$variant/placed-module.cue"; text="$variant/placed-text.cue"; first="$variant/placed-first.cue"; step="$variant/placed-step.cue"
            policy reverse | json > "$variant/native-policy.json"
            query --slurpfile m "$variant/expected-module.json" '.module=$m[0].module' canonical/raw-policy.json | json > "$variant/expected-policy.json"
            cmp "$variant/expected-policy.json" "$variant/raw-policy.json"
            cmp "$variant/expected-policy.json" "$variant/native-policy.json"
            expected_rust > "$variant/expected.rs"
            for mode in raw native; do
              env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json "$variant/$mode-policy.json" > "$variant/$mode.rs"
              cmp "$variant/expected.rs" "$variant/$mode.rs"
            done
            if test "$variant" != canonical; then
              if cmp -s "$variant/expected.rs" canonical/expected.rs; then echo "unobservable $variant" >&2; exit 1; fi
            fi
          done
          cmp canonical/module-forward-first.json ${nativeInstructionModule pkgs}/module.json
          cmp canonical/text-forward-first.json ${nativeInstructionModuleText pkgs}/instruction-module-text.json
          cmp canonical/first-forward-first.json ${nativeInstructionModuleControls pkgs}/first.json
          cmp canonical/step-forward-first.json ${nativeInstructionModuleControls pkgs}/step.json
          json < ${jsonExports pkgs}/lowering.json > published-policy.json
          cmp published-policy.json canonical/native-policy.json
          # Arbitrary helper/root names and unrelated filenames remain ordinary inputs.
          substitute canonical/owner.cue arbitrary.cue --replace-fail '#InstructionModuleText' '#Words' --replace-fail '#InstructionModuleFirst' '#Begin' --replace-fail '#InstructionModuleStep' '#Move' --replace-fail '#InstructionModule:' '#Program:'
          substitute canonical/adapter.cue unrelated.cue --replace-fail '#InstructionModuleText' '#Words' --replace-fail '#InstructionModuleFirst' '#Begin' --replace-fail '#InstructionModuleStep' '#Move' --replace-fail $'#InstructionModule\n' $'#Program\n'
          pair arbitrary.cue unrelated.cue renamed
          cmp renamed-forward-first.json canonical/module-forward-first.json
          # The genuine zero-argument canonical boundary ignores inherited providers.
          module=absent.cue; text=absent.cue; first=absent.cue; step=absent.cue
          moduleProviders=(absent.cue); rightProvider=absent.cue; context=absent.cue; coreShape=absent.cue
          : > actual-arguments; rm -f full-export-started
          status=0
          export_instruction unexpected > extra.json 2> extra.err || status=$?
          test "$status" -eq 2 && test ! -s extra.json && test ! -s actual-arguments && test ! -e full-export-started
          export_instruction > closed-policy.json
          json < closed-policy.json > closed-sorted.json
          cmp closed-sorted.json published-policy.json
          # Assert EVERY actual argument of all gates and the full invocation.
          expected_call() { printf '%s\n' -- invocation "$@" >> expected-arguments; }
          : > expected-arguments
          for helper in InstructionModuleText InstructionModuleFirst InstructionModuleStep InstructionModule; do
            expected_call export "$shape" "$PWD/instruction-module.cue" "$PWD/instruction-module-text.cue" "$PWD/instruction-module-first.cue" "$PWD/instruction-module-step.cue" -e "#$helper"
          done
          expected_call export ${../../bootstrap/instruction-core-cases-schema.cue} ${nativeInstructionCoreCases pkgs}/instruction-core-cases.json
          sources=${specSources pkgs true}
          expected_call export "$sources/schema.cue" "$sources/lowering-fragments.cue" "$PWD/instruction-right-field.cue" -e '#InstructionRightField'
          expected_call export "$sources/schema.cue" "$sources/lowering-fragments.cue" "$PWD/instruction-right-field.cue" "$PWD/instruction-right-operand.cue" -e '#InstructionRightOperand'
          expected_call export "$sources/schema.cue" "$sources/lowering-fragments.cue" "$PWD/instruction-right-field.cue" "$PWD/instruction-right-operand.cue" "$PWD/instruction-operand-cases.cue" -e '#InstructionOperandCases'
          expected_call export "$sources/schema.cue" "$sources/lowering-fragments.cue" "$PWD/instruction-binary-tables.cue" -e '#InstructionBinaryTables'
          expected_call export "$sources/schema.cue" "$sources/lowering-fragments.cue" "$PWD/instruction-right-field.cue" "$PWD/instruction-right-operand.cue" "$PWD/instruction-operand-cases.cue" "$PWD/instruction-binary-tables.cue" "$PWD/instruction-binary-runtime.cue" -e '#InstructionBinaryRuntime'
          expected_call export "$sources/schema.cue" "$sources/lowering-fragments.cue" ${nativeInstructionCoreCases pkgs}/instruction-core-cases.json "$PWD/instruction-right-field.cue" "$PWD/instruction-right-operand.cue" "$PWD/instruction-operand-cases.cue" "$PWD/instruction-binary-runtime.cue" "$PWD/instruction-binary-tables.cue" ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json "$PWD/instruction-module.cue" "$PWD/instruction-module-text.cue" "$PWD/instruction-module-first.cue" "$PWD/instruction-module-step.cue" "$sources/lowering.cue"
          cmp expected-arguments actual-arguments
          coreShape=${../../bootstrap/instruction-core-cases-schema.cue}
          module=canonical/placed-module.cue; text=canonical/placed-text.cue; first=canonical/placed-first.cue; step=canonical/placed-step.cue
          context=${../../bootstrap/lowering.cue}
          consume_module() { policy forward; }
          ${instructionModuleConsumerProbe}
          # Missing upstream members cannot hide behind an already selected Module.
          for slot in Text:header Text:footer First:op Step:op; do
            case "$slot" in Text:*) data=text ;; First:*) data=first ;; Step:*) data=step ;; esac
            query --arg field "''${slot#*:}" 'del(.[$field])' "canonical/$data-forward-first.json" > missing-member.json
            place "#InstructionModule''${slot%%:*}" missing-member.json missing-member.cue
            savedText=$text; savedFirst=$first; savedStep=$step
            case "$data" in text) text=missing-member.cue ;; first) first=missing-member.cue ;; step) step=missing-member.cue ;; esac
            rm -f full-export-started
            if policy forward > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err && test ! -e full-export-started
            text=$savedText; first=$savedFirst; step=$savedStep
          done
          # Selected Module CANNOT hide absent or incomplete helpers at the quad gate.
          query -c 'paths(scalars)' canonical/expected-module.json > slots
          test "$(wc -l < slots)" -eq 13
          while read -r slot; do
            for mutation in 'delpaths([$slot])' 'setpath($slot;42)'; do
              query --argjson slot "$slot" "$mutation" canonical/expected-module.json > malformed.json
              place '#InstructionModule' malformed.json malformed.cue
              module=malformed.cue; rm -f full-export-started
              if policy forward > failed.json 2> failed.err; then exit 1; fi
              test ! -s failed.json && test -s failed.err && test ! -e full-export-started
            done
          done < slots
          query -c 'paths(objects)' canonical/expected-module.json > objects
          printf '%s\n' '[]' >> objects
          while read -r path; do
            query --argjson path "$path" 'setpath($path+["extra"];true)' canonical/expected-module.json > malformed.json
            place '#InstructionModule' malformed.json malformed.cue
            module=malformed.cue
            if policy forward > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          done < objects
          query -c 'paths(arrays)' canonical/expected-module.json > arrays
          test "$(wc -l < arrays)" -eq 2
          while read -r path; do
            for mutation in '.[0:-1]' '.+[.[0]]'; do
              query --argjson path "$path" "getpath(\$path)|$mutation" canonical/expected-module.json > array.json
              query --argjson path "$path" --slurpfile value array.json 'setpath($path;$value[0])' canonical/expected-module.json > malformed.json
              place '#InstructionModule' malformed.json malformed.cue
              module=malformed.cue
              if policy forward > failed.json 2> failed.err; then exit 1; fi
              test ! -s failed.json && test -s failed.err
            done
          done < arrays
          # Real full-schema backfill versus isolated failure, raw from the outset.
          query 'del(.module[0].op)' canonical/expected-module.json > missing-op.json
          place '#InstructionModule' missing-op.json missing-op.cue
          module=missing-op.cue
          if module_gate "$text" "$first" "$step" > failed.json 2> failed.err; then exit 1; fi
          cue export "$schema" "$bridge" "$core" instruction-right-field.cue instruction-right-operand.cue instruction-operand-cases.cue instruction-binary-runtime.cue instruction-binary-tables.cue ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json "$module" "$text" "$first" "$step" "$context" > backfilled.json
          query -e '.module[0].op=="text"' backfilled.json
          # A direct-Go hybrid still passes type shape: never call this coherence.
          module=canonical/placed-module.cue
          for provider in text first step; do
            savedText=$text; savedFirst=$first; savedStep=$step
            case "$provider" in text) text=header/placed-text.cue ;; first) first=first-priority/placed-first.cue ;; step) step=step-priority/placed-step.cue ;; esac
            module_gate "$text" "$first" "$step"
            text=$savedText; first=$savedFirst; step=$savedStep
          done
          # Raw open mode observes controls; returning to the closed call restores ALL.
          module=${../../bootstrap/instruction-module.cue}; text=${../../bootstrap/instruction-module-text.cue}
          first=${../../bootstrap/instruction-module-first.cue}; step=${../../bootstrap/instruction-module-step.cue}
          for state in first-next last-previous last-next first-previous; do
            case "$state" in first-next) initial=firstRule; advance=nextRule ;; last-previous) initial=lastRule; advance=previousRule ;; last-next) initial=lastRule; advance=nextRule ;; first-previous) initial=firstRule; advance=previousRule ;; esac
            printf '%s\n' 'package bootstrap' "#InstructionLowering & {module:[_,{op:\"$initial\"},{body:[_,_,_,_,{op:\"$advance\"}]},_]}" > context.cue
            context=context.cue
            policy reverse | json > "$state-policy.json"
            case "$state" in first-next) reference=canonical ;; last-previous) reference=reverse ;; last-next) reference=first-priority ;; first-previous) reference=step-priority ;; esac
            cmp "$state-policy.json" "$reference/native-policy.json"
            env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json "$state-policy.json" > "$state.rs"
            cmp "$state.rs" "$reference/expected.rs"
          done
          export_instruction | json > recovered-canonical.json
          cmp recovered-canonical.json published-policy.json
          # Inspection is NEVER a retry: no Module EXPORT gate and no full EXPORT.
          context=${../../bootstrap/lowering.cue}
          for missing in Text First Step; do
            providers=("$text" "$first" "$step")
            case "$missing" in Text) providers=("$first" "$step") ;; First) providers=("$text" "$step") ;; Step) providers=("$text" "$first") ;; esac
            cue eval "$schema" "$bridge" "$core" instruction-right-field.cue instruction-right-operand.cue instruction-operand-cases.cue instruction-binary-runtime.cue instruction-binary-tables.cue ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json "$module" "''${providers[@]}" "$context" > /dev/null
            if cue export "$schema" "$bridge" "$core" instruction-right-field.cue instruction-right-operand.cue instruction-operand-cases.cue instruction-binary-runtime.cue instruction-binary-tables.cue ${nativeInstructionUnaryTables pkgs}/instruction-unary-tables.json "$module" "''${providers[@]}" "$context" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          done
          # Shape accepts strings, full schema rejects unknown opcodes.
          query '.module[1].op="unknownModuleOpcode"' canonical/expected-module.json > malformed.json
          place '#InstructionModule' malformed.json malformed.cue
          module=malformed.cue
          module_gate "$text" "$first" "$step"
          if policy forward > failed.json 2> failed.err; then exit 1; fi
          test ! -s failed.json && test -s failed.err
          # VERIFIER ONLY: oversize text is Go-accepted, not native publication.
          query '.module[0].value=("x"*65537)' canonical/expected-module.json > oversize.json
          place '#InstructionModule' oversize.json oversize.cue
          module=oversize.cue
          policy forward > oversize-policy.json
          status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$driver" oversize-policy.json > oversized.rs 2> oversized.err || status=$?
          test "$status" -eq 2 && test ! -s oversized.rs && test -s oversized.err
          grep -F 'module text exceeds 64 KiB' oversized.err
          # Go/typed-valid cursor actions still fail in actual assembly.
          query '.module[1].op="nextRule"' canonical/expected-module.json > protocol.json
          place '#InstructionModule' protocol.json protocol.cue
          module=protocol.cue
          policy forward > protocol-policy.json
          status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json protocol-policy.json > protocol.rs 2> protocol.err || status=$?
          test "$status" -eq 2 && test ! -s protocol.rs
          grep -F 'no current rule' protocol.err
          export_instruction > recovered-policy.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$driver" recovered-policy.json > recovered-driver.rs
          env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json recovered-policy.json > recovered.rs
          cmp recovered.rs canonical/expected.rs
          cp -R . "$out/evidence"
          rm -rf "$out/evidence/home"
          cp ${nativeInstructionModuleBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionModuleBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionModule pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionModuleTextSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/instruction-module-text.cue
            ../../bootstrap/instruction-module-text-export.cue
          ];
        };
      nativeInstructionModuleText =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-module-text" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          helper=${instructionModuleTextSources pkgs}/instruction-module-text.cue
          selector=${instructionModuleTextSources pkgs}/instruction-module-text-export.cue
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$helper" "$selector" > first.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$selector" "$helper" > second.json
          for run in first second; do test "$(wc -c < "$run.json")" -le 65536; done
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/instruction-module-text.json"
        '';
      nativeInstructionModuleTextBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionModuleText pkgs).drvPath
          ];
        };
      nativeInstructionModuleTextVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-module-text-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionModuleTextBuildClosure pkgs}/store-paths; then
              echo "forbidden module text construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionModuleTextSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$jq" "$rustfmt"; do
            test -f "$binary" && test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          cp ${instructionModuleTextSources pkgs}/*.cue .
          helper=instruction-module-text.cue
          textSelector=instruction-module-text-export.cue
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          unary=${../../bootstrap/instruction-unary-tables.cue}
          firstOwner=${../../bootstrap/instruction-module-first.cue}
          stepOwner=${../../bootstrap/instruction-module-step.cue}
          selector=${../../bootstrap/lowering.cue}
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$1" "$textSelector"; }
          place() { cue import -f -p bootstrap -l '#InstructionModuleText:' -o placed.cue "$1"; }
          cue export ${../../bootstrap/native-policy-package.cue} ${../../bootstrap/instruction-binary-tables.cue} > binary.json
          cue import -p bootstrap -l '#InstructionBinaryTables:' -o binary-placed.cue binary.json
          binary_gate() { cue export "$schema" "$bridge" binary-placed.cue -e '#InstructionBinaryTables' > /dev/null; }
          right=${../../bootstrap/instruction-right-field.cue}
          ${instructionOperandConsumer}
          ${instructionCoreConsumer}
          binary_export() { core_gate || return; right_gate || return; binary_gate || return; runtime_gate binary-placed.cue || return; module_gate "''${textProviders[@]}" "$firstOwner" "$stepOwner" || return; touch full-export-started; cue export "$module" "$core" "$right" "$operand" "$group" "$runtime" "$@" binary-placed.cue; }
          raw() { local textProviders=("$helper"); binary_export "$schema" "$bridge" "$unary" "$firstOwner" "$stepOwner" "$helper" "$@"; }
          complete() { local textProviders=(placed.cue); binary_export "$schema" "$bridge" "$unary" "$firstOwner" "$stepOwner" placed.cue "$@"; }
          native "$helper" > text.json
          cmp text.json ${nativeInstructionModuleText pkgs}/instruction-module-text.json
          cue export "$helper" "$textSelector" | json > oracle.json
          cmp text.json oracle.json
          place text.json
          consume_right() { complete right-backfill.cue; }
          ${rightFieldConsumerProbe}
          consume_operand() { consume_right; }
          ${instructionOperandConsumerProbe}
          consume_core() { complete "$selector"; }
          ${instructionCoreConsumerProbe pkgs}
          ${instructionRuntimeConsumerProbe}
          if grep -F '#InstructionModuleText' "$schema"; then
            echo 'instruction module text collides with the existing schema' >&2; exit 1
          fi
          for context in lowering tests/valid/lowering-left-twice tests/valid/lowering-reverse; do
            case "$context" in lowering) source="$selector" ;; *) source=${instructionOverrideFixtures pkgs}/"$(basename "$context").cue" ;; esac
            name=$(basename "$context")
            raw "$source" | json > "$name-raw.json"
            complete "$source" | json > "$name.json"
            textProviders=(placed.cue)
            binary_export "$source" placed.cue "$stepOwner" "$firstOwner" "$unary" "$bridge" "$schema" | json > "$name-reversed.json"
            cmp "$name-raw.json" "$name.json"
            cmp "$name-reversed.json" "$name.json"
            cmp "$name.json" ${jsonExports pkgs}/"$context.json"
          done
          generate ${jsonExports pkgs}/evaluation.json lowering.json > evaluation.rs
          cmp evaluation.rs ${../../bootstrap/generated/evaluation.rs}
          for control in left-twice reverse; do
            generate ${jsonExports pkgs}/evaluation.json "lowering-$control.json" > "evaluation-$control.rs"
            cmp "evaluation-$control.rs" ${../../bootstrap/generated}/evaluation-"$control.rs"
          done
          # Missing slots are readiness failures, not schema conflicts. A header
          # operand of + reports non-concreteness rather than 'incomplete value'.
          not_ready() {
            local textProviders=("$@")
            core_gate || return
            binary_gate
            right_gate
            runtime_gate binary-placed.cue || return
            cue eval "$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$runtime" "$unary" "$firstOwner" "$stepOwner" binary-placed.cue "$@" "$selector" > /dev/null
            if binary_export "$schema" "$bridge" "$unary" "$firstOwner" "$stepOwner" "$@" "$selector" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
            grep -E 'non-concrete value string in operand to \+|incomplete value string' failed.err
          }
          not_ready
          printf '%s\n' 'package bootstrap' '#InstructionModuleText: {header:string,footer:string}' > shape.cue
          not_ready shape.cue
          for slot in header footer; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" --arg slot "$slot" 'del(.[$slot])' text.json > missing.json
            place missing.json
            not_ready placed.cue
          done
          for invalid in '.header=1' '.footer=false' '.extra="unexpected"'; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$invalid" text.json > invalid.json
            place invalid.json
            core_gate
            binary_gate
            right_gate
            runtime_gate binary-placed.cue
            if cue eval "$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$runtime" "$unary" "$firstOwner" "$stepOwner" binary-placed.cue placed.cue "$selector" > failed.eval 2> failed.err; then exit 1; fi
            if complete "$selector" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
            grep -E 'conflicting values|field not allowed' failed.err
          done
          substitute "$textSelector" unknown.cue --replace-fail '#InstructionModuleText' '#MissingInstructionModuleText'
          if env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$helper" unknown.cue > failed.json 2> failed.err; then exit 1; fi
          test ! -s failed.json && test -s failed.err
          grep -F 'unknown' failed.err
          printf '%s\n' '{"formatVersion":1,"rules":{"tiny":{"result":{"op":"int"}}}}' > tiny.json
          # Source-only changes must reach a COMPLETE independently predicted
          # evaluator. This handwritten prediction is never constructor input.
          for variant in header footer; do
            headerComment='// Generated by bootstrap-codegen. Do not edit by hand.'
            footerComment=
            case "$variant" in
              header)
                headerComment='// Module text header variant.'
                substitute "$helper" variant.cue --replace-fail '// Generated by bootstrap-codegen. Do not edit by hand.' "$headerComment" ;;
              footer)
                footerComment=$'// Module text footer variant.\n'
                substitute "$helper" variant.cue --replace-fail '"kind => Err' '"// Module text footer variant.\nkind => Err' ;;
            esac
            native variant.cue > variant.json
            if cmp -s variant.json text.json; then exit 1; fi
            place variant.json
            complete "$selector" | json > variant-policy.json
            textProviders=(variant.cue)
            binary_export "$schema" "$bridge" "$unary" "$firstOwner" "$stepOwner" variant.cue "$selector" | json > variant-raw.json
            cmp variant-policy.json variant-raw.json
            generate tiny.json variant-policy.json > "$variant.rs"
            format > "$variant-expected.rs" <<RUST
          $headerComment
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              match runtime::kind(node)? {
                  "tiny" => Ok::<_, Error>(runtime::Value::Int),
                  $footerComment kind => Err(Error::MissingRule { kind: kind.to_string() }),
              }
          }
          RUST
            cmp "$variant.rs" "$variant-expected.rs"
          done
          cp text.json lowering*.json evaluation*.rs header*.rs footer*.rs "$out/"
          cp ${nativeInstructionModuleTextBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionModuleTextBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionModuleText pkgs).drvPath} > "$out/native-build-roots"
        '';
      sliceLowerSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/slice-lower-default.cue
            ../../bootstrap/slice-lower-default-root.cue
          ];
        };
      nativeSliceLower =
        pkgs:
        pkgs.runCommand "cuestacean-native-slice-lower" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          owner=${sliceLowerSources pkgs}/slice-lower-default.cue
          root=${sliceLowerSources pkgs}/slice-lower-default-root.cue
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$owner" "$root" > first.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$root" "$owner" > second.json
          test "$(wc -c < first.json)" -le 65536
          test "$(wc -c < second.json)" -le 65536
          cmp first.json second.json
          cp first.json "$out/slice-lower.json"
        '';
      nativeSliceLowerBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeSliceLower pkgs).drvPath
          ];
        };
      sliceLowerVerificationSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/schema.cue
            ../../bootstrap/slice-control-fragments.cue
            ../../bootstrap/slice-lower-default.cue
            ../../bootstrap/slice-lower-default-root.cue
            ../../bootstrap/slice-control.cue
            ../../bootstrap/tests/valid/slice-control-lower-one.cue
            ../../bootstrap/tests/valid/slice-control-tail.cue
            ../../bootstrap/generated/slice-control.rs
            ../../bootstrap/generated/slice-control-lower-one.rs
            ../../bootstrap/generated/slice-control-tail.rs
          ];
        };
      nativeSliceLowerVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-slice-lower-verification" { } ''
          set -euo pipefail
          closure=${nativeSliceLowerBuildClosure pkgs}
          sources=${sliceLowerSources pkgs}
          exporterSources=${specSources pkgs false}
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath} \
            ${nativeCodegenCore pkgs} ${(nativeCodegenCore pkgs).drvPath} \
            ${nativeScanCodegen pkgs} ${(nativeScanCodegen pkgs).drvPath} \
            ${nativeUnifyCodegen pkgs} ${(nativeUnifyCodegen pkgs).drvPath} \
            ${nativePrattCodegen pkgs} ${(nativePrattCodegen pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${../../bootstrap/generated/slice-control.rs} \
            ${../../bootstrap/generated/slice-control-lower-one.rs} \
            ${../../bootstrap/generated/slice-control-tail.rs} \
            ${./slice-lower-provider.sh} ${./test-slice-lower.sh} \
            ${sliceLowerVerificationSources pkgs}; do
            if grep -Fx "$forbidden" "$closure/store-paths"; then
              echo "forbidden slice lower construction dependency: $forbidden" >&2
              exit 1
            fi
          done
          for required in ${(specExporter pkgs).drvPath} ${(nativeSliceLower pkgs).drvPath} \
            "$sources" "$exporterSources"; do
            grep -Fx "$required" "$closure/store-paths"
          done
          grep -F "$sources" ${(nativeSliceLower pkgs).drvPath}
          grep -F '${specExporter pkgs}/bin/cuestacean-spec-export' ${(nativeSliceLower pkgs).drvPath}
          (cd "$sources"; find . -type f | LC_ALL=C sort) > source-inventory
          printf '%s\n' ./slice-lower-default-root.cue ./slice-lower-default.cue > expected-inventory
          cmp source-inventory expected-inventory
          cmp "$sources/slice-lower-default.cue" ${../../bootstrap/slice-lower-default.cue}
          cmp "$sources/slice-lower-default-root.cue" ${../../bootstrap/slice-lower-default-root.cue}
          (cd "$exporterSources"; find . -type f | LC_ALL=C sort) > exporter-inventory
          printf '%s\n' ./spec-export/Cargo.lock ./spec-export/Cargo.toml \
            ./spec-export/src/bindings.rs ./spec-export/src/comprehension.rs \
            ./spec-export/src/eval.rs ./spec-export/src/lib.rs ./spec-export/src/main.rs \
            ./spec-export/src/model.rs ./spec-export/src/parser.rs ./spec-export/src/projection.rs \
            > expected-exporter-inventory
          cmp exporter-inventory expected-exporter-inventory
          test "$(find ${nativeSliceLower pkgs} -type f | wc -l)" -eq 1
          mkdir -p "$out"
          ${pkgs.bash}/bin/bash ${./test-slice-lower.sh} \
            ${pkgs.cue}/bin/cue ${pkgs.jq}/bin/jq \
            ${specExporter pkgs}/bin/cuestacean-spec-export \
            ${nativePrattCodegen pkgs}/bin/bootstrap-pratt-codegen \
            ${pkgs.rustfmt}/bin/rustfmt ${nativeSliceLower pkgs}/slice-lower.json \
            ${jsonExports pkgs} ${sliceLowerVerificationSources pkgs} \
            ${./slice-lower-provider.sh} "$out"
          cp "$closure/store-paths" "$out/native-build-paths"
          cp "$closure/registration" "$out/native-build-registration"
          cp source-inventory exporter-inventory "$out/"
          cp -R "$sources" "$out/native-sources"
          cp -R "$exporterSources" "$out/exporter-sources"
          cp ${(nativeSliceLower pkgs).drvPath} "$out/native-producer.drv"
          cp ${(specExporter pkgs).drvPath} "$out/native-exporter.drv"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeSliceLower pkgs).drvPath} \
            > "$out/native-build-roots"
        '';
      # Each instantiation has its OWN two-file source cut and recursive closure.
      primaryMinimumSources =
        pkgs: kind:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            (../../bootstrap + "/${kind}-minimum-input.cue")
            (../../bootstrap + "/${kind}-minimum-input-root.cue")
          ];
        };
      nativePrimaryMinimum =
        pkgs: kind:
        pkgs.runCommand "cuestacean-native-${kind}-minimum" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          owner=${primaryMinimumSources pkgs kind}/${kind}-minimum-input.cue
          root=${primaryMinimumSources pkgs kind}/${kind}-minimum-input-root.cue
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$owner" "$root" > first.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$root" "$owner" > second.json
          test "$(wc -c < first.json)" -le 65536
          test "$(wc -c < second.json)" -le 65536
          cmp first.json second.json
          cp first.json "$out/${kind}-minimum.json"
        '';
      primaryMinimumBuildClosure =
        pkgs: kind:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativePrimaryMinimum pkgs kind).drvPath
          ];
        };
      primaryMinimaVerificationSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/schema.cue
            ../../bootstrap/record-primary-fragments.cue
            ../../bootstrap/conjunction-primary-fragments.cue
            ../../bootstrap/record-minimum-input.cue
            ../../bootstrap/record-minimum-input-root.cue
            ../../bootstrap/conjunction-minimum-input.cue
            ../../bootstrap/conjunction-minimum-input-root.cue
            ../../bootstrap/record-primary.cue
            ../../bootstrap/conjunction-primary.cue
            ../../bootstrap/tests/valid/record-primary-high-minimum.cue
            ../../bootstrap/tests/valid/conjunction-primary-high-minimum.cue
            ../../bootstrap/generated/record-primary.rs
            ../../bootstrap/generated/record-primary-high-minimum.rs
            ../../bootstrap/generated/conjunction-primary.rs
            ../../bootstrap/generated/conjunction-primary-high-minimum.rs
          ];
        };
      primaryMinimumProvenance = pkgs: kind: sibling: ''
        (
          set -euo pipefail
          closure=${primaryMinimumBuildClosure pkgs kind}
          sources=${primaryMinimumSources pkgs kind}
          exporterSources=${specSources pkgs false}
          producer=${nativePrimaryMinimum pkgs kind}
          mkdir -p "$out/provenance/${kind}"
          cd "$out/provenance/${kind}"
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath} \
            ${nativeCodegenCore pkgs} ${(nativeCodegenCore pkgs).drvPath} \
            ${nativeScanCodegen pkgs} ${(nativeScanCodegen pkgs).drvPath} \
            ${nativeUnifyCodegen pkgs} ${(nativeUnifyCodegen pkgs).drvPath} \
            ${nativePrattCodegen pkgs} ${(nativePrattCodegen pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${primaryMinimumSources pkgs sibling} \
            ${nativePrimaryMinimum pkgs sibling} ${(nativePrimaryMinimum pkgs sibling).drvPath} \
            ${./primary-minimum-provider.sh} ${./test-primary-minima.sh} \
            ${./primary-minima-runtime.rs} ${primaryMinimaVerificationSources pkgs} \
            ${../../bootstrap/generated/record-primary.rs} \
            ${../../bootstrap/generated/record-primary-high-minimum.rs} \
            ${../../bootstrap/generated/conjunction-primary.rs} \
            ${../../bootstrap/generated/conjunction-primary-high-minimum.rs}; do
            if grep -Fx "$forbidden" "$closure/store-paths"; then
              echo "forbidden ${kind} minimum construction dependency: $forbidden" >&2
              exit 1
            fi
          done
          for required in ${(specExporter pkgs).drvPath} ${(nativePrimaryMinimum pkgs kind).drvPath} \
            "$sources" "$exporterSources"; do
            grep -Fx "$required" "$closure/store-paths"
          done
          grep -F "$sources" ${(nativePrimaryMinimum pkgs kind).drvPath}
          grep -F '${specExporter pkgs}/bin/cuestacean-spec-export' ${(nativePrimaryMinimum pkgs kind).drvPath}
          (cd "$sources"; find . -type f | LC_ALL=C sort) > source-inventory
          printf '%s\n' ./${kind}-minimum-input-root.cue ./${kind}-minimum-input.cue > expected-inventory
          cmp source-inventory expected-inventory
          cmp "$sources/${kind}-minimum-input.cue" ${../../bootstrap + "/${kind}-minimum-input.cue"}
          cmp "$sources/${kind}-minimum-input-root.cue" ${
            ../../bootstrap + "/${kind}-minimum-input-root.cue"
          }
          (cd "$exporterSources"; find . -type f | LC_ALL=C sort) > exporter-inventory
          printf '%s\n' ./spec-export/Cargo.lock ./spec-export/Cargo.toml \
            ./spec-export/src/bindings.rs ./spec-export/src/comprehension.rs \
            ./spec-export/src/eval.rs ./spec-export/src/lib.rs ./spec-export/src/main.rs \
            ./spec-export/src/model.rs ./spec-export/src/parser.rs ./spec-export/src/projection.rs \
            > expected-exporter-inventory
          cmp exporter-inventory expected-exporter-inventory
          (cd "$producer"; find . -type f | LC_ALL=C sort) > output-inventory
          printf '%s\n' ./${kind}-minimum.json > expected-output-inventory
          cmp output-inventory expected-output-inventory
          cp "$closure/store-paths" native-build-paths
          cp "$closure/registration" native-build-registration
          cp -R "$sources" native-sources
          cp -R "$exporterSources" exporter-sources
          cp ${(nativePrimaryMinimum pkgs kind).drvPath} native-producer.drv
          cp ${(specExporter pkgs).drvPath} native-exporter.drv
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativePrimaryMinimum pkgs kind).drvPath} > native-build-roots
        )
      '';
      nativePrimaryMinimaVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-primary-minima-verification" { } ''
          set -euo pipefail
          mkdir -p "$out"
          ${primaryMinimumProvenance pkgs "record" "conjunction"}
          ${primaryMinimumProvenance pkgs "conjunction" "record"}
          ${pkgs.bash}/bin/bash ${./test-primary-minima.sh} \
            ${pkgs.cue}/bin/cue ${pkgs.jq}/bin/jq \
            ${specExporter pkgs}/bin/cuestacean-spec-export \
            ${nativePrattCodegen pkgs}/bin/bootstrap-pratt-codegen \
            ${pkgs.rustfmt}/bin/rustfmt ${jsonExports pkgs} \
            ${primaryMinimaVerificationSources pkgs} ${./primary-minimum-provider.sh} \
            ${nativePrimaryMinimum pkgs "record"}/record-minimum.json \
            ${nativePrimaryMinimum pkgs "conjunction"}/conjunction-minimum.json "$out"
        '';
      # Check-only consumer: never feeds a native compiler/seed constructor.
      primaryMinimaRuntime =
        pkgs:
        (executor pkgs).overrideAttrs (old: {
          pname = "cuestacean-primary-minima-runtime";
          doCheck = true;
          postPatch = old.postPatch + ''
            cp ${./primary-minima-runtime.rs} tests/primary_minima_runtime.rs
            for kind in record conjunction; do
              for value in 0 100; do
                for extension in rs json; do
                  cp ${nativePrimaryMinimaVerification pkgs}/runtime/"$kind-$value.$extension" tests/
                done
              done
            done
          '';
          preCheck = "";
          checkPhase = ''
            cargo test --offline --test primary_minima_runtime
          '';
        });
      primaryControlMinimaSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/schema.cue
            ../../bootstrap/primary-fragments.cue
            ../../bootstrap/primary-bound-minimum-input.cue
            ../../bootstrap/primary-bound-minimum-input-root.cue
            ../../bootstrap/primary-call-minimum-input.cue
            ../../bootstrap/primary-call-minimum-input-root.cue
            ../../bootstrap/primary-list-minimum-input.cue
            ../../bootstrap/primary-list-minimum-input-root.cue
            ../../bootstrap/primary.cue
            ../../bootstrap/tests/valid/primary-no-conjunction.cue
            ../../bootstrap/tests/valid/primary-bound-high-minimum.cue
            ../../bootstrap/tests/valid/primary-call-high-minimum.cue
            ../../bootstrap/tests/valid/primary-no-lists.cue
            ../../bootstrap/tests/valid/primary-no-records.cue
            ../../bootstrap/generated/primary.rs
            ../../bootstrap/generated/primary-no-conjunction.rs
            ../../bootstrap/generated/primary-bound-high-minimum.rs
            ../../bootstrap/generated/primary-call-high-minimum.rs
            ../../bootstrap/generated/primary-no-lists.rs
            ../../bootstrap/generated/primary-no-records.rs
          ];
        };
      # Independent all-sibling audit; the #298 two-way audits stay unchanged.
      primaryControlMinimumProvenance = pkgs: kind: ''
        (
          set -euo pipefail
          closure=${primaryMinimumBuildClosure pkgs kind}
          sources=${primaryMinimumSources pkgs kind}
          exporterSources=${specSources pkgs false}
          producer=${nativePrimaryMinimum pkgs kind}
          mkdir -p "$out/provenance/${kind}"
          cd "$out/provenance/${kind}"
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath} \
            ${nativeCodegenCore pkgs} ${(nativeCodegenCore pkgs).drvPath} \
            ${nativeScanCodegen pkgs} ${(nativeScanCodegen pkgs).drvPath} \
            ${nativeUnifyCodegen pkgs} ${(nativeUnifyCodegen pkgs).drvPath} \
            ${nativePrattCodegen pkgs} ${(nativePrattCodegen pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${nativeSliceLower pkgs} ${(nativeSliceLower pkgs).drvPath} ${sliceLowerSources pkgs} \
            ${./primary-control-minima-provider.sh} ${./test-primary-control-minima.sh} \
            ${./primary-control-minima-runtime.rs} ${primaryControlMinimaSources pkgs} \
            ${./primary-minimum-provider.sh} ${./test-primary-minima.sh} \
            ${./primary-minima-runtime.rs} ${primaryMinimaVerificationSources pkgs} \
            ${../../bootstrap/generated/primary.rs} \
            ${../../bootstrap/generated/primary-no-conjunction.rs} \
            ${../../bootstrap/generated/primary-bound-high-minimum.rs} \
            ${../../bootstrap/generated/primary-call-high-minimum.rs} \
            ${../../bootstrap/generated/primary-no-lists.rs} \
            ${../../bootstrap/generated/primary-no-records.rs} \
            ${
              builtins.concatStringsSep " " (
                map
                  (
                    sibling:
                    "${primaryMinimumSources pkgs sibling} ${nativePrimaryMinimum pkgs sibling} ${(nativePrimaryMinimum pkgs sibling).drvPath}"
                  )
                  (
                    builtins.filter (sibling: sibling != kind) [
                      "primary-bound"
                      "primary-call"
                      "primary-list"
                      "record"
                      "conjunction"
                    ]
                  )
              )
            }; do
            if grep -Fx "$forbidden" "$closure/store-paths"; then
              echo "forbidden ${kind} minimum construction dependency: $forbidden" >&2
              exit 1
            fi
          done
          for required in ${(specExporter pkgs).drvPath} ${(nativePrimaryMinimum pkgs kind).drvPath} \
            "$sources" "$exporterSources"; do
            grep -Fx "$required" "$closure/store-paths"
          done
          grep -F "$sources" ${(nativePrimaryMinimum pkgs kind).drvPath}
          grep -F '${specExporter pkgs}/bin/cuestacean-spec-export' ${(nativePrimaryMinimum pkgs kind).drvPath}
          (cd "$sources"; find . -type f | LC_ALL=C sort) > source-inventory
          printf '%s\n' ./${kind}-minimum-input-root.cue ./${kind}-minimum-input.cue > expected-inventory
          cmp source-inventory expected-inventory
          cmp "$sources/${kind}-minimum-input.cue" ${../../bootstrap + "/${kind}-minimum-input.cue"}
          cmp "$sources/${kind}-minimum-input-root.cue" ${
            ../../bootstrap + "/${kind}-minimum-input-root.cue"
          }
          (cd "$exporterSources"; find . -type f | LC_ALL=C sort) > exporter-inventory
          printf '%s\n' ./spec-export/Cargo.lock ./spec-export/Cargo.toml \
            ./spec-export/src/bindings.rs ./spec-export/src/comprehension.rs \
            ./spec-export/src/eval.rs ./spec-export/src/lib.rs ./spec-export/src/main.rs \
            ./spec-export/src/model.rs ./spec-export/src/parser.rs ./spec-export/src/projection.rs \
            > expected-exporter-inventory
          cmp exporter-inventory expected-exporter-inventory
          (cd "$producer"; find . -type f | LC_ALL=C sort) > output-inventory
          printf '%s\n' ./${kind}-minimum.json > expected-output-inventory
          cmp output-inventory expected-output-inventory
          cp "$closure/store-paths" native-build-paths
          cp "$closure/registration" native-build-registration
          cp -R "$sources" native-sources
          cp -R "$exporterSources" exporter-sources
          cp ${(nativePrimaryMinimum pkgs kind).drvPath} native-producer.drv
          cp ${(specExporter pkgs).drvPath} native-exporter.drv
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativePrimaryMinimum pkgs kind).drvPath} > native-build-roots
        )
      '';
      nativePrimaryControlMinimaVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-primary-control-minima-verification" { } ''
          set -euo pipefail
          mkdir -p "$out"
          ${primaryControlMinimumProvenance pkgs "primary-bound"}
          ${primaryControlMinimumProvenance pkgs "primary-call"}
          ${primaryControlMinimumProvenance pkgs "primary-list"}
          ${pkgs.bash}/bin/bash ${./test-primary-control-minima.sh} \
            ${pkgs.cue}/bin/cue ${pkgs.jq}/bin/jq \
            ${specExporter pkgs}/bin/cuestacean-spec-export \
            ${nativePrattCodegen pkgs}/bin/bootstrap-pratt-codegen \
            ${pkgs.rustfmt}/bin/rustfmt ${jsonExports pkgs} \
            ${primaryControlMinimaSources pkgs} ${./primary-control-minima-provider.sh} \
            ${nativePrimaryMinimum pkgs "primary-bound"}/primary-bound-minimum.json \
            ${nativePrimaryMinimum pkgs "primary-call"}/primary-call-minimum.json \
            ${nativePrimaryMinimum pkgs "primary-list"}/primary-list-minimum.json "$out"
        '';
      primaryControlMinimaRuntime =
        pkgs:
        (executor pkgs).overrideAttrs (old: {
          pname = "cuestacean-primary-control-minima-runtime";
          doCheck = true;
          postPatch = old.postPatch + ''
            cp ${./primary-control-minima-runtime.rs} tests/primary_control_minima_runtime.rs
            for kind in base bound call list; do
              for extension in rs json; do
                cp ${nativePrimaryControlMinimaVerification pkgs}/runtime/"$kind.$extension" tests/
              done
            done
          '';
          preCheck = "";
          checkPhase = ''
            cargo test --offline --test primary_control_minima_runtime
          '';
        });
      prattMinimaFixtures = [
        "pratt"
        "file-pratt"
        "tests/valid/pratt-no-slices"
        "tests/valid/pratt-slice-high-minimum"
        "tests/valid/pratt-no-selectors"
        "tests/valid/pratt-no-index"
        "tests/valid/pratt-index-high-minimum"
        "tests/valid/pratt-right"
      ];
      prattMinimaSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [
              ../../bootstrap/schema.cue
              ../../bootstrap/pratt-fragments.cue
              ../../bootstrap/pratt-index-minimum-input.cue
              ../../bootstrap/pratt-index-minimum-input-root.cue
              ../../bootstrap/pratt-slice-minimum-input.cue
              ../../bootstrap/pratt-slice-minimum-input-root.cue
            ]
            ++ map (key: ../../bootstrap + "/${key}.cue") prattMinimaFixtures
            ++ map (key: ../../bootstrap + "/generated/${baseNameOf key}.rs") prattMinimaFixtures
          );
        };
      # Separate producer closures; verification dependencies never feed construction.
      prattMinimumProvenance = pkgs: kind: ''
        (
          set -euo pipefail
          closure=${primaryMinimumBuildClosure pkgs kind}
          sources=${primaryMinimumSources pkgs kind}
          exporterSources=${specSources pkgs false}
          producer=${nativePrimaryMinimum pkgs kind}
          mkdir -p "$out/provenance/${kind}"
          cd "$out/provenance/${kind}"
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath} \
            ${nativeCodegenCore pkgs} ${(nativeCodegenCore pkgs).drvPath} \
            ${nativeScanCodegen pkgs} ${(nativeScanCodegen pkgs).drvPath} \
            ${nativeUnifyCodegen pkgs} ${(nativeUnifyCodegen pkgs).drvPath} \
            ${nativePrattCodegen pkgs} ${(nativePrattCodegen pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${nativeSliceLower pkgs} ${(nativeSliceLower pkgs).drvPath} ${sliceLowerSources pkgs} \
            ${./pratt-minima-provider.sh} ${./test-pratt-minima.sh} \
            ${./pratt-minima-runtime.rs} ${prattMinimaSources pkgs} \
            ${./primary-control-minima-provider.sh} ${./test-primary-control-minima.sh} \
            ${./primary-control-minima-runtime.rs} ${primaryControlMinimaSources pkgs} \
            ${./primary-minimum-provider.sh} ${./test-primary-minima.sh} \
            ${./primary-minima-runtime.rs} ${primaryMinimaVerificationSources pkgs} \
            ${
              builtins.concatStringsSep " " (
                map
                  (
                    sibling:
                    "${primaryMinimumSources pkgs sibling} ${nativePrimaryMinimum pkgs sibling} ${(nativePrimaryMinimum pkgs sibling).drvPath}"
                  )
                  (
                    builtins.filter (sibling: sibling != kind) [
                      "pratt-index"
                      "pratt-slice"
                      "primary-bound"
                      "primary-call"
                      "primary-list"
                      "record"
                      "conjunction"
                    ]
                  )
              )
            } \
            ${
              builtins.concatStringsSep " " (
                map (key: toString (../../bootstrap + "/generated/${baseNameOf key}.rs")) prattMinimaFixtures
              )
            }; do
            if grep -Fx "$forbidden" "$closure/store-paths"; then
              echo "forbidden ${kind} minimum construction dependency: $forbidden" >&2
              exit 1
            fi
          done
          for required in ${(specExporter pkgs).drvPath} ${(nativePrimaryMinimum pkgs kind).drvPath} \
            "$sources" "$exporterSources"; do
            grep -Fx "$required" "$closure/store-paths"
          done
          grep -F "$sources" ${(nativePrimaryMinimum pkgs kind).drvPath}
          grep -F '${specExporter pkgs}/bin/cuestacean-spec-export' ${(nativePrimaryMinimum pkgs kind).drvPath}
          (cd "$sources"; find . -type f | LC_ALL=C sort) > source-inventory
          printf '%s\n' ./${kind}-minimum-input-root.cue ./${kind}-minimum-input.cue > expected-inventory
          cmp source-inventory expected-inventory
          cmp "$sources/${kind}-minimum-input.cue" ${../../bootstrap + "/${kind}-minimum-input.cue"}
          cmp "$sources/${kind}-minimum-input-root.cue" ${
            ../../bootstrap + "/${kind}-minimum-input-root.cue"
          }
          (cd "$exporterSources"; find . -type f | LC_ALL=C sort) > exporter-inventory
          printf '%s\n' ./spec-export/Cargo.lock ./spec-export/Cargo.toml \
            ./spec-export/src/bindings.rs ./spec-export/src/comprehension.rs \
            ./spec-export/src/eval.rs ./spec-export/src/lib.rs ./spec-export/src/main.rs \
            ./spec-export/src/model.rs ./spec-export/src/parser.rs ./spec-export/src/projection.rs \
            > expected-exporter-inventory
          cmp exporter-inventory expected-exporter-inventory
          (cd "$producer"; find . -type f | LC_ALL=C sort) > output-inventory
          printf '%s\n' ./${kind}-minimum.json > expected-output-inventory
          cmp output-inventory expected-output-inventory
          cp "$closure/store-paths" native-build-paths
          cp "$closure/registration" native-build-registration
          cp -R "$sources" native-sources
          cp -R "$exporterSources" exporter-sources
          cp ${(nativePrimaryMinimum pkgs kind).drvPath} native-producer.drv
          cp ${(specExporter pkgs).drvPath} native-exporter.drv
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativePrimaryMinimum pkgs kind).drvPath} > native-build-roots
        )
      '';
      nativePrattMinimaVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-pratt-minima-verification" { } ''
          set -euo pipefail
          mkdir -p "$out"
          ${prattMinimumProvenance pkgs "pratt-index"}
          ${prattMinimumProvenance pkgs "pratt-slice"}
          ${pkgs.bash}/bin/bash ${./test-pratt-minima.sh} \
            ${pkgs.cue}/bin/cue ${pkgs.jq}/bin/jq \
            ${specExporter pkgs}/bin/cuestacean-spec-export \
            ${nativePrattCodegen pkgs}/bin/bootstrap-pratt-codegen \
            ${pkgs.rustfmt}/bin/rustfmt ${jsonExports pkgs} \
            ${prattMinimaSources pkgs} ${./pratt-minima-provider.sh} \
            ${nativePrimaryMinimum pkgs "pratt-index"}/pratt-index-minimum.json \
            ${nativePrimaryMinimum pkgs "pratt-slice"}/pratt-slice-minimum.json "$out"
        '';
      prattMinimaRuntime =
        pkgs: row:
        (executor pkgs).overrideAttrs (old: {
          pname = "cuestacean-pratt-minima-runtime-${row}";
          doCheck = true;
          CUESTACEAN_PRATT_MINIMA_ROW = row;
          postPatch = old.postPatch + ''
            # Install proof outputs AFTER normal regeneration, before compilation.
            cp ${./pratt-minima-runtime.rs} tests/pratt_minima_runtime.rs
            cp -R "$CUESTACEAN_BOOTSTRAP_JSON" "$PWD/pratt-minima-json"
            chmod -R u+w "$PWD/pratt-minima-json"
            for control in pratt file-pratt; do
              cp ${nativePrattMinimaVerification pkgs}/runtime/${row}/"$control.rs" "../generated/$control.rs"
              cp ${nativePrattMinimaVerification pkgs}/runtime/${row}/"$control.json" "$PWD/pratt-minima-json/$control.json"
            done
            export CUESTACEAN_BOOTSTRAP_JSON="$PWD/pratt-minima-json"
          '';
          preCheck = "";
          checkPhase = ''
            cargo test --offline --test pratt_minima_runtime
          '';
        });
      instructionModuleControlSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/instruction-module-first.cue
            ../../bootstrap/instruction-module-first-export.cue
            ../../bootstrap/instruction-module-step.cue
            ../../bootstrap/instruction-module-step-export.cue
          ];
        };
      nativeInstructionModuleControls =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-module-controls" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          # Two independent two-file invocations, never a combined preference group.
          for kind in first step; do
            owner=${instructionModuleControlSources pkgs}/instruction-module-"$kind.cue"
            selector=${instructionModuleControlSources pkgs}/instruction-module-"$kind-export.cue"
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$owner" "$selector" > "$kind.json"
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$selector" "$owner" > repeated.json
            test "$(wc -c < "$kind.json")" -le 65536
            test "$(wc -c < repeated.json)" -le 65536
            cmp "$kind.json" repeated.json
            cp "$kind.json" "$out/$kind.json"
          done
        '';
      nativeInstructionModuleControlsBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionModuleControls pkgs).drvPath
          ];
        };
      nativeInstructionModuleControlsVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-module-controls-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionModuleControlsBuildClosure pkgs}/store-paths; then
              echo "forbidden module controls construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionModuleControlSources pkgs} -type f | wc -l)" -eq 4
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$driver" "$jq" "$rustfmt"; do
            test -f "$binary" && test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$1" "$2"; }
          place() { cue import -f -p bootstrap -l "#InstructionModule$2:" -o "$3" "$1"; }
          cp ${instructionModuleControlSources pkgs}/*.cue .
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          text=${../../bootstrap/instruction-module-text.cue}
          unary=${../../bootstrap/instruction-unary-tables.cue}
          selector=${../../bootstrap/lowering.cue}
          first=instruction-module-first.cue
          step=instruction-module-step.cue
          cue export ${../../bootstrap/native-policy-package.cue} ${../../bootstrap/instruction-binary-tables.cue} > binary.json
          cue import -p bootstrap -l '#InstructionBinaryTables:' -o binary-placed.cue binary.json
          binary_gate() { cue export "$schema" "$bridge" binary-placed.cue -e '#InstructionBinaryTables' > /dev/null; }
          right=${../../bootstrap/instruction-right-field.cue}
          ${instructionOperandConsumer}
          ${instructionCoreConsumer}
          binary_export() { core_gate || return; right_gate || return; binary_gate || return; runtime_gate binary-placed.cue || return; module_gate "$text" "''${controlProviders[@]}" || return; touch full-export-started; cue export "$module" "$core" "$right" "$operand" "$group" "$runtime" "$@" binary-placed.cue; }
          base() { binary_export "$schema" "$bridge" "$text" "$unary" "$@"; }
          raw() { local controlProviders=("$first" "$step"); base "$first" "$step" "$@"; }
          selected() { local controlProviders=(first-placed.cue step-placed.cue); base first-placed.cue step-placed.cue "$@"; }
          for kind in first step; do
            native "instruction-module-$kind.cue" "instruction-module-$kind-export.cue" > "$kind.json"
            cmp "$kind.json" ${nativeInstructionModuleControls pkgs}/"$kind.json"
            cue export "instruction-module-$kind.cue" "instruction-module-$kind-export.cue" | json > oracle.json
            cmp "$kind.json" oracle.json
          done
          printf '%s\n' '{"op":"firstRule"}' | json > expected-first.json
          printf '%s\n' '{"op":"nextRule"}' | json > expected-step.json
          cmp first.json expected-first.json
          cmp step.json expected-step.json
          place first.json First first-placed.cue
          place step.json Step step-placed.cue
          consume_right() { selected right-backfill.cue; }
          ${rightFieldConsumerProbe}
          consume_operand() { consume_right; }
          ${instructionOperandConsumerProbe}
          consume_core() { selected "$selector"; }
          ${instructionCoreConsumerProbe pkgs}
          ${instructionRuntimeConsumerProbe}
          if grep -E '#InstructionModule(First|Step)' "$schema"; then exit 1; fi
          selected "$selector" | json > canonical.json
          raw "$selector" | json > raw.json
          cmp canonical.json raw.json
          cmp canonical.json ${jsonExports pkgs}/lowering.json
          controlProviders=(first-placed.cue step-placed.cue)
          binary_export "$selector" step-placed.cue first-placed.cue "$unary" "$text" "$bridge" "$schema" | json > reversed.json
          cmp canonical.json reversed.json
          for control in left-twice reverse; do
            context=${../../bootstrap/tests/valid}/lowering-"$control.cue"
            raw "$context" | json > "$control.json"
            controlProviders=("$first" "$step")
            binary_export "$context" "$step" "$first" "$unary" "$text" "$bridge" "$schema" | json > reversed.json
            cmp "$control.json" reversed.json
            cmp "$control.json" ${jsonExports pkgs}/tests/valid/lowering-"$control.json"
            generate ${jsonExports pkgs}/evaluation.json "$control.json" > "evaluation-$control.rs"
            cmp "evaluation-$control.rs" ${../../bootstrap/generated}/evaluation-"$control.rs"
          done
          if selected ${../../bootstrap/tests/valid/lowering-reverse.cue} > failed.json 2> failed.err; then exit 1; fi
          test ! -s failed.json && test -s failed.err
          grep -F 'conflicting values' failed.err
          # Missing providers AND missing op slots stay incomplete, never filled
          # by the literals removed from the bridge.
          for missing in both first step first-op step-op; do
            providers=()
            case "$missing" in
              first) providers=(step-placed.cue) ;;
              step) providers=(first-placed.cue) ;;
              first-op)
                printf '%s\n' 'package bootstrap' '#InstructionModuleFirst: {}' > shape.cue
                providers=(shape.cue step-placed.cue) ;;
              step-op)
                printf '%s\n' 'package bootstrap' '#InstructionModuleStep: {}' > shape.cue
                providers=(first-placed.cue shape.cue) ;;
            esac
            core_gate
            binary_gate
            right_gate
            runtime_gate binary-placed.cue
            cue eval "$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$runtime" "$text" "$unary" binary-placed.cue "''${providers[@]}" "$selector" > /dev/null
            controlProviders=("''${providers[@]}")
            if base "''${providers[@]}" "$selector" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
            grep -F 'incomplete value' failed.err
          done
          for kind in First Step; do
            case "$kind" in First) data=first.json; other=step-placed.cue ;; Step) data=step.json; other=first-placed.cue ;; esac
            for invalid in '.op=1' '.extra=true' '.op="unknownModuleOp"'; do
              env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$invalid" "$data" > invalid.json
              place invalid.json "$kind" invalid.cue
              controlProviders=(invalid.cue "$other")
              if base invalid.cue "$other" "$selector" > failed.json 2> failed.err; then exit 1; fi
              test ! -s failed.json && test -s failed.err
            done
          done
          # Predict the entire tiny evaluator for all four independent cursor
          # states. No emitted source or committed golden supplies this answer.
          printf '%s\n' '{"formatVersion":1,"rules":{"a":{"result":{"op":"int"}},"b":{"result":{"op":"bool"}}}}' > tiny.json
          for state in first-next last-previous last-next first-previous; do
            case "$state" in
              first-next) initial=firstRule; advance=nextRule ;;
              last-previous) initial=lastRule; advance=previousRule ;;
              last-next) initial=lastRule; advance=nextRule ;;
              first-previous) initial=firstRule; advance=previousRule ;;
            esac
            printf '%s\n' 'package bootstrap' \
              "#InstructionLowering & {module: [_, {op: \"$initial\"}, {body: [_, _, _, _, {op: \"$advance\"}]}, _]}" > context.cue
            raw context.cue | json > "$state.json"
            controlProviders=("$first" "$step")
            binary_export context.cue "$step" "$first" "$unary" "$text" "$bridge" "$schema" | json > reversed.json
            cmp "$state.json" reversed.json
            generate tiny.json "$state.json" > "$state.rs"
            case "$state" in
              first-next) arms=$'"a" => Ok::<_, Error>(runtime::Value::Int),\n"b" => Ok::<_, Error>(runtime::Value::Bool),' ;;
              last-previous) arms=$'"b" => Ok::<_, Error>(runtime::Value::Bool),\n"a" => Ok::<_, Error>(runtime::Value::Int),' ;;
              last-next) arms='"b" => Ok::<_, Error>(runtime::Value::Bool),' ;;
              first-previous) arms='"a" => Ok::<_, Error>(runtime::Value::Int),' ;;
            esac
            format > "$state-expected.rs" <<RUST
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              match runtime::kind(node)? {
                  $arms
                  kind => Err(Error::MissingRule { kind: kind.to_string() }),
              }
          }
          RUST
            cmp "$state.rs" "$state-expected.rs"
          done
          # Source-only priority and literal changes must affect native DATA,
          # standard import, and the complete predicted Rust above.
          for mutation in priority literal; do
            substitute "$first" first-variant.cue --replace-fail \
              'op: *"firstRule" | "lastRule"' \
              "$(case "$mutation" in priority) printf '%s' 'op: "firstRule" | *"lastRule"' ;; literal) printf '%s' 'op: *"lastRule" | "lastRule"' ;; esac)"
            native first-variant.cue instruction-module-first-export.cue > variant.json
            place variant.json First first-placed.cue
            selected "$selector" > variant-policy.json
            controlProviders=(first-variant.cue "$step")
            base first-variant.cue "$step" "$selector" > variant-raw.json
            cmp variant-policy.json variant-raw.json
            generate tiny.json variant-policy.json > variant.rs
            cmp variant.rs last-next-expected.rs
          done
          place first.json First first-placed.cue
          substitute "$step" step-variant.cue --replace-fail \
            'op: *"nextRule" | "previousRule"' 'op: "nextRule" | *"previousRule"'
          native step-variant.cue instruction-module-step-export.cue > variant.json
          place variant.json Step step-placed.cue
          selected "$selector" > variant-policy.json
          controlProviders=("$first" step-variant.cue)
          base "$first" step-variant.cue "$selector" > variant-raw.json
          cmp variant-policy.json variant-raw.json
          generate tiny.json variant-policy.json > variant.rs
          cmp variant.rs first-previous-expected.rs
          generate ${jsonExports pkgs}/evaluation.json canonical.json > evaluation.rs
          cmp evaluation.rs ${../../bootstrap/generated/evaluation.rs}
          generate ${../../bootstrap/executor/tests/fixtures/codegen-escaped-rules.json} canonical.json > escaped.rs
          cmp escaped.rs ${../../bootstrap/generated/codegen-escaped-rules.rs}
          for policy in ${loweringNames}; do
            name=$(basename "$policy")
            env -i HOME="$PWD/home" PATH=/nonexistent "$driver" ${jsonExports pkgs}/"$policy.json" | format > "$name.rs"
            cmp "$name.rs" ${../../bootstrap/generated}/"$name.rs"
          done
          cp first.json step.json canonical.json left-twice.json reverse.json *-expected.rs first-next.rs last-previous.rs last-next.rs first-previous.rs "$out/"
          cp ${nativeInstructionModuleControlsBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionModuleControlsBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionModuleControls pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionCoreSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/instruction-core-cases.cue
          ];
        };
      nativeInstructionCoreCases =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-core-cases" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          package=${instructionCoreSources pkgs}/native-policy-package.cue
          body=${instructionCoreSources pkgs}/instruction-core-cases.cue
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$package" "$body" > first.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$package" "$body" > repeated.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$body" "$package" > reversed.json
          for result in first repeated reversed; do
            test "$(wc -c < "$result.json")" -le 65536
            cmp first.json "$result.json"
          done
          cp first.json "$out/instruction-core-cases.json"
        '';
      nativeInstructionCoreCasesBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionCoreCases pkgs).drvPath
          ];
        };
      nativeInstructionCoreCasesVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-core-cases-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionCoreCasesBuildClosure pkgs}/store-paths; then
              echo "forbidden core-case construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionCoreSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$driver" "$jq" "$rustfmt"; do
            test -f "$binary" && test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          query() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$@"; }
          json() { query -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" native-policy-package.cue "$1"; }
          cp ${instructionCoreSources pkgs}/*.cue .
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          selector=${../../bootstrap/lowering.cue}
          right=${../../bootstrap/instruction-right-field.cue}
          unary=${../../bootstrap/instruction-unary-tables.cue}
          text=${../../bootstrap/instruction-module-text.cue}
          first=${../../bootstrap/instruction-module-first.cue}
          step=${../../bootstrap/instruction-module-step.cue}
          cue export ${../../bootstrap/native-policy-package.cue} ${../../bootstrap/instruction-binary-tables.cue} > binary.json
          cue import -p bootstrap -l '#InstructionBinaryTables:' -o binary-placed.cue binary.json
          ${instructionCoreConsumer}
          core=instruction-core-cases.cue
          ${instructionOperandConsumer}
          binary_gate() { cue export "$schema" "$bridge" binary-placed.cue -e '#InstructionBinaryTables' > /dev/null; }
          policy() {
            core_gate || return
            right_gate || return
            binary_gate || return
            runtime_gate binary-placed.cue || return
            module_gate "$text" "$first" "$step" || return
            touch full-export-started
            if test "$2" = reverse; then
              cue export "$module" "$1" "$step" "$first" "$text" "$unary" binary-placed.cue "$runtime" "$group" "$operand" "$right" "$core" "$bridge" "$schema"
            else
              cue export "$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$runtime" binary-placed.cue "$unary" "$text" "$first" "$step" "$1"
            fi
          }
          native instruction-core-cases.cue > body.json
          cmp body.json ${nativeInstructionCoreCases pkgs}/instruction-core-cases.json
          test "$(wc -c < body.json)" -eq 5298
          cue export native-policy-package.cue instruction-core-cases.cue | json > oracle.json
          cmp body.json oracle.json
          # COMPLETE independent expansion: neither producer output nor schema is an answer.
          query -n '
            def path($v): {op:"path",value:$v};
            def read($op;$field): {op:$op,field:$field};
            def call($p;$a): {op:"call",path:$p,args:$a};
            def wrap($op;$v): {op:$op,value:$v};
            def lower($f): wrap("try";wrap("group";read("lower";$f)));
            path("node") as $node | read("string";"field") as $field |
            path("_context") as $ctx | path("evaluate") as $eval |
            {cases:{
              "null":wrap("ok";path("runtime::Value::Null")),
              top:wrap("ok";path("runtime::Value::Top")),
              int:wrap("ok";path("runtime::Value::Int")),
              bool:wrap("ok";path("runtime::Value::Bool")),
              stringType:wrap("ok";path("runtime::Value::String")),
              text:call("runtime::text";[$node,$field]),
              list:call("runtime::list";[$node,$field]),
              record:call("runtime::record";[$node,$field]),
              recordExpression:call("runtime::record_expression";[$node,$field,$ctx,$eval]),
              listExpression:call("runtime::list_expression";[$node,$field,$ctx,$eval]),
              boolean:wrap("ok";call("runtime::Value::Boolean";[read("boolean";"value")])),
              field:call("runtime::integer";[$node,$field]),
              evaluate:call("evaluate";[wrap("try";call("runtime::field";[$node,$field])),$ctx]),
              conjunction:call("runtime::conjunction";[$node,$field,$ctx,$eval]),
              slice:call("runtime::slice_fields";[$node,read("string";"operandField"),
                read("string";"lowerField"),read("string";"upperField"),$ctx,$eval]),
              listSlice:{op:"concat",parts:[{op:"text",value:"(|_context: &mut runtime::EvaluationContext| "},
                call("runtime::list_slice";[lower("value"),lower("lower"),lower("upper")]),
                {op:"text",value:")(_context)"}]}
            }}' | json > expected.json
          cmp expected.json body.json
          query -c 'paths(scalars)' expected.json > slots
          test "$(wc -l < slots)" -eq 118
          no_shape() {
            if cue export "$coreShape" "$@" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          }
          no_policy() {
            rm -f full-export-started
            if policy "$selector" forward > failed-policy.json 2> failed-policy.err; then exit 1; fi
            test ! -s failed-policy.json && test -s failed-policy.err
            test ! -e full-export-started
          }
          # Absence is incomplete, while wrong types/extras/array lengths conflict.
          cue eval "$coreShape" > /dev/null
          no_shape
          grep -F 'incomplete value' failed.err
          query -r '.cases | keys[]' expected.json > case-names
          test "$(wc -l < case-names)" -eq 16
          while read -r name; do
            query --arg name "$name" 'del(.cases[$name])' expected.json > missing.json
            cue eval "$coreShape" missing.json > /dev/null
            no_shape missing.json
            grep -F 'incomplete value' failed.err
            core=missing.json; no_policy
          done < case-names
          while read -r slot; do
            query --argjson slot "$slot" 'delpaths([$slot])' expected.json > missing.json
            cue eval "$coreShape" missing.json > /dev/null
            no_shape missing.json
            grep -F 'incomplete value' failed.err
            core=missing.json; no_policy
            query --argjson slot "$slot" 'setpath($slot;42)' expected.json > invalid.json
            no_shape invalid.json
            grep -F 'conflicting values' failed.err
            core=invalid.json; no_policy
          done < slots
          query -c 'paths(arrays)' expected.json > arrays
          test "$(wc -l < arrays)" -eq 13
          while read -r array; do
            for operation in shorten extend; do
              case "$operation" in shorten) change='.[0:-1]' ;; extend) change='.+[.[0]]' ;; esac
              query --argjson array "$array" "getpath(\$array) | $change" expected.json > array.json
              query --argjson array "$array" --slurpfile value array.json 'setpath($array;$value[0])' expected.json > invalid.json
              no_shape invalid.json
              grep -E 'incompatible list lengths|not allowed' failed.err
              core=invalid.json; no_policy
            done
          done < arrays
          for extra in '.extra="x"' '.cases.extra={}' '.cases.text.extra="x"' \
            '.cases.text.args[0].extra="x"' '.cases.listSlice.parts[1].args[0].value.value.extra="x"'; do
            query "$extra" expected.json > invalid.json
            no_shape invalid.json
            grep -F 'field not allowed' failed.err
            core=invalid.json; no_policy
          done
          status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" native-policy-package.cue "$coreShape" > native-shape.json 2> native-shape.err || status=$?
          test "$status" -eq 2 && test ! -s native-shape.json && test -s native-shape.err
          grep -Fx 'syntax: declaration separator' native-shape.err
          core=instruction-core-cases.cue
          consume_core() { policy "$selector" forward; }
          ${instructionCoreConsumerProbe pkgs}
          ${instructionRuntimeConsumerProbe}
          consume_operand() { policy "$selector" forward; }
          ${instructionOperandConsumerProbe}
          # Genuine verification-only bypass: the full schema supplies the absent opcode.
          query 'del(.cases.text.op)' expected.json > missing-op.json
          no_shape missing-op.json
          right_gate
          binary_gate
          runtime_gate binary-placed.cue
          module_gate "$text" "$first" "$step"
          cue export "$module" "$schema" "$bridge" missing-op.json "$right" "$operand" "$group" "$runtime" binary-placed.cue "$unary" "$text" "$first" "$step" "$selector" > bypass.json
          query -e '.cases.text.op=="call"' bypass.json > /dev/null
          core=missing-op.json; no_policy
          # Full policy comparisons preserve raw preferences and all cursor contexts.
          for context in lowering tests/valid/lowering-left-twice tests/valid/lowering-reverse; do
            case "$context" in lowering) source="$selector" ;; *) source=${instructionOverrideFixtures pkgs}/"$(basename "$context").cue" ;; esac
            name=$(basename "$context")
            for order in forward reverse; do
              core=instruction-core-cases.cue
              policy "$source" "$order" | json > "$name-raw-$order.json"
              core=body.json
              policy "$source" "$order" | json > "$name-native-$order.json"
              cmp "$name-raw-$order.json" "$name-native-$order.json"
              cmp "$name-native-$order.json" ${jsonExports pkgs}/"$context.json"
            done
          done
          for state in last-next first-previous; do
            case "$state" in last-next) initial=lastRule; advance=nextRule ;; first-previous) initial=firstRule; advance=previousRule ;; esac
            printf '%s\n' 'package bootstrap' "#InstructionLowering & {module:[_,{op:\"$initial\"},{body:[_,_,_,_,{op:\"$advance\"}]},_]}" > "$state.cue"
            core=instruction-core-cases.cue
            policy "$state.cue" forward | json > "$state-raw.json"
            core=body.json
            policy "$state.cue" reverse | json > "$state-native.json"
            cmp "$state-raw.json" "$state-native.json"
          done
          # The shape permits strings, not semantic answers: Go and typed phases differ.
          for invalid in opcode selector-limit; do
            case "$invalid" in
              opcode) mutation='.cases.text.op="unknownCoreOp"' ;;
              selector-limit) mutation='.cases.text.args[1].field=("x"*129)' ;;
            esac
            query "$mutation" expected.json > "$invalid.json"
            core="$invalid.json"
            core_gate
            if test "$invalid" = opcode; then
              if policy "$selector" forward > opcode-policy.json 2> opcode.err; then exit 1; fi
              test ! -s opcode-policy.json && test -s opcode.err
            else
              policy "$selector" forward > selector-policy.json
              status=0
              env -i HOME="$PWD/home" PATH=/nonexistent "$driver" selector-policy.json > invalid.rs 2> invalid.err || status=$?
              test "$status" -eq 2 && test ! -s invalid.rs
              grep -F 'selector must contain 1..=128 bytes' invalid.err
            fi
          done
          # Source-only case/helper changes reach COMPLETE independently predicted Rust.
          printf '%s\n' '{"formatVersion":1,"rules":{"a":{"result":{"op":"null"}},"b":{"result":{"op":"text","field":"value"}},"c":{"result":{"op":"field","field":"count"}}}}' > tiny.json
          for variant in baseline constant helper; do
            mkdir "$variant"
            constant=Null; nodePath=node
            case "$variant" in
              baseline) cp instruction-core-cases.cue "$variant/source.cue" ;;
              constant) substitute instruction-core-cases.cue "$variant/source.cue" --replace-fail 'runtime::Value::Null' 'runtime::Value::Top'; constant=Top ;;
              helper) substitute instruction-core-cases.cue "$variant/source.cue" --replace-fail 'let Node = {op: "path", value: "node"}' 'let Node = {op: "path", value: "different_node"}'; nodePath=different_node ;;
            esac
            native "$variant/source.cue" > "$variant/data.json"
            core="$variant/data.json"
            policy "$selector" forward | json > "$variant/native-policy.json"
            core="$variant/source.cue"
            policy "$selector" reverse | json > "$variant/raw-policy.json"
            cmp "$variant/native-policy.json" "$variant/raw-policy.json"
            generate tiny.json "$variant/native-policy.json" > "$variant/actual.rs"
            format > "$variant/expected.rs" <<RUST
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              match runtime::kind(node)? {
                  "a" => Ok::<_, Error>(runtime::Value::$constant),
                  "b" => runtime::text($nodePath, "value"),
                  "c" => runtime::integer($nodePath, "count"),
                  kind => Err(Error::MissingRule { kind: kind.to_string() }),
              }
          }
          RUST
            cmp "$variant/actual.rs" "$variant/expected.rs"
          done
          for control in lowering lowering-left-twice lowering-reverse; do
            generate ${jsonExports pkgs}/evaluation.json "$control-native-forward.json" > "$control.rs"
            case "$control" in lowering) golden=evaluation ;; *) golden=evaluation-''${control#lowering-} ;; esac
            cmp "$control.rs" ${../../bootstrap/generated}/"$golden.rs"
          done
          for policyName in ${loweringNames}; do
            name=$(basename "$policyName")
            env -i HOME="$PWD/home" PATH=/nonexistent "$driver" ${jsonExports pkgs}/"$policyName.json" | format > "$name-driver.rs"
            cmp "$name-driver.rs" ${../../bootstrap/generated}/"$name.rs"
          done
          cp -R . "$out/evidence"
          rm -rf "$out/evidence/home"
          cp ${nativeInstructionCoreCasesBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionCoreCasesBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionCoreCases pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionRightFieldSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/instruction-right-field.cue
            ../../bootstrap/instruction-right-field-export.cue
          ];
        };
      nativeInstructionRightField =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-right-field" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          owner=${instructionRightFieldSources pkgs}/instruction-right-field.cue
          selector=${instructionRightFieldSources pkgs}/instruction-right-field-export.cue
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$owner" "$selector" > first.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$owner" "$selector" > repeated.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$selector" "$owner" > reversed.json
          for result in first repeated reversed; do
            test "$(wc -c < "$result.json")" -le 65536
            cmp first.json "$result.json"
          done
          cp first.json "$out/right-field.json"
        '';
      nativeInstructionRightFieldBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionRightField pkgs).drvPath
          ];
        };
      nativeInstructionRightFieldVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-right-field-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionRightFieldBuildClosure pkgs}/store-paths; then
              echo "forbidden right-field construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionRightFieldSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$driver" "$jq" "$rustfmt"; do
            test -f "$binary" && test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          query() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$@"; }
          json() { query -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
          place() { cue import -f -p bootstrap -l '#InstructionRightField:' -o "$2" "$1"; }
          cp ${instructionRightFieldSources pkgs}/*.cue .
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          unary=${../../bootstrap/instruction-unary-tables.cue}
          text=${../../bootstrap/instruction-module-text.cue}
          first=${../../bootstrap/instruction-module-first.cue}
          step=${../../bootstrap/instruction-module-step.cue}
          selector=${../../bootstrap/lowering.cue}
          owner=instruction-right-field.cue
          rightSelector=instruction-right-field-export.cue
          cue export ${../../bootstrap/native-policy-package.cue} ${../../bootstrap/instruction-binary-tables.cue} > binary.json
          cue import -p bootstrap -l '#InstructionBinaryTables:' -o binary-placed.cue binary.json
          binary_gate() { cue export "$schema" "$bridge" binary-placed.cue -e '#InstructionBinaryTables' > /dev/null; }
          isolated() { cue "$1" "$schema" "$bridge" "''${@:2}" -e '#InstructionRightField'; }
          operand=${../../bootstrap/instruction-right-operand.cue}
          group=${../../bootstrap/instruction-operand-cases.cue}
          right_gate() {
            isolated export "$@" > /dev/null || return
            cue export "$schema" "$bridge" "$operand" "$@" -e '#InstructionRightOperand' > /dev/null || return
            cue export "$schema" "$bridge" "$operand" "$group" "$@" -e '#InstructionOperandCases' > /dev/null
          }
          ${instructionCoreConsumer}
          ${instructionRuntimeConsumer}
          # All checks are fail-stop even when the caller uses if or ||.
          # Raw validation discards selected stdout and reloads the SAME owner.
          policy() {
            right=$1; context=$2; order=$3
            rightInputs=()
            if test -n "$right"; then rightInputs=("$right"); fi
            core_gate || return
            right_gate "''${rightInputs[@]}" || return
            binary_gate || return
            runtime_gate binary-placed.cue || return
            module_gate "$text" "$first" "$step" || return
            touch full-export-started
            inputs=("$module" "$schema" "$bridge" "$core" "$operand" "$group" "$runtime" "''${rightInputs[@]}" binary-placed.cue "$unary" "$text" "$first" "$step" "$context")
            if test "$order" = reverse; then
              cue export "$module" "$context" "$step" "$first" "$text" "$unary" binary-placed.cue "$runtime" "$group" "$operand" "''${rightInputs[@]}" "$core" "$bridge" "$schema"
            else
              cue export "''${inputs[@]}"
            fi
          }
          native "$owner" "$rightSelector" > right.json
          native "$rightSelector" "$owner" > right-reversed.json
          native "$owner" "$rightSelector" > right-repeat.json
          printf '%s\n' '{"field":"right"}' | json > right-expected.json
          cue export "$owner" "$rightSelector" | json > right-oracle.json
          for result in right-reversed right-repeat right-expected right-oracle; do cmp right.json "$result.json"; done
          cmp right.json ${nativeInstructionRightField pkgs}/right-field.json
          test "$(wc -c < right.json)" -eq 23
          place right.json selected.cue
          consume_core() { policy "$owner" "$selector" forward; }
          ${instructionCoreConsumerProbe pkgs}
          ${instructionRuntimeConsumerProbe}
          consume_operand() { policy "$owner" "$selector" forward; }
          ${instructionOperandConsumerProbe}
          policy selected.cue "$selector" forward | json > canonical.json
          policy "$owner" "$selector" reverse | json > canonical-raw.json
          cmp canonical.json canonical-raw.json
          cmp canonical.json ${jsonExports pkgs}/lowering.json
          # Arbitrary names and ordinary literals use the SAME existing profile.
          printf '%s\n' 'package arbitrary' '#Pick: {answer: "cold" | *"warm"}' > arbitrary.cue
          printf '%s\n' 'package arbitrary' '#Pick' > arbitrary-export.cue
          native arbitrary.cue arbitrary-export.cue > arbitrary.json
          native arbitrary-export.cue arbitrary.cue > arbitrary-reversed.json
          printf '%s\n' '{"answer":"warm"}' | json > arbitrary-expected.json
          cmp arbitrary.json arbitrary-expected.json
          cmp arbitrary.json arbitrary-reversed.json
          # Independent nine-site inventory: the aliases share binaryRuntime,
          # but no other use of the Go lexical R may inherit a site's refinement.
          cat > sites.tsv <<'SITES'
          binaryRuntime|cases:{binaryRuntime:{parts:[_,_,_,_,{value:{value:{field:"%s"}}},_]}}
          sum|cases:{sum:{args:[_,{value:{value:{field:"%s"}}}]}}
          less|cases:{less:{args:[_,{value:{value:{field:"%s"}}}]}}
          lessEqual|cases:{lessEqual:{args:[_,{value:{value:{field:"%s"}}}]}}
          greater|cases:{greater:{args:[_,{value:{value:{field:"%s"}}}]}}
          greaterEqual|cases:{greaterEqual:{args:[_,{value:{value:{field:"%s"}}}]}}
          notEqual|cases:{notEqual:{args:[_,{value:{value:{field:"%s"}}}]}}
          remainder|cases:{remainder:{args:[_,{value:{value:{field:"%s"}}}]}}
          unify|cases:{unify:{parts:[_,{args:[_,{value:{value:{field:"%s"}}}]},_]}}
          SITES
          # Independently authored JSON locations, not scraped from policy DATA.
          cat > paths.json <<'PATHS'
          [["cases","binaryRuntime","parts",4,"value","value","field"],
           ["cases","sum","args",1,"value","value","field"],
           ["cases","less","args",1,"value","value","field"],
           ["cases","lessEqual","args",1,"value","value","field"],
           ["cases","greater","args",1,"value","value","field"],
           ["cases","greaterEqual","args",1,"value","value","field"],
           ["cases","notEqual","args",1,"value","value","field"],
           ["cases","remainder","args",1,"value","value","field"],
           ["cases","unify","parts",1,"args",1,"value","value","field"]]
          PATHS
          # COMPLETE tiny instructions cover every R consumer and all eleven
          # Binary aliases. Left/right produce visibly different complete trees.
          cat > methods.tsv <<'METHODS'
          difference difference 0
          divide divide 0
          equal equal 0
          greater greater 4
          greaterEqual greater_equal 5
          index index 0
          less less 2
          lessEqual less_equal 3
          listIndex list_index 0
          logicalAnd logical_and 0
          logicalOr logical_or 0
          modulo modulo 0
          notEqual not_equal 6
          product product 0
          quotient quotient 0
          recordIndex record_index 0
          remainder remainder 7
          sum sum 1
          unify unify 8
          METHODS
          query -Rn '[inputs | split(" ") | {key:.[0],value:{result:{op:.[0],left:{op:"int"},right:{op:"bool"}}}}] | {formatVersion:1,rules:from_entries}' < methods.tsv > tiny.json
          # The full source prediction is handwritten, never a golden/output edit.
          expected_rust() {
            fields=$1
            : > arms.rs
            while read -r op method slot; do
              choice=$(query -r --argjson slot "$slot" '.[$slot]' <<< "$fields")
              case "$choice" in left) value=Int ;; right) value=Bool ;; *) exit 1 ;; esac
              expression="runtime::$method((Ok::<_, Error>(runtime::Value::Int))?, (Ok::<_, Error>(runtime::Value::$value))?)"
              if test "$op" = unify; then expression="(|_context: &mut runtime::EvaluationContext| $expression)(_context)"; fi
              printf '"%s" => %s,\n' "$op" "$expression" >> arms.rs
            done < methods.tsv
            arms=$(< arms.rs)
            format <<RUST
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              match runtime::kind(node)? {
                  $arms
                  kind => Err(Error::MissingRule { kind: kind.to_string() }),
              }
          }
          RUST
          }
          # Keep EVERY variant's source, context, native DATA, raw/native full
          # policies and complete independently predicted/actual Rust separately.
          for variant in canonical priority literal; do
            mkdir "$variant"
            cp "$rightSelector" "$variant/selector.cue"
            case "$variant" in
              canonical) cp "$owner" "$variant/owner.cue"; preferred=right; alternate=left ;;
              priority)
                substitute "$owner" "$variant/owner.cue" --replace-fail '*"right" | "left"' '"right" | *"left"'
                preferred=left; alternate=right ;;
              literal)
                substitute "$owner" "$variant/owner.cue" --replace-fail '*"right" | "left"' '*"left" | "left"'
                preferred=left; alternate=left ;;
            esac
            native "$variant/owner.cue" "$variant/selector.cue" > "$variant/native.json"
            native "$variant/selector.cue" "$variant/owner.cue" > "$variant/native-reversed.json"
            printf '{"field":"%s"}\n' "$preferred" | json > "$variant/expected.json"
            cmp "$variant/native.json" "$variant/expected.json"
            cmp "$variant/native.json" "$variant/native-reversed.json"
            cue export "$variant/owner.cue" "$variant/selector.cue" | json > "$variant/oracle.json"
            cmp "$variant/native.json" "$variant/oracle.json"
            place "$variant/native.json" "$variant/placed.cue"
            policy "$variant/placed.cue" "$selector" forward | json > "$variant/native-full.json"
            policy "$variant/owner.cue" "$selector" reverse | json > "$variant/raw-full.json"
            cmp "$variant/native-full.json" "$variant/raw-full.json"
            fields=$(query -cn --arg p "$preferred" '[range(9)|$p]')
            expected_rust "$fields" > "$variant/expected.rs"
            generate tiny.json "$variant/native-full.json" > "$variant/actual.rs"
            cmp "$variant/expected.rs" "$variant/actual.rs"
            # Literal-only mutation has no alternative context to force.
            if test "$variant" = literal; then continue; fi
            for mask in none 0 1 2 3 4 5 6 7 8 all even odd; do
              name="$variant/$mask"
              mkdir "$name"
              printf '%s\n' 'package bootstrap' '#InstructionLowering & {' > "$name/context.cue"
              fields=(); index=0
              while IFS='|' read -r site constraint; do
                chosen=$preferred
                case "$mask:$index" in
                  all:*|even:0|even:2|even:4|even:6|even:8|odd:1|odd:3|odd:5|odd:7) chosen=$alternate ;;
                  *) if test "$mask" = "$index"; then chosen=$alternate; fi ;;
                esac
                fields+=("$chosen")
                if test "$chosen" != "$preferred"; then printf "$constraint\n" "$chosen" >> "$name/context.cue"; fi
                index=$((index + 1))
              done < sites.tsv
              printf '%s\n' '}' >> "$name/context.cue"
              printf '%s\n' "''${fields[@]}" | query -Rn '[inputs]' > "$name/expected-fields.json"
              for order in forward reverse; do
                policy "$variant/owner.cue" "$name/context.cue" "$order" | json > "$name/raw-$order.json"
                query --slurpfile paths paths.json '[getpath($paths[0][])]' "$name/raw-$order.json" > "$name/actual-fields-$order.json"
                cmp "$name/expected-fields.json" "$name/actual-fields-$order.json"
                query --slurpfile paths paths.json 'delpaths($paths[0])' "$name/raw-$order.json" | json > "$name/untouched-$order.json"
                query --slurpfile paths paths.json 'delpaths($paths[0])' canonical.json | json > untouched.json
                cmp untouched.json "$name/untouched-$order.json"
                expected_rust "$(< "$name/expected-fields.json")" > "$name/expected-$order.rs"
                generate tiny.json "$name/raw-$order.json" > "$name/actual-$order.rs"
                cmp "$name/expected-$order.rs" "$name/actual-$order.rs"
              done
              cmp "$name/raw-forward.json" "$name/raw-reverse.json"
            done
          done
          # Missing provider/field: eval succeeds in isolation, export is EMPTY.
          printf '%s\n' 'package bootstrap' '#InstructionRightField: {}' > missing.cue
          for missing in owner field; do
            providers=()
            if test "$missing" = field; then providers=(missing.cue); fi
            isolated eval "''${providers[@]}" > "$missing-eval.cue"
            if isolated export "''${providers[@]}" > "$missing.json" 2> "$missing.err"; then exit 1; fi
            test ! -s "$missing.json" && test -s "$missing.err"
          done
          # Explicit verification-only bypass: all nine contexts backfill R.
          core_gate
          binary_gate
          module_gate "$text" "$first" "$step"
          cue export "$module" "$schema" "$bridge" "$core" "$operand" "$group" "$runtime" "$text" "$unary" "$first" "$step" binary-placed.cue canonical/all/context.cue | json > bypass-all-left.json
          cmp bypass-all-left.json canonical/all/raw-forward.json
          # Exercise the ACTUAL policy wrapper under both if and ||, including
          # the context that could conceal a missing input without isolation.
          for missing in missing.cue ""; do
            rm -f full-export-started
            if policy "$missing" canonical/all/context.cue forward > stopped.json 2> stopped.err; then exit 1; fi
            test ! -e full-export-started && test ! -s stopped.json && test -s stopped.err
            status=0
            policy "$missing" canonical/all/context.cue reverse > stopped-or.json 2> stopped-or.err || status=$?
            test "$status" -ne 0
            test ! -e full-export-started && test ! -s stopped-or.json && test -s stopped-or.err
          done
          for invalid in '{field:1}' '{field:"right",extra:true}'; do
            printf '%s\n' 'package bootstrap' "#InstructionRightField: $invalid" > invalid.cue
            if policy invalid.cue "$selector" forward > invalid.json 2> invalid.err; then exit 1; fi
            test ! -s invalid.json && test -s invalid.err
          done
          # Empty is accepted by the string-only helper, rejected by full schema.
          printf '%s\n' 'package bootstrap' '#InstructionRightField:{field:""}' > empty.cue
          right_gate empty.cue
          if policy empty.cue "$selector" forward > empty.json 2> empty.err; then exit 1; fi
          test ! -s empty.json && test -s empty.err
          # 129 nonempty bytes instead pass Go and reach the REAL typed compiler.
          query -n '{field:([range(129)|"x"]|join(""))}' > long.json
          place long.json long.cue
          policy long.cue "$selector" forward > long-policy.json
          status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$driver" long-policy.json > long.rs 2> long.err || status=$?
          test "$status" -eq 2 && test ! -s long.rs && test -s long.err
          grep -F 'selector must contain 1..=128 bytes' long.err
          for control in left-twice reverse; do
            context=${../../bootstrap/tests/valid}/lowering-"$control.cue"
            cp "$context" "$control-context.cue"
            policy "$owner" "$context" forward | json > "$control.json"
            policy "$owner" "$context" reverse | json > "$control-reversed.json"
            cmp "$control.json" "$control-reversed.json"
            cmp "$control.json" ${jsonExports pkgs}/tests/valid/lowering-"$control.json"
            generate ${jsonExports pkgs}/evaluation.json "$control.json" > "evaluation-$control.rs"
            cmp "evaluation-$control.rs" ${../../bootstrap/generated}/evaluation-"$control.rs"
          done
          if policy selected.cue ${../../bootstrap/tests/valid/lowering-left-twice.cue} forward > selected-conflict.json 2> selected-conflict.err; then exit 1; fi
          test ! -s selected-conflict.json && test -s selected-conflict.err
          grep -F 'conflicting values' selected-conflict.err
          # Both mixed cursor contexts retain the existing output exactly.
          for state in last-next first-previous; do
            case "$state" in last-next) initial=lastRule; advance=nextRule ;; first-previous) initial=firstRule; advance=previousRule ;; esac
            printf '%s\n' 'package bootstrap' "#InstructionLowering & {module:[_,{op:\"$initial\"},{body:[_,_,_,_,{op:\"$advance\"}]},_]}" > "$state.cue"
            policy "$owner" "$state.cue" forward | json > "$state.json"
            policy selected.cue "$state.cue" reverse | json > "$state-selected.json"
            cmp "$state.json" "$state-selected.json"
          done
          generate ${jsonExports pkgs}/evaluation.json canonical.json > evaluation.rs
          cmp evaluation.rs ${../../bootstrap/generated/evaluation.rs}
          generate ${jsonExports pkgs}/tests/valid/evaluation-left-twice.json canonical.json > rules-left-twice.rs
          cmp rules-left-twice.rs ${../../bootstrap/generated/evaluation-left-twice.rs}
          generate ${../../bootstrap/executor/tests/fixtures/codegen-escaped-rules.json} canonical.json > escaped.rs
          cmp escaped.rs ${../../bootstrap/generated/codegen-escaped-rules.rs}
          for policyName in ${loweringNames}; do
            name=$(basename "$policyName")
            env -i HOME="$PWD/home" PATH=/nonexistent "$driver" ${jsonExports pkgs}/"$policyName.json" | format > "$name.rs"
            cmp "$name.rs" ${../../bootstrap/generated}/"$name.rs"
          done
          cp -R . "$out/evidence"
          rm -rf "$out/evidence/home"
          cp ${nativeInstructionRightFieldBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionRightFieldBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionRightField pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionOperandSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/instruction-right-operand.cue
            ../../bootstrap/instruction-right-operand-export.cue
          ];
        };
      nativeInstructionRightOperand =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-right-operand" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          owner=${instructionOperandSources pkgs}/instruction-right-operand.cue
          root=${instructionOperandSources pkgs}/instruction-right-operand-export.cue
          right=${nativeInstructionRightField pkgs}/right-field.json
          for input in "$owner" "$root" "$right"; do test "$(wc -c < "$input")" -le 65536; done
          cp "$root" adapter.cue
          chmod u+w adapter.cue
          printf '\n#InstructionRightField: ' >> adapter.cue
          cat "$right" >> adapter.cue
          test "$(wc -c < adapter.cue)" -le 65536
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$owner" adapter.cue > first.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" adapter.cue "$owner" > reversed.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$owner" adapter.cue > repeated.json
          for result in first reversed repeated; do
            test "$(wc -c < "$result.json")" -le 65536
            cmp first.json "$result.json"
          done
          cp first.json "$out/right-operand.json"
        '';
      nativeInstructionRightOperandBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionRightOperand pkgs).drvPath
          ];
        };
      nativeInstructionRightOperandVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-right-operand-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionRightOperandBuildClosure pkgs}/store-paths; then
              echo "forbidden operand construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionOperandSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          compiler=${executor pkgs}/bin/bootstrap-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for tool in "$exporter" "$oracle" "$driver" "$compiler" "$jq" "$rustfmt"; do
            test -f "$tool" && test -r "$tool" && test -x "$tool"
            case "$tool" in /*) ;; *) exit 1 ;; esac
          done
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          query() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$@"; }
          json() { query -S .; }
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          place() { cue import -f -p bootstrap -l "$1:" -o "$3" "$2"; }
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          right=${../../bootstrap/instruction-right-field.cue}
          operand=${../../bootstrap/instruction-right-operand.cue}
          root=${../../bootstrap/instruction-right-operand-export.cue}
          rightRoot=${../../bootstrap/instruction-right-field-export.cue}
          unary=${../../bootstrap/instruction-unary-tables.cue}
          text=${../../bootstrap/instruction-module-text.cue}
          first=${../../bootstrap/instruction-module-first.cue}
          step=${../../bootstrap/instruction-module-step.cue}
          selector=${../../bootstrap/lowering.cue}
          ${instructionCoreConsumer}
          cue export ${../../bootstrap/native-policy-package.cue} ${../../bootstrap/instruction-binary-tables.cue} > binary.json
          place '#InstructionBinaryTables' binary.json binary-placed.cue
          binary_gate() { cue export "$schema" "$bridge" binary-placed.cue -e '#InstructionBinaryTables' > /dev/null; }
          right_gate() { cue export "$schema" "$bridge" "$right" -e '#InstructionRightField' > /dev/null; }
          isolated() { cue "$1" "$schema" "$bridge" "$right" "''${@:2}" -e '#InstructionRightOperand'; }
          operand_gate() { isolated export "$operand" > /dev/null; }
          group=${../../bootstrap/instruction-operand-cases.cue}
          group_gate() { cue export "$schema" "$bridge" "$right" "$operand" "$group" -e '#InstructionOperandCases' > /dev/null; }
          ${instructionRuntimeConsumer}
          policy() {
            mode=$1; context=$2; order=$3
            core_gate || return
            binary_gate || return
            providers=()
            case "$mode" in
              accepted) right_gate || return; operand_gate || return; group_gate || return; runtime_gate binary-placed.cue || return; providers=("$right" "$operand" "$group" "$runtime") ;;
              backfill) right_gate || return; providers=("$right" "$group" "$runtime") ;;
              missing-right) providers=("$operand" "$group" "$runtime") ;;
              legacy) ;;
              *) return 1 ;;
            esac
            module_gate "$text" "$first" "$step" || return
            moduleInputs=("$module")
            if test "$mode" = legacy; then moduleInputs=(); fi
            touch full-export-started
            if test "$order" = reverse; then
              cue export "''${moduleInputs[@]}" "$context" "$step" "$first" "$text" "$unary" binary-placed.cue "''${providers[@]}" "$core" "$bridge" "$schema"
            else
              cue export "''${moduleInputs[@]}" "$schema" "$bridge" "$core" "''${providers[@]}" binary-placed.cue "$unary" "$text" "$first" "$step" "$context"
            fi
          }
          # Restore ALL original cases/L before the existing R/Right inverse.
          ${instructionGroupInverse}
          restore_module "$bridge" module-original.cue
          restore_runtime module-original.cue runtime-original.cue
          restore_group runtime-original.cue group-original.cue
          body=$(tail -n 1 "$operand"); body="''${body#*: }"
          substitute group-original.cue old-bridge.cue \
            --replace-fail "let R = #InstructionRightOperand" "let R = $body" \
            --replace-fail $'#InstructionRightOperand: close({op: string, value: close({op: string, value: close({op: string, field: string})})})\n' "" \
            --replace-fail $'#InstructionRightField: close({field: string})\n' "" \
            --replace-fail '#InstructionRightField.field' '*"right" | "left"'
          originalBridge=$bridge; originalRight=$right; originalOperand=$operand
          # Both independently native stages; the adapter retains the WHOLE Right.
          materialize() {
            native "$right" "$rightRoot" > "$1/right.json"
            cp "$root" "$1/adapter.cue"
            chmod u+w "$1/adapter.cue"
            printf '\n#InstructionRightField: ' >> "$1/adapter.cue"
            cat "$1/right.json" >> "$1/adapter.cue"
            for input in "$right" "$rightRoot" "$operand" "$root" "$1/right.json" "$1/adapter.cue"; do
              test "$(wc -c < "$input")" -le 65536
            done
            native "$operand" "$1/adapter.cue" > "$1/native.json"
            native "$1/adapter.cue" "$operand" > "$1/reverse.json"
            native "$operand" "$1/adapter.cue" > "$1/repeat.json"
            for result in native reverse repeat; do
              test "$(wc -c < "$1/$result.json")" -le 65536
              cmp "$1/native.json" "$1/$result.json"
            done
          }
          printf '%s\n' '{"formatVersion":1,"rules":{"sum":{"result":{"op":"sum","left":{"op":"int"},"right":{"op":"bool"}}}}}' > tiny.json
          expected_rust() {
            value=$1
            rightExpression="(Ok::<_, Error>(runtime::Value::$value))?"
            if test "$2" = group; then rightExpression="((Ok::<_, Error>(runtime::Value::$value)))"; fi
            format <<RUST
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              match runtime::kind(node)? {
                  "sum" => runtime::sum((Ok::<_, Error>(runtime::Value::Int))?, $rightExpression),
                  kind => Err(Error::MissingRule { kind: kind.to_string() }),
              }
          }
          RUST
          }
          for variant in canonical right-owner operand-owner; do
            mkdir "$variant"
            cp "$originalRight" "$variant/right.cue"
            cp "$originalOperand" "$variant/operand.cue"
            cp "$root" "$variant/root.cue"
            cp "$rightRoot" "$variant/right-root.cue"
            expected=right; rustValue=Bool; wrapperOp=try
            case "$variant" in
              right-owner) substituteInPlace "$variant/right.cue" --replace-fail '*"right" | "left"' '"right" | *"left"'; expected=left; rustValue=Int ;;
              operand-owner) substituteInPlace "$variant/operand.cue" --replace-fail 'op: "try"' 'op: "group"'; wrapperOp=group ;;
            esac
            right="$variant/right.cue"; operand="$variant/operand.cue"
            materialize "$variant"
            printf '{"op":"%s","value":{"op":"group","value":{"op":"lower","field":"%s"}}}\n' "$wrapperOp" "$expected" | json > "$variant/expected.json"
            json < "$variant/native.json" > "$variant/sorted.json"
            cmp "$variant/expected.json" "$variant/sorted.json"
            cue export "$right" "$operand" "$root" > "$variant/go.json"
            json < "$variant/go.json" > "$variant/go-sorted.json"
            cmp "$variant/expected.json" "$variant/go-sorted.json"
            policy accepted "$selector" forward | json > "$variant/raw-policy.json"
            place '#InstructionRightOperand' "$variant/native.json" "$variant/placed.cue"
            operand="$variant/placed.cue"
            policy accepted "$selector" reverse | json > "$variant/native-policy.json"
            cmp "$variant/raw-policy.json" "$variant/native-policy.json"
            query --slurpfile leaf "$variant/expected.json" -e '[.cases.binaryRuntime.parts[4],.cases.sum.args[1],.cases.less.args[1],.cases.lessEqual.args[1],.cases.greater.args[1],.cases.greaterEqual.args[1],.cases.notEqual.args[1],.cases.remainder.args[1],.cases.unify.parts[1].args[1]] == [range(9)|$leaf[0]]' "$variant/native-policy.json" > /dev/null
            expected_rust "$rustValue" "$wrapperOp" > "$variant/expected.rs"
            env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json "$variant/native-policy.json" | format > "$variant/actual.rs"
            env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json "$variant/raw-policy.json" | format > "$variant/raw-actual.rs"
            cmp "$variant/expected.rs" "$variant/actual.rs"
            cmp "$variant/expected.rs" "$variant/raw-actual.rs"
          done
          cmp canonical/native.json ${nativeInstructionRightOperand pkgs}/right-operand.json
          test "$(wc -c < canonical/native.json)" -eq 120
          test "$(wc -c < canonical/go.json)" -eq 150
          right=$originalRight; operand=$originalOperand
          for context in "$selector" ${../../bootstrap/tests/valid/lowering-left-twice.cue} ${../../bootstrap/tests/valid/lowering-reverse.cue}; do
            policy accepted "$context" forward | json > current.json
            bridge=old-bridge.cue
            policy legacy "$context" reverse | json > original.json
            bridge=$originalBridge
            cmp current.json original.json
          done
          consume_operand() { policy accepted "$selector" forward; }
          ${instructionOperandConsumerProbe}
          consume_core() { policy accepted "$selector" forward; }
          ${instructionRuntimeConsumerProbe}
          isolated eval > /dev/null
          if isolated export > absent.json 2> absent.err; then exit 1; fi
          test ! -s absent.json && test -s absent.err
          for invalid in '{op:7}' '{op:"unknownInstructionOpcode",value:{op:"group",value:{op:"lower",field:"right"}}}' '{op:"try",value:{op:"group",value:{op:"lower",field:"right",extra:1}}}'; do
            printf '%s\n' 'package bootstrap' "#InstructionRightOperand: $invalid" > invalid.cue
            operand=invalid.cue
            rm -f full-export-started
            if policy accepted "$selector" forward > invalid.json 2> invalid.err; then exit 1; fi
            test ! -s invalid.json && test -s invalid.err
          done
          operand=$originalOperand
          cp "$right" raw-marked.cue
          chmod u+w raw-marked.cue
          printf '\n#InstructionRightOperand\n' >> raw-marked.cue
          status=0
          native raw-marked.cue "$operand" > unsupported.json 2> unsupported.err || status=$?
          test "$status" -eq 2 && test ! -s unsupported.json && test -s unsupported.err
          grep -F 'scalar projection document profile' unsupported.err
          native "$operand" canonical/adapter.cue > recovered.json
          cmp recovered.json canonical/native.json
          for leaf in '[]' '["op"]' '["value","op"]' '["value","value","op"]' '["value","value","field"]'; do
            query --argjson leaf "$leaf" 'if $leaf==[] then {} else delpaths([$leaf]) end' canonical/expected.json > missing.json
            place '#InstructionRightOperand' missing.json missing.cue
            isolated eval missing.cue > /dev/null
            if isolated export missing.cue > missing.out 2> missing.err; then exit 1; fi
            test ! -s missing.out && test -s missing.err
          done
          # Genuine whole-body backfill and missing Right: explicitly bypass BOTH
          # dependent gates, keeping independent Core/Binary isolation live.
          query -n '{op:"try",value:{op:"group",value:{op:"lower",field:"left"}}}' > left.json
          leaf=$(query -c . left.json)
          printf '%s\n' 'package bootstrap' '#InstructionLowering & {cases:{' \
            "binaryRuntime:{parts:[_,_,_,_,$leaf,_]}" \
            "sum:{args:[_,$leaf]},less:{args:[_,$leaf]},lessEqual:{args:[_,$leaf]}" \
            "greater:{args:[_,$leaf]},greaterEqual:{args:[_,$leaf]},notEqual:{args:[_,$leaf]}" \
            "remainder:{args:[_,$leaf]},unify:{parts:[_,{args:[_,$leaf]},_]}" '}}' > all.cue
          policy backfill all.cue forward | json > backfill.json
          policy missing-right all.cue reverse | json > missing-right.json
          cmp backfill.json missing-right.json
          query --slurpfile leaf left.json -e '[.cases.binaryRuntime.parts[4],.cases.sum.args[1],.cases.less.args[1],.cases.lessEqual.args[1],.cases.greater.args[1],.cases.greaterEqual.args[1],.cases.notEqual.args[1],.cases.remainder.args[1],.cases.unify.parts[1].args[1]] == [range(9)|$leaf[0]]' backfill.json > /dev/null
          query -n '{op:"try",value:{op:"group",value:{op:"lower",field:([range(129)|"x"]|join(""))}}}' > long.json
          place '#InstructionRightOperand' long.json long.cue
          operand=long.cue
          policy accepted "$selector" forward > long-policy.json
          status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$driver" long-policy.json > long.rs 2> long.err || status=$?
          test "$status" -eq 2 && test ! -s long.rs && test -s long.err
          grep -F 'selector must contain 1..=128 bytes' long.err
          cp -R . "$out/evidence"
          rm -rf "$out/evidence/home"
          cp ${nativeInstructionRightOperandBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionRightOperandBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionRightOperand pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionOperandCaseSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/instruction-operand-cases.cue
            ../../bootstrap/instruction-operand-cases-export.cue
          ];
        };
      nativeInstructionOperandCases =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-operand-cases" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          owner=${instructionOperandCaseSources pkgs}/instruction-operand-cases.cue
          root=${instructionOperandCaseSources pkgs}/instruction-operand-cases-export.cue
          operand=${nativeInstructionRightOperand pkgs}/right-operand.json
          for input in "$owner" "$root" "$operand"; do test "$(wc -c < "$input")" -le 65536; done
          cp "$root" adapter.cue
          chmod u+w adapter.cue
          printf '\n#InstructionRightOperand: ' >> adapter.cue
          cat "$operand" >> adapter.cue
          test "$(wc -c < adapter.cue)" -le 65536
          for order in forward reverse; do
            for repeat in first second; do
              inputs=("$owner" adapter.cue)
              if test "$order" = reverse; then inputs=(adapter.cue "$owner"); fi
              env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "''${inputs[@]}" > "$order-$repeat.json"
              test "$(wc -c < "$order-$repeat.json")" -le 65536
              cmp forward-first.json "$order-$repeat.json"
            done
          done
          cp forward-first.json "$out/operand-cases.json"
        '';
      nativeInstructionOperandCasesBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionOperandCases pkgs).drvPath
          ];
        };
      nativeInstructionOperandCasesVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-operand-cases-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${../../bootstrap/schema.cue} ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionOperandCasesBuildClosure pkgs}/store-paths; then
              echo "forbidden Group construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionOperandCaseSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          compiler=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for tool in "$exporter" "$oracle" "$compiler" "$driver" "$jq" "$rustfmt"; do
            test -f "$tool" && test -r "$tool" && test -x "$tool"
            case "$tool" in /*) ;; *) exit 1 ;; esac
          done
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          query() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$@"; }
          json() { query -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          place() { cue import -f -p bootstrap -l "$1:" -o "$3" "$2"; }
          bounded() { for input in "$@"; do test "$(wc -c < "$input")" -le 65536; done; }
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          groupRoot=${instructionOperandCaseSources pkgs}/instruction-operand-cases-export.cue
          operandRoot=${instructionOperandSources pkgs}/instruction-right-operand-export.cue
          rightRoot=${../../bootstrap/instruction-right-field-export.cue}
          ${instructionCoreConsumer}
          place '#InstructionBinaryTables' ${nativeInstructionBinaryTables pkgs}/instruction-binary-tables.json binary.cue
          # Independent complete Group expansion, never read back from a producer.
          expected_group() {
            query -n --arg left "$1" --arg right "$2" '
              def wrap($f): {op:"try",value:{op:"group",value:{op:"lower",field:$f}}};
              def call($p): {op:"call",path:$p,args:[wrap($left),wrap($right)]};
              {leftOperand:wrap($left),cases:{
                sum:call("runtime::sum"),less:call("runtime::less"),
                lessEqual:call("runtime::less_equal"),greater:call("runtime::greater"),
                greaterEqual:call("runtime::greater_equal"),notEqual:call("runtime::not_equal"),
                remainder:call("runtime::remainder"),unify:{op:"concat",parts:[
                  {op:"text",value:"(|_context: &mut runtime::EvaluationContext| "},
                  call("runtime::unify"),{op:"text",value:")(_context)"}]}}}' | json
          }
          materialize() {
            native "$right" "$rightRoot" > "$variant/right.json"
            printf '%s\n' '{"field":"right"}' | json > "$variant/expected-right.json"
            json < "$variant/right.json" > "$variant/sorted-right.json"
            cmp "$variant/expected-right.json" "$variant/sorted-right.json"
            cp "$operandRoot" "$variant/operand-adapter.cue"
            chmod u+w "$variant/operand-adapter.cue"
            printf '\n#InstructionRightField: ' >> "$variant/operand-adapter.cue"
            cat "$variant/right.json" >> "$variant/operand-adapter.cue"
            bounded "$right" "$rightRoot" "$operand" "$operandRoot" "$variant/right.json" "$variant/operand-adapter.cue"
            for order in forward reverse; do
              for repeat in first second; do
                inputs=("$operand" "$variant/operand-adapter.cue")
                if test "$order" = reverse; then inputs=("$variant/operand-adapter.cue" "$operand"); fi
                native "''${inputs[@]}" > "$variant/operand-$order-$repeat.json"
                bounded "$variant/operand-$order-$repeat.json"
                cmp "$variant/operand-forward-first.json" "$variant/operand-$order-$repeat.json"
              done
            done
            cp "$variant/operand-forward-first.json" "$variant/operand.json"
            printf '{"op":"try","value":{"op":"group","value":{"op":"lower","field":"%s"}}}\n' "$rightField" | json > "$variant/expected-operand.json"
            json < "$variant/operand.json" > "$variant/sorted-operand.json"
            cmp "$variant/expected-operand.json" "$variant/sorted-operand.json"
            cp "$groupRoot" "$variant/group-adapter.cue"
            chmod u+w "$variant/group-adapter.cue"
            printf '\n#InstructionRightOperand: ' >> "$variant/group-adapter.cue"
            cat "$variant/operand.json" >> "$variant/group-adapter.cue"
            bounded "$group" "$groupRoot" "$variant/operand.json" "$variant/group-adapter.cue"
            for order in forward reverse; do
              for repeat in first second; do
                inputs=("$group" "$variant/group-adapter.cue")
                if test "$order" = reverse; then inputs=("$variant/group-adapter.cue" "$group"); fi
                native "''${inputs[@]}" > "$variant/$order-$repeat.json"
                bounded "$variant/$order-$repeat.json"
                cmp "$variant/forward-first.json" "$variant/$order-$repeat.json"
              done
            done
            expected_group "$leftField" "$rightField" > "$variant/expected-group.json"
            json < "$variant/forward-first.json" > "$variant/sorted-group.json"
            cmp "$variant/expected-group.json" "$variant/sorted-group.json"
            cue export "$right" "$operand" "$group" "$groupRoot" > "$variant/go-group.json"
            json < "$variant/go-group.json" > "$variant/sorted-go-group.json"
            cmp "$variant/expected-group.json" "$variant/sorted-go-group.json"
          }
          ${instructionRuntimeConsumer}
          policy() {
            core_gate || return
            cue export "$schema" "$bridge" binary.cue -e '#InstructionBinaryTables' > /dev/null || return
            cue export "$schema" "$bridge" "$right" -e '#InstructionRightField' > /dev/null || return
            cue export "$schema" "$bridge" "$right" "$operand" -e '#InstructionRightOperand' > /dev/null || return
            cue export "$schema" "$bridge" "$right" "$operand" "$group" -e '#InstructionOperandCases' > /dev/null || return
            runtime_gate binary.cue || return
            module_gate ${../../bootstrap/instruction-module-text.cue} ${../../bootstrap/instruction-module-first.cue} ${../../bootstrap/instruction-module-step.cue} || return
            touch full-export-started
            inputs=("$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$runtime" binary.cue
              ${../../bootstrap/instruction-unary-tables.cue} ${../../bootstrap/instruction-module-text.cue}
              ${../../bootstrap/instruction-module-first.cue} ${../../bootstrap/instruction-module-step.cue} "$context")
            if test "$1" = reverse; then
              reversed=(); for input in "''${inputs[@]}"; do reversed=("$input" "''${reversed[@]}"); done
              inputs=("''${reversed[@]}")
            fi
            cue export "''${inputs[@]}"
          }
          expected_rust() {
            format <<RUST
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
              let mut _context = runtime::EvaluationContext::default();
              runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
              match runtime::kind(node)? {
                  "sum" => runtime::sum((Ok::<_, Error>(runtime::Value::$leftValue))?, (Ok::<_, Error>(runtime::Value::$rightValue))?),
                  kind => Err(Error::MissingRule { kind: kind.to_string() }),
              }
          }
          RUST
          }
          printf '%s\n' '{"formatVersion":1,"rules":{"sum":{"result":{"op":"sum","left":{"op":"int"},"right":{"op":"bool"}}}}}' > tiny.json
          for variant in canonical group-owner operand-owner; do
            mkdir "$variant"
            cp ${../../bootstrap/instruction-right-field.cue} "$variant/right.cue"
            cp ${../../bootstrap/instruction-right-operand.cue} "$variant/operand.cue"
            cp ${instructionOperandCaseSources pkgs}/instruction-operand-cases.cue "$variant/group.cue"
            cp ${../../bootstrap/lowering.cue} "$variant/context.cue"
            leftField=left; rightField=right; leftValue=Int; rightValue=Bool
            case "$variant" in
              group-owner)
                substituteInPlace "$variant/group.cue" --replace-fail 'field: "left"' 'field: "right"'
                leftField=right; leftValue=Bool ;;
              operand-owner)
                substituteInPlace "$variant/operand.cue" --replace-fail 'field: #InstructionRightField.field' 'field: "left"'
                rightField=left; rightValue=Int ;;
            esac
            right="$variant/right.cue"; operand="$variant/operand.cue"; group="$variant/group.cue"; context="$variant/context.cue"
            materialize
            for order in forward reverse; do policy "$order" | json > "$variant/raw-$order.json"; done
            place '#InstructionRightField' "$variant/right.json" "$variant/placed-right.cue"
            place '#InstructionRightOperand' "$variant/operand.json" "$variant/placed-operand.cue"
            place '#InstructionOperandCases' "$variant/forward-first.json" "$variant/placed-group.cue"
            right="$variant/placed-right.cue"; operand="$variant/placed-operand.cue"; group="$variant/placed-group.cue"
            for order in forward reverse; do
              policy "$order" | json > "$variant/native-$order.json"
              cmp "$variant/raw-forward.json" "$variant/raw-$order.json"
              cmp "$variant/raw-forward.json" "$variant/native-$order.json"
            done
            # The unchanged remainder comes from the canonical control; ALL changed
            # subtrees and the expected emitted module are independently specified.
            query --slurpfile g "$variant/expected-group.json" --slurpfile r "$variant/expected-operand.json" \
              '.cases += $g[0].cases | .cases.binaryRuntime.parts[2] = $g[0].leftOperand | .cases.binaryRuntime.parts[4] = $r[0]' \
              canonical/raw-forward.json | json > "$variant/expected-policy.json"
            cmp "$variant/expected-policy.json" "$variant/raw-forward.json"
            expected_rust > "$variant/expected.rs"
            for provider in raw native; do
              env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json "$variant/$provider-forward.json" | format > "$variant/$provider-actual.rs"
              cmp "$variant/expected.rs" "$variant/$provider-actual.rs"
            done
          done
          cmp canonical/forward-first.json ${nativeInstructionOperandCases pkgs}/operand-cases.json
          test "$(wc -c < canonical/forward-first.json)" -eq 4549
          test "$(wc -c < canonical/go-group.json)" -eq 6821
          json < ${jsonExports pkgs}/lowering.json > canonical/bundle.json
          cmp canonical/bundle.json canonical/native-forward.json
          right=canonical/placed-right.cue; operand=canonical/placed-operand.cue; group=canonical/placed-group.cue; context=canonical/context.cue
          consume_operand() { policy forward; }
          ${instructionGroupConsumerProbe}
          consume_core() { policy forward; }
          ${instructionRuntimeConsumerProbe}
          for invalid in opcode type shape; do
            case "$invalid" in
              opcode) expression='.cases.sum.op = "notAnInstructionOpcode"' ;;
              type) expression='.cases.sum.op = 7' ;;
              shape) expression='.cases.sum.args = []' ;;
            esac
            query "$expression" canonical/expected-group.json > "$invalid.json"
            place '#InstructionOperandCases' "$invalid.json" "$invalid.cue"
            group="$invalid.cue"
            rm -f full-export-started
            status=0
            policy forward > "$invalid-policy.json" 2> "$invalid.err" || status=$?
            test "$status" -eq 1 && test ! -s "$invalid-policy.json" && test -s "$invalid.err"
            if test "$invalid" = opcode; then test -e full-export-started; else test ! -e full-export-started; fi
          done
          # Raw Group must consume the SAME long R, not a selected right fixture.
          query '.value.value.field = ([range(129)|"x"]|join(""))' canonical/expected-operand.json > long.json
          place '#InstructionRightOperand' long.json long.cue
          operand=long.cue; group=canonical/group.cue
          policy forward > long-policy.json
          status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$driver" long-policy.json > long.rs 2> long.err || status=$?
          test "$status" -eq 2 && test ! -s long.rs && test -s long.err
          grep -F 'selector must contain 1..=128 bytes' long.err
          operand=canonical/placed-operand.cue; group=canonical/placed-group.cue
          policy reverse | json > recovered.json
          cmp recovered.json canonical/native-forward.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$driver" recovered.json > recovered-driver.rs
          test -s recovered-driver.rs
          native canonical/group.cue canonical/group-adapter.cue > recovered-native.json
          cmp recovered-native.json canonical/forward-first.json
          cp -R . "$out/evidence"
          rm -rf "$out/evidence/home"
          cp ${nativeInstructionOperandCasesBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionOperandCasesBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionOperandCases pkgs).drvPath} > "$out/native-build-roots"
        '';
      instructionBinaryRuntimeSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/instruction-binary-runtime.cue
            ../../bootstrap/instruction-binary-runtime-export.cue
          ];
        };
      nativeInstructionBinaryRuntime =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-binary-runtime" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home" "$out"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -f "$exporter" && test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          owner=${instructionBinaryRuntimeSources pkgs}/instruction-binary-runtime.cue
          root=${instructionBinaryRuntimeSources pkgs}/instruction-binary-runtime-export.cue
          binary=${nativeInstructionBinaryTables pkgs}/instruction-binary-tables.json
          group=${nativeInstructionOperandCases pkgs}/operand-cases.json
          operand=${nativeInstructionRightOperand pkgs}/right-operand.json
          for input in "$owner" "$root" "$binary" "$group" "$operand"; do
            test "$(wc -c < "$input")" -le 65536
          done
          cp "$root" adapter.cue
          chmod u+w adapter.cue
          for provider in BinaryTables OperandCases RightOperand; do
            case "$provider" in
              BinaryTables) input=$binary ;;
              OperandCases) input=$group ;;
              RightOperand) input=$operand ;;
            esac
            printf '\n#Instruction%s: ' "$provider" >> adapter.cue
            cat "$input" >> adapter.cue
          done
          test "$(wc -c < adapter.cue)" -le 65536
          test "$(($(wc -c < "$owner") + $(wc -c < adapter.cue)))" -le 65536
          for order in forward reverse; do
            for repeat in first second; do
              inputs=("$owner" adapter.cue)
              if test "$order" = reverse; then inputs=(adapter.cue "$owner"); fi
              env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "''${inputs[@]}" > "$order-$repeat.json"
              test "$(wc -c < "$order-$repeat.json")" -le 65536
              cmp forward-first.json "$order-$repeat.json"
            done
          done
          cp forward-first.json "$out/binary-runtime.json"
        '';
      nativeInstructionBinaryRuntimeBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeInstructionBinaryRuntime pkgs).drvPath
          ];
        };
      nativeInstructionBinaryRuntimeVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-instruction-binary-runtime-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} ${pkgs.cue.src.drvPath} \
            ${../../bootstrap/schema.cue} ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeInstructionBinaryRuntimeBuildClosure pkgs}/store-paths; then
              echo "forbidden Runtime construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${instructionBinaryRuntimeSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          compiler=${executor pkgs}/bin/bootstrap-codegen
          driver=${nativeCodegenCore pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for tool in "$exporter" "$oracle" "$compiler" "$driver" "$jq" "$rustfmt"; do
            test -f "$tool" && test -r "$tool" && test -x "$tool"
            case "$tool" in /*) ;; *) exit 1 ;; esac
          done
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          query() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" "$@"; }
          json() { query -S .; }
          place() { cue import -f -p bootstrap -l "$1:" -o "$3" "$2"; }
          bounded() { for input in "$@"; do test "$(wc -c < "$input")" -le 65536; done; }
          pair() {
            bounded "$1" "$2"
            test "$(($(wc -c < "$1") + $(wc -c < "$2")))" -le 65536
            for order in forward reverse; do
              for repeat in first second; do
                inputs=("$1" "$2")
                if test "$order" = reverse; then inputs=("$2" "$1"); fi
                native "''${inputs[@]}" > "$3-$order-$repeat.json"
                bounded "$3-$order-$repeat.json"
                cmp "$3-forward-first.json" "$3-$order-$repeat.json"
              done
            done
          }
          adapter() {
            local target=$2
            cp "$1" "$target"
            chmod u+w "$target"
            shift 2
            for provider in "$@"; do
              printf '\n#Instruction%s: ' "''${provider%%=*}" >> "$target"
              cat "''${provider#*=}" >> "$target"
            done
          }
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/lowering-fragments.cue}
          root=${instructionBinaryRuntimeSources pkgs}/instruction-binary-runtime-export.cue
          groupRoot=${instructionOperandCaseSources pkgs}/instruction-operand-cases-export.cue
          operandRoot=${instructionOperandSources pkgs}/instruction-right-operand-export.cue
          rightRoot=${../../bootstrap/instruction-right-field-export.cue}
          ${instructionCoreConsumer}
          ${instructionModuleConsumer}
          # Every answer below is independently specified; none seeds native input.
          expected_runtime() {
            query -n --arg left "$runtimeLeft" --arg right "$rightField" --arg opening "$opening" \
              --arg difference "$differenceMethod" --arg index "$indexMethod" '
              def wrap($f): {op:"try",value:{op:"group",value:{op:"lower",field:$f}}};
              def text($s): {op:"text",value:$s};
              {op:"concat",parts:[{op:"select",field:"op",cases:{
                index:text("runtime::"+$index),difference:text("runtime::"+$difference),
                product:text("runtime::product"),divide:text("runtime::divide"),
                modulo:text("runtime::modulo"),quotient:text("runtime::quotient"),
                recordIndex:text("runtime::record_index"),listIndex:text("runtime::list_index"),
                logicalAnd:text("runtime::logical_and"),logicalOr:text("runtime::logical_or"),
                equal:text("runtime::equal")}},text($opening),wrap($left),text(", "),wrap($right),text(")")]}' | json
          }
          policy() {
            core_gate || return
            cue export "$schema" "$bridge" "$right" -e '#InstructionRightField' > /dev/null || return
            cue export "$schema" "$bridge" "$right" "$operand" -e '#InstructionRightOperand' > /dev/null || return
            cue export "$schema" "$bridge" "$right" "$operand" "$group" -e '#InstructionOperandCases' > /dev/null || return
            cue export "$schema" "$bridge" "$binary" -e '#InstructionBinaryTables' > /dev/null || return
            cue export "$schema" "$bridge" "$right" "$operand" "$group" "$binary" "$runtime" \
              -e '#InstructionBinaryRuntime' > /dev/null || return
            module_gate ${../../bootstrap/instruction-module-text.cue} ${../../bootstrap/instruction-module-first.cue} ${../../bootstrap/instruction-module-step.cue} || return
            touch full-export-started
            inputs=("$module" "$schema" "$bridge" "$core" "$right" "$operand" "$group" "$binary" "$runtime"
              ${../../bootstrap/instruction-unary-tables.cue} ${../../bootstrap/instruction-module-text.cue}
              ${../../bootstrap/instruction-module-first.cue} ${../../bootstrap/instruction-module-step.cue}
              ${../../bootstrap/lowering.cue})
            if test "$1" = reverse; then
              reversed=(); for input in "''${inputs[@]}"; do reversed=("$input" "''${reversed[@]}"); done
              inputs=("''${reversed[@]}")
            fi
            cue export "''${inputs[@]}"
          }
          materialize() {
            pair "$right" "$rightRoot" "$variant/right"
            adapterPath="$variant/operand-adapter.cue"
            adapter "$operandRoot" "$adapterPath" "RightField=$variant/right-forward-first.json"
            pair "$operand" "$adapterPath" "$variant/operand"
            adapterPath="$variant/group-adapter.cue"
            adapter "$groupRoot" "$adapterPath" "RightOperand=$variant/operand-forward-first.json"
            pair "$group" "$adapterPath" "$variant/group"
            pair ${../../bootstrap/native-policy-package.cue} "$variant/binary-owner.cue" "$variant/binary"
            place '#InstructionBinaryTables' "$variant/binary-forward-first.json" "$variant/binary.cue"
            binary="$variant/binary.cue"
            adapterPath="$variant/runtime-adapter.cue"
            adapter "$root" "$adapterPath" "BinaryTables=$variant/binary-forward-first.json" \
              "OperandCases=$variant/group-forward-first.json" "RightOperand=$variant/operand-forward-first.json"
            pair "$runtime" "$adapterPath" "$variant/runtime"
            expected_runtime > "$variant/expected-runtime.json"
            json < "$variant/runtime-forward-first.json" > "$variant/sorted-runtime.json"
            cmp "$variant/expected-runtime.json" "$variant/sorted-runtime.json"
            for order in forward reverse; do
              inputs=("$runtime" "$adapterPath")
              if test "$order" = reverse; then inputs=("$adapterPath" "$runtime"); fi
              cue export "''${inputs[@]}" > "$variant/go-$order.json"
              json < "$variant/go-$order.json" > "$variant/go-sorted.json"
              cmp "$variant/expected-runtime.json" "$variant/go-sorted.json"
            done
          }
          # The unformatted module follows the ORIGINAL header/footer and statement
          # program, including all four blank lines; rustfmt cannot erase the proof.
          expected_rust() {
            cat <<RUST
          // Generated by bootstrap-codegen. Do not edit by hand.
          use bootstrap_executor::{Error, runtime};
          use serde_json::Value;

          pub fn execute(input: &str) -> Result<i64, Error> {
          let mut _context = runtime::EvaluationContext::default();
          runtime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          pub fn execute_json(input: &str) -> Result<Value, Error> {
          let mut _context = runtime::EvaluationContext::default();
          runtime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)
          }

          fn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
          _context.visit(|_context| evaluate_node(node, _context))
          }

          fn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {
          match runtime::kind(node)? {
          "binary" => runtime::$differenceMethod$opening(Ok::<_, Error>(runtime::Value::$binaryLeftValue))?, (Ok::<_, Error>(runtime::Value::$rightValue))?),
          "sum" => runtime::sum((Ok::<_, Error>(runtime::Value::$groupLeftValue))?, (Ok::<_, Error>(runtime::Value::$rightValue))?),
          kind => Err(Error::MissingRule { kind: kind.to_string() }),
          }
          }
          RUST
          }
          printf '%s\n' '{"formatVersion":1,"rules":{"binary":{"result":{"op":"difference","left":{"op":"int"},"right":{"op":"bool"}}},"sum":{"result":{"op":"sum","left":{"op":"int"},"right":{"op":"bool"}}}}}' > tiny.json
          for variant in canonical runtime-punctuation runtime-left binary-method binary-keyswap group-left r-selector; do
            mkdir "$variant"
            cp ${../../bootstrap/instruction-right-field.cue} "$variant/right.cue"
            cp ${../../bootstrap/instruction-right-operand.cue} "$variant/operand.cue"
            cp ${../../bootstrap/instruction-operand-cases.cue} "$variant/group.cue"
            cp ${../../bootstrap/instruction-binary-tables.cue} "$variant/binary-owner.cue"
            cp ${instructionBinaryRuntimeSources pkgs}/instruction-binary-runtime.cue "$variant/runtime.cue"
            runtimeLeft=left; groupLeft=left; rightField=right; opening='('
            binaryLeftValue=Int; groupLeftValue=Int; rightValue=Bool
            differenceMethod=difference; indexMethod=index
            case "$variant" in
              runtime-punctuation)
                substituteInPlace "$variant/runtime.cue" --replace-fail 'value: "("' 'value: "( "'
                opening='( ' ;;
              runtime-left)
                substituteInPlace "$variant/runtime.cue" --replace-fail '#InstructionOperandCases.leftOperand' '#InstructionRightOperand'
                runtimeLeft=right; binaryLeftValue=Bool ;;
              binary-method)
                substituteInPlace "$variant/binary-owner.cue" --replace-fail 'difference:       "difference"' 'difference:       "sum"'
                differenceMethod=sum ;;
              binary-keyswap)
                substituteInPlace "$variant/binary-owner.cue" --replace-fail 'index:       "index"' 'difference:       "index"' \
                  --replace-fail 'difference:       "difference"' 'index:       "difference"'
                differenceMethod=index; indexMethod=difference ;;
              group-left)
                substituteInPlace "$variant/group.cue" --replace-fail 'field: "left"' 'field: "right"'
                runtimeLeft=right; groupLeft=right; binaryLeftValue=Bool; groupLeftValue=Bool ;;
              r-selector)
                substituteInPlace "$variant/operand.cue" --replace-fail '#InstructionRightField.field' '"left"'
                rightField=left; rightValue=Int ;;
            esac
            right="$variant/right.cue"; operand="$variant/operand.cue"; group="$variant/group.cue"; runtime="$variant/runtime.cue"
            materialize
            for order in forward reverse; do policy "$order" | json > "$variant/raw-$order.json"; done
            place '#InstructionRightField' "$variant/right-forward-first.json" "$variant/placed-right.cue"
            place '#InstructionRightOperand' "$variant/operand-forward-first.json" "$variant/placed-operand.cue"
            place '#InstructionOperandCases' "$variant/group-forward-first.json" "$variant/placed-group.cue"
            place '#InstructionBinaryRuntime' "$variant/runtime-forward-first.json" "$variant/placed-runtime.cue"
            right="$variant/placed-right.cue"; operand="$variant/placed-operand.cue"
            group="$variant/placed-group.cue"; runtime="$variant/placed-runtime.cue"
            # Preserve every unaffected sibling from the full canonical control;
            # predict ALL changed subtrees independently, never from variant output.
            query --slurpfile rt "$variant/expected-runtime.json" --arg l "$groupLeft" --arg r "$rightField" '
              def wrap($f): {op:"try",value:{op:"group",value:{op:"lower",field:$f}}};
              .cases.binaryRuntime=$rt[0] |
              (.cases.sum,.cases.less,.cases.lessEqual,.cases.greater,.cases.greaterEqual,
                .cases.notEqual,.cases.remainder,.cases.unify.parts[1]).args=[wrap($l),wrap($r)]' \
              canonical/raw-forward.json | json > "$variant/expected-policy.json"
            for order in forward reverse; do
              policy "$order" | json > "$variant/native-$order.json"
              cmp "$variant/expected-policy.json" "$variant/raw-$order.json"
              cmp "$variant/expected-policy.json" "$variant/native-$order.json"
            done
            expected_rust > "$variant/expected-unformatted.rs"
            if test "$variant" != canonical; then
              if cmp -s canonical/expected-unformatted.rs "$variant/expected-unformatted.rs"; then
                echo "unobservable Runtime source effect: $variant" >&2; exit 1
              fi
            fi
            for provider in raw native; do
              env -i HOME="$PWD/home" PATH=/nonexistent "$compiler" tiny.json "$variant/$provider-forward.json" > "$variant/$provider-unformatted.rs"
              cmp "$variant/expected-unformatted.rs" "$variant/$provider-unformatted.rs"
              env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024 \
                < "$variant/$provider-unformatted.rs" > "$variant/$provider-formatted.rs"
            done
          done
          cmp canonical/runtime-forward-first.json ${nativeInstructionBinaryRuntime pkgs}/binary-runtime.json
          test "$(wc -c < canonical/runtime-forward-first.json)" -eq 1652
          test "$(wc -c < canonical/go-forward.json)" -eq 2286
          json < ${jsonExports pkgs}/lowering.json > canonical/bundle.json
          cmp canonical/bundle.json canonical/native-forward.json
          right=canonical/placed-right.cue; operand=canonical/placed-operand.cue
          group=canonical/placed-group.cue; binary=canonical/binary.cue; runtime=canonical/placed-runtime.cue
          consume_core() { policy forward; }
          ${instructionRuntimeConsumerProbe}
          for invalid in opcode type shape; do
            case "$invalid" in
              opcode) expression='.op = "notAnInstructionOpcode"' ;;
              type) expression='.op = 7' ;;
              shape) expression='.parts = []' ;;
            esac
            query "$expression" canonical/expected-runtime.json > "$invalid.json"
            place '#InstructionBinaryRuntime' "$invalid.json" "$invalid.cue"
            runtime="$invalid.cue"
            rm -f full-export-started
            status=0
            policy forward > "$invalid-policy.json" 2> "$invalid.err" || status=$?
            test "$status" -eq 1 && test ! -s "$invalid-policy.json" && test -s "$invalid.err"
            if test "$invalid" = opcode; then test -e full-export-started; else test ! -e full-export-started; fi
          done
          # A new long R must reach BOTH Group and Runtime: selected old DATA would
          # conceal the dependency. Go accepts this; the REAL typed driver rejects.
          query '.value.value.field = ([range(129)|"x"]|join(""))' canonical/operand-forward-first.json > long.json
          place '#InstructionRightOperand' long.json long.cue
          operand=long.cue; group=canonical/group.cue; runtime=canonical/runtime.cue
          policy forward > long-policy.json
          status=0
          env -i HOME="$PWD/home" PATH=/nonexistent "$driver" long-policy.json > long.rs 2> long.err || status=$?
          test "$status" -eq 2 && test ! -s long.rs && test -s long.err
          grep -F 'selector must contain 1..=128 bytes' long.err
          operand=canonical/placed-operand.cue; group=canonical/placed-group.cue; runtime=canonical/placed-runtime.cue
          policy reverse | json > recovered.json
          cmp recovered.json canonical/native-forward.json
          env -i HOME="$PWD/home" PATH=/nonexistent "$driver" recovered.json > recovered-driver.rs
          test -s recovered-driver.rs
          native canonical/runtime.cue canonical/runtime-adapter.cue > recovered-native.json
          cmp recovered-native.json canonical/runtime-forward-first.json
          cp -R . "$out/evidence"
          rm -rf "$out/evidence/home"
          cp ${nativeInstructionBinaryRuntimeBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeInstructionBinaryRuntimeBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeInstructionBinaryRuntime pkgs).drvPath} > "$out/native-build-roots"
        '';
      prattTableSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/pratt-lowering-tables.cue
          ];
        };
      nativePrattTables =
        pkgs:
        pkgs.runCommand "cuestacean-native-pratt-tables" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -r "$exporter" && test -x "$exporter"
          case "$exporter" in /*) ;; *) exit 1 ;; esac
          for run in first second; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
              ${prattTableSources pkgs}/native-policy-package.cue \
              ${prattTableSources pkgs}/pratt-lowering-tables.cue > "$run.json"
            test "$(wc -c < "$run.json")" -le 65536
          done
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/pratt-lowering-tables.json"
        '';
      nativePrattTablesBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativePrattTables pkgs).drvPath
          ];
        };
      nativePrattTablesVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-pratt-tables-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativePrattTablesBuildClosure pkgs}/store-paths; then
              echo "forbidden Pratt table construction dependency: $forbidden" >&2; exit 1
            fi
          done
          test "$(find ${prattTableSources pkgs} -type f | wc -l)" -eq 2
          test "$(find ${specSources pkgs false} -type f | wc -l)" -eq 10
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-pratt-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$jq" "$rustfmt"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" native-policy-package.cue "$1"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/pratt-lowering-fragments.cue}
          selector=${../../bootstrap/pratt-lowering.cue}
          cp ${prattTableSources pkgs}/*.cue .
          body=${../../bootstrap/pratt-lowering-body.cue}
          add=${../../bootstrap/pratt-add-policy.cue}
          raw() { cue export "$schema" "$bridge" "$selector" "$body" "$add" pratt-lowering-tables.cue "$@"; }
          complete() { cue export "$schema" "$bridge" "$selector" "$body" "$add" "$@"; }
          native pratt-lowering-tables.cue > tables.json
          cmp tables.json ${nativePrattTables pkgs}/pratt-lowering-tables.json
          cue export native-policy-package.cue pratt-lowering-tables.cue | json > oracle-tables.json
          cmp tables.json oracle-tables.json
          raw | json > raw.json
          complete tables.json | json > complete.json
          cmp raw.json complete.json
          cmp complete.json ${jsonExports pkgs}/pratt-lowering.json
          for control in ${prattNames}; do
            generate ${jsonExports pkgs}/"$control.json" complete.json > control.rs
            cmp control.rs ${../../bootstrap/generated}/"$(basename "$control").rs"
          done
          override=${../../bootstrap/tests/valid/pratt-lowering-no-increment.cue}
          raw "$override" | json > raw-override.json
          complete tables.json "$override" | json > override.json
          cmp raw-override.json override.json
          cmp override.json ${jsonExports pkgs}/tests/valid/pratt-lowering-no-increment.json
          generate ${jsonExports pkgs}/pratt.json override.json > override.rs
          cmp override.rs ${../../bootstrap/generated/pratt-no-increment.rs}
          no_export() {
            if complete "$@" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          }
          no_export
          grep -F 'incomplete value' failed.err
          cue eval "$schema" "$bridge" "$selector" "$body" "$add" > /dev/null
          env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -c \
            '([.aliases | keys[] | ["aliases", .]] + [.cases | keys[] | ["cases", .]])[]' tables.json > slots
          test "$(wc -l < slots)" -eq 9
          while read -r slot; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" --argjson slot "$slot" 'delpaths([$slot])' tables.json > missing.json
            no_export missing.json
            grep -F 'incomplete value' failed.err
            cue eval "$schema" "$bridge" "$selector" "$body" "$add" missing.json > /dev/null
          done < slots
          printf '%s\n' '{"formatVersion":1,"body":[{"op":"scanSelector"},{"op":"parseGroup","minimum":{"op":"integer","value":0}}]}' > tiny.json
          # Each variant changes raw source, native DATA and the real compiler's
          # COMPLETE output, predicted independently rather than captured.
          for variant in key method template; do
            group=parse_group; scan=scan_selector
            case "$variant" in
              key) substitute pratt-lowering-tables.cue variant.cue --replace-fail 'parseGroup:' 'temporaryKey:' --replace-fail 'parseNot:' 'parseGroup:' --replace-fail 'temporaryKey:' 'parseNot:'; group=parse_not ;;
              method) substitute pratt-lowering-tables.cue variant.cue --replace-fail '"parse_group"' '"parse_group_changed"'; group=parse_group_changed ;;
              template) substitute pratt-lowering-tables.cue variant.cue --replace-fail 'value: method' 'value: "table_template"'; group=table_template; scan=table_template ;;
            esac
            native variant.cue > variant.json
            if cmp -s variant.json tables.json; then exit 1; fi
            complete variant.json | json > variant-policy.json
            complete variant.cue | json > variant-raw.json
            cmp variant-policy.json variant-raw.json
            generate tiny.json variant-policy.json > "$variant.rs"
            format > "$variant-expected.rs" <<RUST
          // Generated by bootstrap-pratt-codegen. Do not edit by hand.
          use crate::parser::{Error, Frame};
          pub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {
              _frame.step()?;
              _frame.$scan()?;
              _frame.step()?;
              _frame.$group(0u8)?;
              Ok(())
          }
          RUST
            cmp "$variant.rs" "$variant-expected.rs"
          done
          substitute pratt-lowering-tables.cue invalid.cue --replace-fail 'value: method' 'value: int'
          if native invalid.cue > invalid.json 2> invalid.err; then exit 1; fi
          test ! -s invalid.json && test -s invalid.err
          for invalid in opcode shape unbound; do
            case "$invalid" in
              opcode) substitute pratt-lowering-tables.cue invalid.cue --replace-fail 'op: "text", value: method' 'op: "unknownTableOp", value: method' ;;
              shape) substitute pratt-lowering-tables.cue invalid.cue --replace-fail 'value: method' 'value: true' ;;
              unbound) substitute pratt-lowering-tables.cue invalid.cue --replace-fail 'op: "text", value: method' 'op: "get", "name": method' ;;
            esac
            native invalid.cue > invalid.json
            if test "$invalid" = unbound; then complete invalid.json > schema-valid.json
            else no_export invalid.json; cp failed.err "$out/$invalid-schema.err"; fi
            # Replace only these cases in a known COMPLETE policy, so absence of
            # formatVersion or other required slots cannot fake typed rejection.
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" --slurpfile m invalid.json '.cases.selectorAction=$m[0].cases.selectorAction | .cases.parseMinimum=$m[0].cases.parseMinimum' complete.json > malformed.json
            if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" tiny.json malformed.json > invalid.rs 2> invalid.err; then exit 1; fi
            test ! -s invalid.rs && test -s invalid.err
            case "$invalid" in
              opcode) grep -F 'unknown variant `unknownTableOp`' invalid.err ;;
              shape) grep -F 'invalid type: boolean `true`, expected a string' invalid.err ;;
              unbound) grep -F 'unknown binding' invalid.err ;;
            esac
            cp invalid.err "$out/$invalid-typed.err"
          done
          cp tables.json complete.json override.json *-expected.rs key.rs method.rs template.rs "$out/"
          cp ${nativePrattTablesBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativePrattTablesBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativePrattTables pkgs).drvPath} > "$out/native-build-roots"
        '';
      scanLoweringPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/scan-lowering-helpers.cue
            ../../bootstrap/scan-lowering-flow.cue
            ../../bootstrap/scan-lowering-predicates.cue
            ../../bootstrap/scan-advance-policy.cue
          ];
        };
      scanPolicyComposition = pkgs: ''
        compose() {
          test "$(wc -c < "$1")" -le 65536
          test "$(wc -c < "$2")" -le 65536
          env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
            -f ${../../bootstrap/compose-policy.jq} "$1" "$2" > "$3"
          test "$(wc -c < "$3")" -le 65536
        }
        assemble() { compose "$1" "$2" part.json; compose part.json "$3" "$4"; }
      '';
      nativeScanLoweringPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-scan-lowering-policy" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          jq=${pkgs.jq}/bin/jq
          for binary in "$exporter" "$jq"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          ${scanPolicyComposition pkgs}
          for run in first second; do
            for module in flow predicates advance; do
              companion=scan-lowering-helpers
              source=scan-lowering-$module
              if test "$module" = advance; then companion=native-policy-package; source=scan-advance-policy; fi
              env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
                ${scanLoweringPolicySources pkgs}/"$companion.cue" \
                ${scanLoweringPolicySources pkgs}/"$source.cue" > "$run-$module.json"
              test "$(wc -c < "$run-$module.json")" -le 65536
            done
            assemble "$run-flow.json" "$run-predicates.json" "$run-advance.json" "$run.json"
            for order in 'flow predicates advance' 'flow advance predicates' 'predicates flow advance' \
              'predicates advance flow' 'advance flow predicates' 'advance predicates flow'; do
              read -r a b c <<< "$order"
              assemble "$run-$a.json" "$run-$b.json" "$run-$c.json" left.json
              compose "$run-$b.json" "$run-$c.json" right-part.json
              compose "$run-$a.json" right-part.json right.json
              cmp "$run.json" left.json; cmp "$run.json" right.json
            done
          done
          for module in flow predicates advance; do cmp "first-$module.json" "second-$module.json"; done
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/scan-lowering.json"
        '';
      nativeScanLoweringPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeScanLoweringPolicy pkgs).drvPath
          ];
        };
      nativeScanLoweringPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-scan-lowering-policy-verification" { } ''
          set -euo pipefail
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${legacyCodegenSeed pkgs} ${(legacyCodegenSeed pkgs).drvPath} \
            ${codegenSeed pkgs} ${(codegenSeed pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${nativeInstructionCodegen pkgs} ${(nativeInstructionCodegen pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeScanLoweringPolicyBuildClosure pkgs}/store-paths; then
              echo "forbidden scanner construction dependency: $forbidden" >&2; exit 1
            fi
          done
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-scan-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$jq" "$rustfmt"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          ${scanPolicyComposition pkgs}
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/scan-lowering-fragments.cue}
          selector=${../../bootstrap/scan-lowering.cue}
          cp ${scanLoweringPolicySources pkgs}/*.cue .
          helpers=scan-lowering-helpers.cue
          flow=scan-lowering-flow.cue
          predicates=scan-lowering-predicates.cue
          advance=scan-advance-policy.cue
          raw() { cue export "$schema" "$bridge" "$selector" "$helpers" "$flow" "$predicates" "$advance" "$@"; }
          no_export() {
            if cue export "$@" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          }
          native "$helpers" "$flow" > flow.json
          native "$helpers" "$predicates" > predicates.json
          native native-policy-package.cue "$advance" > advance.json
          assemble flow.json predicates.json advance.json policy.json
          cmp policy.json ${nativeScanLoweringPolicy pkgs}/scan-lowering.json
          cmp policy.json ${jsonExports pkgs}/scan-lowering.json
          raw | json > raw.json
          cmp policy.json raw.json
          cue export "$schema" "$bridge" "$selector" policy.json | json > validated.json
          cmp policy.json validated.json
          for missing in "$flow" "$predicates" "$advance"; do
            providers=("$helpers")
            for source in "$flow" "$predicates" "$advance"; do
              if test "$source" != "$missing"; then providers+=("$source"); fi
            done
            no_export "$schema" "$bridge" "$selector" "''${providers[@]}"
            grep -F 'incomplete value' failed.err
            cue eval "$schema" "$bridge" "$selector" "''${providers[@]}" > /dev/null
            cp failed.err "$missing-missing.err"
          done
          no_export "$schema" "$bridge" "$selector" "$flow" "$predicates" "$advance"
          grep -E 'reference .*not found|undefined' failed.err
          cp failed.err helpers-missing.err
          raw ${../../bootstrap/tests/valid/scan-lowering-unicode.cue} | json > unicode.json
          cmp unicode.json ${jsonExports pkgs}/tests/valid/scan-lowering-unicode.json
          no_export "$schema" "$bridge" "$selector" policy.json ${../../bootstrap/tests/valid/scan-lowering-unicode.cue}
          cp failed.err late-override.err
          for machine in ${scannerNames}; do
            generate ${jsonExports pkgs}/"$machine.json" policy.json > scanner.rs
            cmp scanner.rs ${../../bootstrap/generated}/"$(basename "$machine").rs"
          done
          generate ${jsonExports pkgs}/tests/valid/scanner-advance.json unicode.json > unicode.rs
          cmp unicode.rs ${../../bootstrap/generated}/scan-unicode-advance.rs
          # Complete independent expected source; these controls emit, not execute.
          printf '%s\n' '{"formatVersion":1,"body":[{"op":"if","condition":{"op":"hasByte"},"then":[{"op":"setFlag","value":true}],"else":[]},{"op":"advance"}]}' > tiny.json
          for owner in helper flow predicate advance; do
            banner='// Generated by bootstrap-scan-codegen. Do not edit by hand.'
            step='cursor.step()?;'
            condition='cursor.peek().is_some()'
            advancement='cursor.advance()?;'
            case "$owner" in
              helper)
                substitute "$helpers" variant.cue --replace-fail 'cursor.step()?;' '// Helper control.\ncursor.step()?;'
                native variant.cue "$flow" > variant-flow.json
                assemble variant-flow.json predicates.json advance.json variant.json
                step=$'// Helper control.\ncursor.step()?;' ;;
              flow)
                substitute "$flow" variant.cue --replace-fail '// Generated by bootstrap-scan-codegen.' '// Flow control.\n// Generated by bootstrap-scan-codegen.'
                native "$helpers" variant.cue > variant-flow.json
                assemble variant-flow.json predicates.json advance.json variant.json
                banner=$'// Flow control.\n'"$banner" ;;
              predicate)
                substitute "$predicates" variant.cue --replace-fail 'cursor.peek().is_some()' 'cursor.peek().is_none()'
                native "$helpers" variant.cue > variant-predicates.json
                assemble flow.json variant-predicates.json advance.json variant.json
                condition='cursor.peek().is_none()' ;;
              advance)
                substitute "$advance" variant.cue --replace-fail 'cursor.advance()' 'cursor.advance_char()'
                native native-policy-package.cue variant.cue > variant-advance.json
                assemble flow.json predicates.json variant-advance.json variant.json
                advancement='cursor.advance_char()?;' ;;
            esac
            case "$owner" in
              helper) cue export "$schema" "$bridge" "$selector" variant.cue "$flow" "$predicates" "$advance" ;;
              flow) cue export "$schema" "$bridge" "$selector" "$helpers" variant.cue "$predicates" "$advance" ;;
              predicate) cue export "$schema" "$bridge" "$selector" "$helpers" "$flow" variant.cue "$advance" ;;
              advance) cue export "$schema" "$bridge" "$selector" "$helpers" "$flow" "$predicates" variant.cue ;;
            esac | json > variant-raw.json
            cmp variant.json variant-raw.json
            generate tiny.json variant.json > "$owner.rs"
            format > "$owner-expected.rs" <<RUST
          $banner
          use crate::scanner::{Cursor, Error, Scan};
          pub fn scan(source: &str, offset: usize) -> Result<Scan, Error> {
              let mut cursor = Cursor::new(source, offset)?;
              $step
              if $condition {
                  $step
                  cursor.set_flag(true);
              }
              cursor.step()?;
              $advancement
              Ok(cursor.finish())
          }
          RUST
            cmp "$owner.rs" "$owner-expected.rs"
          done
          for invalid in opcode shape; do
            if test "$invalid" = opcode; then
              substitute "$flow" invalid.cue --replace-fail 'op: "concat"' 'op: "unknownNativeControl"'
            else
              substitute "$flow" invalid.cue --replace-fail 'formatVersion: 1' 'formatVersion: "wrong"'
            fi
            native "$helpers" invalid.cue > invalid-flow.json
            assemble invalid-flow.json predicates.json advance.json invalid.json
            no_export "$schema" "$bridge" "$selector" "$helpers" invalid.cue "$predicates" "$advance"
            cp failed.err "$invalid-schema.err"
            if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" tiny.json invalid.json > invalid.rs 2> invalid.err; then exit 1; fi
            test ! -s invalid.rs && test -s invalid.err
            cp invalid.err "$invalid-typed.err"
          done
          # Shape-valid DATA can still violate the consumer's lexical scope.
          substitute "$helpers" unbound.cue --replace-fail 'name: "then"' 'name: "unbound"'
          native unbound.cue "$flow" > unbound-flow.json
          assemble unbound-flow.json predicates.json advance.json unbound.json
          cue export "$schema" "$bridge" "$selector" unbound.cue "$flow" "$predicates" "$advance" | json > unbound-raw.json
          cmp unbound.json unbound-raw.json
          if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" tiny.json unbound.json > unbound.rs 2> unbound-typed.err; then exit 1; fi
          test ! -s unbound.rs && test -s unbound-typed.err
          grep -F 'unknown binding' unbound-typed.err
          cp policy.json unicode.rs *-expected.rs helper.rs flow.rs predicate.rs advance.rs *-missing.err late-override.err *-schema.err *-typed.err "$out/"
          cp ${nativeScanLoweringPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeScanLoweringPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeScanLoweringPolicy pkgs).drvPath} > "$out/native-build-roots"
        '';
      metaLoweringPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/meta-lowering-body.cue
            ../../bootstrap/meta-text-alias.cue
            ../../bootstrap/meta-next-rule-alias.cue
          ];
        };
      nativeMetaLoweringPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-meta-lowering-policy" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          jq=${pkgs.jq}/bin/jq
          for binary in "$exporter" "$jq"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          compose() {
            test "$(wc -c < "$1")" -le 65536
            test "$(wc -c < "$2")" -le 65536
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
              -f ${../../bootstrap/compose-policy.jq} "$1" "$2" > "$3"
            test "$(wc -c < "$3")" -le 65536
          }
          for run in first second; do
            for module in meta-lowering-body meta-text-alias meta-next-rule-alias; do
              # Each complete two-file invocation retains its unchanged ledger.
              env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
                ${metaLoweringPolicySources pkgs}/native-policy-package.cue \
                ${metaLoweringPolicySources pkgs}/"$module.cue" > "$run-$module.json"
              test "$(wc -c < "$run-$module.json")" -le 65536
            done
            compose "$run-meta-lowering-body.json" "$run-meta-text-alias.json" "$run-part.json"
            compose "$run-part.json" "$run-meta-next-rule-alias.json" "$run.json"
            compose "$run-meta-next-rule-alias.json" "$run-meta-text-alias.json" "$run-aliases.json"
            compose "$run-aliases.json" "$run-meta-lowering-body.json" "$run-reversed.json"
            cmp "$run.json" "$run-reversed.json"
          done
          for module in meta-lowering-body meta-text-alias meta-next-rule-alias; do
            cmp "first-$module.json" "second-$module.json"
          done
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/meta-lowering.json"
        '';
      nativeMetaLoweringPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeMetaLoweringPolicy pkgs).drvPath
          ];
        };
      nativeMetaLoweringPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-meta-lowering-policy-verification" { } ''
          set -euo pipefail
          # Schema/oracle/compiler are verifiers, never constructor inputs.
          for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
            ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
            ${executor pkgs} ${(executor pkgs).drvPath} \
            ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
            ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${nativeMetaLoweringPolicyBuildClosure pkgs}/store-paths; then
              echo "forbidden meta construction dependency: $forbidden" >&2
              exit 1
            fi
          done
          mkdir -p "$out" "$PWD/probe/home"
          cd probe
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          oracle=${pkgs.cue}/bin/cue
          codegen=${executor pkgs}/bin/bootstrap-lower-codegen
          jq=${pkgs.jq}/bin/jq
          rustfmt=${pkgs.rustfmt}/bin/rustfmt
          for binary in "$exporter" "$oracle" "$codegen" "$jq" "$rustfmt"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
          cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
          json() { env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -S .; }
          format() { env -i HOME="$PWD/home" PATH=/nonexistent "$rustfmt" --edition 2024; }
          generate() { env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | format; }
          compose() {
            test "$(wc -c < "$1")" -le 65536
            test "$(wc -c < "$2")" -le 65536
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
              -f ${../../bootstrap/compose-policy.jq} "$1" "$2" > "$3"
            test "$(wc -c < "$3")" -le 65536
          }
          assemble() {
            compose "$1" "$2" part.json
            compose part.json "$3" "$4"
          }
          no_export() {
            if cue export "$@" > failed.json 2> failed.err; then exit 1; fi
            test ! -s failed.json && test -s failed.err
          }
          schema=${../../bootstrap/schema.cue}
          bridge=${../../bootstrap/meta-lowering-fragments.cue}
          selector=${../../bootstrap/meta-lowering.cue}
          cp ${metaLoweringPolicySources pkgs}/native-policy-package.cue package.cue
          cp ${metaLoweringPolicySources pkgs}/meta-lowering-body.cue body.cue
          cp ${metaLoweringPolicySources pkgs}/meta-text-alias.cue text.cue
          cp ${metaLoweringPolicySources pkgs}/meta-next-rule-alias.cue next.cue
          for module in body text next; do
            native package.cue "$module.cue" > "$module.json"
            native "$module.cue" package.cue > "$module-reversed.json"
            cmp "$module.json" "$module-reversed.json"
          done
          printf '%s\n' '{"aliases":{"text":"emitValue"}}' | json > expected-text.json
          printf '%s\n' '{"aliases":{"nextRule":"emitModuleAction"}}' | json > expected-next.json
          cmp text.json expected-text.json
          cmp next.json expected-next.json
          assemble body.json text.json next.json policy.json
          compose next.json text.json aliases.json
          compose aliases.json body.json reversed.json
          cmp policy.json reversed.json
          cmp policy.json ${nativeMetaLoweringPolicy pkgs}/meta-lowering.json
          cmp policy.json ${jsonExports pkgs}/meta-lowering.json
          cue export "$schema" "$bridge" body.cue text.cue next.cue "$selector" | json > raw.json
          cmp policy.json raw.json
          cue export "$selector" next.cue text.cue body.cue "$bridge" "$schema" | json > reversed-raw.json
          cmp policy.json reversed-raw.json
          cue export "$schema" "$bridge" "$selector" policy.json | json > validated.json
          cmp policy.json validated.json
          for missing in body text next; do
            providers=()
            for module in body text next; do
              if test "$module" != "$missing"; then providers+=("$module.cue"); fi
            done
            no_export "$schema" "$bridge" "$selector" "''${providers[@]}"
            grep -F 'incomplete value' failed.err
            cue eval "$schema" "$bridge" "$selector" "''${providers[@]}" > /dev/null
            cp failed.err "$missing-missing.err"
          done
          for policy in lowering pratt-lowering unify-lowering ast-lowering scan-lowering meta-lowering; do
            generate ${jsonExports pkgs}/"$policy.json" policy.json > "$policy.rs"
            cmp "$policy.rs" ${../../bootstrap/generated}/"$policy.rs"
          done

          # Source-to-emitted-source control, with a wholly hand-authored driver.
          substitute body.cue variant-body.cue --replace-fail \
            '// Generated lowering driver.' '// Native body control.\n// Generated lowering driver.'
          native package.cue variant-body.cue > variant-body.json
          assemble variant-body.json text.json next.json variant.json
          cue export "$schema" "$bridge" "$selector" variant-body.cue text.cue next.cue | json > raw-variant.json
          cmp variant.json raw-variant.json
          printf '%s\n' '{"formatVersion":1,"cases":{"tiny":{"op":"text","value":"tiny"}}}' > tiny.json
          generate tiny.json variant.json > tiny.rs
          format > expected-tiny.rs <<'RUST'
          // Native body control.
          // Generated lowering driver. Do not edit.
          use bootstrap_executor::lowering::{self, Context, Driver, Error};
          use serde_json::Value;

          pub struct Compiled;

          impl Driver for Compiled {
              fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
                  match lowering::instruction_op(_node)? {
                      "tiny" => ctx.text("tiny"),
                      op => Err(lowering::missing_case(op)),
                  }
              }
          }

          pub fn lower(node: &Value) -> Result<String, Error> {
              Context::run(node, &Compiled)
          }
          RUST
          cmp tiny.rs expected-tiny.rs

          # Ordinary root context supplies the new typed target BEFORE decoding.
          # Native inputs contain no #MetaLowering reference and retain both arms.
          printf '%s\n' 'package bootstrap' \
            '{aliases:{text:"emitForced"},cases:{emitForced:{op:"text",value:"ctx.text(\"forced\")"}}}' > text-context.cue
          printf '%s\n' 'package bootstrap' \
            '{aliases:{nextRule:"emitStepBack"},cases:{emitStepBack:{op:"text",value:"frame.step()?; frame.previous_rule()?;\n"}}}' > next-context.cue
          for which in text next; do
            native "$which.cue" "$which-context.cue" > forced-alias.json
            if test "$which" = text; then
              assemble body.json forced-alias.json next.json forced.json
              fixture=meta-lowering-forced
              input=lowering-numbers
              golden=lowering-meta-forced
            else
              assemble body.json text.json forced-alias.json forced.json
              fixture=meta-driver-previous
              input=meta-driver-keys
              golden=meta-driver-previous
            fi
            cmp forced.json ${jsonExports pkgs}/tests/valid/"$fixture.json"
            cue export "$schema" "$bridge" "$selector" body.cue text.cue next.cue \
              "$which-context.cue" | json > raw-forced.json
            cmp forced.json raw-forced.json
            generate ${jsonExports pkgs}/tests/valid/"$input.json" forced.json > forced.rs
            cmp forced.rs ${../../bootstrap/generated}/"$golden.rs"
            no_export "$schema" "$bridge" "$selector" policy.json "$which-context.cue"
            cp failed.err "$which-late-override.err"
          done

          for invalid in opcode shape; do
            if test "$invalid" = opcode; then
              substitute body.cue invalid.cue --replace-fail 'op: "concat"' 'op: "unknownNativeControl"'
            else
              substitute body.cue invalid.cue --replace-fail 'formatVersion: 1' 'formatVersion: "wrong"'
            fi
            native package.cue invalid.cue > invalid-body.json
            assemble invalid-body.json text.json next.json invalid.json
            no_export "$schema" "$bridge" "$selector" invalid.cue text.cue next.cue
            if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" \
              tiny.json invalid.json > invalid.rs 2> invalid.err; then exit 1; fi
            test ! -s invalid.rs && test -s invalid.err
            cp failed.err "$invalid-schema.err"
            cp invalid.err "$invalid-typed.err"
          done
          cp policy.json tiny.rs expected-tiny.rs *-missing.err *-late-override.err *-schema.err *-typed.err "$out/"
          cp ${nativeMetaLoweringPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
          cp ${nativeMetaLoweringPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
          printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeMetaLoweringPolicy pkgs).drvPath} > "$out/native-build-roots"
        '';
      astLoweringPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/ast-lowering-body.cue
          ];
        };
      nativeAstLoweringPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-lowering-policy" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          jq=${pkgs.jq}/bin/jq
          for binary in "$exporter" "$jq"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          integer=${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json
          test "$(wc -c < "$integer")" -le 65536
          for run in first second; do
            # Each whole export retains its own unchanged native invocation ledger.
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
              ${astLoweringPolicySources pkgs}/native-policy-package.cue \
              ${astLoweringPolicySources pkgs}/ast-lowering-body.cue > "$run-body.json"
            test "$(wc -c < "$run-body.json")" -le 65536
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
              -f ${../../bootstrap/compose-policy.jq} "$run-body.json" "$integer" > "$run.json"
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
              -f ${../../bootstrap/compose-policy.jq} "$integer" "$run-body.json" > "$run-reversed.json"
            test "$(wc -c < "$run.json")" -le 65536
            test "$(wc -c < "$run-reversed.json")" -le 65536
            cmp "$run.json" "$run-reversed.json"
          done
          cmp first-body.json second-body.json
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/ast-lowering.json"
        '';
      nativeAstLoweringPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeAstIntegerPolicy pkgs).drvPath
            (nativeAstLoweringPolicy pkgs).drvPath
          ];
        };
      nativeAstLoweringPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-lowering-policy-verification"
          {
            nativeBuildInputs = [
              pkgs.jq
              pkgs.rustfmt
            ];
          }
          ''
            set -euo pipefail
            # Independent validation is allowed Go/schema/compiler dependencies;
            # none may feed the native construction graph.
            for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
              ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
              ${executor pkgs} ${(executor pkgs).drvPath} \
              ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
              ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeAstLoweringPolicyBuildClosure pkgs}/store-paths; then
                echo "forbidden complete-policy construction dependency: $forbidden" >&2
                exit 1
              fi
            done
            mkdir -p "$out" "$PWD/probe/home"
            cd probe
            exporter=${specExporter pkgs}/bin/cuestacean-spec-export
            oracle=${pkgs.cue}/bin/cue
            codegen=${executor pkgs}/bin/bootstrap-ast-codegen
            jq=${pkgs.jq}/bin/jq
            for binary in "$exporter" "$oracle" "$codegen" "$jq"; do
              test -r "$binary" && test -x "$binary"
              case "$binary" in /*) ;; *) exit 1 ;; esac
            done
            native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
            cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
            compose() {
              test "$(wc -c < "$1")" -le 65536
              test "$(wc -c < "$2")" -le 65536
              env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
                -f ${../../bootstrap/compose-policy.jq} "$@" > composed.json
              test "$(wc -c < composed.json)" -le 65536
            }
            generate() {
              env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | rustfmt --edition 2024
            }
            no_export() {
              if cue export "$@" > failed.json 2> failed.err; then exit 1; fi
              test ! -s failed.json && test -s failed.err
            }
            schema=${../../bootstrap/schema.cue}
            bridge=${../../bootstrap/ast-lowering-fragments.cue}
            selector=${../../bootstrap/ast-lowering.cue}
            cp ${astLoweringPolicySources pkgs}/native-policy-package.cue package.cue
            cp ${astLoweringPolicySources pkgs}/ast-lowering-body.cue body.cue
            cp ${astIntegerPolicySources pkgs}/ast-integer-policy.cue integer.cue
            native package.cue body.cue > body.json
            native body.cue package.cue > reversed-body.json
            cmp body.json reversed-body.json
            native package.cue integer.cue > integer.json
            cmp integer.json ${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json
            compose body.json integer.json
            cp composed.json policy.json
            compose integer.json body.json
            cmp composed.json policy.json
            cmp policy.json ${nativeAstLoweringPolicy pkgs}/ast-lowering.json
            cmp policy.json ${jsonExports pkgs}/ast-lowering.json
            cue export "$schema" "$bridge" body.cue integer.cue "$selector" | jq -S . > raw-policy.json
            cmp policy.json raw-policy.json
            cue export "$selector" integer.cue body.cue "$bridge" "$schema" | jq -S . > reversed-policy.json
            cmp policy.json reversed-policy.json
            # Schema validation remains independent, not an exporter capability.
            cue export "$schema" "$bridge" "$selector" policy.json | jq -S . > validated-policy.json
            cmp policy.json validated-policy.json
            for missing in body integer; do
              if test "$missing" = body; then provider=integer.cue; else provider=body.cue; fi
              no_export "$schema" "$bridge" "$selector" "$provider"
              grep -F 'incomplete value' failed.err
              cue eval "$schema" "$bridge" "$selector" "$provider" > /dev/null
              cp failed.err "$missing-missing.err"
            done
            for builder in ${builderNames}; do
              name=$(basename "$builder")
              generate ${jsonExports pkgs}/"$builder.json" policy.json > "$name.rs"
              cmp "$name.rs" ${../../bootstrap/generated}/"$name.rs"
            done

            # Ordinary lexical body source, not JSON patching: all parameter/use
            # spellings change together, preserving Copy/Owned binding semantics.
            substitute body.cue variant-body.cue --replace-fail '_arg' '_native'
            native package.cue variant-body.cue > variant-body.json
            compose variant-body.json integer.json
            cp composed.json body-policy.json
            cue export "$schema" "$bridge" "$selector" variant-body.cue integer.cue | jq -S . > raw-body-policy.json
            cmp body-policy.json raw-body-policy.json
            if cmp -s policy.json body-policy.json; then exit 1; fi
            generate ${jsonExports pkgs}/ast-atom.json body-policy.json > body-variant.rs
            printf '%s\n' \
              '// Generated by bootstrap-ast-codegen. Do not edit by hand.' \
              'use serde_json::Value;' \
              'pub fn build(_native0: String) -> Value {' \
              '    Value::Object(serde_json::Map::from_iter([(' \
              '        String::from("kind"),' \
              '        Value::String(_native0),' \
              '    )]))' \
              '}' > expected-body.rs
            cmp body-variant.rs expected-body.rs

            # Independently change the preferred integer SOURCE; retain BOTH
            # alternatives and construct the complete native policy before use.
            substitute integer.cue variant-integer.cue \
              --replace-fail 'Value::from(' 'Value::from(1 + '
            native package.cue variant-integer.cue > variant-integer.json
            compose body.json variant-integer.json
            cp composed.json integer-policy.json
            cue export "$schema" "$bridge" "$selector" body.cue variant-integer.cue | jq -S . > raw-integer-policy.json
            cmp integer-policy.json raw-integer-policy.json
            if cmp -s policy.json integer-policy.json; then exit 1; fi
            generate ${jsonExports pkgs}/tests/valid/ast-literal.json integer-policy.json > integer-variant.rs
            printf '%s\n' \
              '// Generated by bootstrap-ast-codegen. Do not edit by hand.' \
              'use serde_json::Value;' \
              'pub fn build() -> Value {' \
              '    Value::from(1 + 7i64)' \
              '}' > expected-integer.rs
            cmp integer-variant.rs expected-integer.rs

            # Invalid instructions are still materializable DATA, never admitted
            # schema/typed compiler programs. Neither validator emits code.
            for invalid in opcode shape; do
              if test "$invalid" = opcode; then
                substitute body.cue invalid.cue --replace-fail 'op: "join"' 'op: "notAnOpcode"'
              else
                substitute body.cue invalid.cue --replace-fail 'parts: [' 'wrongParts: ['
              fi
              native package.cue invalid.cue > invalid-body.json
              compose invalid-body.json integer.json
              no_export "$schema" "$bridge" "$selector" composed.json
              if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" \
                ${jsonExports pkgs}/ast-atom.json composed.json > invalid.rs 2> invalid.err; then exit 1; fi
              test ! -s invalid.rs && test -s invalid.err
              cp failed.err "$invalid-schema.err"
              cp invalid.err "$invalid-typed.err"
            done
            cp body.json integer.json policy.json raw-policy.json body-policy.json integer-policy.json \
              body-variant.rs integer-variant.rs *-missing.err *-schema.err *-typed.err "$out/"
            cp ${nativeAstLoweringPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeAstLoweringPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeAstIntegerPolicy pkgs).drvPath} \
              ${(nativeAstLoweringPolicy pkgs).drvPath} > "$out/native-build-roots"
          '';
      builderSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [ ../../bootstrap/ast-schema.cue ]
            ++ map (fixture: ../../bootstrap + "/${fixture}.cue") builderExportFixtures
          );
        };
      nativeBuilders =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-builders" { } ''
          set -euo pipefail
          for fixture in ${builderExportNames}; do
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${builderSources pkgs}/ast-schema.cue ${builderSources pkgs}/"$fixture.cue" > first.json
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${builderSources pkgs}/ast-schema.cue ${builderSources pkgs}/"$fixture.cue" > second.json
            cmp first.json second.json
            mkdir -p "$(dirname "$out/$fixture.json")"
            cp first.json "$out/$fixture.json"
          done
        '';
      nativeBuilderBuildClosure =
        pkgs:
        pkgs.closureInfo {
          # Attest the entire build graph, not only runtime output references.
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeBuilders pkgs).drvPath
          ];
        };
      nativeBuilderVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-builder-family-verification"
          {
            nativeBuildInputs = [
              pkgs.cue
              pkgs.jq
              pkgs.rustfmt
            ];
          }
          ''
            set -euo pipefail
            for forbidden in ${pkgs.cue.drvPath} ${pkgs.cue.src} ${(jsonExports pkgs).drvPath} ${(executor pkgs).drvPath} ${(defaultObserver pkgs).drvPath} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeBuilderBuildClosure pkgs}/store-paths; then
                echo "forbidden bootstrap dependency: $forbidden" >&2; exit 1
              fi
            done
            mkdir -p "$out"
            for fixture in ${builderExportNames}; do
              cue export ${builderSources pkgs}/ast-schema.cue ${builderSources pkgs}/"$fixture.cue" --out json | jq -S . > expected.json
              cmp expected.json ${nativeBuilders pkgs}/"$fixture.json"
              name=$(basename "$fixture")
              case "$fixture" in
                tests/valid/ast-literal)
                  # The specification contains 7; this alternate compiler policy emits 99.
                  name=ast-forced-integer
                  ${executor pkgs}/bin/bootstrap-ast-codegen ${nativeBuilders pkgs}/"$fixture.json" \
                    ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json \
                    | rustfmt --edition 2024 > actual.rs
                  ;;
                *)
                  ${executor pkgs}/bin/bootstrap-ast-codegen ${nativeBuilders pkgs}/"$fixture.json" \
                    | rustfmt --edition 2024 > actual.rs
                  ;;
              esac
              cmp actual.rs ${../../bootstrap/generated}/"$name.rs"
              cp actual.rs "$out/$name.rs"
            done
            cp ${nativeBuilderBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeBuilderBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeBuilders pkgs).drvPath} > "$out/native-build-roots"
          '';
      atomSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/ast-schema.cue
            ../../bootstrap/ast-atom.cue
          ];
        };
      nativeAtom =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-atom" { } ''
          mkdir -p "$out"
          env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
            ${atomSources pkgs}/ast-schema.cue ${atomSources pkgs}/ast-atom.cue > "$out/ast-atom.json"
        '';
      nativeBuildClosure =
        pkgs:
        pkgs.closureInfo {
          # .drv roots include build dependencies, not just output references.
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeAtom pkgs).drvPath
          ];
        };
      nativeVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-export-verification"
          {
            nativeBuildInputs = [
              pkgs.cue
              pkgs.jq
              pkgs.rustfmt
            ];
          }
          ''
            set -euo pipefail
            for forbidden in ${pkgs.cue.drvPath} ${pkgs.cue.src} ${(jsonExports pkgs).drvPath} ${(executor pkgs).drvPath} ${(defaultObserver pkgs).drvPath} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeBuildClosure pkgs}/store-paths; then
                echo "forbidden bootstrap dependency: $forbidden" >&2; exit 1
              fi
            done
            cue export ${atomSources pkgs}/ast-schema.cue ${atomSources pkgs}/ast-atom.cue --out json | jq -S . > expected
            cmp expected ${nativeAtom pkgs}/ast-atom.json
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${atomSources pkgs}/ast-schema.cue ${atomSources pkgs}/ast-atom.cue > repeated
            cmp repeated ${nativeAtom pkgs}/ast-atom.json
            ${executor pkgs}/bin/bootstrap-ast-codegen ${nativeAtom pkgs}/ast-atom.json \
              | rustfmt --edition 2024 > actual.rs
            cmp actual.rs ${../../bootstrap/generated/ast-atom.rs}
            # Test-only raw fragment/answers: none feed specExporter/nativeAtom.
            # Produce with the minimal production binary BEFORE oracle comparison.
            tagged="$PWD/tagged"
            mkdir -p "$tagged/home"
            cp ${astIntegerPolicySources pkgs}/ast-integer-policy.cue "$tagged/program.cue"
            test -r ${specExporter pkgs}/bin/cuestacean-spec-export
            test -x ${specExporter pkgs}/bin/cuestacean-spec-export
            for name in default text incomplete; do
              case "$name" in
                default) context='{}' ;;
                text) context='{cases:{integer:{op:"text",value:"x"}}}' ;;
                incomplete) context='{cases:{integer:{op:"text"}}}' ;;
              esac
              printf 'package bootstrap\n%s\n' "$context" > "$tagged/schema.cue"
              if [ "$name" = incomplete ]; then
                if env -i HOME="$tagged/home" PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
                  "$tagged/schema.cue" "$tagged/program.cue" > "$tagged/$name.json" 2> "$tagged/$name.err"; then
                  echo "incomplete tagged branch unexpectedly exported" >&2; exit 1
                fi
                test ! -s "$tagged/$name.json"
                grep -F 'incomplete export demand' "$tagged/$name.err"
                if env -i HOME="$tagged/home" PATH=/nonexistent ${pkgs.cue}/bin/cue export \
                  "$tagged/schema.cue" "$tagged/program.cue" > "$tagged/oracle.json" 2> "$tagged/oracle.err"; then
                  echo "incomplete oracle unexpectedly exported" >&2; exit 1
                fi
                test ! -s "$tagged/oracle.json"
                env -i HOME="$tagged/home" PATH=/nonexistent ${pkgs.cue}/bin/cue eval \
                  "$tagged/schema.cue" "$tagged/program.cue" > /dev/null
              else
                env -i HOME="$tagged/home" PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
                  "$tagged/schema.cue" "$tagged/program.cue" > "$tagged/$name.json"
                env -i HOME="$tagged/home" PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
                  "$tagged/program.cue" "$tagged/schema.cue" > "$tagged/repeated.json"
                cmp "$tagged/repeated.json" "$tagged/$name.json"
                case "$name" in
                  default) answer='{"cases":{"integer":{"op":"concat","parts":[{"op":"text","value":"Value::from("},{"op":"signed","field":"value"},{"op":"text","value":"i64)"}]}}}' ;;
                  text) answer='{"cases":{"integer":{"op":"text","value":"x"}}}' ;;
                esac
                printf '%s\n' "$answer" | jq -S . > "$tagged/expected.json"
                cmp "$tagged/expected.json" "$tagged/$name.json"
                env -i HOME="$tagged/home" PATH=/nonexistent ${pkgs.cue}/bin/cue export \
                  "$tagged/schema.cue" "$tagged/program.cue" | jq -S . > "$tagged/oracle.json"
                cmp "$tagged/oracle.json" "$tagged/$name.json"
              fi
            done
            mkdir -p "$out"
            cp "$tagged/default.json" "$tagged/text.json" "$tagged/incomplete.err" "$out/"
            cp ${nativeBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeAtom pkgs).drvPath} > "$out/native-build-roots"
            cp actual.rs "$out/ast-atom.rs"
          '';
      # These constructors are referenced by many verifiers and outputs.
      # Share their lazy values across those references within each system.
      shared = kolohelios-nix.lib.forEachSupportedSystem (
        { pkgs, ... }: {
          jsonExports = makeJsonExports pkgs;
          legacyCodegenSeed = makeLegacyCodegenSeed pkgs;
          codegenSeed = makeCodegenSeed pkgs;
          executor = makeExecutor pkgs;
        }
      );
      sharedFor = pkgs: shared.${pkgs.stdenv.hostPlatform.system};
      jsonExports = pkgs: (sharedFor pkgs).jsonExports;
      legacyCodegenSeed = pkgs: (sharedFor pkgs).legacyCodegenSeed;
      codegenSeed = pkgs: (sharedFor pkgs).codegenSeed;
      executor = pkgs: (sharedFor pkgs).executor;
      cli =
        pkgs:
        pkgs.runCommand "cuestacean" { meta.mainProgram = "cuestacean"; } ''
          install -Dm755 ${executor pkgs}/bin/cuestacean "$out/bin/cuestacean"
        '';
    in
    {
      packages = kolohelios-nix.lib.forEachSupportedSystem (
        { pkgs, ... }: {
          default = executor pkgs;
          cuestacean = cli pkgs;
          exports = jsonExports pkgs;
          spec-export = specExporter pkgs;
          native-ast-atom = nativeAtom pkgs;
          native-ast-builders = nativeBuilders pkgs;
          native-demand-policies = nativeDemandPolicies pkgs;
          native-ast-integer-policy = nativeAstIntegerPolicy pkgs;
          native-ast-lowering-policy = nativeAstLoweringPolicy pkgs;
          native-meta-lowering-policy = nativeMetaLoweringPolicy pkgs;
          native-unify-lowering-policy = nativeUnifyLoweringPolicy pkgs;
          native-scan-lowering-policy = nativeScanLoweringPolicy pkgs;
          native-instruction-core-cases = nativeInstructionCoreCases pkgs;
          native-instruction-right-field = nativeInstructionRightField pkgs;
          native-instruction-right-operand = nativeInstructionRightOperand pkgs;
          native-instruction-operand-cases = nativeInstructionOperandCases pkgs;
          native-instruction-binary-runtime = nativeInstructionBinaryRuntime pkgs;
          native-instruction-binary-tables = nativeInstructionBinaryTables pkgs;
          native-instruction-unary-tables = nativeInstructionUnaryTables pkgs;
          native-instruction-module = nativeInstructionModule pkgs;
          native-instruction-lowering-policy = nativeInstructionLoweringPolicy pkgs;
          native-instruction-module-text = nativeInstructionModuleText pkgs;
          native-instruction-module-controls = nativeInstructionModuleControls pkgs;
          native-slice-lower = nativeSliceLower pkgs;
          native-record-minimum = nativePrimaryMinimum pkgs "record";
          native-conjunction-minimum = nativePrimaryMinimum pkgs "conjunction";
          native-primary-bound-minimum = nativePrimaryMinimum pkgs "primary-bound";
          native-primary-call-minimum = nativePrimaryMinimum pkgs "primary-call";
          native-primary-list-minimum = nativePrimaryMinimum pkgs "primary-list";
          native-pratt-tables = nativePrattTables pkgs;
          native-pratt-lowering-policy = nativePrattLoweringPolicy pkgs;
          native-codegen-core = nativeCodegenCore pkgs;
          native-scan-codegen = nativeScanCodegen pkgs;
          native-unify-codegen = nativeUnifyCodegen pkgs;
          native-pratt-codegen = nativePrattCodegen pkgs;
          native-instruction-codegen = nativeInstructionCodegen pkgs;
          default-observer = defaultObserver pkgs;
          default-trace = defaultTrace pkgs;
        }
      );

      apps = kolohelios-nix.lib.forEachSupportedSystem (
        { pkgs, ... }:
        {
          cuestacean = {
            type = "app";
            program = "${cli pkgs}/bin/cuestacean";
            meta.description = "Export a bounded CUE expression as JSON";
          };
          generated = {
            type = "app";
            program = "${executor pkgs}/bin/bootstrap-generated";
            meta.description = "Evaluate JSON with the CUE-defined rules compiled into Rust";
          };
          source = {
            type = "app";
            program = "${executor pkgs}/bin/bootstrap-source";
            meta.description = "Evaluate the bootstrap decimal integer expression subset";
          };
        }
      );

      devShells = kolohelios-nix.lib.forEachSupportedSystem (
        { system, pkgs, ... }:
        {
          default = pkgs.mkShell {
            CUESTACEAN_BOOTSTRAP_JSON = jsonExports pkgs;
            CUESTACEAN_DEMAND_POLICIES = nativeDemandPolicies pkgs;
            CUESTACEAN_DEFAULT_OBSERVER = "${defaultObserver pkgs}/bin/cuestacean-default-observer";
            CUESTACEAN_DEFAULT_TRACE = "${defaultTrace pkgs}/bin/cuestacean-default-trace";
            packages = [
              pkgs.cue
              pkgs.jq
              pkgs.python3
              pkgs.time
              pkgs.rustc
              pkgs.cargo
              pkgs.rustfmt
              pkgs.clippy
              pkgs.actionlint
              pkgs.shellcheck
              pkgs.taplo
            ]
            ++ kolohelios-nix.lib.workflowPackages pkgs;

            # The shared tooling includes a monorepo-only shaka shim.
            shellHook = ''
              export PATH="${shaka.packages.${system}.default}/bin:$PATH"
            '';
          };
        }
      );

      checks = kolohelios-nix.lib.forEachSupportedSystem (
        { pkgs, ... }:
        {
          bootstrap-executor = executor pkgs;
          cuestacean-cli = pkgs.runCommand "cuestacean-cli-verification" { } ''
            ${pkgs.bash}/bin/bash ${./test-cli.sh} ${cli pkgs}/bin/cuestacean ${pkgs.cue}/bin/cue ${pkgs.jq}/bin/jq
            mkdir cli-parity
            (cd cli-parity; ${pkgs.python3}/bin/python3 -I -B ${./cli-parity}/check.py ${cli pkgs}/bin/cuestacean ${pkgs.cue}/bin/cue)
            status=0
            grep -Fx ${pkgs.cue} ${pkgs.closureInfo { rootPaths = [ (cli pkgs) ]; }}/store-paths || status=$?
            test "$status" -eq 1
            touch "$out"
          '';
          native-spec-tests = specTests pkgs;
          default-observer = defaultObserverVerification pkgs;
          native-spec-closure = nativeVerification pkgs;
          native-builder-family = nativeBuilderVerification pkgs;
          native-demand-policies = nativeDemandPolicyVerification pkgs;
          native-ast-integer-policy = nativeAstIntegerPolicyVerification pkgs;
          native-ast-lowering-policy = nativeAstLoweringPolicyVerification pkgs;
          native-meta-lowering-policy = nativeMetaLoweringPolicyVerification pkgs;
          native-unify-lowering-policy = nativeUnifyLoweringPolicyVerification pkgs;
          native-scan-lowering-policy = nativeScanLoweringPolicyVerification pkgs;
          native-instruction-core-cases = nativeInstructionCoreCasesVerification pkgs;
          native-instruction-right-field = nativeInstructionRightFieldVerification pkgs;
          native-instruction-right-operand = nativeInstructionRightOperandVerification pkgs;
          native-instruction-operand-cases = nativeInstructionOperandCasesVerification pkgs;
          native-instruction-binary-runtime = nativeInstructionBinaryRuntimeVerification pkgs;
          native-instruction-binary-tables = nativeInstructionBinaryTablesVerification pkgs;
          native-instruction-unary-tables = nativeInstructionUnaryTablesVerification pkgs;
          native-instruction-module = nativeInstructionModuleVerification pkgs;
          native-instruction-lowering-policy = nativeInstructionLoweringPolicyVerification pkgs;
          native-instruction-module-text = nativeInstructionModuleTextVerification pkgs;
          native-instruction-module-controls = nativeInstructionModuleControlsVerification pkgs;
          native-slice-lower = nativeSliceLowerVerification pkgs;
          native-primary-minima = nativePrimaryMinimaVerification pkgs;
          primary-minima-runtime = primaryMinimaRuntime pkgs;
          native-primary-control-minima = nativePrimaryControlMinimaVerification pkgs;
          primary-control-minima-runtime = primaryControlMinimaRuntime pkgs;
          native-pratt-minima = nativePrattMinimaVerification pkgs;
          pratt-minima-runtime-base = prattMinimaRuntime pkgs "base";
          pratt-minima-runtime-index = prattMinimaRuntime pkgs "index";
          pratt-minima-runtime-slice = prattMinimaRuntime pkgs "slice";
          native-pratt-tables = nativePrattTablesVerification pkgs;
          native-pratt-lowering-policy = nativePrattLoweringPolicyVerification pkgs;
          native-codegen-core = nativeCoreVerification pkgs;
          native-scan-codegen = nativeScannerVerification pkgs;
          native-unify-codegen = nativeUnifierVerification pkgs;
          native-pratt-codegen = nativePrattVerification pkgs;
          native-instruction-codegen = nativeInstructionVerification pkgs;
          bootstrap-codegen =
            pkgs.runCommand "cuestacean-bootstrap-codegen"
              {
                nativeBuildInputs = [
                  (executor pkgs)
                  pkgs.rustfmt
                ];
              }
              ''
                set -euo pipefail
                for rules in ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/evaluation-left-twice.json ${../../bootstrap/executor/tests/fixtures}/codegen-escaped-rules.json; do
                  name=$(basename "$rules" .json)
                  bootstrap-codegen "$rules" | rustfmt --edition 2024 > first.rs
                  bootstrap-codegen "$rules" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                for policy in ${loweringNames}; do
                  name=$(basename "$policy")
                  bootstrap-lower-codegen ${jsonExports pkgs}/"$policy.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-lower-codegen ${jsonExports pkgs}/"$policy.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-codegen ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/lowering-left-twice.json | rustfmt --edition 2024 > policy.rs
                diff -u ${../../bootstrap/generated}/evaluation-left-twice.rs policy.rs
                bootstrap-codegen ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/lowering-reverse.json | rustfmt --edition 2024 > reverse.rs
                bootstrap-codegen ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/lowering-reverse.json | rustfmt --edition 2024 > reverse-second.rs
                cmp reverse.rs reverse-second.rs
                diff -u ${../../bootstrap/generated}/evaluation-reverse.rs reverse.rs
                bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/lowering-numbers.json ${jsonExports pkgs}/tests/valid/meta-lowering-forced.json | rustfmt --edition 2024 > meta-forced.rs
                bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/lowering-numbers.json ${jsonExports pkgs}/tests/valid/meta-lowering-forced.json | rustfmt --edition 2024 > meta-forced-second.rs
                cmp meta-forced.rs meta-forced-second.rs
                diff -u ${../../bootstrap/generated}/lowering-meta-forced.rs meta-forced.rs
                bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/meta-driver-keys.json ${jsonExports pkgs}/tests/valid/meta-driver-previous.json | rustfmt --edition 2024 > driver-previous.rs
                bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/meta-driver-keys.json ${jsonExports pkgs}/tests/valid/meta-driver-previous.json | rustfmt --edition 2024 > driver-previous-second.rs
                cmp driver-previous.rs driver-previous-second.rs
                diff -u ${../../bootstrap/generated}/meta-driver-previous.rs driver-previous.rs
                for machine in ${scannerNames}; do
                  name=$(basename "$machine")
                  bootstrap-scan-codegen ${jsonExports pkgs}/"$machine.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-scan-codegen ${jsonExports pkgs}/"$machine.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-scan-codegen ${jsonExports pkgs}/tests/valid/scanner-advance.json ${jsonExports pkgs}/tests/valid/scan-lowering-unicode.json | rustfmt --edition 2024 > scan-alternate.rs
                bootstrap-scan-codegen ${jsonExports pkgs}/tests/valid/scanner-advance.json ${jsonExports pkgs}/tests/valid/scan-lowering-unicode.json | rustfmt --edition 2024 > scan-alternate-second.rs
                cmp scan-alternate.rs scan-alternate-second.rs
                diff -u ${../../bootstrap/generated}/scan-unicode-advance.rs scan-alternate.rs
                for builder in ${builderNames}; do
                  name=$(basename "$builder")
                  bootstrap-ast-codegen ${jsonExports pkgs}/"$builder.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-ast-codegen ${jsonExports pkgs}/"$builder.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-ast-codegen ${jsonExports pkgs}/tests/valid/ast-literal.json ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json | rustfmt --edition 2024 > ast-forced.rs
                bootstrap-ast-codegen ${jsonExports pkgs}/tests/valid/ast-literal.json ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json | rustfmt --edition 2024 > ast-forced-second.rs
                cmp ast-forced.rs ast-forced-second.rs
                diff -u ${../../bootstrap/generated}/ast-forced-integer.rs ast-forced.rs
                for control in ${prattNames}; do
                  name=$(basename "$control")
                  bootstrap-pratt-codegen ${jsonExports pkgs}/"$control.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-pratt-codegen ${jsonExports pkgs}/"$control.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                for control in ${filePrattNames}; do
                  bootstrap-pratt-codegen ${jsonExports pkgs}/"$control.json" ${jsonExports pkgs}/file-pratt-lowering.json | rustfmt --edition 2024 > first.rs
                  bootstrap-pratt-codegen ${jsonExports pkgs}/"$control.json" ${jsonExports pkgs}/file-pratt-lowering.json | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$control.rs" first.rs
                done
                bootstrap-pratt-codegen ${jsonExports pkgs}/pratt.json ${jsonExports pkgs}/tests/valid/pratt-lowering-no-increment.json | rustfmt --edition 2024 > pratt-alternate.rs
                bootstrap-pratt-codegen ${jsonExports pkgs}/pratt.json ${jsonExports pkgs}/tests/valid/pratt-lowering-no-increment.json | rustfmt --edition 2024 > pratt-alternate-second.rs
                cmp pratt-alternate.rs pratt-alternate-second.rs
                diff -u ${../../bootstrap/generated}/pratt-no-increment.rs pratt-alternate.rs
                for control in ${unifyNames}; do
                  name=$(basename "$control")
                  bootstrap-unify-codegen ${jsonExports pkgs}/"$control.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-unify-codegen ${jsonExports pkgs}/"$control.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-unify-codegen ${jsonExports pkgs}/tests/valid/unify-equality-probe.json ${jsonExports pkgs}/tests/valid/unify-lowering-permissive.json | rustfmt --edition 2024 > unify-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/tests/valid/unify-equality-probe.json ${jsonExports pkgs}/tests/valid/unify-lowering-permissive.json | rustfmt --edition 2024 > unify-alternate-second.rs
                cmp unify-alternate.rs unify-alternate-second.rs
                diff -u ${../../bootstrap/generated}/unify-permissive-bool.rs unify-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/difference-control.json ${jsonExports pkgs}/tests/valid/integer-kernel-alternate.json | rustfmt --edition 2024 > integer-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/difference-control.json ${jsonExports pkgs}/tests/valid/integer-kernel-alternate.json | rustfmt --edition 2024 > integer-alternate-second.rs
                cmp integer-alternate.rs integer-alternate-second.rs
                diff -u ${../../bootstrap/generated}/difference-as-product.rs integer-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/divide-control.json ${jsonExports pkgs}/tests/valid/division-kernel-alternate.json | rustfmt --edition 2024 > division-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/divide-control.json ${jsonExports pkgs}/tests/valid/division-kernel-alternate.json | rustfmt --edition 2024 > division-alternate-second.rs
                cmp division-alternate.rs division-alternate-second.rs
                diff -u ${../../bootstrap/generated}/divide-as-quotient.rs division-alternate.rs
                touch "$out"
              '';
          bootstrap-source =
            pkgs.runCommand "cuestacean-bootstrap-source"
              {
                nativeBuildInputs = [
                  (executor pkgs)
                  pkgs.cue
                  pkgs.jq
                ];
              }
              ''
                set -euo pipefail
                printf 'package probe\nimport "encoding/json"\nout: json.Marshal(value)\n' > compact.cue
                for source in ${../../bootstrap/source}/valid/*.cue; do
                  cue export "$source" --out json > expected
                  # CUE can spell integer zero as -0; i64 output is canonical 0.
                  # Normalize only that spelling, without rounding any integers.
                  IFS= read -r scalar < expected
                  if [ "$scalar" = "-0" ]; then printf '0\n' > expected; fi
                  bootstrap-source "$source" > actual
                  case "$scalar" in
                    '['*|'{'*)
                      # Reimport/marshal collections with CUE: exact integers,
                      # normalized whitespace/string escapes, never jq numbers.
                      # Record fixtures spell byte-ordered labels at every level;
                      # independent Rust tests pin ordering of arbitrary input.
                      for side in expected actual; do
                        cue import -p probe -l '"value"' --outfile - json: "$side" > value.cue
                        cue export value.cue compact.cue -e out --out text > normalized-"$side"
                      done
                      diff -u normalized-expected normalized-actual
                      ;;
                    \"*)
                      # String encoders may differ in JSON escape spelling.
                      jq -e 'type == "string"' expected > /dev/null
                      jq -e 'type == "string"' actual > /dev/null
                      jq -c . expected > normalized-expected
                      jq -c . actual > normalized-actual
                      diff -u normalized-expected normalized-actual
                      ;;
                    *) diff -u expected actual ;;
                  esac
                done
                for source in ${../../bootstrap/source}/conflicts/*.cue ${../../bootstrap/source}/incomplete/*.cue; do
                  if cue export "$source" --out json > expected 2> cue-error; then
                    echo "Expected upstream export failure: $source" >&2
                    exit 1
                  fi
                  if bootstrap-source "$source" > actual 2> error; then
                    echo "Expected bootstrap export failure: $source" >&2
                    exit 1
                  fi
                  test ! -s expected
                  test -s cue-error
                  test ! -s actual
                  test -s error
                done
                # These selected domain/syntax rejections must also fail upstream.
                for source in ${../../bootstrap/source}/rejected/ordering-{boolean,mixed}.cue ${../../bootstrap/source}/rejected/logic-{left,selected,eager,or-eager}.cue ${../../bootstrap/source}/rejected/call-{domain,arity,empty,name-newline}.cue ${../../bootstrap/source}/rejected/list-index-{range,negative,type,close}.cue ${../../bootstrap/source}/rejected/slice-{range,domain,stride}.cue ${../../bootstrap/source}/rejected/conjunction-{domain,close}.cue ${../../bootstrap/source}/rejected/bounds-exclusive-domain.cue ${../../bootstrap/source}/rejected/not-integer.cue ${../../bootstrap/source}/rejected/null-conflict.cue ${../../bootstrap/source}/rejected/radix-uppercase-{binary,octal}.cue; do
                  if cue export "$source" --out json > expected 2> cue-error; then exit 1; fi
                  test ! -s expected
                  test -s cue-error
                done
                for source in ${../../bootstrap/source}/rejected/*.cue; do
                  if bootstrap-source "$source" > actual 2> error; then
                    echo "Expected rejection: $source" >&2
                    exit 1
                  fi
                  test ! -s actual
                  test -s error
                done
                touch "$out"
              '';
          bootstrap-values =
            pkgs.runCommand "cuestacean-bootstrap-values"
              {
                nativeBuildInputs = [
                  (executor pkgs)
                  pkgs.cue
                  pkgs.jq
                ];
              }
              ''
                set -euo pipefail
                for fixture in lists list-expressions conjunction list-slicing optional-slices nested-lists records records-unify record-expressions nested-records nested-records-unify mixed-collections mixed-unify collection-index-record tests/valid/lists-unify; do
                  probe="''${fixture##*/}"
                  # Marshal in pinned CUE, not jq: keep every i64 exact.
                  printf '%s\n' "$(cue export ${../../bootstrap/tests/upstream}/"$probe.cue" --out text)" > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    diff -u expected actual
                  done
                done
                if cue export ${../../bootstrap/tests/upstream/conjunction-conflict.cue} --out json > expected 2> cue-error; then exit 1; fi
                test ! -s expected
                test -s cue-error
                for binary in bootstrap-executor bootstrap-generated; do
                  if [ "$binary" = bootstrap-executor ]; then
                    if "$binary" ${jsonExports pkgs}/conjunction-conflict.json ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                  else
                    if "$binary" ${jsonExports pkgs}/conjunction-conflict.json > actual 2> error; then exit 1; fi
                  fi
                  test ! -s actual
                  grep -Fx 'conflicting integer values 1 and 2' error
                done
                for pair in list-slicing:range list-slicing:domain optional-slices:range optional-slices:domain; do
                  mode="''${pair#*:}"
                  fixture="''${pair%:*}-$mode"
                  if cue export ${../../bootstrap/tests/upstream}/"$fixture.cue" --out json > expected 2> cue-error; then exit 1; fi
                  test ! -s expected
                  test -s cue-error
                  if [ "$mode" = range ]; then message='list slice out of range'; else message='list slicing requires a concrete closed list and integer endpoints'; fi
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      if "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                    else
                      if "$binary" ${jsonExports pkgs}/"$fixture.json" > actual 2> error; then exit 1; fi
                    fi
                    test ! -s actual
                    grep -Fx "$message" error
                  done
                done
                for fixture in records-conflict records-fields-conflict nested-records-conflict nested-records-scalar-conflict mixed-conflict; do
                  if cue export ${../../bootstrap/tests/upstream}/"$fixture.cue" --out json > expected 2> cue-error; then exit 1; fi
                  test ! -s expected
                  test -s cue-error
                  grep -E 'conflicting values|field not allowed' cue-error
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      if "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                    else
                      if "$binary" ${jsonExports pkgs}/"$fixture.json" > actual 2> error; then exit 1; fi
                    fi
                    test ! -s actual
                    grep -Fx 'conflicting closed record values' error
                  done
                done
                for fixture in list-length lists-equal list-indexing collection-index selectors exclusive-bounds records-index records-length records-equal records-logical; do
                  cue export ${../../bootstrap/tests/upstream}/"$fixture.cue" --out json > expected
                  if [ "$fixture" = lists-equal ]; then input=tests/valid/$fixture; else input=$fixture; fi
                  bootstrap-executor ${jsonExports pkgs}/"$input.json" ${jsonExports pkgs}/evaluation.json > actual
                  diff -u expected actual
                  bootstrap-generated ${jsonExports pkgs}/"$input.json" > actual
                  diff -u expected actual
                done
                cue export ${../../bootstrap/tests/upstream/string-length.cue} --out json > expected
                for binary in bootstrap-executor bootstrap-generated; do
                  if [ "$binary" = bootstrap-executor ]; then
                    "$binary" ${jsonExports pkgs}/string-length.json ${jsonExports pkgs}/evaluation.json > actual
                  else
                    "$binary" ${jsonExports pkgs}/string-length.json > actual
                  fi
                  diff -u expected actual
                done
                for pair in concrete:strings empty:tests/valid/strings-empty equal:tests/valid/strings-equal concatenated:concatenation; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/strings.cue} -e "$property" --out json | jq -c . > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    # Normalize string spelling only; never round numeric scalars.
                    jq -e 'type == "string"' actual > /dev/null
                    jq -c . actual > normalized
                    diff -u expected normalized
                  done
                done
                for pair in equal:equality notEqual:inequality mixed:tests/valid/equality-mixed; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/equality.cue} -e "$property" --out json > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    jq -e 'type == "boolean"' actual > /dev/null
                    diff -u expected actual
                  done
                done
                for pair in integer:ordering text:ordering-text; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/ordering.cue} -e "$property" --out json > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    jq -e 'type == "boolean"' actual > /dev/null
                    diff -u expected actual
                  done
                done
                for pair in and:logic-and or:logic-or not:not; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/logic.cue} -e "$property" --out json > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    jq -e 'type == "boolean"' actual > /dev/null
                    diff -u expected actual
                  done
                done
                for pair in nullValue:null combined:top-null; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/lattice.cue} -e "$property" --out json > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    jq -e 'type == "null"' actual > /dev/null
                    diff -u expected actual
                  done
                done
                if cue export ${../../bootstrap/tests/upstream/lattice.cue} -e top --out json > expected 2> cue-error; then exit 1; fi
                test ! -s expected
                test -s cue-error
                for binary in bootstrap-executor bootstrap-generated; do
                  if [ "$binary" = bootstrap-executor ]; then
                    if "$binary" ${jsonExports pkgs}/top.json ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                  else
                    if "$binary" ${jsonExports pkgs}/top.json > actual 2> error; then exit 1; fi
                  fi
                  test ! -s actual
                  test -s error
                done
                for fixture in strings-incomplete strings-conflict strings-mixed; do
                  case "$fixture" in
                    strings-incomplete) probe=${../../bootstrap/tests/upstream/strings.cue}; field=abstract ;;
                    strings-conflict) probe=${../../bootstrap/tests/upstream/string-conflict.cue}; field=value ;;
                    strings-mixed) probe=${../../bootstrap/tests/upstream/string-mixed.cue}; field=value ;;
                  esac
                  if cue export "$probe" -e "$field" --out json > expected 2> cue-error; then exit 1; fi
                  test ! -s expected
                  test -s cue-error
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      if "$binary" ${jsonExports pkgs}/tests/valid/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                    else
                      if "$binary" ${jsonExports pkgs}/tests/valid/"$fixture.json" > actual 2> error; then exit 1; fi
                    fi
                    test ! -s actual
                    test -s error
                  done
                done
                touch "$out"
              '';
          bootstrap-export = jsonExports pkgs;
        }
      );

      formatter = kolohelios-nix.formatter;
    };
}
