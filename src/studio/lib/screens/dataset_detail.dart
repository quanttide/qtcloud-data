/// 数据集详情页面


import 'package:flutter/material.dart';

import '../models/datasets.dart';



class DatasetDetailScreen extends StatefulWidget {
  Dataset dataset = Dataset(name: 'universities', verboseName: '高校数据集', description: '高校及其院系列表', count: 1000);

  // final Dataset dataset;

  // DatasetDetailScreen({required this.dataset});

  @override
  _DatasetDetailScreenState createState() => _DatasetDetailScreenState();
}

class _DatasetDetailScreenState extends State<DatasetDetailScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('高校数据集'),
      ),
      body: Container(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              widget.dataset.name,
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.bold,
              ),
            ),
            SizedBox(height: 8),
            Text(widget.dataset.description),
          ],
        ),
      ),
    );
  }
}
