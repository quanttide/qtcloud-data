import 'package:flutter/material.dart';
import '../theme.dart';

class PipelinesScreen extends StatelessWidget {
  const PipelinesScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('管道', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 24),
          _PlaceholderCard(
            name: 'csv-standard',
            desc: 'CSV 格式校验 → 元数据注入 → 空值分析',
            steps: 'processor → enricher',
          ),
          const SizedBox(height: 12),
          _PlaceholderCard(
            name: 'annotation',
            desc: 'LLM 辅助数据标注',
            steps: 'preprocess（预留）',
          ),
        ],
      ),
    );
  }
}

class _PlaceholderCard extends StatelessWidget {
  final String name;
  final String desc;
  final String steps;
  const _PlaceholderCard({
    required this.name,
    required this.desc,
    required this.steps,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      color: secondaryColor,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              name,
              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 4),
            Text(desc, style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: Colors.blue.shade900,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(steps, style: const TextStyle(fontSize: 12)),
            ),
          ],
        ),
      ),
    );
  }
}
