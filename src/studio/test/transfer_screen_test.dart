// 传输页组件测试
//
// 覆盖：
//  - 发送：生成模拟分享链接（不依赖服务端）
//  - 接收模式切换：字段随模式切换（分享链接 / 本地保存路径）
//  - 提供商下拉可选

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:qtcloud_data_studio/screens/transfer.dart';
import 'package:qtcloud_data_studio/theme.dart';

Widget _wrap(Widget child) =>
    MaterialApp(theme: defaultThemeData, home: Scaffold(body: child));

void main() {
  testWidgets('传输页：发送后生成模拟分享链接', (tester) async {
    await tester.pumpWidget(_wrap(const TransferScreen()));

    await tester.enterText(
        find.widgetWithText(TextField, '本地路径'), '/tmp/a.csv');
    await tester.enterText(find.widgetWithText(TextField, '远程路径'), 'x/a.csv');
    await tester.tap(find.widgetWithText(ElevatedButton, '发送'));
    await tester.pumpAndSettle();

    expect(find.textContaining('模拟上传完成'), findsOneWidget);
    expect(
      find.textContaining('https://www.dropbox.com/s/'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('传输页：接收模式展示分享链接与保存路径字段', (tester) async {
    await tester.pumpWidget(_wrap(const TransferScreen()));

    // 初始为发送模式：无分享链接字段
    expect(find.widgetWithText(TextField, '分享链接'), findsNothing);

    // 切到接收模式
    await tester.tap(find.text('接收').first);
    await tester.pumpAndSettle();

    expect(find.widgetWithText(TextField, '分享链接'), findsOneWidget);
    expect(find.widgetWithText(TextField, '本地保存路径'), findsOneWidget);
    expect(find.widgetWithText(TextField, '远程路径'), findsNothing);
  });

  testWidgets('传输页：接收后展示模拟结果', (tester) async {
    await tester.pumpWidget(_wrap(const TransferScreen()));

    await tester.tap(find.text('接收').first);
    await tester.pumpAndSettle();
    await tester.enterText(find.widgetWithText(TextField, '分享链接'),
        'https://dropbox.com/s/abc/x.csv');
    await tester.tap(find.widgetWithText(ElevatedButton, '接收'));
    await tester.pumpAndSettle();

    expect(find.textContaining('模拟接收完成'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
