/// 契约（contract）数据模型
///
/// 对应 qtcloud-data CLI 的 contract 命令
/// （见 docs/transfer.md：查看独立的数据契约定义——schema、格式等）。
class ContractField {
  final String name;
  final String type;
  final bool required;
  final String description;

  const ContractField({
    required this.name,
    required this.type,
    required this.required,
    required this.description,
  });

  factory ContractField.fromJson(Map<String, dynamic> json) => ContractField(
    name: json['name'] as String,
    type: json['type'] as String,
    required: json['required'] as bool? ?? false,
    description: json['description'] as String? ?? '',
  );
}

class Contract {
  final String id;
  final String name;

  /// 契约用途说明
  final String description;

  /// 数据格式：csv / yaml / json 等
  final String format;

  /// 字段（schema）定义
  final List<ContractField> fields;

  /// 示例数据
  final String example;

  const Contract({
    required this.id,
    required this.name,
    required this.description,
    required this.format,
    required this.fields,
    required this.example,
  });

  factory Contract.fromJson(Map<String, dynamic> json) => Contract(
    id: json['id'] as String,
    name: json['name'] as String,
    description: json['description'] as String? ?? '',
    format: json['format'] as String? ?? '',
    fields: (json['fields'] as List<dynamic>? ?? const [])
        .map((e) => ContractField.fromJson(e as Map<String, dynamic>))
        .toList(),
    example: json['example'] as String? ?? '',
  );

  /// 必填字段数量
  int get requiredCount => fields.where((f) => f.required).length;
}
