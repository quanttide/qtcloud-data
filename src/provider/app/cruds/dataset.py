"""
CURDs for DataSet
"""
from sqlalchemy.orm import Session

from app.orms import DataSetORM
from app.schemas import DataSet


def create_dataset(session: Session, dataset: DataSet) -> DataSetORM:
    """
    create dataset
    :param session:
    :param dataset:
    :return:
    """
    db_dataset = DataSetORM(**dataset.model_dump())
    # 临时
    session.add(db_dataset)
    session.commit()
    session.refresh(db_dataset)
    return db_dataset
