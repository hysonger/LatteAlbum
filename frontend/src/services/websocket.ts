export interface ScanProgressMessage {
  scanning: boolean
  phase?: string          // 当前阶段: collecting, counting, processing, deleting, completed
  phaseMessage?: string   // 阶段描述信息
  totalFiles: number
  successCount: number
  failureCount: number
  progressPercentage: string
  startTime?: string
  status: 'started' | 'progress' | 'completed' | 'error' | 'cancelled'
  message?: string
  // 各阶段文件数量
  filesToAdd?: number
  filesToUpdate?: number
  filesToDelete?: number
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
