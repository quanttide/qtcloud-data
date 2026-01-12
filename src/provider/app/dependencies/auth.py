"""
Authentication dependencies
"""
from fastapi import HTTPException, Request


async def authenticate(request: Request, call_next):
    """
    authenticate the request
    :param request:
    :param call_next:
    :return:
    """
    # 在请求之前执行的鉴权逻辑
    # 获取请求中的认证信息，如 token、session 等
    authentication = request.headers.get('Authorization')
    if not authentication:
        raise HTTPException(status_code=401, detail='Not authenticated')

    # 进行鉴权逻辑，验证用户身份

    response = await call_next(request)

    # 在响应返回之前执行的逻辑

    return response
