package api

import (
	"net/http"

	"github.com/quanttide/qtcloud-provider/internal/store"
)

func Router(s *store.Store) http.Handler {
	h := NewHandler(s)
	mux := http.NewServeMux()

	mux.HandleFunc("GET /providers", h.ListProviders)
	mux.HandleFunc("GET /blueprints", h.ListBlueprints)
	mux.HandleFunc("GET /blueprints/{name}", h.GetBlueprint)
	mux.HandleFunc("POST /blueprints/{name}/runs", h.RunBlueprint)
	mux.HandleFunc("POST /transfer/send", h.TransferSend)
	mux.HandleFunc("POST /transfer/receive", h.TransferReceive)
	mux.HandleFunc("GET /process/jobs", h.ListProcessJobs)
	mux.HandleFunc("GET /process/jobs/{id}", h.GetProcessJob)
	mux.HandleFunc("GET /version", h.Version)

	return mux
}
