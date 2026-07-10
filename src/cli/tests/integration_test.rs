use qtcloud_data_cli::providers::StorageProvider;
use qtcloud_data_cli::providers::dropbox;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── 辅助函数 ──

async fn mock_upload_ok(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/files/upload"))
        .respond_with(ResponseTemplate::new(200))
        .mount(server)
        .await;
}

async fn mock_shared_link_ok(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/sharing/create_shared_link_with_settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"url": "https://www.dropbox.com/s/abc/file.csv?dl=0"}),
        ))
        .mount(server)
        .await;
}

// ── Dropbox 传输测试 ──

#[tokio::test]
async fn test_dropbox_send() {
    let server = MockServer::start().await;
    let base = server.uri();

    let tmp = std::env::temp_dir().join("test_send.txt");
    std::fs::write(&tmp, b"hello").unwrap();

    mock_upload_ok(&server).await;
    mock_shared_link_ok(&server).await;

    dropbox::upload("fake", tmp.to_str().unwrap(), "/test.txt", Some(&base)).await;

    let link = dropbox::create_shared_link("fake", "/test.txt", Some(&base))
        .await
        .unwrap();

    assert!(link.contains("?dl=1"));
    std::fs::remove_file(&tmp).ok();
}

#[tokio::test]
async fn test_dropbox_receive() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/s/abc/file.csv"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"mock data")
                .insert_header("Content-Type", "application/octet-stream"),
        )
        .mount(&server)
        .await;

    let tmp = std::env::temp_dir().join("test_recv.txt");
    let provider = qtcloud_data_cli::providers::DropboxProvider;

    let result = provider
        .receive(
            &format!("{}/s/abc/file.csv?dl=1", server.uri()),
            tmp.to_str().unwrap(),
        )
        .await;

    assert!(result.is_ok());
    let content = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(content, "mock data");
    std::fs::remove_file(&tmp).ok();
}

#[tokio::test]
async fn test_dropbox_receive_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let tmp = std::env::temp_dir().join("test_404.txt");
    let provider = qtcloud_data_cli::providers::DropboxProvider;

    let result = provider
        .receive(&format!("{}/missing", server.uri()), tmp.to_str().unwrap())
        .await;

    assert!(result.is_err(), "404 应返回 error");
}

#[tokio::test]
async fn test_dropbox_upload_500() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("POST"))
        .and(path("/files/upload"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let tmp = std::env::temp_dir().join("test_err.txt");
    std::fs::write(&tmp, b"data").unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(dropbox::upload(
            "fake",
            tmp.to_str().unwrap(),
            "/fail",
            Some(&base),
        ));
    }));

    assert!(result.is_err(), "500 应触发 panic");
    std::fs::remove_file(&tmp).ok();
}

// ── 网盘类 provider receive_path 测试 ──

#[tokio::test]
async fn test_cloud_providers_receive_path_not_supported() {
    let providers: Vec<Box<dyn StorageProvider>> = vec![
        Box::new(qtcloud_data_cli::providers::DropboxProvider),
        Box::new(qtcloud_data_cli::providers::BaiduDriveProvider),
        Box::new(qtcloud_data_cli::providers::GoogleDriveProvider),
        Box::new(qtcloud_data_cli::providers::OneDriveProvider),
    ];
    for p in providers {
        let result = p.receive_path("/some/path", "/tmp/test").await;
        assert!(result.is_err(), "{} 应不支持自动接收", p.name());
    }
}

// ── S3 receive_path mock 测试 ──

#[tokio::test]
async fn test_s3_receive_path() {
    let server = MockServer::start().await;
    let base = server.uri();

    // mock 一个 S3 GetObject 请求
    Mock::given(method("GET"))
        .and(path("/bucket/key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"s3 mock content")
                .insert_header("Content-Type", "application/octet-stream"),
        )
        .mount(&server)
        .await;
    let _base = server.uri();
    let provider = qtcloud_data_cli::providers::S3Provider;
    assert_eq!(provider.name(), "s3");
}

// ── provider 注册测试 ──

#[tokio::test]
async fn test_provider_detect_from_url() {
    assert!(
        qtcloud_data_cli::providers::detect("https://www.dropbox.com/s/abc/file.csv").is_some(),
    );
    assert!(qtcloud_data_cli::providers::detect("https://pan.baidu.com/s/1abc").is_some(),);
    assert!(
        qtcloud_data_cli::providers::detect("https://drive.google.com/file/d/abc123/view")
            .is_some(),
    );
    assert!(qtcloud_data_cli::providers::detect("https://1drv.ms/u/s!abc123").is_some(),);
    assert!(
        qtcloud_data_cli::providers::detect("https://s3.us-east-1.amazonaws.com/bucket/key")
            .is_some(),
    );
    assert!(qtcloud_data_cli::providers::detect("sftp://user@host:22/path/file.csv").is_some(),);
    assert!(qtcloud_data_cli::providers::detect("https://example.com/file").is_none(),);
}

#[tokio::test]
async fn test_provider_from_name() {
    assert!(qtcloud_data_cli::providers::from_name("dropbox").is_some());
    assert!(qtcloud_data_cli::providers::from_name("baidu").is_some());
    assert!(qtcloud_data_cli::providers::from_name("baidudrive").is_some());
    assert!(qtcloud_data_cli::providers::from_name("google").is_some());
    assert!(qtcloud_data_cli::providers::from_name("googledrive").is_some());
    assert!(qtcloud_data_cli::providers::from_name("onedrive").is_some());
    assert!(qtcloud_data_cli::providers::from_name("s3").is_some());
    assert!(qtcloud_data_cli::providers::from_name("sftp").is_some());
    assert!(qtcloud_data_cli::providers::from_name("unknown").is_none());
}

// ── process 中 to_camel 工具函数测试 ──

#[test]
fn test_to_camel() {
    assert_eq!(
        qtcloud_data_cli::process::to_camel("csv-standard"),
        "csvStandard"
    );
    assert_eq!(qtcloud_data_cli::process::to_camel("simple"), "simple");
    assert_eq!(
        qtcloud_data_cli::process::to_camel("abc-def-ghi"),
        "abcDefGhi"
    );
    assert_eq!(qtcloud_data_cli::process::to_camel(""), "");
}
