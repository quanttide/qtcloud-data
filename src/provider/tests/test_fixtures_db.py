"""
Testing database fixtures
"""
# noqa: F401; pylint: disable=unused-variable
# noinspection PyUnresolvedReferences
from tests.conftest import (get_testing_session, testing_engine)


# pylint: disable=W0613
def test_create_tables(migrate_testing_db):
    """
    Test create tables
    :param migrate_testing_db:
    :return:
    """
    # Verify tables exist
    from sqlalchemy import inspect
    inspector = inspect(testing_engine)
    table_names = inspector.get_table_names()
    print(f'\n\nTables: {table_names}\n\n')
    # Add assertions or other logic based on the existence of tables
    assert 'dataset' in table_names


# pylint: disable=W0613
def test_get_testing_session(migrate_testing_db):
    """
    Test for getting TestingSession
    :param migrate_testing_db:
    :return:
    """
    session = next(get_testing_session())
    # 执行查询操作
    from sqlalchemy import text
    result = session.execute(text('SELECT * FROM dataset')).fetchall()
    # 在这里进行与查询结果相关的测试逻辑
    assert len(result) == 0
    next(get_testing_session())
