import 'package:go_router/go_router.dart';
import 'screens/dashboard.dart';
import 'screens/transfer.dart';
import 'screens/jobs.dart';
import 'screens/pipelines.dart';
import 'screens/blueprints.dart';
import 'screens/contracts.dart';
import 'widgets/layout.dart';

final GoRouter router = GoRouter(
  routes: [
    ShellRoute(
      builder: (context, state, child) => AppLayout(child: child),
      routes: [
        GoRoute(path: '/', builder: (_, __) => const DashboardScreen()),
        GoRoute(path: '/transfer', builder: (_, __) => const TransferScreen()),
        GoRoute(path: '/jobs', builder: (_, __) => const JobsScreen()),
        GoRoute(
          path: '/pipelines',
          builder: (_, __) => const PipelinesScreen(),
        ),
        GoRoute(
          path: '/blueprints',
          builder: (_, __) => const BlueprintsScreen(),
        ),
        GoRoute(
          path: '/contracts',
          builder: (_, __) => const ContractsScreen(),
        ),
      ],
    ),
  ],
);
