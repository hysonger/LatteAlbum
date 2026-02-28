# UI Specification: Truncate Small Aspect Ratio Thumbnails

## Overview

本规范详细描述了瀑布流中窄长图片的截断显示方案。

## Visual Design

### 截断阈值

- **宽高比阈值**: 0.5 (即 height > width * 2 时触发截断)
- **最大显示高度**: 图片宽度 × 1.5

### 截断视觉提示

#### 1. 渐变消失效果

```
┌─────────────────┐
│                 │
│    ████████     │
│    ████████     │
│    ████████     │
│    ████████     │
│  ▓▓▓▓▓▓▓▓▓▓    │  ← 底部渐变消失效果
└─────────────────┘
```

- **渐变方向**: 从上到下
- **渐变颜色**: 白色/透明 (rgba(255,255,255,0) → rgba(255,255,255,1))
- **渐变高度**: 图片底部 30% 区域

#### 2. 截断图标

```
┌─────────────────┐
│            🔗  │  ← 右上角截断图标
│                 │
│    ████████     │
│    ████████     │
│    ████████     │
│  ▓▓▓▓▓▓▓▓▓▓    │
└─────────────────┘
```

- **图标位置**: 右上角
- **图标样式**: 
  - 背景: 半透明黑色 (rgba(0,0,0,0.6))
  - 圆角: 4px
  - 图标: 链条断开图标或"截断"文字
  - 大小: 24px × 24px

### 交互行为

- **悬停效果**: 截断图标变为更醒目样式，提示可点击查看完整图片
- **点击行为**: 点击卡片打开图片查看器，显示完整图片

## Component States

### Normal (正常比例图片)

- 无特殊样式
- 完整显示图片

### Truncated (被截断图片)

- 容器高度受限
- 底部渐变效果
- 右上角截断图标

## Responsive Behavior

### Desktop (≥1024px)

- 列数: 4
- 截断图片最大高度: 列宽 × 1.5

### Tablet (768px - 1023px)

- 列数: 3
- 截断图片最大高度: 列宽 × 1.5

### Mobile (<768px)

- 列数: 2
- 截断图片最大高度: 列宽 × 1.8 (移动端稍微放宽限制)

## Technical Implementation

### CSS 实现方案

```css
/* 截断容器 */
.truncated {
  max-height: calc(var(--column-width) * 1.5);
  overflow: hidden;
  position: relative;
}

/* 渐变效果 */
.truncated::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 30%;
  background: linear-gradient(to bottom, transparent, rgba(255,255,255,0.9));
  pointer-events: none;
}

/* 截断图标 */
.truncate-indicator {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  background: rgba(0,0,0,0.6);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 12px;
  z-index: 1;
}
```

## Acceptance Criteria

1. [ ] 宽高比 < 0.5 的图片在瀑布流中高度受限
2. [ ] 底部渐变效果清晰可见
3. [ ] 右上角截断图标正确显示
4. [ ] 点击截断图片可查看完整原图
5. [ ] 移动端和桌面端显示正确
6. [ ] 正常比例图片不受影响
