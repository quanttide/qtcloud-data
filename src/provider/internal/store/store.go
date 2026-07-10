package store

import "sync"

type JobRecord struct {
	ID         string `json:"id"`
	CustomerID string `json:"customer_id"`
	Pipeline   string `json:"pipeline"`
	Status     string `json:"status"` // running/success/failed
	Input      string `json:"input,omitempty"`
	Output     string `json:"output,omitempty"`
	Error      string `json:"error,omitempty"`
}

type Store struct {
	mu   sync.RWMutex
	jobs map[string]*JobRecord
}

func New() *Store {
	return &Store{jobs: make(map[string]*JobRecord)}
}

func (s *Store) SaveJob(r *JobRecord) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.jobs[r.ID] = r
}

func (s *Store) GetJob(id string) *JobRecord {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.jobs[id]
}

func (s *Store) ListJobs() []*JobRecord {
	s.mu.RLock()
	defer s.mu.RUnlock()
	var out []*JobRecord
	for _, r := range s.jobs {
		out = append(out, r)
	}
	return out
}
