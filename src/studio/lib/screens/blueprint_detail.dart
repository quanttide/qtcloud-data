import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';
import '../models/blueprint.dart';
import '../theme.dart';

// --- Screen ---

class BlueprintDetailScreen extends StatefulWidget {
  final String id;
  const BlueprintDetailScreen({super.key, required this.id});

  @override
  State<BlueprintDetailScreen> createState() => _BlueprintDetailScreenState();
}

class _BlueprintDetailScreenState extends State<BlueprintDetailScreen> {
  static const _seedAsset = 'assets/data/seed_blueprints.json';

  BlueprintDetailData? _bp;
  bool _loaded = false;

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
          .map((e) => BlueprintDetailData.fromJson(e as Map<String, dynamic>))
          .toList();
      BlueprintDetailData? found;
      for (final b in blueprints) {
        if (b.id == widget.id) {
          found = b;
          break;
        }
      }
      if (!mounted) return;
      setState(() {
        _bp = found;
        _loaded = true;
      });
    } catch (e) {
      debugPrint('蓝图详情种子数据加载失败: $e');
      if (!mounted) return;
      setState(() => _loaded = true);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (!_loaded) {
      return const Center(child: CircularProgressIndicator());
    }
    final bp = _bp;
    if (bp == null) {
      return Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _BackButton(),
            const SizedBox(height: 48),
            const Center(child: Text('未找到该蓝图')),
          ],
        ),
      );
    }

    return Padding(
      padding: const EdgeInsets.all(24),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _Header(name: bp.name, version: bp.version),
            const SizedBox(height: 24),
            _ModuleOne(bp: bp),
            const SizedBox(height: 24),
            _ModuleTwo(bp: bp),
            const SizedBox(height: 24),
            _ModuleThree(bp: bp),
          ],
        ),
      ),
    );
  }
}

// --- Header with back button ---

class _BackButton extends StatelessWidget {
  const _BackButton();

  @override
  Widget build(BuildContext context) {
    return TextButton.icon(
      onPressed: () => context.pop(),
      icon: const Icon(Icons.arrow_back, size: 18),
      label: const Text('返回列表'),
    );
  }
}

class _Header extends StatelessWidget {
  final String name;
  final String version;
  const _Header({required this.name, required this.version});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        _BackButton(),
        const SizedBox(width: 16),
        Text(name, style: Theme.of(context).textTheme.headlineSmall),
        const SizedBox(width: 12),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
          decoration: BoxDecoration(
            color: primaryColor.withOpacity(0.2),
            borderRadius: BorderRadius.circular(12),
          ),
          child: Text(
            version,
            style: const TextStyle(
                color: primaryColor, fontSize: 13, fontWeight: FontWeight.w600),
          ),
        ),
      ],
    );
  }
}

// --- Module 1: Blueprint Summary ---

class _ModuleOne extends StatelessWidget {
  final BlueprintDetailData bp;
  const _ModuleOne({required this.bp});

  @override
  Widget build(BuildContext context) {
    return Card(
      color: secondaryColor,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const _SectionTitle(icon: Icons.description, label: '方案概要'),
            const SizedBox(height: 12),
            Text(bp.description,
                style: const TextStyle(fontSize: 14, height: 1.6)),
          ],
        ),
      ),
    );
  }
}

// --- Module 2: Input / Output Contract ---

class _ModuleTwo extends StatelessWidget {
  final BlueprintDetailData bp;
  const _ModuleTwo({required this.bp});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _SectionTitle(icon: Icons.swap_horiz, label: '数据交接规格'),
        const SizedBox(height: 12),
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(child: _FieldTable(isInput: true, bp: bp)),
            const SizedBox(width: 16),
            Expanded(child: _FieldTable(isInput: false, bp: bp)),
          ],
        ),
        const SizedBox(height: 12),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: Colors.orange.withOpacity(0.1),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Colors.orange.withOpacity(0.3)),
          ),
          child: const Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(Icons.warning_amber_rounded, size: 16, color: Colors.orange),
              SizedBox(width: 8),
              Expanded(
                child: Text(
                  '请确保输入数据符合上述规格，否则可能导致处理异常或延迟交付。',
                  style: TextStyle(fontSize: 13, color: Colors.orange),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _FieldTable extends StatelessWidget {
  final bool isInput;
  final BlueprintDetailData bp;
  const _FieldTable({required this.isInput, required this.bp});

  @override
  Widget build(BuildContext context) {
    final fields = isInput ? bp.inputs : bp.outputs;
    final icon = isInput ? Icons.download : Icons.upload;
    final label = isInput ? '您需提供的数据（输入）' : '我们将交付的数据（输出）';
    final hint = isInput ? '约束条件' : '质量承诺';
    final iconColor = isInput ? Colors.orange : Colors.green;

    return Card(
      color: secondaryColor,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, size: 18, color: iconColor),
                const SizedBox(width: 8),
                Text(label,
                    style: const TextStyle(
                        fontSize: 14, fontWeight: FontWeight.bold)),
              ],
            ),
            const Divider(height: 24),
            ...fields.map((f) => _FieldRow(field: f, hint: hint)),
          ],
        ),
      ),
    );
  }
}

class _FieldRow extends StatelessWidget {
  final dynamic field;
  final String hint;
  const _FieldRow({required this.field, required this.hint});

  @override
  Widget build(BuildContext context) {
    final constraintOrCommitment = field is InputField
        ? (field as InputField).constraint
        : (field as OutputField).commitment;

    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                field.name,
                style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    fontFamily: 'monospace'),
              ),
              const SizedBox(width: 8),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: Colors.white10,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(field.type,
                    style:
                        const TextStyle(fontSize: 11, color: Colors.white54)),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Text(field.meaning,
              style: const TextStyle(fontSize: 13, color: Colors.white70)),
          const SizedBox(height: 2),
          Text(
            '$hint: $constraintOrCommitment',
            style: TextStyle(
                fontSize: 12,
                color: field is InputField
                    ? Colors.orange.withOpacity(0.8)
                    : Colors.green.withOpacity(0.8)),
          ),
        ],
      ),
    );
  }
}

// --- Module 3: Process Steps Timeline ---

class _ModuleThree extends StatelessWidget {
  final BlueprintDetailData bp;
  const _ModuleThree({required this.bp});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _SectionTitle(icon: Icons.settings, label: '处理过程详解（透明化业务流）'),
        const SizedBox(height: 4),
        const Text('我们将严格按照以下步骤处理您的数据：',
            style: TextStyle(fontSize: 13, color: Colors.white54)),
        const SizedBox(height: 16),
        ...bp.steps.map((step) =>
            _TimelineStep(step: step, isLast: step.number == bp.steps.length)),
      ],
    );
  }
}

class _TimelineStep extends StatelessWidget {
  final ProcessStep step;
  final bool isLast;
  const _TimelineStep({required this.step, required this.isLast});

  @override
  Widget build(BuildContext context) {
    return IntrinsicHeight(
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Timeline indicator
          SizedBox(
            width: 40,
            child: Column(
              children: [
                Container(
                  width: 32,
                  height: 32,
                  decoration: const BoxDecoration(
                    color: primaryColor,
                    shape: BoxShape.circle,
                  ),
                  alignment: Alignment.center,
                  child: Text(
                    '${step.number}',
                    style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.bold,
                        color: Colors.white),
                  ),
                ),
                if (!isLast)
                  Expanded(
                    child: Container(
                      width: 2,
                      color: primaryColor.withOpacity(0.4),
                    ),
                  ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          // Content
          Expanded(
            child: Padding(
              padding: EdgeInsets.only(bottom: isLast ? 0 : 24),
              child: Card(
                color: secondaryColor,
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        '步骤 ${step.number}：${step.name}',
                        style: const TextStyle(
                            fontSize: 15, fontWeight: FontWeight.bold),
                      ),
                      const SizedBox(height: 8),
                      _StepDetail(label: '逻辑', text: step.logic),
                      const SizedBox(height: 6),
                      _StepDetail(label: '动作', text: step.action),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _StepDetail extends StatelessWidget {
  final String label;
  final String text;
  const _StepDetail({required this.label, required this.text});

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 36,
          child: Text(
            '$label:',
            style: const TextStyle(fontSize: 13, color: Colors.white38),
          ),
        ),
        Expanded(
          child: Text(text, style: const TextStyle(fontSize: 13, height: 1.5)),
        ),
      ],
    );
  }
}

// --- Shared ---

class _SectionTitle extends StatelessWidget {
  final IconData icon;
  final String label;
  const _SectionTitle({required this.icon, required this.label});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, size: 20, color: primaryColor),
        const SizedBox(width: 8),
        Text(label,
            style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
      ],
    );
  }
}
