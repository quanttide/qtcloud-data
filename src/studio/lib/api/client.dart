import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;

class ApiClient {
  final String baseUrl;
  final http.Client _client = http.Client();

  ApiClient({String? baseUrl})
      : baseUrl = baseUrl ?? _defaultBaseUrl();

  static String _defaultBaseUrl() {
    return Platform.environment['PROVIDER_URL'] ?? 'http://localhost:8080';
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
