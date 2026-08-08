// 传输页组件测试
//
// 覆盖：
//  - 发送模式：提交后展示返回的分享链接
//  - 发送失败：展示错误信息且不崩溃
//  - 接收模式切换：字段随模式切换（分享链接 / 本地保存路径）

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:qtcloud_data_studio/api/client.dart';
import 'package:qtcloud_data_studio/screens/transfer.dart';
import 'package:qtcloud_data_studio/theme.dart';

Widget _wrap(Widget child) =>
    MaterialApp(theme: defaultThemeData, home: Scaffold(body: child));

void main() {
  testWidgets('传输页：发送成功后展示分享链接', (tester) async {
    final mock = MockClient((req) async {
      if (req.url.path == '/transfer/send') {
        return http.Response(
          jsonEncode({'url': 'https://dropbox.com/s/abc/result.csv?dl=1'}),
          200,
        );
      }
      return http.Response('not found', 404);
    });
    final client = ApiClient(baseUrl: 'http://test', client: mock);

    await tester.pumpWidget(_wrap(TransferScreen(client: client)));

    await tester.enterText(
        find.widgetWithText(TextField, '本地路径'), '/tmp/a.csv');
    await tester.enterText(find.widgetWithText(TextField, '远程路径'), 'x/a.csv');
    await tester.tap(find.widgetWithText(ElevatedButton, '发送'));
    await tester.pumpAndSettle();

    expect(
      find.textContaining('https://dropbox.com/s/abc/result.csv'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('传输页：发送失败时展示错误且不崩溃', (tester) async {
    final mock = MockClient((req) async => http.Response('boom', 500));
    final client = ApiClient(baseUrl: 'http://test', client: mock);

    await tester.pumpWidget(_wrap(TransferScreen(client: client)));

    await tester.enterText(
        find.widgetWithText(TextField, '本地路径'), '/tmp/a.csv');
    await tester.enterText(find.widgetWithText(TextField, '远程路径'), 'x/a.csv');
    await tester.tap(find.widgetWithText(ElevatedButton, '发送'));
    await tester.pumpAndSettle();

    expect(find.textContaining('错误:'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('传输页：切换到接收模式后展示分享链接与保存路径字段', (tester) async {
    final client = ApiClient(
      baseUrl: 'http://test',
      client: MockClient((req) async => http.Response('{}', 200)),
    );

    await tester.pumpWidget(_wrap(TransferScreen(client: client)));

    // 初始为发送模式：无分享链接字段
    expect(find.widgetWithText(TextField, '分享链接'), findsNothing);

    // 切到接收模式
    await tester.tap(find.text('接收').first);
    await tester.pumpAndSettle();

    expect(find.widgetWithText(TextField, '分享链接'), findsOneWidget);
    expect(find.widgetWithText(TextField, '本地保存路径'), findsOneWidget);
    expect(find.widgetWithText(TextField, '远程路径'), findsNothing);
  });
}
