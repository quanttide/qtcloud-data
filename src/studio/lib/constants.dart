/// 应用版本常量
///
/// 构建时注入（deploy-studio.yml 从 tag 提取）：
///   flutter build web --dart-define=APP_VERSION=0.1.0-beta.2
/// 未注入（本地开发）时为空字符串，界面隐藏版本号，
/// 避免与 seed 数据/发布版本产生双源不一致。
const String appVersion = String.fromEnvironment('APP_VERSION');
