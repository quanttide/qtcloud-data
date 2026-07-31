package pipeline

import (
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/quanttide/qtcloud-provider/internal/specstore"
)

func TestExecuteStateMachineRunsBuiltinCopyInOrder(t *testing.T) {
	dir := t.TempDir()
	input := filepath.Join(dir, "raw.csv")
	if err := os.WriteFile(input, []byte("a,b\n1,2\n"), 0o600); err != nil {
		t.Fatalf("write input: %v", err)
	}

	result, err := ExecuteBlueprint(context.Background(), specstore.Blueprint{
		Name: "copy-demo",
		Pipeline: specstore.Pipeline{
			Name:    "copy-pipeline",
			StartAt: "load",
			States: map[string]specstore.State{
				"load": {
					Type:     "task",
					Resource: "builtin:copy",
					Next:     "finish",
				},
				"finish": {
					Type:     "task",
					Resource: "builtin:copy",
					End:      true,
				},
			},
		},
	}, input, dir)
	if err != nil {
		t.Fatalf("ExecuteBlueprint returned error: %v", err)
	}

	if result.Output != filepath.Join(dir, "final.csv") {
		t.Fatalf("output = %q, want final.csv in work dir", result.Output)
	}
	if got, want := len(result.Steps), 2; got != want {
		t.Fatalf("len(result.Steps) = %d, want %d", got, want)
	}
	if result.Steps[0].Name != "load" || result.Steps[1].Name != "finish" {
		t.Fatalf("step order = %#v", result.Steps)
	}

	content, err := os.ReadFile(result.Output)
	if err != nil {
		t.Fatalf("read output: %v", err)
	}
	if string(content) != "a,b\n1,2\n" {
		t.Fatalf("output content = %q", content)
	}
}

func TestExecuteBlueprintRejectsMissingStartAt(t *testing.T) {
	_, err := ExecuteBlueprint(context.Background(), specstore.Blueprint{
		Name: "bad",
		Pipeline: specstore.Pipeline{
			Name: "bad-pipeline",
			States: map[string]specstore.State{
				"only": {Type: "task", Resource: "builtin:copy", End: true},
			},
		},
	}, "input.csv", t.TempDir())

	if !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("error = %v, want ErrInvalidDefinition", err)
	}
}

func TestExecuteBlueprintRejectsStateWithoutResource(t *testing.T) {
	_, err := ExecuteBlueprint(context.Background(), specstore.Blueprint{
		Name: "bad",
		Pipeline: specstore.Pipeline{
			Name:    "bad-pipeline",
			StartAt: "only",
			States: map[string]specstore.State{
				"only": {Type: "task", End: true},
			},
		},
	}, "input.csv", t.TempDir())

	if !errors.Is(err, ErrNoExecutableResource) {
		t.Fatalf("error = %v, want ErrNoExecutableResource", err)
	}
}

func TestExecuteBlueprintRejectsStateCycles(t *testing.T) {
	_, err := ExecuteBlueprint(context.Background(), specstore.Blueprint{
		Name: "bad",
		Pipeline: specstore.Pipeline{
			Name:    "bad-pipeline",
			StartAt: "a",
			States: map[string]specstore.State{
				"a": {Type: "task", Resource: "builtin:copy", Next: "b"},
				"b": {Type: "task", Resource: "builtin:copy", Next: "a"},
			},
		},
	}, "input.csv", t.TempDir())

	if !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("error = %v, want ErrInvalidDefinition", err)
	}
}

func TestExecuteBlueprintRunsLinearStepsWhenStatesAreMissing(t *testing.T) {
	dir := t.TempDir()
	input := filepath.Join(dir, "raw.csv")
	if err := os.WriteFile(input, []byte("a,b\n1,2\n"), 0o600); err != nil {
		t.Fatalf("write input: %v", err)
	}

	result, err := ExecuteBlueprint(context.Background(), specstore.Blueprint{
		Name: "linear",
		Pipeline: specstore.Pipeline{
			Name: "linear-pipeline",
			Steps: []specstore.Step{
				{Name: "first", Resource: "builtin:copy"},
				{Name: "second", Resource: "builtin:copy"},
			},
		},
	}, input, dir)
	if err != nil {
		t.Fatalf("ExecuteBlueprint returned error: %v", err)
	}

	if result.Output != filepath.Join(dir, "final.csv") {
		t.Fatalf("output = %q, want final.csv in work dir", result.Output)
	}
	if got, want := len(result.Steps), 2; got != want {
		t.Fatalf("len(result.Steps) = %d, want %d", got, want)
	}
}

func TestExecuteLegacyPipelineKeepsStubCompatibility(t *testing.T) {
	dir := t.TempDir()
	input := filepath.Join(dir, "raw.csv")
	if err := os.WriteFile(input, []byte("a,b\n1,2\n"), 0o600); err != nil {
		t.Fatalf("write input: %v", err)
	}

	output, err := Execute(context.Background(), &Pipeline{
		Name:  "legacy",
		Steps: []Step{{Name: "copy", Command: "builtin:copy"}},
	}, input, dir)
	if err != nil {
		t.Fatalf("Execute returned error: %v", err)
	}
	if output != filepath.Join(dir, "final.csv") {
		t.Fatalf("output = %q, want final.csv in work dir", output)
	}
}

func TestExecuteBlueprintRejectsInvalidInputs(t *testing.T) {
	blueprint := specstore.Blueprint{
		Name: "copy",
		Pipeline: specstore.Pipeline{
			Name:    "copy-pipeline",
			StartAt: "copy",
			States: map[string]specstore.State{
				"copy": {Type: "task", Resource: "builtin:copy", End: true},
			},
		},
	}

	if _, err := ExecuteBlueprint(context.Background(), blueprint, "", t.TempDir()); !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("empty input error = %v, want ErrInvalidDefinition", err)
	}
	if _, err := ExecuteBlueprint(context.Background(), blueprint, "input.csv", ""); !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("empty work dir error = %v, want ErrInvalidDefinition", err)
	}
	if _, err := Execute(context.Background(), nil, "input.csv", t.TempDir()); !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("nil pipeline error = %v, want ErrInvalidDefinition", err)
	}
}

func TestStateExecutionPlanRejectsUnsupportedDefinitions(t *testing.T) {
	tests := []struct {
		name     string
		pipeline specstore.Pipeline
	}{
		{
			name: "unsupported type",
			pipeline: specstore.Pipeline{
				StartAt: "choice",
				States: map[string]specstore.State{
					"choice": {Type: "choice", Resource: "builtin:copy", End: true},
				},
			},
		},
		{
			name: "missing next",
			pipeline: specstore.Pipeline{
				StartAt: "copy",
				States: map[string]specstore.State{
					"copy": {Type: "task", Resource: "builtin:copy"},
				},
			},
		},
		{
			name: "unknown next",
			pipeline: specstore.Pipeline{
				StartAt: "copy",
				States: map[string]specstore.State{
					"copy": {Type: "task", Resource: "builtin:copy", Next: "missing"},
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := stateExecutionPlan(tt.pipeline)
			if !errors.Is(err, ErrInvalidDefinition) {
				t.Fatalf("error = %v, want ErrInvalidDefinition", err)
			}
		})
	}
}

func TestExecuteLinearStepsRejectsMissingExecutableResource(t *testing.T) {
	_, err := executeLinearSteps(context.Background(), specstore.Pipeline{
		Name:  "plan-only",
		Steps: []specstore.Step{{Name: "describe"}},
	}, "input.csv", t.TempDir())

	if !errors.Is(err, ErrNoExecutableResource) {
		t.Fatalf("error = %v, want ErrNoExecutableResource", err)
	}
}

func TestExecuteStateMachineReturnsPartialStepsOnFailure(t *testing.T) {
	dir := t.TempDir()
	input := filepath.Join(dir, "raw.csv")
	if err := os.WriteFile(input, []byte("a,b\n1,2\n"), 0o600); err != nil {
		t.Fatalf("write input: %v", err)
	}
	t.Setenv("PIPELINE_SCRIPT_DIR", t.TempDir())

	result, err := ExecuteBlueprint(context.Background(), specstore.Blueprint{
		Name: "partial-failure",
		Pipeline: specstore.Pipeline{
			Name:    "partial-failure-pipeline",
			StartAt: "copy",
			States: map[string]specstore.State{
				"copy": {
					Type:     "task",
					Resource: "builtin:copy",
					Next:     "missing-script",
				},
				"missing-script": {
					Type:     "task",
					Resource: "python:missing.py",
					End:      true,
				},
			},
		},
	}, input, dir)

	if err == nil {
		t.Fatal("expected missing script to fail")
	}
	if got, want := len(result.Steps), 2; got != want {
		t.Fatalf("len(result.Steps) = %d, want %d", got, want)
	}
	if result.Steps[0].Status != "success" {
		t.Fatalf("first step status = %q", result.Steps[0].Status)
	}
	if result.Steps[1].Status != "failed" {
		t.Fatalf("second step status = %q", result.Steps[1].Status)
	}
	if result.Steps[1].Input != result.Steps[0].Output {
		t.Fatalf("failed step input = %q, want previous output %q", result.Steps[1].Input, result.Steps[0].Output)
	}
}

func TestResolvePipelineReportsTodoAndRejectsBlankName(t *testing.T) {
	if _, err := ResolvePipeline(""); !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("blank name error = %v, want ErrInvalidDefinition", err)
	}

	_, err := ResolvePipeline("customer-chat")
	if err == nil || !strings.Contains(err.Error(), "TODO") {
		t.Fatalf("ResolvePipeline error = %v, want TODO error", err)
	}
}

func TestExecuteStepRejectsUnsupportedResource(t *testing.T) {
	_, err := executeStep(context.Background(), executableStep{
		Name:     "unsupported",
		Resource: "arn:aws:states:::lambda",
		End:      true,
	}, "input.csv", filepath.Join(t.TempDir(), "final.csv"))

	if !errors.Is(err, ErrNoExecutableResource) {
		t.Fatalf("error = %v, want ErrNoExecutableResource", err)
	}
}

func TestResolveScriptPathStaysUnderPipelineScriptDir(t *testing.T) {
	root := t.TempDir()
	script := filepath.Join(root, "step.py")
	if err := os.WriteFile(script, []byte("print('ok')\n"), 0o600); err != nil {
		t.Fatalf("write script: %v", err)
	}
	t.Setenv("PIPELINE_SCRIPT_DIR", root)

	resolved, err := resolveScriptPath("step.py")
	if err != nil {
		t.Fatalf("resolveScriptPath returned error: %v", err)
	}
	if resolved != script {
		t.Fatalf("resolved = %q, want %q", resolved, script)
	}

	_, err = resolveScriptPath("../secret.py")
	if !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("traversal error = %v, want ErrInvalidDefinition", err)
	}

	_, err = resolveScriptPath("")
	if !errors.Is(err, ErrNoExecutableResource) {
		t.Fatalf("blank script error = %v, want ErrNoExecutableResource", err)
	}

	_, err = resolveScriptPath(".")
	if !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("directory error = %v, want ErrInvalidDefinition", err)
	}
}

func TestResolveScriptPathRejectsSymlinkEscapingRoot(t *testing.T) {
	root := t.TempDir()
	outside := t.TempDir()
	outsideScript := filepath.Join(outside, "escape.py")
	if err := os.WriteFile(outsideScript, []byte("print('escape')\n"), 0o600); err != nil {
		t.Fatalf("write outside script: %v", err)
	}
	link := filepath.Join(root, "escape.py")
	if err := os.Symlink(outsideScript, link); err != nil {
		t.Skipf("cannot create symlink on this platform: %v", err)
	}
	t.Setenv("PIPELINE_SCRIPT_DIR", root)

	_, err := resolveScriptPath("escape.py")
	if !errors.Is(err, ErrInvalidDefinition) {
		t.Fatalf("symlink escape error = %v, want ErrInvalidDefinition", err)
	}
}

func TestResourceAndNameHelpers(t *testing.T) {
	if got := resourceFromCommand("clean.py"); got != "python:clean.py" {
		t.Fatalf("python resource = %q", got)
	}
	if got := resourceFromCommand("clean.sh"); got != "bash:clean.sh" {
		t.Fatalf("bash resource = %q", got)
	}
	if got := resourceFromCommand("builtin:copy"); got != "builtin:copy" {
		t.Fatalf("builtin resource = %q", got)
	}
	if got := sanitizeFileSegment("数据 load #1"); got != "load--1" {
		t.Fatalf("sanitized segment = %q", got)
	}
	if got := sanitizeFileSegment("数据"); got != "step" {
		t.Fatalf("empty sanitized segment = %q", got)
	}
	if got := stepOutputPath("work", 0, "load", false); got != filepath.Join("work", "step_0_load.csv") {
		t.Fatalf("step output = %q", got)
	}
	if got := stepOutputPath("work", 1, "finish", true); got != filepath.Join("work", "final.csv") {
		t.Fatalf("final output = %q", got)
	}
}

func TestPythonBinaryUsesEnvironmentOverride(t *testing.T) {
	t.Setenv("PYTHON_BIN", "custom-python")

	if got := pythonBinary(); got != "custom-python" {
		t.Fatalf("pythonBinary = %q, want custom-python", got)
	}
}
