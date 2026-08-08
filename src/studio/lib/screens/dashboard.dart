import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';
import '../theme.dart';

/// 总览页：版本信息与提供商入口
///
/// 数据来自 seed 资源（assets/data/seed_dashboard.json），
/// 不依赖服务端——客户端无需先关联服务端即可使用。
class DashboardScreen extends StatefulWidget {
  const DashboardScreen({super.key});

  @override
  State<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends State<DashboardScreen> {
  static const _seedAsset = 'assets/data/seed_dashboard.json';

  String _version = '';
  List<String> _providers = [];

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final raw = await rootBundle.loadString(_seedAsset);
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      if (!mounted) return;
      setState(() {
        _version = decoded['version'] as String? ?? '';
        _providers = (decoded['providers'] as List<dynamic>? ?? const [])
            .map((e) => e as String)
            .toList();
      });
    } catch (e) {
      debugPrint('总览种子数据加载失败: $e');
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
            '控制台 v$_version',
            style: Theme.of(context).textTheme.bodySmall,
          ),
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
