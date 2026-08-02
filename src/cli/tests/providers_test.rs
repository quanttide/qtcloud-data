use qtcloud_data_cli::providers::StorageProvider;
use qtcloud_data_cli::providers::dropbox;
use qtcloud_data_cli::providers::google_drive::{receive_with_base, send_with_base};
use qtcloud_data_cli::providers::onedrive;
use std::sync::Mutex;
use wiremock::matchers::query_param;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// s3 测试通过进程级 AWS_* 环境变量指向 wiremock，并行线程会互相覆盖，
// 因此用静态锁串行化这两个测试（仅限本测试进程内）。
static AWS_ENV_LOCK: Mutex<()> = Mutex::new(());

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

    dropbox::upload("fake", tmp.to_str().unwrap(), "/test.txt", Some(&base))
        .await
        .unwrap();

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

    let result = dropbox::upload("fake", tmp.to_str().unwrap(), "/fail", Some(&base)).await;

    assert!(result.is_err(), "500 应返回错误");
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

/// 设置 AWS SDK 指向 wiremock 的静态配置。
/// 与其它测试无 AWS 环境变量冲突，因此不需要额外的锁。
fn set_aws_mock_env(endpoint: &str) {
    unsafe {
        std::env::set_var("AWS_ENDPOINT_URL", endpoint);
        std::env::set_var("AWS_ACCESS_KEY_ID", "test");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test");
        std::env::set_var("AWS_REGION", "us-east-1");
        std::env::set_var("AWS_S3_USE_PATH_STYLE_ENDPOINT", "true");
        std::env::set_var("S3_BUCKET", "bucket");
    }
}

fn clear_aws_mock_env() {
    for var in [
        "AWS_ENDPOINT_URL",
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_REGION",
        "AWS_S3_USE_PATH_STYLE_ENDPOINT",
        "S3_BUCKET",
    ] {
        unsafe {
            std::env::remove_var(var);
        }
    }
}

#[tokio::test]
async fn test_s3_receive_path_downloads_from_configured_endpoint() {
    let _guard = AWS_ENV_LOCK.lock().unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/bucket/key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"s3 mock content")
                .insert_header("Content-Type", "application/octet-stream"),
        )
        .mount(&server)
        .await;

    set_aws_mock_env(&server.uri());
    let provider = qtcloud_data_cli::providers::S3Provider;
    let out = std::env::temp_dir().join("s3-receive.txt");

    let result = provider.receive_path("/key", out.to_str().unwrap()).await;

    clear_aws_mock_env();
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "s3 mock content");
    std::fs::remove_file(&out).ok();
}

#[tokio::test]
async fn test_s3_send_uploads_and_presigns_url() {
    let _guard = AWS_ENV_LOCK.lock().unwrap();
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/bucket/key"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    set_aws_mock_env(&server.uri());
    let provider = qtcloud_data_cli::providers::S3Provider;
    let file = std::env::temp_dir().join("s3-send.txt");
    std::fs::write(&file, b"upload me").unwrap();

    let result = provider.send(file.to_str().unwrap(), "/key").await;

    clear_aws_mock_env();
    assert!(result.is_ok(), "{result:?}");
    let url = result.unwrap();
    assert!(url.contains("/bucket/key"), "{url}");
    assert!(url.contains("X-Amz-Signature"), "{url}");
    std::fs::remove_file(&file).ok();
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

// ── Google Drive / OneDrive（自 provider_test.rs 并入）──

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
