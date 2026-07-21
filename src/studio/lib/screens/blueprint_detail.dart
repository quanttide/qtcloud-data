import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../theme.dart';

// --- Models ---

class InputField {
  final String name;
  final String type;
  final String meaning;
  final String constraint;
  const InputField({
    required this.name,
    required this.type,
    required this.meaning,
    required this.constraint,
  });
}

class OutputField {
  final String name;
  final String type;
  final String meaning;
  final String commitment;
  const OutputField({
    required this.name,
    required this.type,
    required this.meaning,
    required this.commitment,
  });
}

class ProcessStep {
  final int number;
  final String name;
  final String logic;
  final String action;
  const ProcessStep({
    required this.number,
    required this.name,
    required this.logic,
    required this.action,
  });
}

class BlueprintDetailData {
  final String id;
  final String name;
  final String version;
  final String description;
  final List<InputField> inputs;
  final List<OutputField> outputs;
  final List<ProcessStep> steps;
  const BlueprintDetailData({
    required this.id,
    required this.name,
    required this.version,
    required this.description,
    required this.inputs,
    required this.outputs,
    required this.steps,
  });
}

// --- Mock data ---

const _mockBlueprints = {
  'csv-standard': BlueprintDetailData(
    id: 'csv-standard',
    name: '客户画像标准化方案',
    version: 'v1.2',
    description: '将多源原始客户数据清洗、标准化，输出统一的用户画像格式，确保数据质量符合下游分析要求。',
    inputs: [
      InputField(name: 'raw_user_id', type: '文本', meaning: '原始用户标识', constraint: '必填，不可全数字'),
      InputField(name: 'raw_age', type: '数字', meaning: '原始年龄', constraint: '必填，范围 0-120'),
      InputField(name: 'raw_gender', type: '文本', meaning: '原始性别', constraint: '必填，枚举值 [M, F]'),
      InputField(name: 'raw_date', type: '日期', meaning: '注册日期', constraint: '必填，格式 YYYY-MM-DD'),
    ],
    outputs: [
      OutputField(name: 'standard_user_id', type: '文本', meaning: '标准化用户标识', commitment: '去重，非空，长度固定 16 位'),
      OutputField(name: 'age_group', type: '文本', meaning: '年龄段', commitment: '枚举值 [18-25, 26-35, 36-45, 46+]'),
      OutputField(name: 'gender_full', type: '文本', meaning: '性别全称', commitment: '枚举值 [男, 女]，空值率 < 1%'),
      OutputField(name: 'register_quarter', type: '文本', meaning: '注册季度', commitment: '格式 YYYY-QX，去重率 100%'),
    ],
    steps: [
      ProcessStep(
        number: 1,
        name: '数据格式与完整性校验',
        logic: '检查必填字段是否为空，验证日期/数字格式是否合规。',
        action: '不合规的记录将被剔除并记录在《异常数据报告》中。',
      ),
      ProcessStep(
        number: 2,
        name: '缺失值与异常值处理',
        logic: '对于年龄为负数或大于 120 的记录，标记为异常并置空。',
        action: '确保进入下一步的数据在逻辑上是合理的。',
      ),
      ProcessStep(
        number: 3,
        name: '字段标准化映射',
        logic: '将性别缩写 (M/F) 统一转换为全称 (男/女)，将连续的年龄数值映射为年龄段。',
        action: '如 28 岁映射为 "26-35"，M 映射为 "男"。',
      ),
      ProcessStep(
        number: 4,
        name: '去重与最终封装',
        logic: '基于 standard_user_id 进行去重，保留最新记录。',
        action: '生成最终符合输出规格的交付文件。',
      ),
    ],
  ),
  'survey-clean': BlueprintDetailData(
    id: 'survey-clean',
    name: '问卷清洗',
    version: 'v1.0',
    description: '对原始问卷数据进行清洗和结构化处理，去除无效回答，标准化字段格式，输出可直接用于分析的结构化数据集。',
    inputs: [
      InputField(name: 'respondent_id', type: '文本', meaning: '受访者编号', constraint: '必填，唯一标识'),
      InputField(name: 'answer_text', type: '文本', meaning: '开放题回答', constraint: '必填，不超过 5000 字符'),
      InputField(name: 'choice_ids', type: '文本', meaning: '选择题选项', constraint: '必填，逗号分隔的数字列表'),
      InputField(name: 'submit_time', type: '日期时间', meaning: '提交时间', constraint: '必填，ISO 8601 格式'),
    ],
    outputs: [
      OutputField(name: 'clean_respondent_id', type: '文本', meaning: '清洗后受访者编号', commitment: '去重，非空'),
      OutputField(name: 'answer_tokens', type: '文本数组', meaning: '分词结果', commitment: '已去停用词，空值率 < 5%'),
      OutputField(name: 'choice_labels', type: '文本数组', meaning: '选项标签', commitment: '已映射为可读文本'),
      OutputField(name: 'duration_minutes', type: '数字', meaning: '答题时长(分钟)', commitment: '非负，异常值已剔除'),
    ],
    steps: [
      ProcessStep(
        number: 1,
        name: '问卷完整性检查',
        logic: '检查每份问卷的必答题是否全部作答，筛除答题时长异常的记录。',
        action: '不合格问卷标记为无效，不进入后续处理。',
      ),
      ProcessStep(
        number: 2,
        name: '开放题文本清洗',
        logic: '对开放题回答进行分词、去停用词处理。',
        action: '生成标准化的词条列表。',
      ),
      ProcessStep(
        number: 3,
        name: '选择题选项映射',
        logic: '将选项 ID 映射为对应的标签文本。',
        action: '便于分析人员直接阅读。',
      ),
      ProcessStep(
        number: 4,
        name: '输出格式化',
        logic: '汇总所有清洗后的字段，生成最终数据集。',
        action: '附带数据质量报告。',
      ),
    ],
  ),
};

// --- Screen ---

class BlueprintDetailScreen extends StatelessWidget {
  final String id;
  const BlueprintDetailScreen({super.key, required this.id});

  @override
  Widget build(BuildContext context) {
    final bp = _mockBlueprints[id];
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
            style: const TextStyle(color: primaryColor, fontSize: 13, fontWeight: FontWeight.w600),
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
            Text(bp.description, style: const TextStyle(fontSize: 14, height: 1.6)),
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
                Text(label, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold)),
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
                style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600, fontFamily: 'monospace'),
              ),
              const SizedBox(width: 8),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: Colors.white10,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(field.type, style: const TextStyle(fontSize: 11, color: Colors.white54)),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Text(field.meaning, style: const TextStyle(fontSize: 13, color: Colors.white70)),
          const SizedBox(height: 2),
          Text(
            '$hint: $constraintOrCommitment',
            style: TextStyle(fontSize: 12, color: field is InputField ? Colors.orange.withOpacity(0.8) : Colors.green.withOpacity(0.8)),
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
        const Text('我们将严格按照以下步骤处理您的数据：', style: TextStyle(fontSize: 13, color: Colors.white54)),
        const SizedBox(height: 16),
        ...bp.steps.map((step) => _TimelineStep(step: step, isLast: step.number == bp.steps.length)),
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
                    style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: Colors.white),
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
                        style: const TextStyle(fontSize: 15, fontWeight: FontWeight.bold),
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
        Text(label, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
      ],
    );
  }
}
