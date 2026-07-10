import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../theme.dart';

class AppLayout extends StatelessWidget {
  final Widget child;
  const AppLayout({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const _Sidebar(),
            Expanded(flex: 5, child: child),
          ],
        ),
      ),
    );
  }
}

class _Sidebar extends StatelessWidget {
  const _Sidebar();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 200,
      color: secondaryColor,
      child: Column(
        children: [
          const SizedBox(height: 24),
          Text(
            '量潮数据云',
            style: Theme.of(
              context,
            ).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold),
          ),
          const Divider(height: 32),
          ..._items.map(
            (item) => _SidebarItem(
              icon: item.icon,
              label: item.label,
              path: item.path,
            ),
          ),
          const Spacer(),
          const Divider(),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Text(
              'v0.1.0-alpha',
              style: Theme.of(
                context,
              ).textTheme.bodySmall?.copyWith(color: Colors.grey),
            ),
          ),
        ],
      ),
    );
  }
}

class _NavItem {
  final IconData icon;
  final String label;
  final String path;
  const _NavItem(this.icon, this.label, this.path);
}

const _items = [
  _NavItem(Icons.dashboard, '总览', '/'),
  _NavItem(Icons.swap_horiz, '传输', '/transfer'),
  _NavItem(Icons.receipt_long, '执行记录', '/jobs'),
  _NavItem(Icons.account_tree, '管道', '/pipelines'),
  _NavItem(Icons.map, '蓝图', '/blueprints'),
  _NavItem(Icons.description, '契约', '/contracts'),
];

class _SidebarItem extends StatelessWidget {
  final IconData icon;
  final String label;
  final String path;
  const _SidebarItem({
    required this.icon,
    required this.label,
    required this.path,
  });

  @override
  Widget build(BuildContext context) {
    final active = GoRouterState.of(context).uri.toString() == path;
    return ListTile(
      leading: Icon(
        icon,
        size: 20,
        color: active ? Colors.blue : Colors.white54,
      ),
      title: Text(
        label,
        style: TextStyle(
          color: active ? Colors.blue : Colors.white54,
          fontSize: 14,
        ),
      ),
      dense: true,
      onTap: () => context.go(path),
    );
  }
}
