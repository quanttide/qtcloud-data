import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';
import '../constants.dart';
import '../models/job.dart';
import '../theme.dart';

/// 总览页：数据云系统汇总
///
/// 遵循 spec 四层框架（Requirement → Specification → Implementation → Task，
/// transfer 衔接交付）汇总各模块统计与最近动态。
/// 数据全部来自 seed 资源，不依赖服务端。
class DashboardScreen extends StatefulWidget {
  const DashboardScreen({super.key});

  @override
  State<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends State<DashboardScreen> {
  static const _seeds = {
    '需求': 'assets/data/seed_requirements.json',
    '蓝图': 'assets/data/seed_blueprints.json',
    '契约': 'assets/data/seed_contracts.json',
    '管道': 'assets/data/seed_pipelines.json',
    '执行': 'assets/data/seed_jobs.json',
    '传输': 'assets/data/seed_dashboard.json',
  };

  Map<String, int> _counts = {};
  List<Job> _recentJobs = [];
  bool _loaded = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final counts = <String, int>{};
      for (final entry in _seeds.entries) {
        final raw = await rootBundle.loadString(entry.value);
        final decoded = jsonDecode(raw) as Map<String, dynamic>;
        // 各 seed 顶层键即模块数据数组；传输模块统计提供商数
        final key = entry.value.contains('seed_dashboard')
            ? 'providers'
            : decoded.keys.firstWhere(
                (k) => decoded[k] is List,
                orElse: () => '',
              );
        if (key.isEmpty) continue;
        counts[entry.key] = (decoded[key] as List<dynamic>).length;
      }
      final jobsRaw = await rootBundle.loadString(_seeds['执行']!);
      final jobs = ((jsonDecode(jobsRaw) as Map<String, dynamic>)['jobs']
              as List<dynamic>)
          .map((e) => Job.fromJson(e as Map<String, dynamic>))
          .toList();
      if (!mounted) return;
      setState(() {
        _counts = counts;
        _recentJobs = jobs.take(3).toList();
        _loaded = true;
      });
    } catch (e) {
      debugPrint('总览种子数据加载失败: $e');
      if (!mounted) return;
      setState(() => _loaded = true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('总览', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 4),
          Text(
            appVersion.isEmpty
                ? '数据云系统汇总：需求 → 规格 → 实现 → 执行 → 交付'
                : '数据云系统汇总 v$appVersion：需求 → 规格 → 实现 → 执行 → 交付',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 24),
          Expanded(
            child: _loaded
                ? _buildBody(context)
                : const Center(child: CircularProgressIndicator()),
          ),
        ],
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return ListView(
      children: [
        // 模块统计卡片
        _buildStatGrid(context),
        const SizedBox(height: 24),
        // 最近执行记录
        Text('最近执行', style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: 12),
        if (_recentJobs.isEmpty)
          Text('暂无执行记录', style: Theme.of(context).textTheme.bodySmall)
        else
          ..._recentJobs.map(
            (job) => Card(
              color: secondaryColor,
              child: Padding(
                padding:
                    const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                child: Row(
                  children: [
                    Icon(
                      job.status == 'success'
                          ? Icons.check_circle
                          : (job.status == 'running'
                              ? Icons.hourglass_top
                              : Icons.error),
                      size: 18,
                      color: job.status == 'success'
                          ? const Color(0xFFA5D6A7)
                          : (job.status == 'running'
                              ? const Color(0xFF90CAF9)
                              : const Color(0xFFEF9A9A)),
                    ),
                    const SizedBox(width: 10),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            '${job.ref} · ${job.blueprint}',
                            style: const TextStyle(fontSize: 13),
                          ),
                          Text(
                            job.created,
                            style: Theme.of(context).textTheme.bodySmall,
                          ),
                        ],
                      ),
                    ),
                    Text(
                      job.statusLabel,
                      style: const TextStyle(fontSize: 12),
                    ),
                  ],
                ),
              ),
            ),
          ),
        const SizedBox(height: 8),
        TextButton.icon(
          onPressed: () => context.go('/jobs'),
          icon: const Icon(Icons.receipt_long, size: 16),
          label: const Text('查看全部执行记录'),
        ),
      ],
    );
  }

  Widget _buildStatGrid(BuildContext context) {
    const items = [
      ('需求', '/requirements', Icons.description_outlined),
      ('蓝图', '/blueprints', Icons.account_tree),
      ('契约', '/contracts', Icons.article_outlined),
      ('管道', '/pipelines', Icons.schema_outlined),
      ('执行', '/jobs', Icons.receipt_long),
      ('传输', '/transfer', Icons.swap_horiz),
    ];
    return GridView.count(
      crossAxisCount: 3,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      mainAxisSpacing: 12,
      crossAxisSpacing: 12,
      childAspectRatio: 1.6,
      children: items.map((item) {
        final (label, path, icon) = item;
        final count = _counts[label] ?? 0;
        return InkWell(
          onTap: () => context.go(path),
          borderRadius: BorderRadius.circular(12),
          child: Container(
            decoration: BoxDecoration(
              color: secondaryColor,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(icon, size: 24, color: Colors.blue.shade300),
                const SizedBox(height: 8),
                Text(
                  '$count',
                  style: const TextStyle(
                    fontSize: 22,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  label,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
        );
      }).toList(),
    );
  }
}
