/// 数据管道（Pipeline）数据模型
///
/// 对应 spec 四层框架的 Implementation 层（动词 implement），
/// 见 docs/specification/implementation/pipeline.md。
class Pipeline {
  final String id;

  /// 管道说明
  final String desc;

  /// 步骤链（展示用）
  final String steps;

  const Pipeline({
    required this.id,
    required this.desc,
    required this.steps,
  });

  factory Pipeline.fromJson(Map<String, dynamic> json) => Pipeline(
        id: json['id'] as String,
        desc: json['desc'] as String? ?? '',
        steps: json['steps'] as String? ?? '',
      );
}
