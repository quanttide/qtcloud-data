import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../api/client.dart';
import '../theme.dart';

class DashboardScreen extends StatefulWidget {
  const DashboardScreen({super.key});
  @override
  State<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends State<DashboardScreen> {
  final _client = ApiClient();
  String _version = '';
  List<String> _providers = [];
  String _error = '';

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final v = await _client.getVersion();
      final p = await _client.listProviders();
      setState(() {
        _version = v['version'] ?? '';
        _providers = p;
        _error = '';
      });
    } catch (e) {
      setState(() => _error = e.toString());
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('量潮数据云', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 8),
          Text(
            'Provider: $_version',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          if (_error.isNotEmpty) ...[
            const SizedBox(height: 16),
            Card(
              color: Colors.red.shade900,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Text(
                  '连接失败: $_error',
                  style: const TextStyle(color: Colors.white),
                ),
              ),
            ),
          ],
          const SizedBox(height: 32),
          Text('提供商', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 16),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: _providers
                .map(
                  (p) => _ProviderCard(
                    name: p,
                    onTap: () => context.go('/transfer', extra: p),
                  ),
                )
                .toList(),
          ),
          const SizedBox(height: 32),
          Text('快速操作', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 16),
          Row(
            children: [
              _ActionChip(
                icon: Icons.swap_horiz,
                label: '传输文件',
                onTap: () => context.go('/transfer'),
              ),
              const SizedBox(width: 12),
              _ActionChip(
                icon: Icons.receipt_long,
                label: '执行记录',
                onTap: () => context.go('/jobs'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _ProviderCard extends StatelessWidget {
  final String name;
  final VoidCallback onTap;
  const _ProviderCard({required this.name, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        width: 140,
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: secondaryColor,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          children: [
            Icon(Icons.cloud, size: 32, color: Colors.blue.shade300),
            const SizedBox(height: 8),
            Text(name, style: const TextStyle(fontSize: 14)),
          ],
        ),
      ),
    );
  }
}

class _ActionChip extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onTap;
  const _ActionChip({
    required this.icon,
    required this.label,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return ActionChip(avatar: Icon(icon), label: Text(label), onPressed: onTap);
  }
}
