package provider

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
)

type DropboxProvider struct{}

func (p *DropboxProvider) Name() string { return "dropbox" }

func (p *DropboxProvider) Send(ctx context.Context, localPath, remotePath string) (string, error) {
	token := os.Getenv("DROPBOX_ACCESS_TOKEN")
	if token == "" {
		return "", fmt.Errorf("请设置 DROPBOX_ACCESS_TOKEN")
	}
	data, err := os.ReadFile(localPath)
	if err != nil {
		return "", fmt.Errorf("读取文件失败: %w", err)
	}
	// POST /files/upload
	req, _ := http.NewRequestWithContext(ctx, "POST", "https://content.dropboxapi.com/2/files/upload", nil)
	req.Header.Set("Authorization", "Bearer "+token)
	req.Header.Set("Dropbox-API-Arg", fmt.Sprintf(`{"path":%q,"mode":"overwrite"}`, remotePath))
	req.Header.Set("Content-Type", "application/octet-stream")
	// TODO: 完整实现
	_ = data
	return "https://dropbox.com/s/..." + remotePath, nil
}

func (p *DropboxProvider) Receive(ctx context.Context, url, localPath string) error {
	resp, err := http.Get(url)
	if err != nil {
		return fmt.Errorf("下载失败: %w", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != 200 {
		return fmt.Errorf("下载失败 [%d]", resp.StatusCode)
	}
	out, _ := os.Create(localPath)
	defer out.Close()
	io.Copy(out, resp.Body)
	return nil
}

func (p *DropboxProvider) ReceivePath(ctx context.Context, remote, local string) error {
	return fmt.Errorf("Dropbox 不支持自动接收")
}
