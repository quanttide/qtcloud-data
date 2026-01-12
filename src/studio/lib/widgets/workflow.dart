/// 工作流组件

import 'package:flutter/material.dart';
import 'package:graphview/graphview.dart';


class Workflow extends StatelessWidget {
  const Workflow({super.key});

  Graph buildGraph() {
    final Graph graph = Graph();

    // 创建节点
    final Node node1 = Node.Id('1');
    final Node node2 = Node.Id('2');
    final Node node3 = Node.Id('3');
    final Node node4 = Node.Id('4');

    // 添加节点到图中
    graph.addNode(node1);
    graph.addNode(node2);
    graph.addNode(node3);
    graph.addNode(node4);

    // 添加边到图中
    graph.addEdge(node1, node2);
    graph.addEdge(node2, node3);
    graph.addEdge(node3, node4);

    return graph;
  }

  String getNodeText(String id){
    Map map = {
      '1': '高校名单',
      '2': '高校数据集',
      '3': '高校教师数据集',
      '4': '高校排名网站',
    };
    return map[id];
  }

  @override
  Widget build(BuildContext context) {
    SugiyamaConfiguration config = SugiyamaConfiguration();
    config.orientation = SugiyamaConfiguration.ORIENTATION_LEFT_RIGHT;
    return InteractiveViewer(
      constrained: false,
      child: GraphView(
        graph: buildGraph(),
        algorithm: SugiyamaAlgorithm(config),
        builder: (Node node) {
          return Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(4),
                boxShadow: const [
                  BoxShadow(color: Colors.blue, spreadRadius: 1),
                ],
              ),
              child: Text(getNodeText(node.key?.value))
          );
        },
      ),
    );
  }
}