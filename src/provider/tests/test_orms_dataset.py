"""
Test ORMs for DataSet subdomain
"""
from app.orms.dataset import DataSetORM


def test_init_dataset_orm():
    """
    Test creation of DataSetORM
    """
    dataset_orm = DataSetORM(name='test')
    assert dataset_orm.name == 'test'
