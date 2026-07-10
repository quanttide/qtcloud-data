package api

import (
	"encoding/json"
	"net/http"

	"github.com/quanttide/qtcloud-provider/internal/provider"
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

// GET /runs — 查看执行记录
func (h *Handler) ListRuns(w http.ResponseWriter, r *http.Request) {
	json.NewEncoder(w).Encode(h.Store.ListRuns())
}
