import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/pipeline.dart';
import '../theme.dart';

/// 管道页：数据管道列表
///
/// 对应 spec 四层框架的 Implementation 层（动词 implement），
/// seed 数据：assets/data/seed_pipelines.json。
class PipelinesScreen extends StatefulWidget {
  const PipelinesScreen({super.key});

  @override
  State<PipelinesScreen> createState() => _PipelinesScreenState();
}

class _PipelinesScreenState extends State<PipelinesScreen> {
  static const _seedAsset = 'assets/data/seed_pipelines.json';

  List<Pipeline>? _pipelines;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final raw = await rootBundle.loadString(_seedAsset);
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      final pipelines = (decoded['pipelines'] as List<dynamic>)
          .map((e) => Pipeline.fromJson(e as Map<String, dynamic>))
          .toList();
      if (!mounted) return;
      setState(() => _pipelines = pipelines);
    } catch (e) {
      debugPrint('管道种子数据加载失败: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('管道', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 4),
          Text(
            '数据管道（Pipeline）— 实现阶段产出',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 24),
          Expanded(child: _buildBody()),
        ],
      ),
    );
  }

  Widget _buildBody() {
    final pipelines = _pipelines;
    if (pipelines == null) {
      return const Center(child: CircularProgressIndicator());
    }
    if (pipelines.isEmpty) {
      return const Center(child: Text('暂无管道'));
    }
    return ListView.separated(
      itemCount: pipelines.length,
      separatorBuilder: (context, index) => const SizedBox(height: 12),
      itemBuilder: (context, index) {
        final p = pipelines[index];
        return Card(
          color: secondaryColor,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  p.id,
                  style: const TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 4),
                Text(p.desc, style: Theme.of(context).textTheme.bodySmall),
                const SizedBox(height: 8),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: Colors.blue.shade900,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(p.steps, style: const TextStyle(fontSize: 12)),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
