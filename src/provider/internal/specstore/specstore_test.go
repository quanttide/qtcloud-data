package specstore

import (
	"errors"
	"os"
	"path/filepath"
	"testing"
)

func TestLoadBlueprintsReadsLegacyAndSpecificationYAML(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "customer-chat-blueprint.yaml"), legacyBlueprintYAML())
	writeFile(t, filepath.Join(dir, "customer-chat-contract.yaml"), contractOnlyYAML())
	writeFile(t, filepath.Join(dir, "wrapped-spec.yaml"), envelopedSpecificationYAML())

	blueprints, err := LoadBlueprints(dir)
	if err != nil {
		t.Fatalf("LoadBlueprints returned error: %v", err)
	}

	if got, want := len(blueprints), 2; got != want {
		t.Fatalf("len(blueprints) = %d, want %d", got, want)
	}
	if blueprints[0].Name != "customer-chat" {
		t.Fatalf("first blueprint name = %q, want customer-chat", blueprints[0].Name)
	}
	if blueprints[0].Pipeline.Name != "customer-chat-pipeline" {
		t.Fatalf("pipeline name = %q", blueprints[0].Pipeline.Name)
	}
	if blueprints[0].SourcePath != "customer-chat-blueprint.yaml" {
		t.Fatalf("source path = %q, want file name only", blueprints[0].SourcePath)
	}
	if got := len(blueprints[0].Pipeline.Steps); got != 2 {
		t.Fatalf("pipeline steps = %d, want 2", got)
	}
	if blueprints[0].Pipeline.StartAt != "数据加载与校验" {
		t.Fatalf("pipeline start_at = %q", blueprints[0].Pipeline.StartAt)
	}
	if blueprints[0].Pipeline.States["数据加载与校验"].Next != "字段选择与输出" {
		t.Fatalf("first state next = %q", blueprints[0].Pipeline.States["数据加载与校验"].Next)
	}
	if !blueprints[0].Pipeline.States["字段选择与输出"].End {
		t.Fatal("last state should end the workflow")
	}
	if blueprints[1].Name != "wrapped" {
		t.Fatalf("second blueprint name = %q, want wrapped", blueprints[1].Name)
	}
}

func TestDefaultSpecDirFindsCliOutputWhenRunningFromProviderDir(t *testing.T) {
	root := t.TempDir()
	providerDir := filepath.Join(root, "src", "provider")
	cliSpecDir := filepath.Join(root, "src", "cli", ".quanttide", "data", "spec")
	if err := os.MkdirAll(providerDir, 0o700); err != nil {
		t.Fatalf("create provider dir: %v", err)
	}
	if err := os.MkdirAll(cliSpecDir, 0o700); err != nil {
		t.Fatalf("create cli spec dir: %v", err)
	}

	if got := defaultSpecDir(providerDir); got != cliSpecDir {
		t.Fatalf("defaultSpecDir() = %q, want %q", got, cliSpecDir)
	}
}

func TestLoadBlueprintsPrefersSpecificationEnvelopeOverLegacyBlueprint(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "customer-chat-blueprint.yaml"), legacyBlueprintYAML())
	writeFile(t, filepath.Join(dir, "customer-chat-spec.yaml"), envelopedCustomerChatSpecYAML())

	blueprints, err := LoadBlueprints(dir)
	if err != nil {
		t.Fatalf("LoadBlueprints returned error: %v", err)
	}

	if got, want := len(blueprints), 1; got != want {
		t.Fatalf("len(blueprints) = %d, want %d", got, want)
	}
	if blueprints[0].Description != "envelope wins" {
		t.Fatalf("description = %q, want envelope wins", blueprints[0].Description)
	}
	if blueprints[0].SourcePath != "customer-chat-spec.yaml" {
		t.Fatalf("source path = %q, want customer-chat-spec.yaml", blueprints[0].SourcePath)
	}
}

func TestLoadBlueprintByNameRejectsPathTraversal(t *testing.T) {
	dir := t.TempDir()

	_, err := LoadBlueprintByName(dir, "../secret")
	if !errors.Is(err, ErrInvalidName) {
		t.Fatal("expected path traversal name to fail")
	}
}

func TestLoadBlueprintByNameFindsMatchingBlueprint(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "customer-chat-blueprint.yaml"), legacyBlueprintYAML())

	blueprint, err := LoadBlueprintByName(dir, "customer-chat")
	if err != nil {
		t.Fatalf("LoadBlueprintByName returned error: %v", err)
	}

	if blueprint.Name != "customer-chat" {
		t.Fatalf("blueprint name = %q", blueprint.Name)
	}
}

func TestLoadBlueprintByNameReturnsNotFound(t *testing.T) {
	dir := t.TempDir()

	_, err := LoadBlueprintByName(dir, "missing")
	if !errors.Is(err, ErrNotFound) {
		t.Fatalf("error = %v, want ErrNotFound", err)
	}
}

func TestLoadBlueprintsTreatsMissingDirAsEmptyList(t *testing.T) {
	dir := filepath.Join(t.TempDir(), "missing")

	blueprints, err := LoadBlueprints(dir)
	if err != nil {
		t.Fatalf("LoadBlueprints returned error: %v", err)
	}
	if len(blueprints) != 0 {
		t.Fatalf("len(blueprints) = %d, want 0", len(blueprints))
	}
}

func TestLoadBlueprintSummariesReturnsListShapeForStudio(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "customer-chat-blueprint.yaml"), legacyBlueprintYAML())

	summaries, err := LoadBlueprintSummaries(dir)
	if err != nil {
		t.Fatalf("LoadBlueprintSummaries returned error: %v", err)
	}

	if got, want := len(summaries), 1; got != want {
		t.Fatalf("len(summaries) = %d, want %d", got, want)
	}
	if summaries[0].Name != "customer-chat" {
		t.Fatalf("summary name = %q", summaries[0].Name)
	}
	if summaries[0].Pipeline != "customer-chat-pipeline" {
		t.Fatalf("summary pipeline = %q", summaries[0].Pipeline)
	}
}

func TestLoadBlueprintsRejectsUnsupportedSpecificationVersion(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "bad-spec.yaml"), `api_version: qtcloud.quanttide.com/v9
kind: Specification
spec:
  blueprint:
    name: bad
`)

	_, err := LoadBlueprints(dir)
	if err == nil {
		t.Fatal("expected unsupported specification version to fail")
	}
}

func TestLoadBlueprintsRejectsUnsupportedSpecificationKind(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "bad-spec.yaml"), `api_version: qtcloud.quanttide.com/v1alpha1
kind: Contract
spec:
  blueprint:
    name: bad
`)

	_, err := LoadBlueprints(dir)
	if err == nil {
		t.Fatal("expected unsupported specification kind to fail")
	}
}

func TestSpecDirUsesEnvOverrides(t *testing.T) {
	t.Setenv("SPEC_DIR", "custom-spec")
	t.Setenv("DATA_ROOT", "ignored-root")
	if got, want := SpecDir(), "custom-spec"; got != want {
		t.Fatalf("SpecDir() = %q, want %q", got, want)
	}

	t.Setenv("SPEC_DIR", "")
	t.Setenv("DATA_ROOT", "data-root")
	if got, want := SpecDir(), filepath.Join("data-root", "spec"); got != want {
		t.Fatalf("SpecDir() = %q, want %q", got, want)
	}
}

func TestLoadBlueprintsRejectsDuplicateLegacyBlueprintNames(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "a-blueprint.yaml"), legacyBlueprintYAML())
	writeFile(t, filepath.Join(dir, "b-blueprint.yaml"), legacyBlueprintYAML())

	_, err := LoadBlueprints(dir)
	if !errors.Is(err, ErrDuplicateBlueprint) {
		t.Fatalf("error = %v, want ErrDuplicateBlueprint", err)
	}
}

func TestBlueprintSummaryCountsOutputRules(t *testing.T) {
	bp := Blueprint{
		Name:        "customer-chat",
		Description: "客户订单清洗",
		Contract: Contract{
			Input:  ContractSide{Schema: "订单号,客户名,金额", Format: "CSV"},
			Output: ContractSide{Schema: "订单号,客户名,金额", Format: "CSV", Rules: []string{"订单号非空", "金额保留两位小数"}},
		},
		Pipeline: Pipeline{Name: "customer-chat-pipeline"},
		Status:   "draft",
	}

	summary := bp.Summary("customer-chat-blueprint.yaml")

	if summary.Name != "customer-chat" {
		t.Fatalf("summary name = %q", summary.Name)
	}
	if summary.ContractSummary != "CSV -> CSV" {
		t.Fatalf("contract summary = %q", summary.ContractSummary)
	}
	if summary.RulesCount != 2 {
		t.Fatalf("rules count = %d, want 2", summary.RulesCount)
	}
}

func TestBlueprintSummaryFallsBackToSchemaWhenFormatMissing(t *testing.T) {
	bp := Blueprint{
		Name: "schema-only",
		Contract: Contract{
			Input:  ContractSide{Schema: "raw: string"},
			Output: ContractSide{Schema: "clean: string"},
		},
	}

	summary := bp.Summary("schema-only-blueprint.yaml")

	if summary.ContractSummary != "raw: string -> clean: string" {
		t.Fatalf("contract summary = %q", summary.ContractSummary)
	}
}

func TestBlueprintSummaryUsesDefaultContractLabels(t *testing.T) {
	bp := Blueprint{Name: "empty-contract"}

	summary := bp.Summary("empty-contract-blueprint.yaml")

	if summary.ContractSummary != "input -> output" {
		t.Fatalf("contract summary = %q", summary.ContractSummary)
	}
}

func writeFile(t *testing.T, path string, content string) {
	t.Helper()
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("write %s: %v", path, err)
	}
}

func legacyBlueprintYAML() string {
	return `name: "customer-chat"
description: "客户订单清洗"
contract:
  input:
    schema: "订单号,客户名,金额,下单日期"
    format: "CSV"
  output:
    schema: "订单号,客户名,金额,下单日期"
    format: "CSV"
    rules:
      - "订单号非空"
      - "金额保留两位小数"
pipeline:
  name: "customer-chat-pipeline"
  start_at: "数据加载与校验"
  states:
    数据加载与校验:
      type: task
      from: "原始 CSV 文件"
      to: "校验后的数据"
      desc: "检查必填字段"
      next: "字段选择与输出"
    字段选择与输出:
      type: task
      from: "校验后的数据"
      to: "最终交付 CSV"
      desc: "生成交付文件"
      end: true
  steps:
    - name: "数据加载与校验"
      from: "原始 CSV 文件"
      to: "校验后的数据"
      desc: "检查必填字段"
    - name: "字段选择与输出"
      from: "校验后的数据"
      to: "最终交付 CSV"
      desc: "生成交付文件"
      depends:
        - "数据加载与校验"
status: draft
created_at: "2026-07-30T00:00:00+00:00"
updated_at: "2026-07-30T00:00:00+00:00"
`
}

func envelopedSpecificationYAML() string {
	return `api_version: qtcloud.quanttide.com/v1alpha1
kind: Specification
metadata:
  name: wrapped
  generated_by: qtcloud-data-cli
  source_path: wrapped-blueprint.yaml
spec:
  blueprint:
    name: "wrapped"
    description: "enveloped blueprint"
    contract:
      input:
        schema: "raw: string"
        format: "CSV"
      output:
        schema: "clean: string"
        format: "CSV"
    pipeline:
      name: "wrapped-pipeline"
      steps:
        - name: "clean"
          from: "raw"
          to: "clean"
          desc: "trim whitespace"
    status: draft
    created_at: "2026-07-30T00:00:00+00:00"
    updated_at: "2026-07-30T00:00:00+00:00"
`
}

func envelopedCustomerChatSpecYAML() string {
	return `api_version: qtcloud.quanttide.com/v1alpha1
kind: Specification
metadata:
  name: customer-chat
  generated_by: qtcloud-data-cli
spec:
  blueprint:
    name: "customer-chat"
    description: "envelope wins"
    contract:
      input:
        schema: "raw: string"
        format: "CSV"
      output:
        schema: "clean: string"
        format: "CSV"
    pipeline:
      name: "customer-chat-pipeline"
      steps:
        - name: "clean"
          from: "raw"
          to: "clean"
          desc: "trim whitespace"
    status: draft
    created_at: "2026-07-30T00:00:00+00:00"
    updated_at: "2026-07-30T00:00:00+00:00"
`
}

func contractOnlyYAML() string {
	return `contract:
  input:
    schema: "订单号,客户名,金额,下单日期"
    format: "CSV"
  output:
    schema: "订单号,客户名,金额,下单日期"
    format: "CSV"
`
}
