// 总览页组件测试
//
// 覆盖：
//  - Provider 加载成功：版本号与提供商卡片展示
//  - Provider 不可达：显示错误卡而不崩溃（Web 上无后端时的兜底行为）

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:qtcloud_data_studio/api/client.dart';
import 'package:qtcloud_data_studio/screens/dashboard.dart';
import 'package:qtcloud_data_studio/theme.dart';

Widget _wrap(Widget child) =>
    MaterialApp(theme: defaultThemeData, home: Scaffold(body: child));

void main() {
  testWidgets('总览页：Provider 加载成功时展示版本号与提供商', (tester) async {
    final mock = MockClient((req) async {
      if (req.url.path == '/version') {
        return http.Response(jsonEncode({'version': 'v0.1.0'}), 200);
      }
      if (req.url.path == '/providers') {
        return http.Response(jsonEncode(['dropbox', 's3']), 200);
      }
      return http.Response('not found', 404);
    });
    final client = ApiClient(baseUrl: 'http://test', client: mock);

    await tester.pumpWidget(_wrap(DashboardScreen(client: client)));
    await tester.pumpAndSettle();

    expect(find.text('量潮数据云'), findsOneWidget);
    expect(find.text('Provider: v0.1.0'), findsOneWidget);
    expect(find.text('dropbox'), findsOneWidget);
    expect(find.text('s3'), findsOneWidget);
    expect(find.text('快速操作'), findsOneWidget);
    expect(find.textContaining('连接失败'), findsNothing);
  });

  testWidgets('总览页：Provider 不可达时显示错误卡且不崩溃', (tester) async {
    final mock = MockClient((req) async => http.Response('service down', 500));
    final client = ApiClient(baseUrl: 'http://test', client: mock);

    await tester.pumpWidget(_wrap(DashboardScreen(client: client)));
    await tester.pumpAndSettle();

    expect(find.text('量潮数据云'), findsOneWidget);
    expect(find.textContaining('连接失败'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
