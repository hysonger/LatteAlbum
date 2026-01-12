import SockJS from 'sockjs-client'
import { Client, IMessage } from '@stomp/stompjs'

export interface ScanProgressMessage {
  scanning: boolean
  totalFiles: number
  successCount: number
  failureCount: number
  progressPercentage: string
  startTime?: string
  status: 'started' | 'progress' | 'completed' | 'error' | 'cancelled'
  message?: string
}

class ScanProgressWebSocketService {
  private client: Client | null = null
  private isConnected = false
  private progressCallback: ((progress: ScanProgressMessage) => void) | null = null
  private connectionPromise: Promise<void> | null = null

  /**
   * 获取 WebSocket URL
   */
  private getWebSocketUrl(): string {
    // 始终使用相对路径，避免协议问题
    return `/ws/scan`
  }

  /**
   * 连接到 WebSocket 服务器
   */
  connect(): Promise<void> {
    if (this.isConnected && this.client?.connected) {
      return Promise.resolve()
    }

    if (this.connectionPromise) {
      return this.connectionPromise
    }

    this.connectionPromise = new Promise((resolve, reject) => {
      const wsUrl = this.getWebSocketUrl()
      console.log('[WebSocket] 连接到:', wsUrl)

      this.client = new Client({
        webSocketFactory: () => new SockJS(wsUrl),
        reconnectDelay: 5000,
        heartbeatIncoming: 4000,
        heartbeatOutgoing: 4000,
        onConnect: () => {
          console.log('[WebSocket] 连接成功')
          this.isConnected = true
          this.connectionPromise = null

          // 订阅扫描进度主题
          this.client?.subscribe('/topic/scan/progress', (message: IMessage) => {
            try {
              const progress: ScanProgressMessage = JSON.parse(message.body)
              console.log('[WebSocket] 收到进度更新:', progress)
              if (this.progressCallback) {
                this.progressCallback(progress)
              }
            } catch (e) {
              console.error('[WebSocket] 解析进度消息失败:', e)
            }
          })

          resolve()
        },
        onStompError: (frame) => {
          console.error('[WebSocket] STOMP 错误:', frame.headers['message'])
          this.isConnected = false
          this.connectionPromise = null
          reject(new Error(frame.headers['message']))
        },
        onDisconnect: () => {
          console.log('[WebSocket] 连接断开')
          this.isConnected = false
        },
        onWebSocketClose: (frame) => {
          console.log('[WebSocket] 连接关闭:', frame.code, frame.reason)
          this.isConnected = false
          // 尝试重连
          if (this.connectionPromise === null) {
            setTimeout(() => {
              if (!this.isConnected) {
                this.connect().catch(() => {})
              }
            }, 3000)
          }
        }
      })

      this.client.activate()
    })

    return this.connectionPromise
  }

  /**
   * 断开连接
   */
  disconnect(): void {
    if (this.client) {
      this.client.deactivate()
      this.client = null
      this.isConnected = false
      this.connectionPromise = null
      console.log('[WebSocket] 已断开连接')
    }
  }

  /**
   * 设置进度回调
   */
  onProgress(callback: (progress: ScanProgressMessage) => void): void {
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
    return this.isConnected && this.client?.connected === true
  }
}

// 导出单例
export const scanProgressWs = new ScanProgressWebSocketService()
