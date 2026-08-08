import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;

class ApiClient {
  final String baseUrl;
  final http.Client _client;

  ApiClient({String? baseUrl, http.Client? client})
      : baseUrl = baseUrl ?? _defaultBaseUrl(),
        _client = client ?? http.Client();

  /// 跨平台默认地址（Web 部署必须用编译期注入）：
  /// 1. `flutter build web --dart-define=PROVIDER_URL=https://...`（推荐）
  /// 2. 桌面/移动端运行时环境变量 PROVIDER_URL
  /// 3. 本地开发兜底 localhost:8080
  ///
  /// 注意：不能在 Web 上使用 dart:io 的 Platform.environment
  /// （浏览器环境不支持，会导致页面初始化崩溃）。
  static String _defaultBaseUrl() {
    const injected = String.fromEnvironment('PROVIDER_URL');
    if (injected.isNotEmpty) return injected;
    if (!kIsWeb) {
      // 非 Web 平台支持运行时环境变量
      const env = String.fromEnvironment('PROVIDER_URL');
      if (env.isNotEmpty) return env;
    }
    return 'http://localhost:8080';
  }

  Future<List<String>> listProviders() async {
    final resp = await _client.get(Uri.parse('$baseUrl/providers'));
    _check(resp);
    return List<String>.from(jsonDecode(resp.body));
  }

  Future<Map<String, String>> sendFile({
    required String provider,
    required String localPath,
    required String remotePath,
  }) async {
    final resp = await _client.post(
      Uri.parse('$baseUrl/transfer/send'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'provider': provider,
        'local_path': localPath,
        'remote_path': remotePath,
      }),
    );
    _check(resp);
    return Map<String, String>.from(jsonDecode(resp.body));
  }

  Future<void> receiveFile({
    required String provider,
    required String url,
    required String localPath,
  }) async {
    final resp = await _client.post(
      Uri.parse('$baseUrl/transfer/receive'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'provider': provider,
        'url': url,
        'local_path': localPath,
      }),
    );
    _check(resp);
  }

  Future<Map<String, dynamic>> getVersion() async {
    final resp = await _client.get(Uri.parse('$baseUrl/version'));
    _check(resp);
    return Map<String, dynamic>.from(jsonDecode(resp.body));
  }

  void _check(http.Response resp) {
    if (resp.statusCode >= 400) {
      throw Exception('${resp.statusCode}: ${resp.body}');
    }
  }

  void dispose() => _client.close();
}
