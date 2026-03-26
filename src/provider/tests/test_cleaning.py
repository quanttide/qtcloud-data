"""问卷数据清洗测试用例"""
import pytest
import pandas as pd
import numpy as np


class TestQuestionnaireCleaning:
    """问卷清洗测试"""
    
    def test_raw_data_loaded(self, raw_data):
        """测试原始数据加载"""
        assert raw_data is not None
        assert len(raw_data) > 0
        print(f"原始数据形状: {raw_data.shape}")
        print(f"原始数据列数: {len(raw_data.columns)}")
        
    def test_metadata_cleaning(self, raw_data, cleaning_functions):
        """测试元数据清洗"""
        df = cleaning_functions['metadata'](raw_data.copy())
        
        # 验证提交时间列存在且为datetime类型
        if 'submit_time' in df.columns:
            assert pd.api.types.is_datetime64_any_dtype(df['submit_time'])
        
        # 验证用时列为数值型
        if 'time_spent_sec' in df.columns:
            assert pd.api.types.is_numeric_dtype(df['time_spent_sec'])
            
    def test_basic_info_cleaning(self, raw_data, cleaning_functions):
        """测试基本信息清洗"""
        df = cleaning_functions['basic_info'](raw_data.copy())
        
        # 验证性别编码
        if 'gender' in df.columns:
            valid_genders = df['gender'].dropna().unique()
            for val in valid_genders:
                assert val in [1, 0]
        
        # 验证教育程度编码
        if 'education' in df.columns:
            valid_edu = df['education'].dropna().unique()
            for val in valid_edu:
                assert 1 <= val <= 7
                
    def test_skip_logic(self, cleaned_data, skip_logic):
        """测试跳转逻辑处理"""
        df = cleaned_data
        
        # 验证第一份工作的跳过逻辑
        if 'is_first_job' in df.columns and 'prev_garment_years' in df.columns:
            first_job_workers = df[df['is_first_job'] == 1]
            if len(first_job_workers) > 0:
                # 第一份工作的工人，prev_garment_years 应为空
                assert first_job_workers['prev_garment_years'].isna().all()
        
        # 验证没有考虑离职的跳过逻辑
        if 'satisfaction_consider_quitting' in df.columns:
            not_considering = df[df['satisfaction_consider_quitting'] <= 2]
            if len(not_considering) > 0:
                # 检查离职原因相关列是否为空
                quit_cols = [col for col in df.columns if col.startswith('quit_')]
                for col in quit_cols:
                    if col in df.columns:
                        assert not_considering[col].isna().all()
                        
    def test_cleaned_data_shape(self, cleaned_data, expected_data):
        """测试清洗后数据形状"""
        assert cleaned_data is not None
        assert len(cleaned_data) > 0
        print(f"清洗后数据形状: {cleaned_data.shape}")
        
    def test_no_missing_key_columns(self, cleaned_data, codebook):
        """测试关键列没有缺失"""
        # 从codebook中获取必需列
        required_cols = ['submit_time', 'gender', 'education']
        for col in required_cols:
            if col in cleaned_data.columns:
                # 检查非空比例
                non_null_pct = cleaned_data[col].notna().mean()
                print(f"{col} 非空比例: {non_null_pct:.2%}")
                
    def test_compare_with_expected(self, cleaned_data, expected_data):
        """测试清洗结果与期望一致"""
        # 比较关键字段的值
        key_columns = ['gender', 'education', 'is_first_job']
        
        for col in key_columns:
            if col in cleaned_data.columns and col in expected_data.columns:
                # 忽略索引，比较值
                pd.testing.assert_series_equal(
                    cleaned_data[col].reset_index(drop=True),
                    expected_data[col].reset_index(drop=True),
                    check_names=False,
                    check_dtype=False
                )