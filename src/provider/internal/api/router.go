package api

import (
	"net/http"

	"github.com/quanttide/qtcloud-provider/internal/store"
)

func Router(s *store.Store) http.Handler {
	h := NewHandler(s)
	mux := http.NewServeMux()

	mux.HandleFunc("GET /providers", h.ListProviders)
	mux.HandleFunc("POST /transfer/send", h.TransferSend)
	mux.HandleFunc("POST /transfer/receive", h.TransferReceive)
	mux.HandleFunc("GET /process/runs", h.ListProcessRuns)

	return mux
}
