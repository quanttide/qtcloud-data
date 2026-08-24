#![allow(clippy::await_holding_lock)]

use qtcloud_data_cli::storage::Storage;
use qtcloud_data_cli::storage::baidu_drive::BaiduDriveStorage;
use qtcloud_data_cli::storage::dropbox;
use qtcloud_data_cli::storage::google_drive::{receive_with_base, send_with_base};
use qtcloud_data_cli::storage::onedrive;
use russh::keys::{Algorithm, PrivateKey};
use russh::server::{Auth, ChannelOpenHandle, Msg, Session};
use russh::{Channel, ChannelId};
use russh_sftp::protocol::{
    Attrs, Data, File, FileAttributes, Handle, Name, OpenFlags, Status, StatusCode, Version,
};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::{Duration, timeout};
use wiremock::matchers::query_param;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// s3 测试通过进程级 AWS_* 环境变量指向 wiremock，并行线程会互相覆盖，
// 因此用静态锁串行化这两个测试（仅限本测试进程内）。
static AWS_ENV_LOCK: Mutex<()> = Mutex::new(());
static BAIDU_ENV_LOCK: Mutex<()> = Mutex::new(());
static SFTP_ENV_LOCK: Mutex<()> = Mutex::new(());
static SFTP_PATH_COUNTER: AtomicUsize = AtomicUsize::new(0);

const SFTP_TEST_USER: &str = "sftp-user";
const SFTP_TEST_PASSWORD: &str = "sftp-password";

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

fn unique_temp_path(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        SFTP_PATH_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let path = unique_temp_path(prefix);
    fs::create_dir_all(&path).unwrap();
    path
}

fn resolve_remote_path(root: &Path, remote: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    for component in Path::new(remote).components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            #[cfg(windows)]
            Component::Prefix(_) => {}
            Component::ParentDir => {
                path.pop();
            }
            Component::Normal(part) => path.push(part),
            #[cfg(not(windows))]
            _ => {}
        }
    }
    path
}

fn ok_status(id: u32) -> Status {
    Status {
        id,
        status_code: StatusCode::Ok,
        error_message: "Ok".to_string(),
        language_tag: "en-US".to_string(),
    }
}

fn io_error_to_status(err: std::io::Error) -> StatusCode {
    match err.kind() {
        std::io::ErrorKind::NotFound => StatusCode::NoSuchFile,
        std::io::ErrorKind::PermissionDenied => StatusCode::PermissionDenied,
        _ => StatusCode::Failure,
    }
}

fn metadata_to_attrs(id: u32, metadata: fs::Metadata) -> Attrs {
    Attrs {
        id,
        attrs: (&metadata).into(),
    }
}

#[derive(Clone)]
struct SftpTestServer {
    root: PathBuf,
    password: String,
}

struct SshSession {
    root: PathBuf,
    password: String,
    clients: Arc<AsyncMutex<HashMap<ChannelId, Channel<Msg>>>>,
}

impl SshSession {
    fn new(root: PathBuf, password: String) -> Self {
        Self {
            root,
            password,
            clients: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    async fn get_channel(&mut self, channel_id: ChannelId) -> Channel<Msg> {
        let mut clients = self.clients.lock().await;
        clients.remove(&channel_id).unwrap()
    }
}

impl russh::server::Server for SftpTestServer {
    type Handler = SshSession;

    fn new_client(&mut self, _: Option<SocketAddr>) -> Self::Handler {
        SshSession::new(self.root.clone(), self.password.clone())
    }
}

impl russh::server::Handler for SshSession {
    type Error = russh::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::reject())
    }

    async fn auth_password(&mut self, user: &str, password: &str) -> Result<Auth, Self::Error> {
        if user == SFTP_TEST_USER && password == self.password {
            Ok(Auth::Accept)
        } else {
            Ok(Auth::reject())
        }
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        reply: ChannelOpenHandle,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        {
            let mut clients = self.clients.lock().await;
            clients.insert(channel.id(), channel);
        }
        reply.accept().await;
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.close(channel)?;
        Ok(())
    }

    async fn subsystem_request(
        &mut self,
        channel_id: ChannelId,
        name: &str,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if name == "sftp" {
            let channel = self.get_channel(channel_id).await;
            session.channel_success(channel_id)?;
            russh_sftp::server::run(
                channel.into_stream(),
                SftpSession {
                    root: self.root.clone(),
                },
            )
            .await;
        } else {
            session.channel_failure(channel_id)?;
        }

        Ok(())
    }
}

struct SftpSession {
    root: PathBuf,
}

impl SftpSession {
    fn path(&self, remote: &str) -> PathBuf {
        resolve_remote_path(&self.root, remote)
    }
}

impl russh_sftp::server::Handler for SftpSession {
    type Error = StatusCode;

    fn unimplemented(&self) -> Self::Error {
        StatusCode::OpUnsupported
    }

    async fn init(
        &mut self,
        _version: u32,
        _extensions: HashMap<String, String>,
    ) -> Result<Version, Self::Error> {
        Ok(Version::new())
    }

    async fn open(
        &mut self,
        id: u32,
        filename: String,
        pflags: OpenFlags,
        _attrs: FileAttributes,
    ) -> Result<Handle, Self::Error> {
        let path = self.path(&filename);
        let write_like = pflags.intersects(
            OpenFlags::WRITE | OpenFlags::APPEND | OpenFlags::CREATE | OpenFlags::TRUNCATE,
        );

        if write_like {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(io_error_to_status)?;
            }
        }

        let mut options = fs::OpenOptions::new();
        if pflags.contains(OpenFlags::READ) {
            options.read(true);
        }
        if pflags.contains(OpenFlags::WRITE) {
            options.write(true);
        }
        if pflags.contains(OpenFlags::APPEND) {
            options.append(true);
        }
        if pflags.contains(OpenFlags::TRUNCATE) {
            options.truncate(true);
        }
        if write_like {
            options.create(true);
        }

        options.open(&path).map_err(io_error_to_status)?;

        Ok(Handle {
            id,
            handle: path.to_string_lossy().into_owned(),
        })
    }

    async fn close(&mut self, id: u32, _handle: String) -> Result<Status, Self::Error> {
        Ok(ok_status(id))
    }

    async fn read(
        &mut self,
        id: u32,
        handle: String,
        offset: u64,
        len: u32,
    ) -> Result<Data, Self::Error> {
        let path = PathBuf::from(handle);
        let mut file = fs::File::open(&path).map_err(io_error_to_status)?;
        file.seek(SeekFrom::Start(offset))
            .map_err(io_error_to_status)?;
        let mut limited = file.take(len as u64);
        let mut data = Vec::new();
        let read = limited.read_to_end(&mut data).map_err(io_error_to_status)?;
        if read == 0 {
            return Err(StatusCode::Eof);
        }
        Ok(Data { id, data })
    }

    async fn write(
        &mut self,
        id: u32,
        handle: String,
        offset: u64,
        data: Vec<u8>,
    ) -> Result<Status, Self::Error> {
        let path = PathBuf::from(handle);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_error_to_status)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .map_err(io_error_to_status)?;
        file.seek(SeekFrom::Start(offset))
            .map_err(io_error_to_status)?;
        file.write_all(&data).map_err(io_error_to_status)?;
        Ok(ok_status(id))
    }

    async fn mkdir(
        &mut self,
        id: u32,
        path: String,
        _attrs: FileAttributes,
    ) -> Result<Status, Self::Error> {
        fs::create_dir_all(self.path(&path)).map_err(io_error_to_status)?;
        Ok(ok_status(id))
    }

    async fn stat(&mut self, id: u32, path: String) -> Result<Attrs, Self::Error> {
        let metadata = fs::metadata(self.path(&path)).map_err(io_error_to_status)?;
        Ok(metadata_to_attrs(id, metadata))
    }

    async fn lstat(&mut self, id: u32, path: String) -> Result<Attrs, Self::Error> {
        let metadata = fs::symlink_metadata(self.path(&path)).map_err(io_error_to_status)?;
        Ok(metadata_to_attrs(id, metadata))
    }

    async fn fstat(&mut self, id: u32, handle: String) -> Result<Attrs, Self::Error> {
        let metadata = fs::metadata(PathBuf::from(handle)).map_err(io_error_to_status)?;
        Ok(metadata_to_attrs(id, metadata))
    }

    async fn realpath(&mut self, id: u32, path: String) -> Result<Name, Self::Error> {
        let normalized = if path.starts_with('/') {
            path
        } else {
            format!("/{path}")
        };
        Ok(Name {
            id,
            files: vec![File::dummy(normalized)],
        })
    }
}

async fn start_sftp_fixture() -> SftpTestServerFixture {
    let root = unique_temp_dir("qtcloud-sftp");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut config = russh::server::Config::default();
    config.inactivity_timeout = None;
    config.auth_rejection_time = Duration::from_secs(0);
    config.auth_rejection_time_initial = Some(Duration::from_secs(0));
    config
        .keys
        .push(PrivateKey::random(&mut rand::rng(), Algorithm::Rsa { hash: None }).unwrap());
    let config = Arc::new(config);

    let root_for_server = root.clone();
    let join = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut server = SftpTestServer {
            root: root_for_server,
            password: SFTP_TEST_PASSWORD.to_string(),
        };
        let handler = russh::server::Server::new_client(&mut server, None);
        let running = russh::server::run_stream(config, socket, handler)
            .await
            .unwrap();
        let _ = running.await;
    });

    SftpTestServerFixture { root, addr, join }
}

struct SftpTestServerFixture {
    root: PathBuf,
    addr: SocketAddr,
    join: tokio::task::JoinHandle<()>,
}

impl SftpTestServerFixture {
    async fn shutdown(self) {
        let SftpTestServerFixture { root, join, .. } = self;
        timeout(Duration::from_secs(5), join)
            .await
            .expect("SFTP server did not stop")
            .expect("SFTP server task panicked");
        let _ = fs::remove_dir_all(root);
    }
}

fn clear_sftp_env() -> std::sync::MutexGuard<'static, ()> {
    let guard = SFTP_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::remove_var("SFTP_HOST");
        std::env::remove_var("SFTP_PORT");
        std::env::remove_var("SFTP_USER");
        std::env::remove_var("SFTP_PASSWORD");
        std::env::remove_var("SFTP_KEY_PATH");
    }
    guard
}

fn set_sftp_env(addr: SocketAddr) {
    unsafe {
        std::env::set_var("SFTP_HOST", addr.ip().to_string());
        std::env::set_var("SFTP_PORT", addr.port().to_string());
        std::env::set_var("SFTP_USER", SFTP_TEST_USER);
        std::env::set_var("SFTP_PASSWORD", SFTP_TEST_PASSWORD);
        std::env::remove_var("SFTP_KEY_PATH");
    }
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
    let provider = qtcloud_data_cli::storage::DropboxStorage;

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
    let provider = qtcloud_data_cli::storage::DropboxStorage;

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

#[tokio::test]
async fn baidu_send_runs_precreate_upload_create_and_share_flow() {
    let server = MockServer::start().await;
    for method_name in ["precreate", "upload", "create"] {
        let response = if method_name == "precreate" {
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"uploadid": "upload-1"}))
        } else if method_name == "create" {
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"fs_id": 42}))
        } else {
            ResponseTemplate::new(200)
        };
        Mock::given(method("POST"))
            .and(path("/file"))
            .and(query_param("method", method_name))
            .respond_with(response)
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path("/share"))
        .and(query_param("method", "create"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "link": "https://pan.baidu.com/s/abc",
            "pwd": "1234"
        })))
        .mount(&server)
        .await;

    let file = tmp_file("baidu-send.csv", "a,b\n1,2\n");
    let link = BaiduDriveStorage
        .send_with_base(
            "fake-token",
            &file,
            "/apps/report.csv",
            &format!("{}/file", server.uri()),
            &format!("{}/share", server.uri()),
        )
        .await
        .unwrap();

    assert_eq!(link, "https://pan.baidu.com/s/abc?pwd=1234");
    std::fs::remove_file(&file).ok();
}

#[tokio::test]
async fn baidu_receive_reads_share_listing_and_downloads_file() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/share"))
        .and(query_param("method", "list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "list": [{
                "fs_id": 42,
                "dlink": format!("{}/download?x=1", server.uri())
            }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/download"))
        .respond_with(ResponseTemplate::new(200).set_body_string("id,value\n1,x\n"))
        .mount(&server)
        .await;

    let out = std::env::temp_dir().join("baidu-receive.csv");
    BaiduDriveStorage
        .receive_with_base(
            "fake-token",
            &format!("{}/s/abc", server.uri()),
            out.to_str().unwrap(),
            &format!("{}/file", server.uri()),
            &format!("{}/share", server.uri()),
        )
        .await
        .unwrap();

    assert_eq!(std::fs::read_to_string(&out).unwrap(), "id,value\n1,x\n");
    std::fs::remove_file(&out).ok();
}

#[tokio::test]
async fn baidu_provider_reports_missing_token_without_network_call() {
    let _guard = BAIDU_ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::remove_var("BAIDU_ACCESS_TOKEN");
        std::env::remove_var("BAIDUDRIVE_ACCESS_TOKEN");
    }

    let result = BaiduDriveStorage.send("missing.csv", "/report.csv").await;

    assert!(result.unwrap_err().contains("BAIDU_ACCESS_TOKEN"));
}

#[tokio::test]
async fn sftp_send_writes_file_to_remote_path_and_returns_url() {
    let _guard = clear_sftp_env();
    let fixture = start_sftp_fixture().await;
    set_sftp_env(fixture.addr);

    let local = unique_temp_path("sftp-send-source.csv");
    fs::write(&local, b"id,value\n1,ok\n").unwrap();
    let remote_path = "/incoming/report.csv";

    let result = qtcloud_data_cli::storage::SftpStorage
        .send(local.to_str().unwrap(), remote_path)
        .await
        .unwrap();

    assert_eq!(
        result,
        format!(
            "sftp://{}@{}:{}/{}",
            SFTP_TEST_USER,
            fixture.addr.ip(),
            fixture.addr.port(),
            remote_path.trim_start_matches('/')
        )
    );

    let remote_file = fixture.root.join("incoming/report.csv");
    assert_eq!(
        fs::read_to_string(&remote_file).unwrap(),
        "id,value\n1,ok\n"
    );

    fs::remove_file(&local).ok();
    fixture.shutdown().await;
}

#[tokio::test]
async fn sftp_receive_path_downloads_remote_file_to_local_disk() {
    let _guard = clear_sftp_env();
    let fixture = start_sftp_fixture().await;
    set_sftp_env(fixture.addr);

    let remote_file = fixture.root.join("reports/summary.csv");
    if let Some(parent) = remote_file.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&remote_file, b"month,total\naug,42\n").unwrap();

    let local = unique_temp_path("sftp-receive-target.csv");
    let result = qtcloud_data_cli::storage::SftpStorage
        .receive_path("/reports/summary.csv", local.to_str().unwrap())
        .await;

    assert!(result.is_ok(), "{result:?}");
    assert_eq!(fs::read_to_string(&local).unwrap(), "month,total\naug,42\n");

    fs::remove_file(&local).ok();
    fixture.shutdown().await;
}

// ── 网盘类 provider receive_path 测试 ──

#[tokio::test]
async fn test_cloud_providers_receive_path_not_supported() {
    let providers: Vec<Box<dyn Storage>> = vec![
        Box::new(qtcloud_data_cli::storage::DropboxStorage),
        Box::new(qtcloud_data_cli::storage::BaiduDriveStorage),
        Box::new(qtcloud_data_cli::storage::GoogleDriveStorage),
        Box::new(qtcloud_data_cli::storage::OneDriveStorage),
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
    let provider = qtcloud_data_cli::storage::S3Storage;
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
    let provider = qtcloud_data_cli::storage::S3Storage;
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
    assert!(qtcloud_data_cli::storage::detect("https://www.dropbox.com/s/abc/file.csv").is_some(),);
    assert!(qtcloud_data_cli::storage::detect("https://pan.baidu.com/s/1abc").is_some(),);
    assert!(
        qtcloud_data_cli::storage::detect("https://drive.google.com/file/d/abc123/view").is_some(),
    );
    assert!(qtcloud_data_cli::storage::detect("https://1drv.ms/u/s!abc123").is_some(),);
    assert!(
        qtcloud_data_cli::storage::detect("https://s3.us-east-1.amazonaws.com/bucket/key")
            .is_some(),
    );
    assert!(qtcloud_data_cli::storage::detect("sftp://user@host:22/path/file.csv").is_some(),);
    assert!(qtcloud_data_cli::storage::detect("https://example.com/file").is_none(),);
}

#[tokio::test]
async fn test_provider_from_name() {
    assert!(qtcloud_data_cli::storage::from_name("dropbox").is_some());
    assert!(qtcloud_data_cli::storage::from_name("baidu").is_some());
    assert!(qtcloud_data_cli::storage::from_name("baidudrive").is_some());
    assert!(qtcloud_data_cli::storage::from_name("google").is_some());
    assert!(qtcloud_data_cli::storage::from_name("googledrive").is_some());
    assert!(qtcloud_data_cli::storage::from_name("onedrive").is_some());
    assert!(qtcloud_data_cli::storage::from_name("s3").is_some());
    assert!(qtcloud_data_cli::storage::from_name("sftp").is_some());
    assert!(qtcloud_data_cli::storage::from_name("unknown").is_none());
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
        .and(path(format!("/upload/drive/v3/files/{file_id}")))
        .respond_with(ResponseTemplate::new(200))
        .mount(server)
        .await;
    // 3. 设置权限
    Mock::given(method("POST"))
        .and(path(format!("/drive/v3/files/{file_id}/permissions")))
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
