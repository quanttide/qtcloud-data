"""
Unittests for domain models of DataSetORM subdomain
"""
from app.schemas import DataSet


def test_init_dataset():
    """
    Test init DataSet domain model
    """
    dataset = DataSet(name='test', verbose_name='测试数据集', readme='这是一个测试数据集')
    assert dataset.name == 'test'
    assert dataset.verbose_name == '测试数据集'
    assert dataset.readme == '这是一个测试数据集'
