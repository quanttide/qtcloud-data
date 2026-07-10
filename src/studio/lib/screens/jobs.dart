import 'package:flutter/material.dart';
import '../theme.dart';

class JobsScreen extends StatelessWidget {
  const JobsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('执行记录', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 24),
          Card(
            color: secondaryColor,
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                children: [
                  Icon(Icons.hourglass_empty, size: 48, color: Colors.grey),
                  const SizedBox(height: 12),
                  const Text('暂无执行记录'),
                  const SizedBox(height: 8),
                  Text(
                    '通过 CLI 或 API 执行 process 后将在此显示',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
