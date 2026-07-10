import 'package:flutter/material.dart';
import '../api/client.dart';
import '../theme.dart';

class TransferScreen extends StatefulWidget {
  const TransferScreen({super.key});
  @override
  State<TransferScreen> createState() => _TransferScreenState();
}

class _TransferScreenState extends State<TransferScreen> {
  final _client = ApiClient();
  final _providerCtl = TextEditingController(text: 'dropbox');
  final _localCtl = TextEditingController();
  final _remoteCtl = TextEditingController();
  final _urlCtl = TextEditingController();
  String _result = '';
  bool _loading = false;
  bool _sendMode = true;

  @override
  void dispose() {
    _providerCtl.dispose();
    _localCtl.dispose();
    _remoteCtl.dispose();
    _urlCtl.dispose();
    _client.dispose();
    super.dispose();
  }

  Future<void> _send() async {
    setState(() {
      _loading = true;
      _result = '';
    });
    try {
      final res = await _client.sendFile(
        provider: _providerCtl.text,
        localPath: _localCtl.text,
        remotePath: _remoteCtl.text,
      );
      setState(() => _result = 'URL: ${res['url']}');
    } catch (e) {
      setState(() => _result = '错误: $e');
    } finally {
      setState(() => _loading = false);
    }
  }

  Future<void> _receive() async {
    setState(() {
      _loading = true;
      _result = '';
    });
    try {
      await _client.receiveFile(
        provider: _providerCtl.text,
        url: _urlCtl.text,
        localPath: _localCtl.text,
      );
      setState(() => _result = '接收成功');
    } catch (e) {
      setState(() => _result = '错误: $e');
    } finally {
      setState(() => _loading = false);
    }
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
          SegmentedButton<bool>(
            segments: const [
              ButtonSegment(value: true, label: Text('发送')),
              ButtonSegment(value: false, label: Text('接收')),
            ],
            selected: {_sendMode},
            onSelectionChanged: (v) => setState(() => _sendMode = v.first),
          ),
          const SizedBox(height: 24),
          _Field('提供商', _providerCtl),
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
            onPressed: _loading ? null : (_sendMode ? _send : _receive),
            child: _loading
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Text(_sendMode ? '发送' : '接收'),
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
