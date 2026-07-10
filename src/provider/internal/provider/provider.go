package provider

import "context"

type Provider interface {
	Name() string
	Send(ctx context.Context, localPath, remotePath string) (string, error)
	Receive(ctx context.Context, url, localPath string) error
	ReceivePath(ctx context.Context, remote, local string) error
}

type Credential struct {
	AccessToken string            `json:"access_token"`
	Extra       map[string]string `json:"extra,omitempty"`
}
