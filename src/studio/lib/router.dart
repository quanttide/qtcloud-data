import 'package:go_router/go_router.dart';
import 'package:flutter/material.dart';

import 'screens/home.dart';
import 'screens/dataset_list.dart';
import 'screens/dataset_detail.dart';
import 'widgets/side_menu.dart';

/// 页面路由
final GoRouter router = GoRouter(routes: [
  ShellRoute(
    builder: (BuildContext context, GoRouterState state, Widget child){
      return Scaffold(
        appBar: AppBar(title: const Text('量潮数据云')),
        body: SafeArea(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // We want this side menu only for large screen
              const Expanded(
                  // default flex = 1
                  // and it takes 1/6 part of the screen
                  child: SideMenu(),
              ),
              Expanded(
                // It takes 5/6 part of the screen
                flex: 5,
                child: child,
              ),
            ],
          ),
        ),
      );
    },
    routes: [
      // 首页
      GoRoute(
        name: 'home',
        path: '/',
        builder: (BuildContext context, GoRouterState state) {
          return const HomeScreen();
        },
      ),
      // 数据集列表
      GoRoute(
        name: 'dataset-list',
        path: '/datasets',
        builder: (BuildContext context, GoRouterState state) =>
        const DatasetListScreen(),
      ),
      // 数据集详情页面
      GoRoute(
        name: 'dataset-detail',
        path: '/datasets/universities',
        builder: (BuildContext context, GoRouterState state) =>
            DatasetDetailScreen(),
      )
    ]
  )
]);
