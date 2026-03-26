import pandas as pd
import numpy as np
from datetime import datetime
import re

def is_skipped_entry(x):
    """判断条目是否被跳过"""
    if pd.isna(x):
        return True
    if isinstance(x, str) and (x.strip() == '' or x.strip() == '跳过' or '跳过' in x):
        return True
    return False


def extract_seconds(time_str):
    """提取用时中的秒数"""
    if pd.isna(time_str) or not isinstance(time_str, str):
        return np.nan
    match = re.search(r'(\d+)秒', str(time_str))
    if match:
        return int(match.group(1))
    try:
        return float(time_str)
    except:
        return np.nan


def extract_other_content(text):
    """提取〖〗括号中的内容"""
    if not isinstance(text, str):
        return ''
    match = re.search(r'〖(.*?)〗', text)
    if match:
        return match.group(1)
    return ''


def clean_metadata(df):
    """清洗元数据"""
    result = df.copy()
    
    if '提交答卷时间' in result.columns:
        result['submit_time'] = pd.to_datetime(result['提交答卷时间'], errors='coerce')
    
    if '所用时间' in result.columns:
        result['time_spent_sec'] = result['所用时间'].apply(extract_seconds)
    
    if '来源' in result.columns:
        result['source'] = result['来源'].astype(str)
    
    if '来自IP' in result.columns:
        result['ip_address'] = result['来自IP'].astype(str)
    
    if '总分' in result.columns:
        result['total_score'] = pd.to_numeric(result['总分'], errors='coerce')
    
    return result


def clean_basic_info(df):
    """清洗基本信息"""
    result = df.copy()
    
    # 性别编码
    gender_mapping = {'男': 1, '女': 0}
    if '3. 您的性别：' in result.columns:
        result['gender'] = result['3. 您的性别：'].map(gender_mapping)
    
    # 教育程度编码
    education_mapping = {
        '小学': 1, '初中': 2, '高中': 3, '中专': 4,
        '大专': 5, '本科': 6, '研究生及以上': 7
    }
    if '5. 您的最高教育程度：' in result.columns:
        result['education'] = result['5. 您的最高教育程度：'].map(education_mapping)
    
    # 出生年份
    if '4. 您的出生年份' in result.columns:
        result['birth_year'] = pd.to_numeric(result['4. 您的出生年份'], errors='coerce')
    
    return result


def clean_work_experience(df):
    """清洗工作经历"""
    result = df.copy()
    
    # 是否第一份工作
    first_job_mapping = {'是': 1, '不是': 0}
    if '5. 这是您的第一份工作吗?' in result.columns:
        result['is_first_job'] = result['5. 这是您的第一份工作吗?'].map(first_job_mapping)
    
    # 之前行业工作年限
    if '6. 在进入本公司之前' in result.columns:
        result['prev_garment_years'] = pd.to_numeric(result['6. 在进入本公司之前'], errors='coerce')
    
    return result


def clean_satisfaction(df):
    """清洗满意度量表"""
    result = df.copy()
    
    # 满意度映射
    satisfaction_mapping = {
        '非常不同意': 1, '比较不同意': 2, '不确定': 3,
        '比较同意': 4, '非常同意': 5
    }
    
    reverse_mapping = {
        '非常不同意': 5, '比较不同意': 4, '不确定': 3,
        '比较同意': 2, '非常同意': 1
    }
    
    # 正向计分题
    positive_items = [
        ('总的来说，我对过去三个月分配给我的任务感到满意', 'satisfaction_task')
    ]
    
    for orig_col, new_col in positive_items:
        matching = [col for col in result.columns if orig_col in col]
        if matching:
            result[new_col] = result[matching[0]].map(satisfaction_mapping)
    
    # 反向计分题
    negative_items = [
        ('我的上级不体谅下属的感受', 'satisfaction_supervisor_inconsiderate'),
        ('在过去三个月里，我多次考虑辞职离开公司', 'satisfaction_consider_quitting')
    ]
    
    for orig_col, new_col in negative_items:
        matching = [col for col in result.columns if orig_col in col]
        if matching:
            result[new_col] = result[matching[0]].map(reverse_mapping)
    
    return result


def clean_communication(df):
    """清洗沟通渠道"""
    result = df.copy()
    
    # 是否曾反馈
    has_feedback_mapping = {'是': 1, '否': 0}
    if '2. 您是否曾通过上述渠道反馈建议或表达看法?' in result.columns:
        result['has_feedback'] = result['2. 您是否曾通过上述渠道反馈建议或表达看法?'].map(has_feedback_mapping)
    
    return result


def apply_skip_logic(df, skip_logic):
    """应用跳转逻辑，设置合理缺失值"""
    result = df.copy()
    
    for rule in skip_logic.get('skip_logic', []):
        trigger = rule['trigger']
        
        if trigger not in result.columns:
            continue
        
        skip_cols = rule.get('skip_questions', [])
        
        # 判断触发条件
        if 'trigger_value' in rule:
            condition = result[trigger] == rule['trigger_value']
        elif 'trigger_condition' in rule:
            if rule['trigger_condition'] == '<= 2':
                condition = result[trigger] <= 2
            else:
                continue
        else:
            continue
        
        # 设置跳转字段为缺失值
        for col in skip_cols:
            if col in result.columns:
                result.loc[condition, col] = np.nan
    
    return result


def handle_skip_logic(df, skip_logic):
    """处理跳转逻辑（别名）"""
    return apply_skip_logic(df, skip_logic)