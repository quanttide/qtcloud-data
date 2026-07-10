use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_upload_ok(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/files/upload"))
        .respond_with(ResponseTemplate::new(200))
        .mount(server)
        .await;
}

async fn mock_shared_link_ok(server: &MockServer, remote_path: &str) {
    let expected_body = serde_json::json!({
        "path": remote_path,
        "settings": { "requested_visibility": { ".tag": "public" } }
    });

    Mock::given(method("POST"))
        .and(path("/sharing/create_shared_link_with_settings"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"url": "https://www.dropbox.com/s/abc123/file.csv?dl=0"}),
        ))
        .mount(server)
        .await;
}

#[tokio::test]
async fn test_send_uploads_file_and_returns_shared_link() {
    let server = MockServer::start().await;
    let base = server.uri();

    let tmp = std::env::temp_dir().join("test_send.txt");
    std::fs::write(&tmp, b"hello, world!").unwrap();

    mock_upload_ok(&server).await;
    mock_shared_link_ok(&server, "/Customers/send/test_send.txt").await;

    qtcloud_data_cli::dropbox::upload(
        "fake_token",
        tmp.to_str().unwrap(),
        "/Customers/send/test_send.txt",
        Some(&base),
    )
    .await;

    let link = qtcloud_data_cli::dropbox::create_shared_link(
        "fake_token",
        "/Customers/send/test_send.txt",
        Some(&base),
    )
    .await
    .unwrap();

    assert!(link.contains("dropbox.com/s/"));
    assert!(link.contains("?dl=1"));
    std::fs::remove_file(&tmp).ok();
}

#[tokio::test]
async fn test_receive_downloads_file_from_shared_link() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/s/abc123/file.csv"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"mock file content")
                .insert_header("Content-Type", "application/octet-stream"),
        )
        .mount(&server)
        .await;

    let url = format!("{}/s/abc123/file.csv", server.uri());
    let tmp = std::env::temp_dir().join("test_received.txt");

    qtcloud_data_cli::dropbox::download_and_save(
        "fake_token",
        &format!("{url}?dl=1"),
        tmp.to_str().unwrap(),
    )
    .await;

    let content = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(content, "mock file content");
    std::fs::remove_file(&tmp).ok();
}

#[tokio::test]
async fn test_receive_404_should_panic() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let url = format!("{}/not-found", server.uri());
    let tmp = std::env::temp_dir().join("test_404.txt");

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(qtcloud_data_cli::dropbox::download_and_save(
            "fake_token",
            &url,
            tmp.to_str().unwrap(),
        ));
    }));

    assert!(result.is_err(), "404 应触发 panic");
}

#[tokio::test]
async fn test_upload_500_should_panic() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("POST"))
        .and(path("/files/upload"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let tmp = std::env::temp_dir().join("test_upload_error.txt");
    std::fs::write(&tmp, b"data").unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(qtcloud_data_cli::dropbox::upload(
            "fake_token",
            tmp.to_str().unwrap(),
            "/fail",
            Some(&base),
        ));
    }));

    assert!(result.is_err(), "500 应触发 panic");
    std::fs::remove_file(&tmp).ok();
}

#[tokio::test]
async fn test_create_shared_link_404_returns_error() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("POST"))
        .and(path("/sharing/create_shared_link_with_settings"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({"error_summary": "path/not_found"})),
        )
        .mount(&server)
        .await;

    let result =
        qtcloud_data_cli::dropbox::create_shared_link("fake_token", "/nonexistent", Some(&base))
            .await;

    assert!(result.is_err(), "404 应返回 error");
}
