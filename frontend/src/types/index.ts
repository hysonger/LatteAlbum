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
  content: T[]
  totalPages: number
  totalElements: number
  numberOfElements: number
  size: number
  number: number
  first: boolean
  last: boolean
}