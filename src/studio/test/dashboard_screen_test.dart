// 总览页组件测试
//
// 覆盖：
//  - 系统汇总：各模块统计卡片（需求/蓝图/契约/管道/执行/传输）
//  - 最近执行记录展示
//  - 无服务端依赖：不展示任何"连接失败"类错误卡

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:qtcloud_data_studio/screens/dashboard.dart';
import 'package:qtcloud_data_studio/theme.dart';

Widget _wrap(Widget child) =>
    MaterialApp(theme: defaultThemeData, home: Scaffold(body: child));

void main() {
  testWidgets('总览页：汇总各模块统计与最近执行（seed 数据）', (tester) async {
    await tester.pumpWidget(_wrap(const DashboardScreen()));
    await tester.pumpAndSettle();

    // 标题与模块统计卡
    expect(find.text('总览'), findsWidgets);
    expect(find.textContaining('数据云系统汇总'), findsOneWidget);
    for (final label in ['需求', '蓝图', '契约', '管道', '执行', '传输']) {
      expect(find.text(label), findsWidgets);
    }
    // 需求 1 条、蓝图 2 条、契约 3 条、管道 2 条、执行 3 条、传输 6 个提供商
    expect(find.text('1'), findsOneWidget);
    expect(find.text('2'), findsNWidgets(2));
    expect(find.text('3'), findsNWidgets(2));
    expect(find.text('6'), findsOneWidget);

    // 最近执行记录（量潮科技数字化案例）
    expect(find.text('最近执行'), findsOneWidget);
    expect(find.textContaining('resolution-33w'), findsOneWidget);

    // 无服务端错误卡
    expect(find.textContaining('连接失败'), findsNothing);
    expect(tester.takeException(), isNull);
  });
}
