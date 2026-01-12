"""
Test CRUDs for dataset
"""
from app.cruds.dataset import create_dataset
from app.schemas import DataSet
from tests.conftest import TestingSession


def test_create_dataset(migrate_testing_db):
    """
    Test create dataset
    """
    session = TestingSession()
    dataset = DataSet(name='test', verbose_name='测试数据集', readme='这是一个测试数据集')
    db_dataset = create_dataset(session, dataset)
    assert db_dataset.id is not None
    assert db_dataset.created_at is not None
    assert db_dataset.updated_at is not None
    assert db_dataset.name == 'test'
    assert db_dataset.verbose_name == '测试数据集'
    assert db_dataset.readme == '这是一个测试数据集'
    session.close()
