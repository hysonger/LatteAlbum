export interface MediaFile {
  id: string
  fileName: string
  fileType: 'image' | 'video'
  mimeType: string
  fileSize: number
  width?: number
  height?: number
  exifTimestamp?: string
  exifTimezoneOffset?: string
  createTime: string
  modifyTime: string
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