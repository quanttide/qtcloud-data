import 'package:flutter/material.dart';
import '../theme.dart';

class BlueprintsScreen extends StatelessWidget {
  const BlueprintsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('蓝图', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 24),
          _Card(
            name: 'CSV 数据标准化',
            contract: 'input: CSV → output: CSV + 元数据',
            pipeline: 'csv-standard',
            rules: '4 条验收规则',
          ),
          const SizedBox(height: 12),
          _Card(
            name: '问卷清洗',
            contract: 'input: 原始问卷 → output: 清洗后问卷',
            pipeline: 'csv-standard',
            rules: '4 条验收规则',
          ),
        ],
      ),
    );
  }
}

class _Card extends StatelessWidget {
  final String name;
  final String contract;
  final String pipeline;
  final String rules;
  const _Card({
    required this.name,
    required this.contract,
    required this.pipeline,
    required this.rules,
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
            const SizedBox(height: 8),
            _Row('契约', contract),
            _Row('管道', pipeline),
            _Row('验收', rules),
          ],
        ),
      ),
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
