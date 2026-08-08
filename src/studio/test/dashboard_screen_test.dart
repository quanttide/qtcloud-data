// 总览页组件测试
//
// 覆盖：
//  - seed 数据加载成功：控制台版本与提供商卡片展示
//  - 无服务端依赖：不展示任何"连接失败"类错误卡

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:qtcloud_data_studio/screens/dashboard.dart';
import 'package:qtcloud_data_studio/theme.dart';

Widget _wrap(Widget child) =>
    MaterialApp(theme: defaultThemeData, home: Scaffold(body: child));

void main() {
  testWidgets('总览页：展示控制台版本与提供商列表（seed 数据，无服务端依赖）', (tester) async {
    await tester.pumpWidget(_wrap(const DashboardScreen()));
    await tester.pumpAndSettle();

    expect(find.text('量潮数据云'), findsOneWidget);
    expect(find.text('控制台 v0.1.0-alpha'), findsOneWidget);
    expect(find.text('dropbox'), findsOneWidget);
    expect(find.text('s3'), findsOneWidget);
    expect(find.text('快速操作'), findsOneWidget);
    // 无服务端错误卡
    expect(find.textContaining('连接失败'), findsNothing);
    expect(tester.takeException(), isNull);
  });
}
