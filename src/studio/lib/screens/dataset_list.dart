import 'package:flutter/material.dart';

import '../models/datasets.dart';
import '../theme.dart';

/// 数据集页面
class DatasetListScreen extends StatefulWidget {
  const DatasetListScreen({super.key});

  @override
  _DatasetListScreenState createState() => _DatasetListScreenState();
}


class _DatasetListScreenState extends State<DatasetListScreen> {
  @override
  Widget build(BuildContext context) {
    List<Dataset> datasetList = [
      Dataset(name: 'universities', description: '高校及其院系列表', verboseName: '高校数据集', count: 1000),
      Dataset(name: 'weibo', description: '上市公司微博简介', verboseName: '微博数据集', count: 20110),
      Dataset(name: 'university-fellows', description: '高校教师简历', count: 200000, verboseName: "高校教师数据集"),
    ];
    return Container(
      padding: const EdgeInsets.all(defaultPadding),
      decoration: const BoxDecoration(
        color: secondaryColor,
        borderRadius: BorderRadius.all(Radius.circular(10)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: double.infinity,
            child: DataTable(
              columnSpacing: defaultPadding,
              // minWidth: 600,
              columns: const [
                DataColumn(
                  label: Text("标识"),
                ),
                DataColumn(
                  label: Text("名称"),
                ),
                DataColumn(
                  label: Text("描述"),
                ),
                DataColumn(
                  label: Text("数量"),
                ),
              ],
              rows: List.generate(
                datasetList.length,
                    (index) => DataRow(
                      cells: [
                        DataCell(Text(datasetList[index].name)),
                        DataCell(Text(datasetList[index].verboseName)),
                        DataCell(Text(datasetList[index].description)),
                        DataCell(Text('${datasetList[index].count}条')),
                      ],
                    ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
