import 'dart:convert';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../theme.dart';

/// 传输页：发送 / 接收演示
///
/// 本地模拟交互，不依赖服务端——客户端无需先关联服务端即可使用。
/// 分享链接格式对齐 CLI 输出（见 docs/transfer.md）：
///   https://www.dropbox.com/s/<id>/<文件名>?dl=1
class TransferScreen extends StatefulWidget {
  const TransferScreen({super.key, this.initialProvider});

  /// 从总览页提供商卡片进入时预填的提供商（可选）
  final String? initialProvider;

  @override
  State<TransferScreen> createState() => _TransferScreenState();
}

class _TransferScreenState extends State<TransferScreen> {
  static const _seedAsset = 'assets/data/seed_dashboard.json';

  List<String> _providers = const ['dropbox'];

  late final TextEditingController _providerCtl =
      TextEditingController(text: widget.initialProvider ?? 'dropbox');
  final _localCtl = TextEditingController();
  final _remoteCtl = TextEditingController();
  final _urlCtl = TextEditingController();
  String _result = '';
  bool _sendMode = true;

  @override
  void initState() {
    super.initState();
    _loadProviders();
  }

  /// 提供商列表与总览页同源（seed_dashboard.json）
  Future<void> _loadProviders() async {
    try {
      final raw = await rootBundle.loadString(_seedAsset);
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      final providers = (decoded['providers'] as List<dynamic>? ?? const [])
          .map((e) => e as String)
          .toList();
      if (!mounted || providers.isEmpty) return;
      setState(() => _providers = providers);
    } catch (e) {
      debugPrint('提供商列表加载失败: $e');
    }
  }

  @override
  void dispose() {
    _providerCtl.dispose();
    _localCtl.dispose();
    _remoteCtl.dispose();
    _urlCtl.dispose();
    super.dispose();
  }

  /// 本地模拟发送：生成分享链接（不发起网络请求）
  void _send() {
    final fileName = _remoteCtl.text.split('/').last;
    final safeName = fileName.isEmpty ? 'file' : fileName;
    final id = List.generate(8, (_) => _randomChar()).join();
    setState(() {
      _result = '模拟上传完成\nhttps://www.dropbox.com/s/$id/$safeName?dl=1';
    });
  }

  /// 本地模拟接收（不发起网络请求）
  void _receive() {
    setState(() {
      _result = '模拟接收完成\n${_urlCtl.text.isEmpty ? '（未填写链接）' : _urlCtl.text}'
          '\n→ ${_localCtl.text.isEmpty ? '（未填写保存路径）' : _localCtl.text}';
    });
  }

  String _randomChar() {
    const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
    return chars[Random().nextInt(chars.length)];
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('数据传输', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 8),
          Text(
            '本地演示：发送 / 接收结果模拟生成，不依赖服务端',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 16),
          SegmentedButton<bool>(
            segments: const [
              ButtonSegment(value: true, label: Text('发送')),
              ButtonSegment(value: false, label: Text('接收')),
            ],
            selected: {_sendMode},
            onSelectionChanged: (v) => setState(() => _sendMode = v.first),
          ),
          const SizedBox(height: 24),
          _ProviderField(controller: _providerCtl, providers: _providers),
          const SizedBox(height: 12),
          if (_sendMode) ...[
            _Field('本地路径', _localCtl),
            const SizedBox(height: 12),
            _Field('远程路径', _remoteCtl),
          ] else ...[
            _Field('分享链接', _urlCtl),
            const SizedBox(height: 12),
            _Field('本地保存路径', _localCtl),
          ],
          const SizedBox(height: 24),
          ElevatedButton(
            onPressed: _sendMode ? _send : _receive,
            child: Text(_sendMode ? '发送' : '接收'),
          ),
          if (_result.isNotEmpty) ...[
            const SizedBox(height: 16),
            Card(
              color: secondaryColor,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: SelectableText(_result),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _ProviderField extends StatelessWidget {
  final TextEditingController controller;
  final List<String> providers;
  const _ProviderField({required this.controller, required this.providers});

  @override
  Widget build(BuildContext context) {
    return DropdownButtonFormField<String>(
      initialValue: controller.text,
      items: providers
          .map((p) => DropdownMenuItem(value: p, child: Text(p)))
          .toList(),
      onChanged: (v) {
        if (v != null) controller.text = v;
      },
      decoration: const InputDecoration(
        labelText: '提供商',
        border: OutlineInputBorder(),
        filled: true,
        fillColor: secondaryColor,
      ),
    );
  }
}

class _Field extends StatelessWidget {
  final String label;
  final TextEditingController controller;
  const _Field(this.label, this.controller);

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      decoration: InputDecoration(
        labelText: label,
        border: const OutlineInputBorder(),
        filled: true,
        fillColor: secondaryColor,
      ),
    );
  }
}
