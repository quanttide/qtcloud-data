import 'package:flutter/material.dart';
import '../theme.dart';

class ContractsScreen extends StatelessWidget {
  const ContractsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('契约', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 24),
          Card(
            color: secondaryColor,
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                children: [
                  Icon(
                    Icons.description_outlined,
                    size: 48,
                    color: Colors.grey,
                  ),
                  const SizedBox(height: 12),
                  const Text('契约定义'),
                  const SizedBox(height: 8),
                  Text(
                    '配置 CONTRACTS_DIR 后，蓝图中的契约定义将在此展示',
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
