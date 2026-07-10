package pipeline

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"strings"
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

func Execute(ctx context.Context, p *Pipeline, input, workDir string) (string, error) {
	prev := input
	for i, step := range p.Steps {
		stepOutput := fmt.Sprintf("%s/step_%d.csv", workDir, i)
		if i == len(p.Steps)-1 {
			stepOutput = fmt.Sprintf("%s/final.csv", workDir)
		}

		cmd := exec.CommandContext(ctx, "python3", step.Command, prev, stepOutput)
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			return "", fmt.Errorf("step %q 失败: %w", step.Name, err)
		}
		prev = stepOutput
	}
	return prev, nil
}

func ResolvePipeline(name string) (*Pipeline, error) {
	// 从 CUE 解析（与 CLI 相同逻辑）
	dir := os.Getenv("PIPELINES_DIR")
	if dir == "" {
		dir = "./pipelines"
	}
	key := strings.ReplaceAll(name, "-", "")
	key = strings.Replace(key, string(key[0]), strings.ToLower(string(key[0])), 1)
	// Todo: cue export --out yaml --expression ...
	return nil, fmt.Errorf("TODO: CUE integration")
}
