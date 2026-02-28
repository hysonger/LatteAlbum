# Tasks: Truncate Small Aspect Ratio Thumbnails

## Implementation Checklist

### Phase 1: Core Implementation

- [ ] **T1.1** 在 `MediaCard.vue` 中添加 `columnWidth` prop
- [ ] **T1.2** 实现 `aspectRatio` computed 属性
- [ ] **T1.3** 实现 `isTruncated` computed 属性（阈值 0.5）
- [ ] **T1.4** 实现 `maxHeight` computed 属性

### Phase 2: Styling

- [ ] **T2.1** 添加截断容器样式（max-height + overflow: hidden）
- [ ] **T2.2** 添加底部渐变效果（::after 伪元素）
- [ ] **T2.3** 添加截断指示器图标样式

### Phase 3: Integration

- [ ] **T3.1** 在 `Gallery.vue` 中计算 columnWidth
- [ ] **T3.2** 传递 columnWidth 给 MediaCard 组件

### Phase 4: Testing

- [ ] **T4.1** 测试正常比例图片显示
- [ ] **T4.2** 测试窄长图片截断效果
- [ ] **T4.3** 测试渐变效果和图标显示
- [ ] **T4.4** 测试移动端响应式
- [ ] **T4.5** 测试点击截断图片打开查看器

## Implementation Order

```
1. MediaCard.vue - Props + Computed
           │
           ▼
2. MediaCard.vue - Styles (truncation)
           │
           ▼
3. Gallery.vue - Pass columnWidth
           │
           ▼
4. Testing & Refinement
```

## Notes

- 优先实现核心逻辑，确保功能可用
- 样式可以后续优化
- 测试时需要准备不同宽高比的测试图片
