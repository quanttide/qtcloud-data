import 'package:flutter/material.dart';
import 'package:flutter_web_plugins/url_strategy.dart';
import 'theme.dart';
import 'router.dart';

void main() async {
  usePathUrlStrategy();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: '量潮数据云',
      theme: defaultThemeData,
      routerConfig: router,
      debugShowCheckedModeBanner: false,
    );
  }
}
