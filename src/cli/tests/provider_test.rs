//! google_drive / onedrive provider 的 wiremock 集成测试。
//!
//! 与 `integration_test.rs` 中 dropbox 的测试模式一致：通过 `*_with_base`
//! helper 把 API 端点指向 wiremock，不依赖真实网络。

use qtcloud_data_cli::providers::google_drive::{receive_with_base, send_with_base};
use qtcloud_data_cli::providers::onedrive;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn tmp_file(name: &str, content: &str) -> String {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, content).unwrap();
    path.to_string_lossy().to_string()
}

// ── Google Drive ──

async fn mock_gdrive_upload_flow(server: &MockServer, file_id: &str) {
    // 1. 初始化 resumable 会话，返回上传 URL（含 fileId）
    Mock::given(method("POST"))
        .and(path("/upload/drive/v3/files"))
        .and(query_param("uploadType", "resumable"))
        .respond_with(ResponseTemplate::new(200).insert_header(
            "location",
            format!(
                "{}/upload/drive/v3/files/{file_id}?uploadType=resumable",
                server.uri()
            ),
        ))
        .mount(server)
        .await;
    // 2. 上传内容
    Mock::given(method("PUT"))
        .and(path(&format!("/upload/drive/v3/files/{file_id}")))
        .respond_with(ResponseTemplate::new(200))
        .mount(server)
        .await;
    // 3. 设置权限
    Mock::given(method("POST"))
        .and(path(&format!("/drive/v3/files/{file_id}/permissions")))
        .respond_with(ResponseTemplate::new(200))
        .mount(server)
        .await;
}

#[tokio::test]
async fn google_drive_send_uploads_and_returns_web_view_link() {
    let server = MockServer::start().await;
    mock_gdrive_upload_flow(&server, "file123").await;
    // 4. 获取分享链接
    Mock::given(method("GET"))
        .and(path("/drive/v3/files/file123"))
        .and(query_param("fields", "webViewLink"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"webViewLink": "https://drive.google.com/file/d/file123/view"}),
        ))
        .mount(&server)
        .await;

    let file = tmp_file("gdrive-send.csv", "a,b\n1,2\n");
    let api_base = format!("{}/drive/v3", server.uri());
    let upload_base = format!("{}/upload/drive/v3", server.uri());

    let link = send_with_base(
        "fake-token",
        &file,
        "/folder1/data.csv",
        Some(&api_base),
        Some(&upload_base),
    )
    .await
    .unwrap();

    assert_eq!(link, "https://drive.google.com/file/d/file123/view");
    std::fs::remove_file(&file).ok();
}

#[tokio::test]
async fn google_drive_send_reports_missing_upload_url() {
    let server = MockServer::start().await;
    // 只 mock 会话初始化，但不返回 Location header
    Mock::given(method("POST"))
        .and(path("/upload/drive/v3/files"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let file = tmp_file("gdrive-no-location.csv", "x\n");
    let api_base = format!("{}/drive/v3", server.uri());
    let upload_base = format!("{}/upload/drive/v3", server.uri());

    let err = send_with_base(
        "fake-token",
        &file,
        "/data.csv",
        Some(&api_base),
        Some(&upload_base),
    )
    .await
    .unwrap_err();
    assert!(err.contains("未获取到上传 URL"), "{err}");

    std::fs::remove_file(&file).ok();
}

#[tokio::test]
async fn google_drive_receive_downloads_file_content() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/drive/v3/files/file456"))
        .and(query_param("alt", "media"))
        .respond_with(ResponseTemplate::new(200).set_body_string("id,value\n1,x\n"))
        .mount(&server)
        .await;

    let out = tmp_file("gdrive-receive.csv", "");
    let api_base = format!("{}/drive/v3", server.uri());
    let share_url = format!("{}/file/d/file456/view", server.uri());

    receive_with_base("fake-token", &share_url, &out, Some(&api_base))
        .await
        .unwrap();

    assert_eq!(std::fs::read_to_string(&out).unwrap(), "id,value\n1,x\n");
    std::fs::remove_file(&out).ok();
}

#[tokio::test]
async fn google_drive_receive_rejects_unparseable_share_url() {
    let err = receive_with_base(
        "fake-token",
        "https://drive.google.com/nope",
        "/tmp/x.csv",
        None,
    )
    .await
    .unwrap_err();
    assert!(err.contains("无法从 URL 提取 fileId"), "{err}");
}

// ── OneDrive ──

#[tokio::test]
async fn onedrive_send_uploads_and_creates_view_link() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1.0/me/drive/root:/folder/data.csv:/content"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1.0/me/drive/root:/folder/data.csv:/createLink"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"link": {"webUrl": "https://1drv.ms/s/abc123"}})),
        )
        .mount(&server)
        .await;

    let file = tmp_file("onedrive-send.csv", "a,b\n");
    let graph_api = format!("{}/v1.0", server.uri());

    let link = onedrive::send_with_base("fake-token", &file, "/folder/data.csv", Some(&graph_api))
        .await
        .unwrap();

    assert_eq!(link, "https://1drv.ms/s/abc123");
    std::fs::remove_file(&file).ok();
}

#[tokio::test]
async fn onedrive_send_reports_share_link_failure() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1.0/me/drive/root:/data.csv:/content"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1.0/me/drive/root:/data.csv:/createLink"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"error": "denied"})),
        )
        .mount(&server)
        .await;

    let file = tmp_file("onedrive-share-fail.csv", "x\n");
    let graph_api = format!("{}/v1.0", server.uri());

    let err = onedrive::send_with_base("fake-token", &file, "/data.csv", Some(&graph_api))
        .await
        .unwrap_err();
    assert!(err.contains("创建分享失败"), "{err}");

    std::fs::remove_file(&file).ok();
}

#[tokio::test]
async fn onedrive_receive_downloads_with_download_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/share/file.csv"))
        .and(query_param("download", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("report,value\n1,2\n"))
        .mount(&server)
        .await;

    let out = tmp_file("onedrive-receive.csv", "");
    let share_url = format!("{}/share/file.csv", server.uri());

    onedrive::receive_with_base("fake-token", &share_url, &out, None)
        .await
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        "report,value\n1,2\n"
    );
    std::fs::remove_file(&out).ok();
}

#[tokio::test]
async fn onedrive_receive_reports_download_failure() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/missing.csv"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let out = tmp_file("onedrive-receive-fail.csv", "");
    let share_url = format!("{}/missing.csv", server.uri());

    let err = onedrive::receive_with_base("fake-token", &share_url, &out, None)
        .await
        .unwrap_err();
    assert!(err.contains("下载失败 [404"), "{err}");

    std::fs::remove_file(&out).ok();
}
