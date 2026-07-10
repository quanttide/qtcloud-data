package main

import (
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/quanttide/qtcloud-provider/internal/api"
	"github.com/quanttide/qtcloud-provider/internal/store"
)

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	s := store.New()
	handler := api.Router(s)

	addr := fmt.Sprintf(":%s", port)
	log.Printf("量潮数据云 Provider 启动: http://localhost%s", addr)
	log.Fatal(http.ListenAndServe(addr, handler))
}
