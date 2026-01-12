"""
ORM for dataset
"""

from sqlalchemy.orm import Mapped

from app.dependencies.db import BaseORM
from app.orms.base import CreatedAt, Id, Name, Readme, UpdatedAt, VerboseName


class DataSetORM(BaseORM):
    """
    ORM for DataSet
    """
    # pylint: disable=R0903
    # Too few public methods
    __tablename__ = 'dataset'

    id: Mapped[Id]
    created_at: Mapped[CreatedAt]
    updated_at: Mapped[UpdatedAt]
    name: Mapped[Name]
    verbose_name: Mapped[VerboseName]
    readme: Mapped[Readme]
