package api

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/quanttide/qtcloud-provider/internal/store"
)

func TestBlueprintEndpointsReadSpecDir(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "customer-chat-blueprint.yaml"), `name: "customer-chat"
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
pipeline:
  name: "customer-chat-pipeline"
  start_at: "数据加载与校验"
  states:
    数据加载与校验:
      type: task
      from: "原始 CSV"
      to: "校验后数据"
      desc: "检查必填字段"
      end: true
  steps:
    - name: "数据加载与校验"
      from: "原始 CSV"
      to: "校验后数据"
      desc: "检查必填字段"
status: draft
created_at: "2026-07-30T00:00:00+00:00"
updated_at: "2026-07-30T00:00:00+00:00"
`)

	t.Setenv("SPEC_DIR", dir)
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	listResp, err := http.Get(server.URL + "/blueprints")
	if err != nil {
		t.Fatalf("GET /blueprints: %v", err)
	}
	defer listResp.Body.Close()

	if listResp.StatusCode != http.StatusOK {
		t.Fatalf("GET /blueprints status = %d", listResp.StatusCode)
	}

	var list []map[string]any
	if err := json.NewDecoder(listResp.Body).Decode(&list); err != nil {
		t.Fatalf("decode /blueprints: %v", err)
	}
	if got, want := len(list), 1; got != want {
		t.Fatalf("len(list) = %d, want %d", got, want)
	}
	if list[0]["name"] != "customer-chat" {
		t.Fatalf("list[0].name = %v", list[0]["name"])
	}
	if list[0]["pipeline"] != "customer-chat-pipeline" {
		t.Fatalf("list[0].pipeline = %v", list[0]["pipeline"])
	}

	detailResp, err := http.Get(server.URL + "/blueprints/customer-chat")
	if err != nil {
		t.Fatalf("GET /blueprints/customer-chat: %v", err)
	}
	defer detailResp.Body.Close()

	if detailResp.StatusCode != http.StatusOK {
		t.Fatalf("GET /blueprints/customer-chat status = %d", detailResp.StatusCode)
	}

	var detail map[string]any
	if err := json.NewDecoder(detailResp.Body).Decode(&detail); err != nil {
		t.Fatalf("decode /blueprints/customer-chat: %v", err)
	}
	if detail["name"] != "customer-chat" {
		t.Fatalf("detail.name = %v", detail["name"])
	}
	pipeline := detail["pipeline"].(map[string]any)
	if pipeline["name"] != "customer-chat-pipeline" {
		t.Fatalf("detail.pipeline.name = %v", pipeline["name"])
	}
	if pipeline["start_at"] != "数据加载与校验" {
		t.Fatalf("detail.pipeline.start_at = %v", pipeline["start_at"])
	}
	states := pipeline["states"].(map[string]any)
	first := states["数据加载与校验"].(map[string]any)
	if first["type"] != "task" {
		t.Fatalf("state type = %v", first["type"])
	}
}

func TestBlueprintDetailReturns404WhenMissing(t *testing.T) {
	t.Setenv("SPEC_DIR", t.TempDir())
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Get(server.URL + "/blueprints/missing")
	if err != nil {
		t.Fatalf("GET /blueprints/missing: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("status = %d, want 404", resp.StatusCode)
	}
}

func TestBlueprintDetailReturns500WhenYAMLInvalid(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "bad-blueprint.yaml"), "name: [")
	t.Setenv("SPEC_DIR", dir)
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Get(server.URL + "/blueprints/bad")
	if err != nil {
		t.Fatalf("GET /blueprints/bad: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusInternalServerError {
		t.Fatalf("status = %d, want 500", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("read response body: %v", err)
	}
	if bytes.Contains(body, []byte(dir)) || bytes.Contains(body, []byte("yaml:")) {
		t.Fatalf("response leaked internal error details: %s", body)
	}
}

func TestBlueprintListReturns500WithoutLeakingYAMLDetails(t *testing.T) {
	dir := t.TempDir()
	writeFile(t, filepath.Join(dir, "bad-blueprint.yaml"), "name: [")
	t.Setenv("SPEC_DIR", dir)
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Get(server.URL + "/blueprints")
	if err != nil {
		t.Fatalf("GET /blueprints: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusInternalServerError {
		t.Fatalf("status = %d, want 500", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("read response body: %v", err)
	}
	if bytes.Contains(body, []byte(dir)) || bytes.Contains(body, []byte("yaml:")) {
		t.Fatalf("response leaked internal error details: %s", body)
	}
}

func TestRunBlueprintExecutesPipelineAndStoresJob(t *testing.T) {
	dir := t.TempDir()
	input := filepath.Join(dir, "raw.csv")
	workDir := filepath.Join(dir, "work")
	writeFile(t, input, "a,b\n1,2\n")
	writeFile(t, filepath.Join(dir, "copy-blueprint.yaml"), `name: "copy"
description: "copy demo"
contract:
  input:
    schema: "raw: string"
    format: "CSV"
  output:
    schema: "clean: string"
    format: "CSV"
pipeline:
  name: "copy-pipeline"
  start_at: "copy"
  states:
    copy:
      type: task
      resource: "builtin:copy"
      end: true
status: draft
`)

	t.Setenv("SPEC_DIR", dir)
	s := store.New()
	server := httptest.NewServer(Router(s))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/blueprints/copy/runs",
		"application/json",
		bytes.NewBufferString(`{"customer_id":"ABC-001","input_path":"`+filepath.ToSlash(input)+`","work_dir":"`+filepath.ToSlash(workDir)+`"}`),
	)
	if err != nil {
		t.Fatalf("POST /blueprints/copy/runs: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusCreated {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("status = %d, body = %s", resp.StatusCode, body)
	}

	var job map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&job); err != nil {
		t.Fatalf("decode run response: %v", err)
	}
	if job["blueprint"] != "copy" {
		t.Fatalf("job.blueprint = %v", job["blueprint"])
	}
	if job["pipeline"] != "copy-pipeline" {
		t.Fatalf("job.pipeline = %v", job["pipeline"])
	}
	if job["status"] != "success" {
		t.Fatalf("job.status = %v", job["status"])
	}
	output, ok := job["output"].(string)
	if !ok || output == "" {
		t.Fatalf("job.output = %v", job["output"])
	}
	content, err := os.ReadFile(filepath.FromSlash(output))
	if err != nil {
		t.Fatalf("read output: %v", err)
	}
	if string(content) != "a,b\n1,2\n" {
		t.Fatalf("output content = %q", content)
	}

	jobsResp, err := http.Get(server.URL + "/process/jobs")
	if err != nil {
		t.Fatalf("GET /process/jobs: %v", err)
	}
	defer jobsResp.Body.Close()
	var jobs []map[string]any
	if err := json.NewDecoder(jobsResp.Body).Decode(&jobs); err != nil {
		t.Fatalf("decode jobs: %v", err)
	}
	if got, want := len(jobs), 1; got != want {
		t.Fatalf("len(jobs) = %d, want %d", got, want)
	}
	if jobs[0]["id"] != job["id"] {
		t.Fatalf("stored job id = %v, want %v", jobs[0]["id"], job["id"])
	}
}

func TestRunBlueprintRejectsNonExecutablePipeline(t *testing.T) {
	dir := t.TempDir()
	input := filepath.Join(dir, "raw.csv")
	writeFile(t, input, "a,b\n1,2\n")
	writeFile(t, filepath.Join(dir, "plan-only-blueprint.yaml"), `name: "plan-only"
pipeline:
  name: "plan-only-pipeline"
  start_at: "review"
  states:
    review:
      type: task
      desc: "human-readable plan only"
      end: true
`)

	t.Setenv("SPEC_DIR", dir)
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/blueprints/plan-only/runs",
		"application/json",
		bytes.NewBufferString(`{"customer_id":"ABC-001","input_path":"`+filepath.ToSlash(input)+`"}`),
	)
	if err != nil {
		t.Fatalf("POST /blueprints/plan-only/runs: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusUnprocessableEntity {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("status = %d, body = %s", resp.StatusCode, body)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("read response body: %v", err)
	}
	if bytes.Contains(body, []byte(dir)) {
		t.Fatalf("response leaked internal paths: %s", body)
	}
}

func TestRunBlueprintRejectsBadRequests(t *testing.T) {
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(server.URL+"/blueprints/copy/runs", "application/json", bytes.NewBufferString("{"))
	if err != nil {
		t.Fatalf("POST invalid json: %v", err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("invalid json status = %d, want 400", resp.StatusCode)
	}

	resp, err = http.Post(server.URL+"/blueprints/copy/runs", "application/json", bytes.NewBufferString(`{"customer_id":"ABC"}`))
	if err != nil {
		t.Fatalf("POST missing input: %v", err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("missing input status = %d, want 400", resp.StatusCode)
	}
}

func TestRunBlueprintRejectsInputOutsideConfiguredRoot(t *testing.T) {
	specDir := t.TempDir()
	allowedInputDir := t.TempDir()
	outsideInputDir := t.TempDir()
	input := filepath.Join(outsideInputDir, "raw.csv")
	writeFile(t, input, "a,b\n1,2\n")
	writeFile(t, filepath.Join(specDir, "copy-blueprint.yaml"), `name: "copy"
pipeline:
  name: "copy-pipeline"
  start_at: "copy"
  states:
    copy:
      type: task
      resource: "builtin:copy"
      end: true
`)

	t.Setenv("SPEC_DIR", specDir)
	t.Setenv("PIPELINE_INPUT_DIR", allowedInputDir)
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/blueprints/copy/runs",
		"application/json",
		bytes.NewBufferString(`{"customer_id":"ABC-001","input_path":"`+filepath.ToSlash(input)+`"}`),
	)
	if err != nil {
		t.Fatalf("POST /blueprints/copy/runs: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusBadRequest {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("status = %d, body = %s", resp.StatusCode, body)
	}
}

func TestRunBlueprintRejectsInputSymlinkOutsideConfiguredRoot(t *testing.T) {
	specDir := t.TempDir()
	allowedInputDir := t.TempDir()
	outsideInputDir := t.TempDir()
	outsideInput := filepath.Join(outsideInputDir, "raw.csv")
	writeFile(t, outsideInput, "a,b\n1,2\n")
	inputLink := filepath.Join(allowedInputDir, "raw.csv")
	if err := os.Symlink(outsideInput, inputLink); err != nil {
		t.Skipf("cannot create symlink on this platform: %v", err)
	}
	writeExecutableCopyBlueprint(t, specDir)

	t.Setenv("SPEC_DIR", specDir)
	t.Setenv("PIPELINE_INPUT_DIR", allowedInputDir)
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/blueprints/copy/runs",
		"application/json",
		bytes.NewBufferString(`{"customer_id":"ABC-001","input_path":"`+filepath.ToSlash(inputLink)+`"}`),
	)
	if err != nil {
		t.Fatalf("POST /blueprints/copy/runs: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusBadRequest {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("status = %d, body = %s", resp.StatusCode, body)
	}
}

func TestRunBlueprintRejectsWorkDirSymlinkOutsideConfiguredRoot(t *testing.T) {
	specDir := t.TempDir()
	inputDir := t.TempDir()
	workRoot := t.TempDir()
	outsideWorkDir := t.TempDir()
	input := filepath.Join(inputDir, "raw.csv")
	workLink := filepath.Join(workRoot, "escape")
	writeFile(t, input, "a,b\n1,2\n")
	if err := os.Symlink(outsideWorkDir, workLink); err != nil {
		t.Skipf("cannot create symlink on this platform: %v", err)
	}
	writeExecutableCopyBlueprint(t, specDir)

	t.Setenv("SPEC_DIR", specDir)
	t.Setenv("PIPELINE_INPUT_DIR", inputDir)
	t.Setenv("PIPELINE_WORK_ROOT", workRoot)
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/blueprints/copy/runs",
		"application/json",
		bytes.NewBufferString(`{"customer_id":"ABC-001","input_path":"`+filepath.ToSlash(input)+`","work_dir":"`+filepath.ToSlash(workLink)+`"}`),
	)
	if err != nil {
		t.Fatalf("POST /blueprints/copy/runs: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusBadRequest {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("status = %d, body = %s", resp.StatusCode, body)
	}
}

func TestRunBlueprintCreatesConfiguredWorkRoot(t *testing.T) {
	specDir := t.TempDir()
	inputDir := t.TempDir()
	workRoot := filepath.Join(t.TempDir(), "provider-work")
	input := filepath.Join(inputDir, "raw.csv")
	writeFile(t, input, "a,b\n1,2\n")
	writeExecutableCopyBlueprint(t, specDir)

	t.Setenv("SPEC_DIR", specDir)
	t.Setenv("PIPELINE_INPUT_DIR", inputDir)
	t.Setenv("PIPELINE_WORK_ROOT", workRoot)
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/blueprints/copy/runs",
		"application/json",
		bytes.NewBufferString(`{"customer_id":"ABC-001","input_path":"`+filepath.ToSlash(input)+`"}`),
	)
	if err != nil {
		t.Fatalf("POST /blueprints/copy/runs: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusCreated {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("status = %d, body = %s", resp.StatusCode, body)
	}

	var job map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&job); err != nil {
		t.Fatalf("decode job: %v", err)
	}
	output := filepath.FromSlash(job["output"].(string))
	if _, err := os.Stat(output); err != nil {
		t.Fatalf("expected output under created work root: %v", err)
	}
	if rel, err := filepath.Rel(workRoot, output); err != nil || rel == ".." || strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
		t.Fatalf("output = %q, want under work root %q", output, workRoot)
	}
}

func TestRunBlueprintStoresFailedJobWithPartialSteps(t *testing.T) {
	specDir := t.TempDir()
	input := filepath.Join(specDir, "raw.csv")
	writeFile(t, input, "a,b\n1,2\n")
	writeFile(t, filepath.Join(specDir, "partial-blueprint.yaml"), `name: "partial"
pipeline:
  name: "partial-pipeline"
  start_at: "copy"
  states:
    copy:
      type: task
      resource: "builtin:copy"
      next: "missing-script"
    missing-script:
      type: task
      resource: "python:missing.py"
      end: true
`)

	t.Setenv("SPEC_DIR", specDir)
	t.Setenv("PIPELINE_SCRIPT_DIR", t.TempDir())
	s := store.New()
	server := httptest.NewServer(Router(s))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/blueprints/partial/runs",
		"application/json",
		bytes.NewBufferString(`{"customer_id":"ABC-001","input_path":"`+filepath.ToSlash(input)+`"}`),
	)
	if err != nil {
		t.Fatalf("POST /blueprints/partial/runs: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusInternalServerError {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("status = %d, body = %s", resp.StatusCode, body)
	}

	jobs := s.ListJobs()
	if got, want := len(jobs), 1; got != want {
		t.Fatalf("len(jobs) = %d, want %d", got, want)
	}
	job := jobs[0]
	if job.Status != "failed" {
		t.Fatalf("job status = %q, want failed", job.Status)
	}
	if got, want := len(job.Steps), 2; got != want {
		t.Fatalf("len(job.Steps) = %d, want %d", got, want)
	}
	if job.Steps[0].Status != "success" || job.Steps[1].Status != "failed" {
		t.Fatalf("step statuses = %#v", job.Steps)
	}
}

func TestReadOnlyEndpointsReturnJSON(t *testing.T) {
	s := store.New()
	_ = s.SaveJob(&store.JobRecord{ID: "job-1", CustomerID: "ABC", Pipeline: "csv", Status: "success"})
	server := httptest.NewServer(Router(s))
	defer server.Close()

	for _, path := range []string{"/providers", "/version", "/process/jobs"} {
		resp, err := http.Get(server.URL + path)
		if err != nil {
			t.Fatalf("GET %s: %v", path, err)
		}
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("GET %s status = %d", path, resp.StatusCode)
		}
		resp.Body.Close()
	}
}

func TestGetProcessJobReturnsSingleJob(t *testing.T) {
	s := store.New()
	_ = s.SaveJob(&store.JobRecord{
		ID:         "job-1",
		CustomerID: "ABC",
		Blueprint:  "customer-chat",
		Pipeline:   "customer-chat-pipeline",
		Status:     "success",
		Output:     "final.csv",
		Steps: []store.JobStep{
			{Name: "copy", Resource: "builtin:copy", Input: "raw.csv", Output: "final.csv", Status: "success"},
		},
	})
	server := httptest.NewServer(Router(s))
	defer server.Close()

	resp, err := http.Get(server.URL + "/process/jobs/job-1")
	if err != nil {
		t.Fatalf("GET /process/jobs/job-1: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("status = %d, want 200", resp.StatusCode)
	}

	var job map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&job); err != nil {
		t.Fatalf("decode job: %v", err)
	}
	if job["id"] != "job-1" {
		t.Fatalf("job.id = %v", job["id"])
	}
	if job["blueprint"] != "customer-chat" {
		t.Fatalf("job.blueprint = %v", job["blueprint"])
	}
	steps := job["steps"].([]any)
	first := steps[0].(map[string]any)
	if first["input"] != "raw.csv" {
		t.Fatalf("step input = %v", first["input"])
	}
}

func TestGetProcessJobReturns404WhenMissing(t *testing.T) {
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Get(server.URL + "/process/jobs/missing")
	if err != nil {
		t.Fatalf("GET /process/jobs/missing: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("status = %d, want 404", resp.StatusCode)
	}
}

func TestTransferSendRejectsBadRequests(t *testing.T) {
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(server.URL+"/transfer/send", "application/json", bytes.NewBufferString("{"))
	if err != nil {
		t.Fatalf("POST /transfer/send invalid json: %v", err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("invalid json status = %d, want 400", resp.StatusCode)
	}

	resp, err = http.Post(
		server.URL+"/transfer/send",
		"application/json",
		bytes.NewBufferString(`{"provider":"unknown","local_path":"a.csv","remote_path":"b.csv"}`),
	)
	if err != nil {
		t.Fatalf("POST /transfer/send unknown provider: %v", err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("unknown provider status = %d, want 400", resp.StatusCode)
	}
}

func TestTransferSendReturnsProviderResult(t *testing.T) {
	t.Setenv("DROPBOX_ACCESS_TOKEN", "test-token")
	dir := t.TempDir()
	input := filepath.Join(dir, "input.csv")
	writeFile(t, input, "a,b\n1,2\n")
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/transfer/send",
		"application/json",
		bytes.NewBufferString(`{"provider":"dropbox","local_path":"`+filepath.ToSlash(input)+`","remote_path":"/input.csv"}`),
	)
	if err != nil {
		t.Fatalf("POST /transfer/send: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("status = %d, body = %s", resp.StatusCode, body)
	}

	var body map[string]string
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if body["url"] == "" {
		t.Fatal("expected url in response")
	}
}

func TestTransferReceiveRejectsBadRequests(t *testing.T) {
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(server.URL+"/transfer/receive", "application/json", bytes.NewBufferString("{"))
	if err != nil {
		t.Fatalf("POST /transfer/receive invalid json: %v", err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("invalid json status = %d, want 400", resp.StatusCode)
	}

	resp, err = http.Post(
		server.URL+"/transfer/receive",
		"application/json",
		bytes.NewBufferString(`{"provider":"unknown","url":"https://example.com/a.csv","local_path":"a.csv"}`),
	)
	if err != nil {
		t.Fatalf("POST /transfer/receive unknown provider: %v", err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("unknown provider status = %d, want 400", resp.StatusCode)
	}
}

func TestTransferReceiveReturnsOKWhenProviderSucceeds(t *testing.T) {
	source := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte("a,b\n1,2\n"))
	}))
	defer source.Close()

	dir := t.TempDir()
	output := filepath.Join(dir, "received.csv")
	server := httptest.NewServer(Router(store.New()))
	defer server.Close()

	resp, err := http.Post(
		server.URL+"/transfer/receive",
		"application/json",
		bytes.NewBufferString(`{"provider":"dropbox","url":"`+source.URL+`","local_path":"`+filepath.ToSlash(output)+`"}`),
	)
	if err != nil {
		t.Fatalf("POST /transfer/receive: %v", err)
	}
	resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("status = %d, want 200", resp.StatusCode)
	}
	if _, err := os.Stat(output); err != nil {
		t.Fatalf("expected received file: %v", err)
	}
}

func writeFile(t *testing.T, path string, content string) {
	t.Helper()
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("write %s: %v", path, err)
	}
}

func writeExecutableCopyBlueprint(t *testing.T, dir string) {
	t.Helper()
	writeFile(t, filepath.Join(dir, "copy-blueprint.yaml"), `name: "copy"
pipeline:
  name: "copy-pipeline"
  start_at: "copy"
  states:
    copy:
      type: task
      resource: "builtin:copy"
      end: true
`)
}
