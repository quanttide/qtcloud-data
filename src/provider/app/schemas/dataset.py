"""
Domain models for DataSetORM subdomain
"""


from .base import BaseModel


class DataSet(BaseModel):
    """
    DataSetORM domain model
    """

    # pylint: disable=R0903
    # Too few public methods
    name: str
    verbose_name: str
    readme: str


class DataSchema(BaseModel):
    """
    DataSchema domain model
    """

    # pylint: disable=R0903
    # Too few public methods

    name: str
    verbose_name: str
    readme: str

    # pylint: disable=R0903  # Too few public methods


class DataRecord(BaseModel):
    """
    DataRecord domain model
    """

    # pylint: disable=R0903  # Too few public methods
