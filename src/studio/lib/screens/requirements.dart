import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/requirement.dart';
import '../theme.dart';

/// 需求页：数据需求文档（DRD）列表
///
/// 对应 spec 四层框架的 Requirement 层（动词 clarify），
/// seed 数据：assets/data/seed_requirements.json。
class RequirementsScreen extends StatefulWidget {
  const RequirementsScreen({super.key});

  @override
  State<RequirementsScreen> createState() => _RequirementsScreenState();
}

class _RequirementsScreenState extends State<RequirementsScreen> {
  static const _seedAsset = 'assets/data/seed_requirements.json';

  List<Requirement>? _requirements;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final raw = await rootBundle.loadString(_seedAsset);
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      final requirements = (decoded['requirements'] as List<dynamic>)
          .map((e) => Requirement.fromJson(e as Map<String, dynamic>))
          .toList();
      if (!mounted) return;
      setState(() => _requirements = requirements);
    } catch (e) {
      debugPrint('需求种子数据加载失败: $e');
    }
  }

  (Color, Color) _statusColors(Requirement r) {
    switch (r.status) {
      case 'done':
        return (const Color(0xFF1B5E20), const Color(0xFFA5D6A7));
      case 'active':
        return (const Color(0xFF0D47A1), const Color(0xFF90CAF9));
      case 'approved':
        return (const Color(0xFF4A148C), const Color(0xFFCE93D8));
      default:
        return (const Color(0xFF37474F), const Color(0xFFB0BEC5));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('需求', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 4),
          Text(
            '数据需求文档（DRD）— 澄清阶段产出',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 24),
          Expanded(child: _buildBody()),
        ],
      ),
    );
  }

  Widget _buildBody() {
    final requirements = _requirements;
    if (requirements == null) {
      return const Center(child: CircularProgressIndicator());
    }
    if (requirements.isEmpty) {
      return const Center(child: Text('暂无需求'));
    }
    return ListView.separated(
      itemCount: requirements.length,
      separatorBuilder: (context, index) => const SizedBox(height: 12),
      itemBuilder: (context, index) {
        final r = requirements[index];
        final colors = _statusColors(r);
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
                        r.name,
                        style: const TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 10, vertical: 4),
                      decoration: BoxDecoration(
                        color: colors.$1,
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Text(
                        r.statusLabel,
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w500,
                          color: colors.$2,
                        ),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  '${r.project.isNotEmpty ? '${r.project} · ' : ''}${r.client} · ${r.created}',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
                const SizedBox(height: 12),
                Text(r.summary, style: const TextStyle(fontSize: 13)),
                const SizedBox(height: 8),
                Text(
                  '目标：${r.goal}',
                  style: const TextStyle(fontSize: 13, color: Colors.grey),
                ),
                Text(
                  '输出：${r.outputExpectation}',
                  style: const TextStyle(fontSize: 13, color: Colors.grey),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
