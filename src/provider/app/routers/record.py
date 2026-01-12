"""
DataRecord Endpoints
"""
from fastapi import APIRouter

from app.schemas import DataRecord

router = APIRouter(prefix='/schemas')
# In-memory database (for demonstration purposes)
data_records: list[DataRecord] = []


@router.get('/')
async def list_data_records():
    """
    list all records
    :return:
    """
    return data_records
