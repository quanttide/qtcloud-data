/// 数据需求（Requirement / DRD）数据模型
///
/// 对应 spec 四层框架的 Requirement 层（动词 clarify，产物 DRD），
/// 见 docs/specification/requirement/index.md。
class Requirement {
  final String id;
  final String name;

  /// 所属项目（完整案例，如 量潮科技数字化）
  final String project;

  /// 客户
  final String client;

  /// 状态：draft / approved / active / done
  final String status;
  final String created;

  /// 需求摘要
  final String summary;

  /// 业务目标（DRD 章节）
  final String goal;

  /// 输出期望（DRD 章节）
  final String outputExpectation;

  const Requirement({
    required this.id,
    required this.name,
    required this.project,
    required this.client,
    required this.status,
    required this.created,
    required this.summary,
    required this.goal,
    required this.outputExpectation,
  });

  factory Requirement.fromJson(Map<String, dynamic> json) => Requirement(
        id: json['id'] as String,
        name: json['name'] as String,
        project: json['project'] as String? ?? '',
        client: json['client'] as String? ?? '',
        status: json['status'] as String? ?? 'draft',
        created: json['created'] as String? ?? '',
        summary: json['summary'] as String? ?? '',
        goal: json['goal'] as String? ?? '',
        outputExpectation: json['output_expectation'] as String? ?? '',
      );

  /// 状态展示文案
  String get statusLabel => switch (status) {
        'draft' => '草稿',
        'approved' => '已确认',
        'active' => '进行中',
        'done' => '已完成',
        _ => status,
      };
}
