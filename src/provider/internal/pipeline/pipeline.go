package pipeline

import (
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/quanttide/qtcloud-provider/internal/specstore"
)

var (
	ErrInvalidDefinition    = errors.New("invalid pipeline definition")
	ErrNoExecutableResource = errors.New("pipeline step has no executable resource")
)

type Step struct {
	Name    string `json:"name"`
	Command string `json:"command"`
}

type Pipeline struct {
	Name        string `json:"name"`
	Description string `json:"description,omitempty"`
	Steps       []Step `json:"steps"`
}

type Run struct {
	CustomerID string `json:"customer_id"`
	Pipeline   string `json:"pipeline"`
	Status     string `json:"status"` // running / success / failed
	Log        string `json:"log,omitempty"`
}

type ExecutionResult struct {
	Output string       `json:"output"`
	Steps  []StepResult `json:"steps"`
}

type StepResult struct {
	Name     string `json:"name"`
	Resource string `json:"resource"`
	Input    string `json:"input"`
	Output   string `json:"output"`
	Status   string `json:"status"`
}

type executableStep struct {
	Name     string
	Resource string
	End      bool
	Next     string
}

func ExecuteBlueprint(ctx context.Context, blueprint specstore.Blueprint, input, workDir string) (ExecutionResult, error) {
	if strings.TrimSpace(input) == "" {
		return ExecutionResult{}, fmt.Errorf("%w: input_path is required", ErrInvalidDefinition)
	}
	if strings.TrimSpace(workDir) == "" {
		return ExecutionResult{}, fmt.Errorf("%w: work_dir is required", ErrInvalidDefinition)
	}
	if err := os.MkdirAll(workDir, 0o700); err != nil {
		return ExecutionResult{}, fmt.Errorf("create work_dir: %w", err)
	}

	if len(blueprint.Pipeline.States) > 0 {
		return executeStateMachine(ctx, blueprint.Pipeline, input, workDir)
	}
	return executeLinearSteps(ctx, blueprint.Pipeline, input, workDir)
}

func Execute(ctx context.Context, p *Pipeline, input, workDir string) (string, error) {
	if p == nil {
		return "", fmt.Errorf("%w: pipeline is nil", ErrInvalidDefinition)
	}
	steps := make([]specstore.Step, 0, len(p.Steps))
	for _, step := range p.Steps {
		steps = append(steps, specstore.Step{
			Name:     step.Name,
			Resource: resourceFromCommand(step.Command),
			Command:  step.Command,
		})
	}
	result, err := executeLinearSteps(ctx, specstore.Pipeline{Name: p.Name, Steps: steps}, input, workDir)
	if err != nil {
		return "", err
	}
	return result.Output, nil
}

func ResolvePipeline(name string) (*Pipeline, error) {
	dir := os.Getenv("PIPELINES_DIR")
	if dir == "" {
		dir = ".quanttide/data/pipelines"
	}
	if strings.TrimSpace(name) == "" {
		return nil, fmt.Errorf("%w: pipeline name is required", ErrInvalidDefinition)
	}
	return nil, fmt.Errorf("TODO: CUE integration in %s", dir)
}

func executeStateMachine(ctx context.Context, p specstore.Pipeline, input, workDir string) (ExecutionResult, error) {
	steps, err := stateExecutionPlan(p)
	if err != nil {
		return ExecutionResult{}, err
	}

	prev := input
	result := ExecutionResult{Steps: make([]StepResult, 0, len(steps))}
	for i, step := range steps {
		output := stepOutputPath(workDir, i, step.Name, step.End)
		stepResult, err := executeStep(ctx, step, prev, output)
		if err != nil {
			result.Steps = append(result.Steps, failedStep(step, prev, output))
			result.Output = prev
			return result, err
		}
		result.Steps = append(result.Steps, stepResult)
		prev = output
	}
	result.Output = prev
	return result, nil
}

func stateExecutionPlan(p specstore.Pipeline) ([]executableStep, error) {
	if strings.TrimSpace(p.StartAt) == "" {
		return nil, fmt.Errorf("%w: pipeline.start_at is required", ErrInvalidDefinition)
	}

	current := p.StartAt
	visited := make(map[string]bool)
	steps := make([]executableStep, 0, len(p.States))
	for {
		if visited[current] {
			return nil, fmt.Errorf("%w: state cycle at %q", ErrInvalidDefinition, current)
		}
		visited[current] = true

		state, ok := p.States[current]
		if !ok {
			return nil, fmt.Errorf("%w: state %q not found", ErrInvalidDefinition, current)
		}
		if state.Type != "" && !strings.EqualFold(state.Type, "task") {
			return nil, fmt.Errorf("%w: unsupported state type %q", ErrInvalidDefinition, state.Type)
		}

		step := executableStep{
			Name:     current,
			Resource: firstNonEmpty(state.Resource, resourceFromCommand(state.Command)),
			End:      state.End,
			Next:     state.Next,
		}
		if step.Resource == "" {
			return nil, fmt.Errorf("%w: state %q", ErrNoExecutableResource, current)
		}
		steps = append(steps, step)

		if state.End {
			return steps, nil
		}
		if strings.TrimSpace(state.Next) == "" {
			return nil, fmt.Errorf("%w: state %q must define next or end", ErrInvalidDefinition, current)
		}
		current = state.Next
	}
}

func executeLinearSteps(ctx context.Context, p specstore.Pipeline, input, workDir string) (ExecutionResult, error) {
	if len(p.Steps) == 0 {
		return ExecutionResult{}, fmt.Errorf("%w: pipeline has no states or steps", ErrInvalidDefinition)
	}

	prev := input
	result := ExecutionResult{Steps: make([]StepResult, 0, len(p.Steps))}
	for i, step := range p.Steps {
		name := firstNonEmpty(step.Name, fmt.Sprintf("step-%d", i+1))
		executable := executableStep{
			Name:     name,
			Resource: firstNonEmpty(step.Resource, resourceFromCommand(step.Command)),
			End:      i == len(p.Steps)-1,
		}
		if executable.Resource == "" {
			return ExecutionResult{}, fmt.Errorf("%w: step %q", ErrNoExecutableResource, name)
		}

		output := stepOutputPath(workDir, i, name, executable.End)
		stepResult, err := executeStep(ctx, executable, prev, output)
		if err != nil {
			result.Steps = append(result.Steps, failedStep(executable, prev, output))
			result.Output = prev
			return result, err
		}
		result.Steps = append(result.Steps, stepResult)
		prev = output
	}
	result.Output = prev
	return result, nil
}

func executeStep(ctx context.Context, step executableStep, input, output string) (StepResult, error) {
	if err := os.MkdirAll(filepath.Dir(output), 0o700); err != nil {
		return StepResult{}, fmt.Errorf("create output dir: %w", err)
	}

	if step.Resource == "builtin:copy" {
		if err := copyFile(input, output); err != nil {
			return StepResult{}, fmt.Errorf("execute %q: %w", step.Name, err)
		}
		return successfulStep(step, input, output), nil
	}

	if err := executeScript(ctx, step.Resource, input, output); err != nil {
		return StepResult{}, fmt.Errorf("execute %q: %w", step.Name, err)
	}
	return successfulStep(step, input, output), nil
}

func successfulStep(step executableStep, input, output string) StepResult {
	return StepResult{
		Name:     step.Name,
		Resource: step.Resource,
		Input:    input,
		Output:   output,
		Status:   "success",
	}
}

func failedStep(step executableStep, input, output string) StepResult {
	return StepResult{
		Name:     step.Name,
		Resource: step.Resource,
		Input:    input,
		Output:   output,
		Status:   "failed",
	}
}

func copyFile(input, output string) error {
	src, err := os.Open(input)
	if err != nil {
		return err
	}
	defer src.Close()

	dst, err := os.OpenFile(output, os.O_CREATE|os.O_TRUNC|os.O_WRONLY, 0o600)
	if err != nil {
		return err
	}
	defer dst.Close()

	_, err = io.Copy(dst, src)
	return err
}

func executeScript(ctx context.Context, resource, input, output string) error {
	kind, scriptRef, ok := strings.Cut(resource, ":")
	if !ok {
		return fmt.Errorf("%w: unsupported resource %q", ErrNoExecutableResource, resource)
	}

	var runner string
	switch strings.ToLower(kind) {
	case "python", "python3":
		runner = pythonBinary()
	case "bash", "sh":
		runner = "bash"
	default:
		return fmt.Errorf("%w: unsupported resource %q", ErrNoExecutableResource, resource)
	}

	scriptPath, err := resolveScriptPath(scriptRef)
	if err != nil {
		return err
	}

	cmd := exec.CommandContext(ctx, runner, scriptPath, input, output)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func resolveScriptPath(scriptRef string) (string, error) {
	if strings.TrimSpace(scriptRef) == "" {
		return "", fmt.Errorf("%w: script path is required", ErrNoExecutableResource)
	}

	root := os.Getenv("PIPELINE_SCRIPT_DIR")
	if root == "" {
		wd, err := os.Getwd()
		if err != nil {
			return "", err
		}
		root = wd
	}

	candidateAbs, err := ResolvePathUnderRoot(root, scriptRef, true)
	if err != nil {
		return "", err
	}
	info, err := os.Stat(candidateAbs)
	if err != nil {
		return "", err
	}
	if info.IsDir() {
		return "", fmt.Errorf("%w: script path is a directory", ErrInvalidDefinition)
	}
	return candidateAbs, nil
}

func ResolvePathUnderRoot(root, pathValue string, mustExist bool) (string, error) {
	root = strings.TrimSpace(root)
	pathValue = strings.TrimSpace(pathValue)
	if root == "" {
		return "", fmt.Errorf("%w: root is required", ErrInvalidDefinition)
	}
	if pathValue == "" {
		return "", fmt.Errorf("%w: path is required", ErrInvalidDefinition)
	}

	rootAbs, err := filepath.Abs(filepath.Clean(root))
	if err != nil {
		return "", err
	}
	rootReal, err := filepath.EvalSymlinks(rootAbs)
	if err != nil {
		return "", fmt.Errorf("%w: resolve root: %w", ErrInvalidDefinition, err)
	}
	rootReal, err = filepath.Abs(filepath.Clean(rootReal))
	if err != nil {
		return "", err
	}

	var candidate string
	if filepath.IsAbs(pathValue) {
		candidate = filepath.Clean(pathValue)
	} else {
		candidate = filepath.Join(rootAbs, filepath.Clean(pathValue))
	}
	candidateAbs, err := filepath.Abs(candidate)
	if err != nil {
		return "", err
	}
	if !pathWithinRoot(rootAbs, candidateAbs) {
		return "", fmt.Errorf("%w: path must stay under configured root", ErrInvalidDefinition)
	}

	candidateReal, err := resolveCandidatePath(candidateAbs, mustExist)
	if err != nil {
		return "", err
	}
	if !pathWithinRoot(rootReal, candidateReal) {
		return "", fmt.Errorf("%w: path must stay under configured root", ErrInvalidDefinition)
	}
	return candidateReal, nil
}

func resolveCandidatePath(candidateAbs string, mustExist bool) (string, error) {
	resolved, err := filepath.EvalSymlinks(candidateAbs)
	if err == nil {
		return filepath.Abs(filepath.Clean(resolved))
	}
	if mustExist || !os.IsNotExist(err) {
		return "", err
	}

	existing := candidateAbs
	for {
		if _, statErr := os.Lstat(existing); statErr == nil {
			break
		} else if !os.IsNotExist(statErr) {
			return "", statErr
		}
		parent := filepath.Dir(existing)
		if parent == existing {
			return "", err
		}
		existing = parent
	}

	existingReal, err := filepath.EvalSymlinks(existing)
	if err != nil {
		return "", err
	}
	rel, err := filepath.Rel(existing, candidateAbs)
	if err != nil {
		return "", err
	}
	combined := filepath.Join(existingReal, rel)
	return filepath.Abs(filepath.Clean(combined))
}

func pathWithinRoot(root, candidate string) bool {
	rel, err := filepath.Rel(root, candidate)
	if err != nil {
		return false
	}
	return rel == "." || (rel != ".." && !strings.HasPrefix(rel, ".."+string(filepath.Separator)))
}

func stepOutputPath(workDir string, index int, name string, isFinal bool) string {
	if isFinal {
		return filepath.Join(workDir, "final.csv")
	}
	return filepath.Join(workDir, fmt.Sprintf("step_%d_%s.csv", index, sanitizeFileSegment(name)))
}

func sanitizeFileSegment(value string) string {
	var b strings.Builder
	for _, ch := range value {
		if ch >= 'a' && ch <= 'z' || ch >= 'A' && ch <= 'Z' || ch >= '0' && ch <= '9' || ch == '-' || ch == '_' {
			b.WriteRune(ch)
			continue
		}
		b.WriteByte('-')
	}
	out := strings.Trim(b.String(), "-")
	if out == "" {
		return "step"
	}
	return out
}

func resourceFromCommand(command string) string {
	command = strings.TrimSpace(command)
	if command == "" {
		return ""
	}
	switch {
	case strings.HasSuffix(command, ".py"):
		return "python:" + command
	case strings.HasSuffix(command, ".sh"):
		return "bash:" + command
	default:
		return command
	}
}

func pythonBinary() string {
	if bin := os.Getenv("PYTHON_BIN"); bin != "" {
		return bin
	}
	if runtime.GOOS == "windows" {
		return "python"
	}
	return "python3"
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if strings.TrimSpace(value) != "" {
			return strings.TrimSpace(value)
		}
	}
	return ""
}
