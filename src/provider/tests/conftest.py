"""
Testing fixtures
"""
import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from starlette.testclient import TestClient

from app.dependencies.db import get_session
from app.main import app

# 使用测试数据库连接字符串
TEST_DATABASE_URL = 'sqlite:///:memory:'

# 创建测试数据库引擎
testing_engine = create_engine(
    TEST_DATABASE_URL,
    connect_args={'check_same_thread': False},
    echo=True
)

# 创建一个会话类
TestingSession = sessionmaker(
    autocommit=False, autoflush=False,
    bind=testing_engine,
    info={'name': 'testing-session'},
)


def get_testing_session():
    """
    测试会话
    """
    session = TestingSession()
    try:
        yield session
    finally:
        session.close()


@pytest.fixture
def migrate_testing_db():
    """
    测试数据库
    """
    # set up
    from app.dependencies.db import BaseORM
    BaseORM.metadata.create_all(bind=testing_engine)
    yield
    # tear down
    BaseORM.metadata.drop_all(bind=testing_engine)


@pytest.fixture
def client():
    """
    测试客户端
    :return:
    """
    app.dependency_overrides[get_session] = get_testing_session
    return TestClient(app)
