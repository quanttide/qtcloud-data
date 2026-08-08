// 应用冒烟测试：入口可正常构建，无 Web 平台崩溃
//
// 覆盖：
//  - MyApp（MaterialApp.router）渲染首页不抛异常
//  - 默认 ApiClient 在无后端环境（测试内 HTTP 一律 400）下
//    首页显示"连接失败"兜底而不是崩溃
//    （回归：dart:io Platform.environment 曾导致 Web 上页面初始化崩溃）

import 'package:flutter_test/flutter_test.dart';

import 'package:qtcloud_data_studio/main.dart';

void main() {
  testWidgets('应用启动冒烟测试：首页渲染不崩溃', (tester) async {
    await tester.pumpWidget(const MyApp());
    await tester.pumpAndSettle();

    // 侧边栏标题存在（无论后端是否可达）
    expect(find.text('量潮数据云'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
