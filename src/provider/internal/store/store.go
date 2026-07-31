package store

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
)

type JobRecord struct {
	ID         string    `json:"id"`
	CustomerID string    `json:"customer_id"`
	Blueprint  string    `json:"blueprint,omitempty"`
	Pipeline   string    `json:"pipeline"`
	Status     string    `json:"status"` // running/success/failed
	Input      string    `json:"input,omitempty"`
	Output     string    `json:"output,omitempty"`
	Error      string    `json:"error,omitempty"`
	StartedAt  string    `json:"started_at,omitempty"`
	FinishedAt string    `json:"finished_at,omitempty"`
	Steps      []JobStep `json:"steps,omitempty"`
}

type JobStep struct {
	Name     string `json:"name"`
	Resource string `json:"resource,omitempty"`
	Input    string `json:"input,omitempty"`
	Output   string `json:"output,omitempty"`
	Status   string `json:"status"`
}

type Store struct {
	mu   sync.RWMutex
	jobs map[string]*JobRecord
	path string
}

func New() *Store {
	return &Store{jobs: make(map[string]*JobRecord)}
}

func NewPersistent(path string) (*Store, error) {
	s := New()
	s.path = strings.TrimSpace(path)
	if s.path == "" {
		return s, nil
	}
	if err := s.load(); err != nil {
		return nil, err
	}
	return s, nil
}

func NewFromEnv() (*Store, error) {
	return NewPersistent(jobStorePath())
}

func (s *Store) SaveJob(r *JobRecord) error {
	if r == nil {
		return nil
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	s.jobs[r.ID] = cloneJob(r)
	return s.persistLocked()
}

func (s *Store) GetJob(id string) *JobRecord {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return cloneJob(s.jobs[id])
}

func (s *Store) ListJobs() []*JobRecord {
	s.mu.RLock()
	defer s.mu.RUnlock()
	out := make([]*JobRecord, 0, len(s.jobs))
	for _, r := range s.jobs {
		out = append(out, cloneJob(r))
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].StartedAt == out[j].StartedAt {
			return out[i].ID < out[j].ID
		}
		return out[i].StartedAt > out[j].StartedAt
	})
	return out
}

func (s *Store) load() error {
	content, err := os.ReadFile(s.path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return err
	}
	if strings.TrimSpace(string(content)) == "" {
		return nil
	}

	jobs := make(map[string]*JobRecord)
	if err := json.Unmarshal(content, &jobs); err != nil {
		return err
	}
	for id, job := range jobs {
		if job == nil {
			continue
		}
		if job.ID == "" {
			job.ID = id
		}
		s.jobs[job.ID] = cloneJob(job)
	}
	return nil
}

func (s *Store) persistLocked() error {
	if s.path == "" {
		return nil
	}
	if err := os.MkdirAll(filepath.Dir(s.path), 0o700); err != nil {
		return err
	}
	content, err := json.MarshalIndent(s.jobs, "", "  ")
	if err != nil {
		return err
	}
	tmp := s.path + ".tmp"
	if err := os.WriteFile(tmp, content, 0o600); err != nil {
		return err
	}
	return os.Rename(tmp, s.path)
}

func jobStorePath() string {
	if path := strings.TrimSpace(os.Getenv("JOB_STORE_PATH")); path != "" {
		return path
	}
	if dir := strings.TrimSpace(os.Getenv("CATALOG_DIR")); dir != "" {
		return filepath.Join(dir, "provider-jobs.json")
	}
	if root := strings.TrimSpace(os.Getenv("DATA_ROOT")); root != "" {
		return filepath.Join(root, "catalog", "provider-jobs.json")
	}
	return filepath.Join(".quanttide", "data", "catalog", "provider-jobs.json")
}

func cloneJob(job *JobRecord) *JobRecord {
	if job == nil {
		return nil
	}
	out := *job
	if job.Steps != nil {
		out.Steps = append([]JobStep(nil), job.Steps...)
	}
	return &out
}
