package store

import "sync"

type RunRecord struct {
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
	runs map[string]*RunRecord
}

func New() *Store {
	return &Store{runs: make(map[string]*RunRecord)}
}

func (s *Store) SaveRun(r *RunRecord) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.runs[r.ID] = r
}

func (s *Store) GetRun(id string) *RunRecord {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.runs[id]
}

func (s *Store) ListRuns() []*RunRecord {
	s.mu.RLock()
	defer s.mu.RUnlock()
	var out []*RunRecord
	for _, r := range s.runs {
		out = append(out, r)
	}
	return out
}
