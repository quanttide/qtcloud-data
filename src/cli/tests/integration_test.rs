use qtcloud_data_cli::providers::StorageProvider;
use qtcloud_data_cli::providers::dropbox;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

#[tokio::test]
async fn test_provider_detect_from_url() {
    assert!(
        qtcloud_data_cli::providers::detect("https://www.dropbox.com/s/abc/file.csv").is_some(),
        "应识别 Dropbox 链接"
    );
    assert!(
        qtcloud_data_cli::providers::detect("https://pan.baidu.com/s/1abc").is_some(),
        "应识别百度网盘链接"
    );
    assert!(
        qtcloud_data_cli::providers::detect("https://example.com/file").is_none(),
        "未知链接应返回 None"
    );
}

#[tokio::test]
async fn test_provider_from_name() {
    assert!(qtcloud_data_cli::providers::from_name("dropbox").is_some());
    assert!(qtcloud_data_cli::providers::from_name("baidu").is_some());
    assert!(qtcloud_data_cli::providers::from_name("baidudrive").is_some());
    assert!(qtcloud_data_cli::providers::from_name("unknown").is_none());
}
