import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/job.dart';
import '../theme.dart';

/// 执行记录页：展示 CLI/API 执行 process 后的 job 记录
/// （seed 数据：assets/data/seed_jobs.json，字段对齐 CLI jobs.json）
class JobsScreen extends StatefulWidget {
  const JobsScreen({super.key});

  @override
  State<JobsScreen> createState() => _JobsScreenState();
}

class _JobsScreenState extends State<JobsScreen> {
  static const _seedAsset = 'assets/data/seed_jobs.json';

  List<Job>? _jobs;
  bool _loadFailed = false;

  @override
  void initState() {
    super.initState();
    _loadJobs();
  }

  Future<void> _loadJobs() async {
    try {
      final raw = await rootBundle.loadString(_seedAsset);
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      final jobs = (decoded['jobs'] as List<dynamic>)
          .map((e) => Job.fromJson(e as Map<String, dynamic>))
          .toList();
      if (!mounted) return;
      setState(() => _jobs = jobs);
    } catch (e) {
      debugPrint('执行记录种子数据加载失败: $e');
      if (!mounted) return;
      setState(() => _loadFailed = true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('执行记录', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 4),
          Text(
            '通过 CLI 或 API 执行 process 后自动记录',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 24),
          Expanded(child: _buildBody()),
        ],
      ),
    );
  }

  Widget _buildBody() {
    if (_loadFailed) {
      return const _EmptyState(
        icon: Icons.error_outline,
        title: '加载失败',
        subtitle: '种子数据读取失败，请检查构建资源',
      );
    }
    final jobs = _jobs;
    if (jobs == null) {
      return const Center(child: CircularProgressIndicator());
    }
    if (jobs.isEmpty) {
      return const _EmptyState(
        icon: Icons.hourglass_empty,
        title: '暂无执行记录',
        subtitle: '通过 CLI 或 API 执行 process 后将在此显示',
      );
    }
    return ListView.separated(
      itemCount: jobs.length,
      separatorBuilder: (context, index) => const SizedBox(height: 12),
      itemBuilder: (context, index) => _JobCard(job: jobs[index]),
    );
  }
}

class _JobCard extends StatelessWidget {
  final Job job;
  const _JobCard({required this.job});

  (Color, Color) get _statusColors {
    switch (job.status) {
      case 'success':
        return (const Color(0xFF1B5E20), const Color(0xFFA5D6A7));
      case 'failed':
        return (const Color(0xFFB71C1C), const Color(0xFFEF9A9A));
      default:
        return (const Color(0xFF0D47A1), const Color(0xFF90CAF9));
    }
  }

  String get _durationText => job.durationSec > 0 ? '${job.durationSec}s' : '—';

  @override
  Widget build(BuildContext context) {
    final statusColors = _statusColors;
    return Card(
      color: secondaryColor,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    '${job.ref} · ${job.blueprint}',
                    style: const TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 10,
                    vertical: 4,
                  ),
                  decoration: BoxDecoration(
                    color: statusColors.$1,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    job.statusLabel,
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                      color: statusColors.$2,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            Text(job.client, style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 12),
            _Row('管道', job.pipeline),
            _Row('原始文件', job.input),
            _Row('结果文件', job.output.isEmpty ? '—' : job.output),
            if (job.shareLink.isNotEmpty) ...[
              _Row('分享链接', job.shareLink, link: true),
            ],
            const SizedBox(height: 8),
            Row(
              children: [
                Text(job.created, style: Theme.of(context).textTheme.bodySmall),
                const Spacer(),
                Text(
                  '耗时 $_durationText',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _Row extends StatelessWidget {
  final String label;
  final String value;
  final bool link;
  const _Row(this.label, this.value, {this.link = false});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 76,
            child: Text(
              label,
              style: const TextStyle(color: Colors.grey, fontSize: 13),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: TextStyle(fontSize: 13, color: link ? primaryColor : null),
            ),
          ),
        ],
      ),
    );
  }
}

class _EmptyState extends StatelessWidget {
  final IconData icon;
  final String title;
  final String subtitle;
  const _EmptyState({
    required this.icon,
    required this.title,
    required this.subtitle,
  });

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Card(
        color: secondaryColor,
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, size: 48, color: Colors.grey),
              const SizedBox(height: 12),
              Text(title),
              const SizedBox(height: 8),
              Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
            ],
          ),
        ),
      ),
    );
  }
}
