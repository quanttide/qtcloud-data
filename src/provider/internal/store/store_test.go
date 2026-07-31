package store

import (
	"os"
	"path/filepath"
	"testing"
)

func TestPersistentStoreSavesAndLoadsJobs(t *testing.T) {
	path := filepath.Join(t.TempDir(), "provider-jobs.json")

	s, err := NewPersistent(path)
	if err != nil {
		t.Fatalf("NewPersistent: %v", err)
	}
	if err := s.SaveJob(&JobRecord{
		ID:         "job-1",
		CustomerID: "ABC",
		Blueprint:  "customer-chat",
		Pipeline:   "customer-chat-pipeline",
		Status:     "success",
		Output:     "final.csv",
		Steps: []JobStep{
			{Name: "copy", Resource: "builtin:copy", Status: "success"},
		},
	}); err != nil {
		t.Fatalf("SaveJob: %v", err)
	}

	reloaded, err := NewPersistent(path)
	if err != nil {
		t.Fatalf("reload NewPersistent: %v", err)
	}
	job := reloaded.GetJob("job-1")
	if job == nil {
		t.Fatal("expected persisted job to be loaded")
	}
	if job.Blueprint != "customer-chat" {
		t.Fatalf("job.Blueprint = %q", job.Blueprint)
	}
	if job.Steps[0].Resource != "builtin:copy" {
		t.Fatalf("job step resource = %q", job.Steps[0].Resource)
	}
}

func TestPersistentStoreUpdatesExistingJobFile(t *testing.T) {
	path := filepath.Join(t.TempDir(), "provider-jobs.json")
	s, err := NewPersistent(path)
	if err != nil {
		t.Fatalf("NewPersistent: %v", err)
	}

	if err := s.SaveJob(&JobRecord{ID: "job-1", CustomerID: "ABC", Pipeline: "p", Status: "running"}); err != nil {
		t.Fatalf("SaveJob running: %v", err)
	}
	if err := s.SaveJob(&JobRecord{ID: "job-1", CustomerID: "ABC", Pipeline: "p", Status: "success", Output: "final.csv"}); err != nil {
		t.Fatalf("SaveJob success: %v", err)
	}

	reloaded, err := NewPersistent(path)
	if err != nil {
		t.Fatalf("reload NewPersistent: %v", err)
	}
	job := reloaded.GetJob("job-1")
	if job == nil {
		t.Fatal("expected updated job to be loaded")
	}
	if job.Status != "success" {
		t.Fatalf("job.Status = %q", job.Status)
	}
	if job.Output != "final.csv" {
		t.Fatalf("job.Output = %q", job.Output)
	}
}

func TestNewFromEnvUsesJobStorePath(t *testing.T) {
	path := filepath.Join(t.TempDir(), "jobs.json")
	t.Setenv("JOB_STORE_PATH", path)
	t.Setenv("CATALOG_DIR", filepath.Join(t.TempDir(), "ignored"))
	t.Setenv("DATA_ROOT", filepath.Join(t.TempDir(), "ignored-root"))

	s, err := NewFromEnv()
	if err != nil {
		t.Fatalf("NewFromEnv: %v", err)
	}
	if err := s.SaveJob(&JobRecord{ID: "job-1", CustomerID: "ABC", Pipeline: "p", Status: "success"}); err != nil {
		t.Fatalf("SaveJob: %v", err)
	}

	if _, err := os.Stat(path); err != nil {
		t.Fatalf("expected JOB_STORE_PATH file to be written: %v", err)
	}
}

func TestNewFromEnvFallsBackToCatalogDir(t *testing.T) {
	catalogDir := t.TempDir()
	t.Setenv("JOB_STORE_PATH", "")
	t.Setenv("CATALOG_DIR", catalogDir)
	t.Setenv("DATA_ROOT", filepath.Join(t.TempDir(), "ignored-root"))

	s, err := NewFromEnv()
	if err != nil {
		t.Fatalf("NewFromEnv: %v", err)
	}
	if err := s.SaveJob(&JobRecord{ID: "job-1", CustomerID: "ABC", Pipeline: "p", Status: "success"}); err != nil {
		t.Fatalf("SaveJob: %v", err)
	}

	if _, err := os.Stat(filepath.Join(catalogDir, "provider-jobs.json")); err != nil {
		t.Fatalf("expected catalog job file to be written: %v", err)
	}
}

func TestMemoryStoreDoesNotPersist(t *testing.T) {
	s := New()
	if err := s.SaveJob(&JobRecord{ID: "job-1", CustomerID: "ABC", Pipeline: "p", Status: "success"}); err != nil {
		t.Fatalf("SaveJob: %v", err)
	}
	if got := s.GetJob("job-1"); got == nil {
		t.Fatal("expected memory store to keep job")
	}
}
