import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';
import '../models/blueprint.dart';
import '../theme.dart';

/// 蓝图页：数据蓝图列表
///
/// 对应 spec 四层框架的 Specification 层（动词 design），
/// seed 数据：assets/data/seed_blueprints.json。
class BlueprintsScreen extends StatefulWidget {
  const BlueprintsScreen({super.key});

  @override
  State<BlueprintsScreen> createState() => _BlueprintsScreenState();
}

class _BlueprintsScreenState extends State<BlueprintsScreen> {
  static const _seedAsset = 'assets/data/seed_blueprints.json';

  List<BlueprintSummary>? _blueprints;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final raw = await rootBundle.loadString(_seedAsset);
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      final blueprints = (decoded['blueprints'] as List<dynamic>)
          .map((e) => BlueprintSummary.fromJson(e as Map<String, dynamic>))
          .toList();
      if (!mounted) return;
      setState(() => _blueprints = blueprints);
    } catch (e) {
      debugPrint('蓝图种子数据加载失败: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('蓝图', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 4),
          Text(
            '数据蓝图（Blueprint）— 设计阶段产出',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 24),
          Expanded(child: _buildBody()),
        ],
      ),
    );
  }

  Widget _buildBody() {
    final blueprints = _blueprints;
    if (blueprints == null) {
      return const Center(child: CircularProgressIndicator());
    }
    if (blueprints.isEmpty) {
      return const Center(child: Text('暂无蓝图'));
    }
    return ListView.separated(
      itemCount: blueprints.length,
      separatorBuilder: (context, index) => const SizedBox(height: 12),
      itemBuilder: (context, index) {
        final b = blueprints[index];
        return Card(
          color: secondaryColor,
          child: InkWell(
            onTap: () => context.push('/blueprints/${b.id}'),
            borderRadius: BorderRadius.circular(12),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    b.name,
                    style: const TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  const SizedBox(height: 8),
                  _Row('契约', b.contract),
                  _Row('管道', b.pipeline),
                  _Row('验收', '${b.rules} 条规则'),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}

class _Row extends StatelessWidget {
  final String label;
  final String value;
  const _Row(this.label, this.value);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          SizedBox(
            width: 60,
            child: Text(
              label,
              style: const TextStyle(color: Colors.grey, fontSize: 13),
            ),
          ),
          Expanded(child: Text(value, style: const TextStyle(fontSize: 13))),
        ],
      ),
    );
  }
}
