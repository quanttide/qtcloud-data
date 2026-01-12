"""
DataSchema endpoints
"""
from fastapi import APIRouter

from app.schemas import DataSchema

router = APIRouter(prefix='/schemas')
# In-memory database (for demonstration purposes)
data_schemas: list[DataSchema] = []


@router.get('/')
async def list_data_schemas():
    """
    list all DataSchema
    :return:
    """
    return data_schemas
