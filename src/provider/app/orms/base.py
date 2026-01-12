"""
Base tools for SQLAlchemy ORM
"""
from datetime import datetime
from typing import Annotated, Optional
from uuid import UUID, uuid4

from sqlalchemy import Text
from sqlalchemy.orm import mapped_column

Id = Annotated[
    UUID,
    mapped_column(primary_key=True, index=True, default=uuid4)
]
CreatedAt = Annotated[
    datetime,
    mapped_column(default=datetime.now)
]
UpdatedAt = Annotated[
    datetime,
    mapped_column(default=datetime.now, onupdate=datetime.now)
]
Name = Annotated[str, mapped_column(index=True)]
VerboseName = Optional[str]
Readme = Annotated[Optional[str], mapped_column(Text)]
