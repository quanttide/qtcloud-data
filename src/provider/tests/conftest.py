import pytest
import pandas as pd
import yaml
from pathlib import Path

FIXTURES_DIR = Path(__file__).parent / "fixtures"

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))


@pytest.fixture(scope="session")
def raw_data_path():
    """返回原始数据文件路径"""
    return FIXTURES_DIR / "raw" / "sample_questionnaire.xlsx"


@pytest.fixture(scope="session")
def raw_data(raw_data_path):
    """读取原始问卷数据"""
    if not raw_data_path.exists():
        pytest.skip(f"原始数据文件不存在: {raw_data_path}")
    return pd.read_excel(raw_data_path)


@pytest.fixture(scope="session")
def expected_data_path():
    """返回期望结果文件路径"""
    return FIXTURES_DIR / "expected" / "expected_cleaned.csv"


@pytest.fixture(scope="session")
def expected_data(expected_data_path):
    """读取期望的清洗结果"""
    if not expected_data_path.exists():
        pytest.skip(f"期望结果文件不存在: {expected_data_path}")
    return pd.read_csv(expected_data_path)


@pytest.fixture(scope="session")
def codebook():
    """读取数据契约"""
    codebook_path = FIXTURES_DIR / "codebook" / "codebook.yaml"
    if not codebook_path.exists():
        pytest.skip(f"codebook文件不存在: {codebook_path}")
    with open(codebook_path, 'r', encoding='utf-8') as f:
        return yaml.safe_load(f)


@pytest.fixture(scope="session")
def skip_logic():
    """读取跳转逻辑定义"""
    skip_path = FIXTURES_DIR / "codebook" / "skip_logic.yaml"
    if not skip_path.exists():
        pytest.skip(f"跳转逻辑文件不存在: {skip_path}")
    with open(skip_path, 'r', encoding='utf-8') as f:
        return yaml.safe_load(f)


@pytest.fixture(scope="session")
def cleaning_functions():
    """导入清洗函数模块"""
    try:
        from cleaning_functions import (
            clean_metadata,
            clean_basic_info,
            clean_work_experience,
            clean_satisfaction,
            clean_communication,
            handle_skip_logic,
            apply_skip_logic
        )
        return {
            'metadata': clean_metadata,
            'basic_info': clean_basic_info,
            'work_experience': clean_work_experience,
            'satisfaction': clean_satisfaction,
            'communication': clean_communication,
            'skip_logic': handle_skip_logic,
            'apply_skip_logic': apply_skip_logic
        }
    except ImportError as e:
        pytest.skip(f"无法导入清洗函数: {e}")


@pytest.fixture(scope="session")
def cleaned_data(raw_data, cleaning_functions, codebook, skip_logic):
    """执行完整清洗流程，返回清洗后的数据"""
    df = raw_data.copy()
    
    # 执行各步骤清洗
    if 'metadata' in cleaning_functions:
        df = cleaning_functions['metadata'](df)
    
    if 'basic_info' in cleaning_functions:
        df = cleaning_functions['basic_info'](df)
    
    if 'work_experience' in cleaning_functions:
        df = cleaning_functions['work_experience'](df)
    
    if 'satisfaction' in cleaning_functions:
        df = cleaning_functions['satisfaction'](df)
    
    if 'communication' in cleaning_functions:
        df = cleaning_functions['communication'](df)
    
    # 应用跳转逻辑
    if 'apply_skip_logic' in cleaning_functions:
        df = cleaning_functions['apply_skip_logic'](df, skip_logic)
    
    return df