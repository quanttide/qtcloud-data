"""
Entry module
"""

from fastapi import FastAPI

from app.dependencies.db import BaseORM, engine
from app.routers import dataset, record, schema

BaseORM.metadata.create_all(bind=engine)

app = FastAPI()
app.include_router(dataset.router)
app.include_router(schema.router)
app.include_router(record.router)


@app.get('/')
async def root():
    """
    Root endpoint.
    """
    return {'msg': 'Hello World!'}
