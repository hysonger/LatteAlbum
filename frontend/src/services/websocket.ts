export interface ScanProgressMessage {
  scanning: boolean
  phase?: string          // 当前阶段: collecting, counting, processing, writing, deleting, completed, error, cancelled
  totalFiles: number
  successCount: number
  failureCount: number
  progressPercentage: string
  startTime?: string
  status: 'idle' | 'progress' | 'completed' | 'error' | 'cancelled'
  // 各阶段文件数量
  filesToAdd?: number
  filesToUpdate?: number
  filesToDelete?: number
}

// 阶段名称中文映射
export const PHASE_LABELS: Record<string, string> = {
  idle: '空闲',
  collecting: '收集中',
  counting: '检查中',
  processing: '处理中',
  writing: '保存中',
  deleting: '清理中',
  completed: '完成',
  error: '错误',
  cancelled: '已取消',
}

// 根据状态获取中文描述
export function getPhaseMessage(progress: Partial<ScanProgressMessage>): string {
  const phase = progress.phase?.toLowerCase()

  if (!phase || phase === 'idle') {
    return '就绪'
  }

  const status = progress.status

  if (status === 'error') {
    return '扫描出错'
  }

  if (status === 'cancelled') {
    return '扫描已取消'
  }

  if (status === 'completed') {
    return '扫描完成'
  }

  // 处理中显示进度信息
  const processed = (progress.successCount || 0) + (progress.failureCount || 0)
  const total = progress.totalFiles || 0

  if (total > 0) {
    const percentage = progress.progressPercentage || '0'
    return `${PHASE_LABELS[phase] || phase} (${processed}/${total}, ${percentage}%)`
  }

  return PHASE_LABELS[phase] || phase
}

type ProgressCallback = (progress: ScanProgressMessage) => void

class ScanProgressWebSocketService {
  private ws: WebSocket | null = null
  private isConnected = false
  private progressCallback: ProgressCallback | null = null
  private reconnectTimer: number | null = null

  /**
   * 获取 WebSocket URL
   */
  private getWebSocketUrl(): string {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const host = window.location.host
    return `${protocol}//${host}/ws/scan`
  }

  /**
   * 连接到 WebSocket 服务器
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.isConnected && this.ws?.readyState === WebSocket.OPEN) {
        resolve()
        return
      }

      const wsUrl = this.getWebSocketUrl()
      console.log('[WebSocket] 连接到:', wsUrl)

      try {
        this.ws = new WebSocket(wsUrl)

        this.ws.onopen = () => {
          console.log('[WebSocket] 连接成功')
          this.isConnected = true
          resolve()
        }

        this.ws.onmessage = (event) => {
          try {
            const progress: ScanProgressMessage = JSON.parse(event.data)
            console.log('[WebSocket] 收到进度更新:', progress)
            if (this.progressCallback) {
              this.progressCallback(progress)
            }
          } catch (e) {
            console.error('[WebSocket] 解析进度消息失败:', e)
          }
        }

        this.ws.onerror = (error) => {
          console.error('[WebSocket] 连接错误:', error)
          this.isConnected = false
          reject(new Error('WebSocket 连接失败'))
        }

        this.ws.onclose = (event) => {
          console.log('[WebSocket] 连接关闭:', event.code, event.reason)
          this.isConnected = false

          // 自动重连
          if (this.reconnectTimer === null) {
            this.reconnectTimer = window.setTimeout(() => {
              this.reconnectTimer = null
              console.log('[WebSocket] 尝试重连...')
              this.connect().catch(() => {})
            }, 5000)
          }
        }
      } catch (error) {
        reject(error)
      }
    })
  }

  /**
   * 断开连接
   */
  disconnect(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.ws) {
      this.ws.close()
      this.ws = null
      this.isConnected = false
      console.log('[WebSocket] 已断开连接')
    }
  }

  /**
   * 设置进度回调
   */
  onProgress(callback: ProgressCallback): void {
    this.progressCallback = callback
  }

  /**
   * 移除进度回调
   */
  offProgress(): void {
    this.progressCallback = null
  }

  /**
   * 检查是否已连接
   */
  isReady(): boolean {
    return this.isConnected && this.ws?.readyState === WebSocket.OPEN
  }
}

// 导出单例
export const scanProgressWs = new ScanProgressWebSocketService()
