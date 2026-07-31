package main

import (
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/quanttide/qtcloud-provider/internal/api"
	"github.com/quanttide/qtcloud-provider/internal/provider"
	"github.com/quanttide/qtcloud-provider/internal/store"
)

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	s, err := store.NewFromEnv()
	if err != nil {
		log.Fatalf("加载 process job store 失败: %v", err)
	}
	handler := api.Router(s)

	addr := fmt.Sprintf(":%s", port)
	log.Printf("量潮数据云 Provider %s 启动: http://localhost%s", provider.Version, addr)
	log.Fatal(http.ListenAndServe(addr, handler))
}
