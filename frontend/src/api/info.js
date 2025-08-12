import request from '@/utils/request'

export const checkMyInfo = () => {
    return request({
        url:'usr/bio',
        method: 'get',
        headers: {
        'Authorization': `Bearer ${localStorage.getItem('token')}`, // 从本地获取 token
        }
    })
}

export const checkOthersInfo = (id) => {
    return request({
        url:`usr/${id}`,
        method: 'get',
    })
}