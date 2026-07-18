# 图片大图页面缩放与拖拽功能设计

- 日期：2026-06-20
- 范围：前端 `PhotoViewer.vue` 大图查看器
- 目标：为图片类型媒体添加缩放（鼠标滚轮 / Ctrl+Command±）与点按拖拽平移功能；视频类型不启用。

> **更新（2026-07-18）**：滚轮公式已改为按事件特征自适应系数（触控板捏合 `ctrlKey` / 小步长 delta 用 0.01，鼠标大步长保持 0.0015，`deltaMode=1` 行模式按 33px/行归一化，单事件倍率 clamp 到 [0.5, 2]），解决触控板缩放过慢；同时新增移动端触屏支持：双指捏合以双指中点为锚点缩放、捏合结束剩一指接续平移、单指放大后可平移（`<img>` 设 `touch-action: none`），第 42、59 行的原始描述以代码为准。

## 背景与现状

`PhotoViewer.vue` 同时承载图片与视频：

- `.media-container` 通过 `containerStyle` 依据**原始图片宽高比**计算出固定宽高，使图片"适配屏幕"。
- `<img>` 使用 `width:100%; height:100%; object-fit: contain`，因容器已按图片宽高比定尺寸，图片**正好填满** `<img>` 元素，无 letterbox 空隙。
- 键盘事件通过 `document.addEventListener('keydown', handleKeydown)` 全局处理（Esc / 左右方向键；视频播放时不拦截方向键）。
- 点击背景（`.photo-viewer @click.self="close"`）关闭查看器。

关键结论：1x 时图片贴合容器、无空隙，因此用 CSS `transform` 缩放/平移不会出现"空隙判定"问题，clamp 边界严格成立。

## 确认的交互参数

| 项 | 选择 |
| --- | --- |
| 滚轮缩放中心 | 鼠标光标处 |
| 缩放范围 | 1x – 5x |
| 双击 | 在 1x 与 2x 间切换 |
| 切换上一张/下一张 | 重置为 1x |

## 方案

基于 CSS `transform` 的缩放/平移，逻辑封装为组合式函数 `useImageZoom.ts`。零依赖，浏览器走 GPU 合成保证流畅。

- 对 `<img>` 应用 `transform: translate(ox, oy) scale(s)`，`transform-origin: 0 0`。
- 仅在 `isImage` 为真时启用；视频保持原生控件与方向键逻辑不变。

### 数学锚点公式

设当前 scale `s0`、偏移 `o0`（x/y 两轴独立）、锚点（相对容器左上的像素坐标）`c`、目标 `s1`。保持锚点下同一图像点不动的偏移：

```
o1 = c - (c - o0) * (s1 / s0)
```

- 滚轮：`c = (cursorX, cursorY)`；`s1 = clamp(s0 * Math.pow(1.0015, -deltaY), 1, 5)`。该公式既能处理鼠标离散滚轮（`deltaY=±100` → 倍率 ≈1.16），也能平滑处理触控板惯性滚动的细粒度 `deltaY`。
- 键盘（Ctrl/Cmd + `+`/`-`）：`c = (W/2, H/2)`（图片中心）；步进 ×1.25 / ÷1.25，clamp。
- 双击：`c = 双击点`；在 1x 与 2x 间翻转。

### 边界 clamp

缩放后图片须始终覆盖容器（不露黑边）。以容器布局尺寸 `W × H`（用 `offsetWidth / offsetHeight` 读取，不受 transform 影响）：

```
ox ∈ [W * (1 - s), 0]
oy ∈ [H * (1 - s), 0]
```

因 1x 时图片正好填满，该边界严格成立，无空隙特例。

### 拖拽平移

- `pointerdown` 仅响应 `pointerType === 'mouse'`（不劫持移动端原生手势），记录起点并 `setPointerCapture`。
- `pointermove` 累加位移。
- 缩放 ≤ 1 时不可拖（拖动无效）。
- `pointerup` 结束。

### 光标

- scale > 1 且未拖拽：`grab`
- 拖拽中：`grabbing`
- 其他：默认

### 状态与重置

- 状态：`scale`、`offsetX`、`offsetY`。
- `reset()`：scale=1、offset=0。
- 在 `watch(currentFile)`（图片切换）中调用 `reset()`；视频类型 `enabled=false` 无副作用。

### 作用范围与边界

- 视频类型：`enabled=false`，不绑定 wheel/pointer/transform，`<video>` 维持原生控件。
- 滚轮 `preventDefault`：wheel 监听器以 `{ passive: false }` 手动挂载，确保阻止页面滚动。
- 现有关闭逻辑（点背景 → `close`）不受影响：拖拽图片不触发背景点击；Esc 仍关闭。

## 组件接口（`useImageZoom`）

```ts
useImageZoom(containerRef: Ref<HTMLElement | null>, {
  enabled: ComputedRef<boolean>,  // isImage
  minScale = 1,
  maxScale = 5,
}): {
  imgStyle: ComputedRef<CSSProperties>  // 绑定到 <img>：transform / transform-origin / cursor / user-select
  cursor: ComputedRef<string>
  onWheel: (e: WheelEvent) => void
  onPointerDown: (e: PointerEvent) => void
  onPointerMove: (e: PointerEvent) => void
  onPointerUp: (e: PointerEvent) => void
  onDoubleClick: (e: MouseEvent) => void
  reset: () => void
}
```

## 文件清单

- 新增：`frontend/src/composables/useImageZoom.ts`
- 改动：`frontend/src/components/PhotoViewer.vue`
  - 给 `.media-container` 加 `ref`
  - 给 `<img>` 绑定 `imgStyle`
  - `watch(currentFile)` 调用 `reset()`
  - 扩展 `handleKeydown`：图片下 Ctrl/Cmd + `+`/`=`/`-` 缩放并 `preventDefault`
  - `onMounted/onUnmounted` 挂载/卸载 wheel 监听（passive:false）
- 文档：更新 `docs/architecture.md`（新增组合式函数说明）；按需更新 `AGENTS.md`。

## 验证

- 前端无测试框架（package.json 仅有 dev/build/preview）。本特性不擅自引入测试运行器。
- 依赖 `npm run build`（含 `vue-tsc` 类型检查）+ 手动验证：
  - 滚轮缩放：以光标为锚点，范围 1–5，阻止页面滚动。
  - Ctrl/Cmd ± 缩放，以中心为锚点，阻止浏览器缩放。
  - 双击 1x↔2x 翻转。
  - 拖拽平移 + clamp 不露黑边；1x 时拖拽无效。
  - 换图重置；视频不受影响；Esc 仍关闭；点击背景仍关闭。
- 缩放/平移数学保持为纯函数，便于将来引入测试框架时覆盖。
