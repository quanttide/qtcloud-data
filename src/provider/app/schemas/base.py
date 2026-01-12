"""
BaseORM classes for pydantic schemas.
"""
from datetime import datetime
from typing import Annotated, Optional
from uuid import UUID

from pydantic import BaseModel as PydanticBaseModel
from pydantic import ConfigDict, Field


class BaseModel(PydanticBaseModel):
    """
    BaseModel with default fields.
    """
    # Fields
    id: Optional[Annotated[UUID, Field(frozen=True)]] = None
    created_at: Optional[Annotated[datetime, Field(frozen=True)]] = None
    updated_at: Optional[datetime] = None
    # created_by: Optional[Annotated[UUID, Field(frozen=True)]]
    # updated_by: Optional[UUID]

    # Config
    model_config = ConfigDict(from_attributes=True)
