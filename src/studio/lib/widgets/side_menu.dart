import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:go_router/go_router.dart';


class SideMenu extends StatelessWidget {
  const SideMenu({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Drawer(
      child: ListView(
        children: [
          DrawerListTile(
            title: "数据集",
            svgSrc: "assets/icons/menu_dashboard.svg",
            press: () {
              context.goNamed('dataset-list');
            },
          ),
          DrawerListTile(
            title: "数据应用",
            svgSrc: "assets/icons/menu_tran.svg",
            press: () {
              context.goNamed('dataset-list');
            },
          ),
          DrawerListTile(
            title: "数据处理器",
            svgSrc: "assets/icons/menu_task.svg",
            press: () {
              context.goNamed('dataset-list');
            },
          ),
          DrawerListTile(
            title: "数据工作流",
            svgSrc: "assets/icons/menu_doc.svg",
            press: () {
              context.goNamed('dataset-list');
            },
          ),
          DrawerListTile(
            title: "数据共享",
            svgSrc: "assets/icons/menu_store.svg",
            press: () {
              context.goNamed('dataset-list');
            },
          ),
          const Divider(),
          DrawerListTile(
            title: "通知",
            svgSrc: "assets/icons/menu_notification.svg",
            press: () {
              context.goNamed('dataset-list');
            },
          ),
          DrawerListTile(
            title: "主页",
            svgSrc: "assets/icons/menu_profile.svg",
            press: () {
              context.goNamed('dataset-list');
            },
          ),
          DrawerListTile(
            title: "设置",
            svgSrc: "assets/icons/menu_setting.svg",
            press: () {
              context.goNamed('dataset-list');
            },
          ),
        ],
      ),
    );
  }
}

class DrawerListTile extends StatelessWidget {
  const DrawerListTile({
    Key? key,
    // For selecting those three line once press "Command+D"
    required this.title,
    required this.svgSrc,
    required this.press,
  }) : super(key: key);

  final String title, svgSrc;
  final VoidCallback press;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      onTap: press,
      horizontalTitleGap: 0.0,
      leading: SvgPicture.asset(
        svgSrc,
        colorFilter: const ColorFilter.mode(Colors.white54, BlendMode.srcIn),
        height: 16,
      ),
      title: Text(
        title,
        style: const TextStyle(color: Colors.white54),
      ),
    );
  }
}
