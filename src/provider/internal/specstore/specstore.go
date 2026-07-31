package specstore

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"gopkg.in/yaml.v3"
)

const (
	SpecificationAPIVersion = "qtcloud.quanttide.com/v1alpha1"
	SpecificationKind       = "Specification"
)

type Blueprint struct {
	Name        string   `json:"name" yaml:"name"`
	Description string   `json:"description,omitempty" yaml:"description"`
	Contract    Contract `json:"contract" yaml:"contract"`
	Pipeline    Pipeline `json:"pipeline" yaml:"pipeline"`
	Status      string   `json:"status,omitempty" yaml:"status"`
	CreatedAt   string   `json:"created_at,omitempty" yaml:"created_at"`
	UpdatedAt   string   `json:"updated_at,omitempty" yaml:"updated_at"`
	SourcePath  string   `json:"source_path,omitempty" yaml:"-"`
}

type Contract struct {
	Input  ContractSide `json:"input" yaml:"input"`
	Output ContractSide `json:"output" yaml:"output"`
}

type ContractSide struct {
	Schema string   `json:"schema,omitempty" yaml:"schema"`
	Format string   `json:"format,omitempty" yaml:"format"`
	Rules  []string `json:"rules,omitempty" yaml:"rules"`
}

type Pipeline struct {
	Name    string           `json:"name" yaml:"name"`
	StartAt string           `json:"start_at,omitempty" yaml:"start_at"`
	States  map[string]State `json:"states,omitempty" yaml:"states"`
	Steps   []Step           `json:"steps" yaml:"steps"`
}

type Step struct {
	Name     string   `json:"name" yaml:"name"`
	From     string   `json:"from,omitempty" yaml:"from"`
	To       string   `json:"to,omitempty" yaml:"to"`
	Desc     string   `json:"desc,omitempty" yaml:"desc"`
	Resource string   `json:"resource,omitempty" yaml:"resource"`
	Command  string   `json:"command,omitempty" yaml:"command"`
	Depends  []string `json:"depends,omitempty" yaml:"depends"`
}

type State struct {
	Type     string   `json:"type" yaml:"type"`
	From     string   `json:"from,omitempty" yaml:"from"`
	To       string   `json:"to,omitempty" yaml:"to"`
	Desc     string   `json:"desc,omitempty" yaml:"desc"`
	Resource string   `json:"resource,omitempty" yaml:"resource"`
	Command  string   `json:"command,omitempty" yaml:"command"`
	Next     string   `json:"next,omitempty" yaml:"next"`
	End      bool     `json:"end,omitempty" yaml:"end"`
	Depends  []string `json:"depends,omitempty" yaml:"depends"`
}

type BlueprintSummary struct {
	Name            string `json:"name"`
	Description     string `json:"description,omitempty"`
	ContractSummary string `json:"contract_summary"`
	Pipeline        string `json:"pipeline"`
	RulesCount      int    `json:"rules_count"`
	Status          string `json:"status,omitempty"`
	SourcePath      string `json:"source_path,omitempty"`
}

type specificationEnvelope struct {
	APIVersion string `yaml:"api_version"`
	Kind       string `yaml:"kind"`
	Metadata   struct {
		Name        string `yaml:"name"`
		GeneratedBy string `yaml:"generated_by"`
		SourcePath  string `yaml:"source_path"`
	} `yaml:"metadata"`
	Spec struct {
		Blueprint Blueprint `yaml:"blueprint"`
	} `yaml:"spec"`
}

func SpecDir() string {
	if dir := os.Getenv("SPEC_DIR"); dir != "" {
		return dir
	}
	if root := os.Getenv("DATA_ROOT"); root != "" {
		return filepath.Join(root, "spec")
	}

	wd, err := os.Getwd()
	if err != nil {
		return filepath.Join(".quanttide", "data", "spec")
	}
	return defaultSpecDir(wd)
}

func LoadBlueprintSummaries(dir string) ([]BlueprintSummary, error) {
	blueprints, err := LoadBlueprints(dir)
	if err != nil {
		return nil, err
	}

	summaries := make([]BlueprintSummary, 0, len(blueprints))
	for _, blueprint := range blueprints {
		summaries = append(summaries, blueprint.Summary(blueprint.SourcePath))
	}
	return summaries, nil
}

func LoadBlueprints(dir string) ([]Blueprint, error) {
	paths, err := yamlPaths(dir)
	if err != nil {
		return nil, err
	}

	blueprintsByName := make(map[string]blueprintFile)
	for _, path := range paths {
		file, err := loadBlueprintFile(path)
		if err != nil {
			return nil, err
		}
		existing, exists := blueprintsByName[file.Blueprint.Name]
		if exists && existing.Priority == file.Priority {
			return nil, fmt.Errorf("%w: %s", ErrDuplicateBlueprint, file.Blueprint.Name)
		}
		if !exists || file.Priority > existing.Priority {
			blueprintsByName[file.Blueprint.Name] = file
		}
	}

	blueprints := make([]Blueprint, 0, len(blueprintsByName))
	for _, file := range blueprintsByName {
		blueprints = append(blueprints, file.Blueprint)
	}
	sort.Slice(blueprints, func(i, j int) bool {
		return blueprints[i].Name < blueprints[j].Name
	})
	return blueprints, nil
}

func LoadBlueprintByName(dir string, name string) (*Blueprint, error) {
	if !isSafeName(name) {
		return nil, fmt.Errorf("%w: %s", ErrInvalidName, name)
	}

	blueprints, err := LoadBlueprints(dir)
	if err != nil {
		return nil, err
	}
	for _, blueprint := range blueprints {
		if blueprint.Name == name {
			return &blueprint, nil
		}
	}
	return nil, ErrNotFound
}

func (b Blueprint) Summary(sourcePath string) BlueprintSummary {
	return BlueprintSummary{
		Name:            b.Name,
		Description:     b.Description,
		ContractSummary: contractSummary(b.Contract),
		Pipeline:        b.Pipeline.Name,
		RulesCount:      len(b.Contract.Output.Rules),
		Status:          b.Status,
		SourcePath:      sourcePath,
	}
}

var ErrNotFound = errors.New("blueprint not found")
var ErrInvalidName = errors.New("invalid blueprint name")
var ErrDuplicateBlueprint = errors.New("duplicate blueprint name")

type blueprintFile struct {
	Blueprint Blueprint
	Priority  int
}

func defaultSpecDir(cwd string) string {
	current := filepath.Join(cwd, ".quanttide", "data", "spec")
	candidates := []string{
		current,
		filepath.Clean(filepath.Join(cwd, "src", "cli", ".quanttide", "data", "spec")),
		filepath.Clean(filepath.Join(cwd, "..", "cli", ".quanttide", "data", "spec")),
	}

	for _, candidate := range candidates {
		if stat, err := os.Stat(candidate); err == nil && stat.IsDir() {
			return candidate
		}
	}
	return current
}

func yamlPaths(dir string) ([]string, error) {
	if _, err := os.Stat(dir); errors.Is(err, os.ErrNotExist) {
		return []string{}, nil
	}

	yamlFiles, err := filepath.Glob(filepath.Join(dir, "*.yaml"))
	if err != nil {
		return nil, err
	}
	ymlFiles, err := filepath.Glob(filepath.Join(dir, "*.yml"))
	if err != nil {
		return nil, err
	}

	paths := filterBlueprintSpecPaths(append(yamlFiles, ymlFiles...))
	sort.Strings(paths)
	return paths, nil
}

func filterBlueprintSpecPaths(paths []string) []string {
	out := make([]string, 0, len(paths))
	for _, path := range paths {
		name := filepath.Base(path)
		if strings.HasSuffix(name, "-blueprint.yaml") ||
			strings.HasSuffix(name, "-blueprint.yml") ||
			strings.HasSuffix(name, "-spec.yaml") ||
			strings.HasSuffix(name, "-spec.yml") {
			out = append(out, path)
		}
	}
	return out
}

func loadBlueprintFile(path string) (blueprintFile, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return blueprintFile{}, fmt.Errorf("read blueprint yaml %s: %w", path, err)
	}

	blueprint, isEnvelope, err := parseBlueprintYAML(content)
	if err != nil {
		return blueprintFile{}, fmt.Errorf("parse blueprint yaml %s: %w", path, err)
	}
	blueprint.SourcePath = filepath.Base(path)
	priority := 1
	if isEnvelope {
		priority = 2
	}
	return blueprintFile{Blueprint: blueprint, Priority: priority}, nil
}

func parseBlueprintYAML(content []byte) (Blueprint, bool, error) {
	var probe map[string]any
	if err := yaml.Unmarshal(content, &probe); err != nil {
		return Blueprint{}, false, err
	}

	if isSpecificationEnvelope(probe) {
		blueprint, err := parseSpecificationEnvelope(content)
		return blueprint, true, err
	}

	var blueprint Blueprint
	if err := yaml.Unmarshal(content, &blueprint); err != nil {
		return Blueprint{}, false, err
	}
	if blueprint.Name == "" {
		return Blueprint{}, false, errors.New("blueprint name is required")
	}
	return blueprint, false, nil
}

func isSpecificationEnvelope(probe map[string]any) bool {
	if _, ok := probe["api_version"]; ok {
		return true
	}
	if _, ok := probe["kind"]; ok {
		return true
	}
	if _, ok := probe["spec"]; ok {
		return true
	}
	return false
}

func parseSpecificationEnvelope(content []byte) (Blueprint, error) {
	var envelope specificationEnvelope
	if err := yaml.Unmarshal(content, &envelope); err != nil {
		return Blueprint{}, err
	}
	if envelope.APIVersion != SpecificationAPIVersion {
		return Blueprint{}, fmt.Errorf("unsupported api_version %q", envelope.APIVersion)
	}
	if envelope.Kind != SpecificationKind {
		return Blueprint{}, fmt.Errorf("unsupported kind %q", envelope.Kind)
	}
	if envelope.Spec.Blueprint.Name == "" {
		return Blueprint{}, errors.New("spec.blueprint.name is required")
	}
	return envelope.Spec.Blueprint, nil
}

func contractSummary(contract Contract) string {
	input := strings.TrimSpace(contract.Input.Format)
	if input == "" {
		input = strings.TrimSpace(contract.Input.Schema)
	}
	output := strings.TrimSpace(contract.Output.Format)
	if output == "" {
		output = strings.TrimSpace(contract.Output.Schema)
	}
	if input == "" {
		input = "input"
	}
	if output == "" {
		output = "output"
	}
	return input + " -> " + output
}

func isSafeName(name string) bool {
	if name == "" || name == "." || name == ".." {
		return false
	}
	return !strings.ContainsAny(name, `/\`)
}
