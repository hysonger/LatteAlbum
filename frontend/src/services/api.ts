import axios from 'axios'
import type { MediaFile, Directory, PaginatedResponse, DateInfo } from '@/types'

const API_BASE = '/api'

// 创建axios实例
const apiClient = axios.create({
  baseURL: API_BASE,
  timeout: 30000
})

// 添加请求拦截器
apiClient.interceptors.request.use(
  (config) => {
    return config
  },
  (error) => {
    console.error('请求错误:', error)
    return Promise.reject(error)
  }
)

// 添加响应拦截器
apiClient.interceptors.response.use(
  (response) => {
    return response
  },
  (error) => {
    console.error('响应错误:', error.response?.status, error.response?.config?.url, error.message)
    return Promise.reject(error)
  }
)

// 文件API
export const fileApi = {
  // 获取文件列表
  getFiles: (params: {
    path?: string
    page?: number
    size?: number
    sortBy?: string
    order?: string
    filterType?: string
    cameraModel?: string
    date?: string
  }) => {
    return apiClient.get<PaginatedResponse<MediaFile>>('/files', { params })
  },

  // 获取文件详情
  getFileDetail: (id: string) => {
    return apiClient.get<MediaFile>(`/files/${id}`)
  },

  // 获取缩略图（支持full size用于大图显示）
  getThumbnail: (id: string, size: string = 'small') => {
    return apiClient.get(`/files/${id}/thumbnail`, {
      params: { size },
      responseType: 'blob'
    })
  },

  // 获取缩略图URL（直接返回URL，无需下载）
  getThumbnailUrl: (id: string, size: string = 'small'): string => {
    return `/api/files/${id}/thumbnail?size=${size}`
  },

  // 获取原始文件URL（用于视频流式播放，直接返回URL）
  getOriginalFileUrl: (id: string): string => {
    return `/api/files/${id}/original`
  },

  // 获取原始文件（下载到blob）
  getOriginalFile: (id: string) => {
    return apiClient.get(`/files/${id}/original`, {
      responseType: 'blob'
    })
  },

  // 更新文件信息
  updateFile: (id: string, data: Partial<MediaFile>) => {
    return apiClient.put<MediaFile>(`/files/${id}`, data)
  },

  // 获取包含照片的日期列表
  getDates: (params: {
    sortBy?: string
    filterType?: string
    cameraModel?: string
  }) => {
    return apiClient.get<DateInfo[]>('/files/dates', { params })
  }
}

// 目录API
export const directoryApi = {
  // 获取目录树
  getDirectoryTree: () => {
    return apiClient.get<Directory[]>('/directories')
  }
}

// 系统API
export const systemApi = {
  // 重新扫描
  rescan: () => {
    return apiClient.post('/system/rescan')
  },

  // 获取扫描进度
  getScanProgress: () => {
    return apiClient.get<{
      scanning: boolean
      phase?: string
      totalFiles: number
      successCount: number
      failureCount: number
      progressPercentage: string
      startTime?: string
      filesToAdd: number
      filesToUpdate: number
      filesToDelete: number
    }>('/system/scan/progress')
  },

  // 取消扫描
  cancelScan: () => {
    return apiClient.post('/system/scan/cancel')
  },

  // 获取系统状态
  getStatus: () => {
    return apiClient.get('/system/status')
  }
}