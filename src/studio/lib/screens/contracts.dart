import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/contract.dart';
import '../theme.dart';

/// 契约页：展示数据契约定义（schema、格式等）
/// （seed 数据：assets/data/seed_contracts.json，对应 CLI contract 命令）
class ContractsScreen extends StatefulWidget {
  const ContractsScreen({super.key});

  @override
  State<ContractsScreen> createState() => _ContractsScreenState();
}

class _ContractsScreenState extends State<ContractsScreen> {
  static const _seedAsset = 'assets/data/seed_contracts.json';

  List<Contract>? _contracts;
  bool _loadFailed = false;

  @override
  void initState() {
    super.initState();
    _loadContracts();
  }

  Future<void> _loadContracts() async {
    try {
      final raw = await rootBundle.loadString(_seedAsset);
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      final contracts = (decoded['contracts'] as List<dynamic>)
          .map((e) => Contract.fromJson(e as Map<String, dynamic>))
          .toList();
      if (!mounted) return;
      setState(() => _contracts = contracts);
    } catch (e) {
      debugPrint('契约种子数据加载失败: $e');
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
          Text('契约', style: Theme.of(context).textTheme.headlineMedium),
          const SizedBox(height: 4),
          Text(
            '蓝图使用的数据契约定义（CLI contract list/show）',
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
      return const Center(child: Text('加载失败'));
    }
    final contracts = _contracts;
    if (contracts == null) {
      return const Center(child: CircularProgressIndicator());
    }
    if (contracts.isEmpty) {
      return const Center(child: Text('暂无契约定义'));
    }
    return ListView.separated(
      itemCount: contracts.length,
      separatorBuilder: (context, index) => const SizedBox(height: 12),
      itemBuilder: (context, index) =>
          _ContractCard(contract: contracts[index]),
    );
  }
}

class _ContractCard extends StatelessWidget {
  final Contract contract;
  const _ContractCard({required this.contract});

  @override
  Widget build(BuildContext context) {
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
                    contract.name,
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
                    color: Colors.blue.shade900,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    contract.format,
                    style: const TextStyle(fontSize: 12),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              contract.description,
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 12),
            ...contract.fields.map(
              (f) => Padding(
                padding: const EdgeInsets.symmetric(vertical: 2),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    SizedBox(
                      width: 140,
                      child: Text(
                        f.name + (f.required ? ' *' : ''),
                        style: TextStyle(
                          fontSize: 13,
                          color: f.required ? primaryColor : Colors.grey,
                          fontWeight:
                              f.required ? FontWeight.w600 : FontWeight.w400,
                        ),
                      ),
                    ),
                    Expanded(
                      child: Text(
                        '${f.type}${f.description.isEmpty ? '' : ' — ${f.description}'}',
                        style: const TextStyle(fontSize: 13),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 8),
            Text(
              '共 ${contract.fields.length} 个字段（必填 ${contract.requiredCount}）',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            if (contract.example.isNotEmpty) ...[
              const SizedBox(height: 8),
              Container(
                width: double.infinity,
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: bgColor,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(
                  contract.example,
                  style: const TextStyle(
                    fontSize: 12,
                    fontFamily: 'monospace',
                    color: Colors.grey,
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
