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
