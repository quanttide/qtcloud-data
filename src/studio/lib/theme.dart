import 'package:flutter/material.dart';

/// 颜色系统
const primaryColor = Color(0xFF2697FF);
const secondaryColor = Color(0xFF2A2D3E);
const bgColor = Color(0xFF212332);

/// 间距
const defaultPadding = 16.0;

/// 默认主题
final ThemeData defaultThemeData = ThemeData.dark().copyWith(
  scaffoldBackgroundColor: bgColor,
  canvasColor: secondaryColor,
);