/// 数据蓝图（Blueprint）数据模型
///
/// 对应 spec 四层框架的 Specification 层（动词 design），
/// 见 docs/specification/specification/blueprint.md。
class BlueprintSummary {
  final String id;
  final String name;

  /// 契约摘要（input → output）
  final String contract;

  /// 关联管道
  final String pipeline;

  /// 验收规则数
  final int rules;

  const BlueprintSummary({
    required this.id,
    required this.name,
    required this.contract,
    required this.pipeline,
    required this.rules,
  });

  factory BlueprintSummary.fromJson(Map<String, dynamic> json) =>
      BlueprintSummary(
        id: json['id'] as String,
        name: json['name'] as String,
        contract: json['contract'] as String? ?? '',
        pipeline: json['pipeline'] as String? ?? '',
        rules: json['rules'] as int? ?? 0,
      );
}

/// 蓝图输入字段（数据服务规格书 SRS：输入契约）
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

  factory InputField.fromJson(Map<String, dynamic> json) => InputField(
        name: json['name'] as String,
        type: json['type'] as String? ?? '',
        meaning: json['meaning'] as String? ?? '',
        constraint: json['constraint'] as String? ?? '',
      );
}

/// 蓝图输出字段（数据服务规格书 SRS：输出契约）
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

  factory OutputField.fromJson(Map<String, dynamic> json) => OutputField(
        name: json['name'] as String,
        type: json['type'] as String? ?? '',
        meaning: json['meaning'] as String? ?? '',
        commitment: json['commitment'] as String? ?? '',
      );
}

/// 蓝图处理步骤（数据服务规格书 SRS：处理过程详解）
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

  factory ProcessStep.fromJson(Map<String, dynamic> json) => ProcessStep(
        number: json['number'] as int? ?? 0,
        name: json['name'] as String,
        logic: json['logic'] as String? ?? '',
        action: json['action'] as String? ?? '',
      );
}

/// 蓝图详情（数据服务规格书 SRS 完整数据）
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

  factory BlueprintDetailData.fromJson(Map<String, dynamic> json) =>
      BlueprintDetailData(
        id: json['id'] as String,
        name: json['name'] as String,
        version: json['version'] as String? ?? '',
        description: json['description'] as String? ?? '',
        inputs: (json['inputs'] as List<dynamic>? ?? const [])
            .map((e) => InputField.fromJson(e as Map<String, dynamic>))
            .toList(),
        outputs: (json['outputs'] as List<dynamic>? ?? const [])
            .map((e) => OutputField.fromJson(e as Map<String, dynamic>))
            .toList(),
        steps: (json['steps'] as List<dynamic>? ?? const [])
            .map((e) => ProcessStep.fromJson(e as Map<String, dynamic>))
            .toList(),
      );
}
