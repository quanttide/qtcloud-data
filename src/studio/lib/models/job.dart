/// 执行记录（job）数据模型
///
/// 字段对齐 qtcloud-data CLI 的 jobs.json 记录
/// （见 docs/transfer.md：客户、来源、blueprint、pipeline、原始文件、
///  最终结果、分享链接文件、状态和日志路径）。
class Job {
  final String id;
  final String ref;

  /// 客户名称
  final String client;

  /// 使用的蓝图 / 管道
  final String blueprint;
  final String pipeline;

  /// 来源链接
  final String source;

  /// 原始文件 / 最终结果文件
  final String input;
  final String output;

  /// 交付分享链接（成功后生成）
  final String shareLink;

  /// 状态：success / failed / running
  final String status;

  /// 创建时间（展示用）
  final String created;

  /// 耗时（秒）
  final int durationSec;

  const Job({
    required this.id,
    required this.ref,
    required this.client,
    required this.blueprint,
    required this.pipeline,
    required this.source,
    required this.input,
    required this.output,
    required this.shareLink,
    required this.status,
    required this.created,
    required this.durationSec,
  });

  factory Job.fromJson(Map<String, dynamic> json) => Job(
    id: json['id'] as String,
    ref: json['ref'] as String,
    client: json['client'] as String,
    blueprint: json['blueprint'] as String,
    pipeline: json['pipeline'] as String,
    source: json['source'] as String? ?? '',
    input: json['input'] as String? ?? '',
    output: json['output'] as String? ?? '',
    shareLink: json['share_link'] as String? ?? '',
    status: json['status'] as String,
    created: json['created'] as String? ?? '',
    durationSec: json['duration_sec'] as int? ?? 0,
  );

  /// 状态展示文案
  String get statusLabel => switch (status) {
    'success' => '成功',
    'failed' => '失败',
    'running' => '进行中',
    _ => status,
  };
}
