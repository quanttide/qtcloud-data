package api

import (
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/quanttide/qtcloud-provider/internal/pipeline"
	"github.com/quanttide/qtcloud-provider/internal/provider"
	"github.com/quanttide/qtcloud-provider/internal/specstore"
	"github.com/quanttide/qtcloud-provider/internal/store"
)

type Handler struct {
	Store *store.Store
}

func NewHandler(s *store.Store) *Handler {
	return &Handler{Store: s}
}

// GET /providers — 列出支持的提供商
func (h *Handler) ListProviders(w http.ResponseWriter, r *http.Request) {
	json.NewEncoder(w).Encode(provider.List())
}

// GET /blueprints — 从 CLI 生成的 Specification YAML 目录列出蓝图
func (h *Handler) ListBlueprints(w http.ResponseWriter, r *http.Request) {
	summaries, err := specstore.LoadBlueprintSummaries(specstore.SpecDir())
	if err != nil {
		log.Printf("load blueprints failed: %v", err)
		http.Error(w, "failed to load blueprints", http.StatusInternalServerError)
		return
	}
	writeJSON(w, summaries)
}

// GET /blueprints/{name} — 查看单个蓝图详情
func (h *Handler) GetBlueprint(w http.ResponseWriter, r *http.Request) {
	blueprint, err := specstore.LoadBlueprintByName(specstore.SpecDir(), r.PathValue("name"))
	if err != nil {
		if errors.Is(err, specstore.ErrNotFound) {
			http.Error(w, "blueprint not found", http.StatusNotFound)
			return
		}
		if errors.Is(err, specstore.ErrInvalidName) {
			http.Error(w, "invalid blueprint name", http.StatusBadRequest)
			return
		}
		log.Printf("load blueprint failed: %v", err)
		http.Error(w, "failed to load blueprint", http.StatusInternalServerError)
		return
	}
	writeJSON(w, blueprint)
}

// POST /blueprints/{name}/runs — 执行 Blueprint Pipeline
func (h *Handler) RunBlueprint(w http.ResponseWriter, r *http.Request) {
	var body struct {
		CustomerID string `json:"customer_id"`
		InputPath  string `json:"input_path"`
		WorkDir    string `json:"work_dir"`
	}
	decoder := json.NewDecoder(r.Body)
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&body); err != nil {
		http.Error(w, "invalid request", http.StatusBadRequest)
		return
	}
	body.CustomerID = strings.TrimSpace(body.CustomerID)
	body.InputPath = strings.TrimSpace(body.InputPath)
	body.WorkDir = strings.TrimSpace(body.WorkDir)
	if body.CustomerID == "" || body.InputPath == "" {
		http.Error(w, "customer_id and input_path are required", http.StatusBadRequest)
		return
	}
	if err := validatePathUnderOptionalRoot(body.InputPath, "PIPELINE_INPUT_DIR"); err != nil {
		http.Error(w, "input_path is outside allowed root", http.StatusBadRequest)
		return
	}

	blueprint, err := specstore.LoadBlueprintByName(specstore.SpecDir(), r.PathValue("name"))
	if err != nil {
		if errors.Is(err, specstore.ErrNotFound) {
			http.Error(w, "blueprint not found", http.StatusNotFound)
			return
		}
		if errors.Is(err, specstore.ErrInvalidName) {
			http.Error(w, "invalid blueprint name", http.StatusBadRequest)
			return
		}
		log.Printf("load blueprint for run failed: %v", err)
		http.Error(w, "failed to load blueprint", http.StatusInternalServerError)
		return
	}

	startedAt := time.Now().UTC()
	job := &store.JobRecord{
		ID:         newJobID(body.CustomerID, startedAt),
		CustomerID: body.CustomerID,
		Blueprint:  blueprint.Name,
		Pipeline:   blueprint.Pipeline.Name,
		Status:     "running",
		Input:      body.InputPath,
		StartedAt:  startedAt.Format(time.RFC3339),
	}
	if body.WorkDir == "" {
		workRoot := strings.TrimSpace(os.Getenv("PIPELINE_WORK_ROOT"))
		if workRoot == "" {
			workRoot = filepath.Join(os.TempDir(), "qtcloud-provider")
		}
		body.WorkDir = filepath.Join(workRoot, job.ID)
	}
	if err := validateWorkDirUnderOptionalRoot(body.WorkDir); err != nil {
		http.Error(w, "work_dir is outside allowed root", http.StatusBadRequest)
		return
	}
	if err := h.Store.SaveJob(job); err != nil {
		log.Printf("save running job failed: %v", err)
		http.Error(w, "failed to save job", http.StatusInternalServerError)
		return
	}

	result, err := pipeline.ExecuteBlueprint(r.Context(), *blueprint, body.InputPath, body.WorkDir)
	job.FinishedAt = time.Now().UTC().Format(time.RFC3339)
	if err != nil {
		job.Status = "failed"
		job.Error = safePipelineRunError(err)
		applyExecutionResult(job, result)
		if saveErr := h.Store.SaveJob(job); saveErr != nil {
			log.Printf("save failed job failed: %v", saveErr)
		}
		if errors.Is(err, pipeline.ErrInvalidDefinition) || errors.Is(err, pipeline.ErrNoExecutableResource) {
			log.Printf("blueprint pipeline is not executable: %v", err)
			http.Error(w, "blueprint pipeline is not executable", http.StatusUnprocessableEntity)
			return
		}
		log.Printf("pipeline execution failed: %v", err)
		http.Error(w, "pipeline execution failed", http.StatusInternalServerError)
		return
	}

	job.Status = "success"
	applyExecutionResult(job, result)
	if err := h.Store.SaveJob(job); err != nil {
		log.Printf("save completed job failed: %v", err)
		http.Error(w, "failed to save job", http.StatusInternalServerError)
		return
	}

	writeJSONStatus(w, http.StatusCreated, job)
}

// POST /transfer/send — 发送文件
func (h *Handler) TransferSend(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Provider   string `json:"provider"`
		LocalPath  string `json:"local_path"`
		RemotePath string `json:"remote_path"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		http.Error(w, "invalid request", 400)
		return
	}
	p, ok := provider.Get(body.Provider)
	if !ok {
		http.Error(w, "unknown provider: "+body.Provider, 400)
		return
	}
	link, err := p.Send(r.Context(), body.LocalPath, body.RemotePath)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}
	json.NewEncoder(w).Encode(map[string]string{"url": link})
}

// POST /transfer/receive — 接收文件
func (h *Handler) TransferReceive(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Provider  string `json:"provider"`
		URL       string `json:"url"`
		LocalPath string `json:"local_path"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		http.Error(w, "invalid request", 400)
		return
	}
	p, ok := provider.Get(body.Provider)
	if !ok {
		http.Error(w, "unknown provider", 400)
		return
	}
	if err := p.Receive(r.Context(), body.URL, body.LocalPath); err != nil {
		http.Error(w, err.Error(), 500)
		return
	}
	w.WriteHeader(200)
}

// GET /version — 版本信息
func (h *Handler) Version(w http.ResponseWriter, r *http.Request) {
	json.NewEncoder(w).Encode(map[string]string{
		"service": "qtcloud-provider",
		"version": provider.Version,
	})
}

// GET /process/jobs — 查看 process 执行记录
func (h *Handler) ListProcessJobs(w http.ResponseWriter, r *http.Request) {
	json.NewEncoder(w).Encode(h.Store.ListJobs())
}

// GET /process/jobs/{id} — 查看单个 process 执行记录
func (h *Handler) GetProcessJob(w http.ResponseWriter, r *http.Request) {
	job := h.Store.GetJob(r.PathValue("id"))
	if job == nil {
		http.Error(w, "job not found", http.StatusNotFound)
		return
	}
	writeJSON(w, job)
}

func writeJSON(w http.ResponseWriter, v any) {
	writeJSONStatus(w, http.StatusOK, v)
}

func writeJSONStatus(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(v)
}

func newJobID(customerID string, now time.Time) string {
	return sanitizeID(customerID) + "-" + strconv.FormatInt(now.UnixNano(), 10)
}

func sanitizeID(value string) string {
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
		return "job"
	}
	return out
}

func safePipelineRunError(err error) string {
	if errors.Is(err, pipeline.ErrInvalidDefinition) || errors.Is(err, pipeline.ErrNoExecutableResource) {
		return "blueprint pipeline is not executable"
	}
	return "pipeline execution failed"
}

func applyExecutionResult(job *store.JobRecord, result pipeline.ExecutionResult) {
	if result.Output != "" {
		job.Output = result.Output
	}
	job.Steps = make([]store.JobStep, 0, len(result.Steps))
	for _, step := range result.Steps {
		job.Steps = append(job.Steps, store.JobStep{
			Name:     step.Name,
			Resource: step.Resource,
			Input:    step.Input,
			Output:   step.Output,
			Status:   step.Status,
		})
	}
}

func validatePathUnderOptionalRoot(pathValue string, rootEnv string) error {
	root := strings.TrimSpace(os.Getenv(rootEnv))
	if root == "" {
		return nil
	}
	_, err := pipeline.ResolvePathUnderRoot(root, pathValue, false)
	return err
}

func validateWorkDirUnderOptionalRoot(pathValue string) error {
	root := strings.TrimSpace(os.Getenv("PIPELINE_WORK_ROOT"))
	if root == "" {
		return nil
	}
	if err := os.MkdirAll(root, 0o700); err != nil {
		return err
	}
	_, err := pipeline.ResolvePathUnderRoot(root, pathValue, false)
	return err
}
