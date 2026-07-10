package provider

import (
	"context"
	"fmt"
)

type S3Provider struct{}

func (p *S3Provider) Name() string { return "s3" }

func (p *S3Provider) Send(ctx context.Context, localPath, remotePath string) (string, error) {
	return "", fmt.Errorf("S3 provider: 需要 AWS SDK，请在联网环境编译")
}

func (p *S3Provider) Receive(ctx context.Context, url, localPath string) error {
	return fmt.Errorf("S3 provider: 需要 AWS SDK，请在联网环境编译")
}

func (p *S3Provider) ReceivePath(ctx context.Context, remote, local string) error {
	return fmt.Errorf("S3 provider: 需要 AWS SDK，请在联网环境编译")
}
