export interface MediaFile {
  id: string
  fileName: string
  fileType: 'image' | 'video'
  mimeType?: string
  fileSize?: number
  width?: number
  height?: number
  exifTimestamp?: string
  exifTimezoneOffset?: string
  createTime?: string
  modifyTime?: string
  lastScanned?: string
  cameraMake?: string
  cameraModel?: string
  lensModel?: string
  exposureTime?: string
  aperture?: string
  iso?: number
  focalLength?: string
  duration?: number
  videoCodec?: string
}

export interface DateInfo {
  date: string
  count: number
}

export interface Directory {
  id: number
  path: string
  parentId?: number
  fileCount: number
  lastModified: string
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  size: number
  totalPages: number
}

// GPS 坐标（敏感信息）。通过专用端点 /api/files/{id}/gps 按需获取，
// MediaFile 列表/详情默认不带 GPS。
export interface GpsInfo {
  hasGps: boolean
  latitude?: number
  longitude?: number
}