# Technical Design: Truncate Small Aspect Ratio Thumbnails

## Overview

本文档描述了实现窄长图片截断功能的技术方案。

## Architecture

### Components Affected

```
Gallery.vue (父组件)
    │
    ├── 传递 columnWidth 给 MediaCard
    │
    └── MediaCard.vue (子组件)
            │
            ├── 计算宽高比
            ├── 决定是否截断
            ├── 应用截断样式
            └── 显示截断图标
```

## Implementation Details

### 1. MediaCard 组件修改

#### Props 扩展

```typescript
// 新增 props
interface Props {
  // ... 现有 props
  columnWidth?: number  // 列宽度，用于计算截断高度
}
```

#### 宽高比计算

```typescript
const aspectRatio = computed(() => {
  if (!props.item.width || !props.item.height) return 1
  return props.item.width / props.item.height
})

const isTruncated = computed(() => {
  return aspectRatio.value < 0.5
})

const maxHeight = computed(() => {
  if (!props.columnWidth) return 'none'
  return `${props.columnWidth * 1.5}px`
})
```

### 2. Gallery 组件修改

#### 传递列宽

```typescript
// MediaCard 调用时传递 columnWidth
<MediaCard
  :item="item"
  :thumbnail-size="thumbnailSize"
  :column-width="columnWidth"
  @click="handleClick(item)"
/>
```

### 3. 样式实现

#### 截断容器样式

```scss
.thumbnail-container {
  position: relative;
  overflow: hidden;
  
  &.truncated {
    max-height: var(--max-height, none);
    
    .thumbnail {
      object-fit: cover;
      height: 100%;
    }
    
    // 渐变效果
    &::after {
      content: '';
      position: absolute;
      bottom: 0;
      left: 0;
      right: 0;
      height: 30%;
      background: linear-gradient(
        to bottom,
        transparent,
        rgba(255, 255, 255, 0.9)
      );
      pointer-events: none;
    }
  }
}
```

#### 截断图标样式

```scss
.truncate-indicator {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.2s;
  z-index: 1;
  
  &:hover {
    background: rgba(0, 0, 0, 0.8);
  }
  
  // 图标可以使用 SVG 或文字
  &::before {
    content: '🔗'; // 或使用 SVG 图标
  }
}
```

## Data Flow

```
Backend API (MediaFile with width/height)
    │
    ▼
Gallery.vue (columnWidth calculation)
    │
    ▼
MediaCard.vue
    │
    ├── aspectRatio = width / height
    ├── isTruncated = aspectRatio < 0.5
    ├── maxHeight = columnWidth * 1.5
    │
    ▼
Render (with truncated styles)
```

## Edge Cases

### 1. 缺少宽高信息

- 如果 API 返回的图片没有宽高信息，不进行截断
- 默认显示原图

### 2. 极端宽高比

- 宽高比极小（如 0.1）的图片也应该正常显示
- 最大高度限制防止过度截断

### 3. 视频缩略图

- 视频缩略图同样应用截断逻辑
- 视频时长标识应正常显示在截断区域之上

## Performance Considerations

1. **computed 属性**: 使用 Vue computed 缓存计算结果
2. **CSS 实现**: 使用 CSS 而非 JS 实现渐变效果，减少重绘
3. **条件渲染**: 仅在 isTruncated 为 true 时渲染截断相关 DOM

## Testing Plan

### Unit Tests

- [ ] aspectRatio 计算正确性
- [ ] isTruncated 阈值判断
- [ ] maxHeight 计算

### Visual Tests

- [ ] 正常比例图片显示
- [ ] 截断图片显示（渐变 + 图标）
- [ ] 移动端响应式
- [ ] 悬停效果

## File Changes

| File | Change Type | Description |
|------|-------------|-------------|
| `frontend/src/components/MediaCard.vue` | Modify | 添加截断逻辑和样式 |
| `frontend/src/components/Gallery.vue` | Modify | 传递 columnWidth |
