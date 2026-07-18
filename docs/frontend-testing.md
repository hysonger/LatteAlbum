# 前端测试框架

LatteAlbum 前端（Vue 3 + TypeScript + Vite 5）使用 [Vitest](https://vitest.dev/) 作为测试框架。本文记录选型、目录约定与各层测试范式，供后续覆盖用例时参考。

## 选型理由

| 依赖 | 作用 |
|---|---|
| `vitest@^2` | 测试运行器，与 Vite 5 原生集成，复用 `vite.config.ts` 的 alias / define |
| `jsdom@^29` | DOM 环境，与 Element Plus 组件兼容更稳（happy-dom 更快但偶有兼容问题） |
| `@vue/test-utils@^2.4` | Vue 3 组件挂载库 |
| `@vitest/coverage-v8@^2` | 覆盖率（v8 provider） |
| `@vitest/ui@^2` | 可选的可视化测试面板 |

> 版本说明：Vitest 3.x/4.x 要求 Vite ≥6，本项目用 Vite 5，故锁定 2.x。

## 配置位置

测试配置合并进 `frontend/vite.config.ts` 的 `test` 块（顶部 `/// <reference types="vitest/config" />`），复用 `@` → `src` 路径别名。全局 API（`describe`/`it`/`expect`/`vi` 等）开启 `globals: true`，类型由 `tsconfig.json` 的 `types: ["vitest/globals"]` 提供，测试文件无需 import。

## 目录约定

- **就近平铺**：测试文件与被测源码同目录，命名为 `*.spec.ts`（如 `src/utils/format.spec.ts`）。
- **全局 setup**：`src/test/setup.ts`，补齐 jsdom 缺失的浏览器 API（`URL.createObjectURL`、`matchMedia`、`IntersectionObserver`、`ResizeObserver`）。
- 仅 `*.spec.ts` / `*.test.ts` 被收集运行。

## 各层测试范式（模板）

每个层级已有一个示例用例，新增用例照此扩展：

| 层级 | 示例文件 | 范式要点 |
|---|---|---|
| 纯函数 | [src/utils/format.spec.ts](../frontend/src/utils/format.spec.ts) | 输入→输出直接断言；`vi.useFakeTimers` 测 `debounce` |
| composable + DOM | [src/composables/useScreenSize.spec.ts](../frontend/src/composables/useScreenSize.spec.ts) | 用 `mount(defineComponent(...))` 触发 `onMounted`；改 `window.innerWidth` 后 `dispatchEvent('resize')` |
| 复杂交互 composable | [src/composables/useImageZoom.spec.ts](../frontend/src/composables/useImageZoom.spec.ts) | 按需 stub `offsetWidth/Height`、`getBoundingClientRect` |
| Pinia store | [src/stores/gallery.spec.ts](../frontend/src/stores/gallery.spec.ts) | `vi.mock('@/services/api')`；`createPinia()` + `setActivePinia()` 隔离实例 |
| 组件挂载 | [src/components/RefreshButton.spec.ts](../frontend/src/components/RefreshButton.spec.ts) | `@vue/test-utils` 的 `mount`，验证 props 渲染与 emit |

## 常见坑

- **jsdom 无真实布局**：`offsetWidth`/`offsetHeight`/`getBoundingClientRect` 默认返回 0。依赖布局尺寸的用例需在测试内用 `Object.defineProperty` 按需 stub，**不要**放进全局 setup，避免污染其它用例。
- **mock `@/services/api`**：`vi.mock('@/services/api', () => ({ fileApi: { getFiles: vi.fn() } }))`，用 `vi.mocked(fileApi.getFiles)` 获取带类型的 mock。注意工厂内只能用 `vi.*`，不能引用外部变量。
- **模块级单例状态**：如 `useScreenSize` 的 `screenWidth` 是模块级 ref，跨用例保留；改属性后需 dispatch 事件同步。
- **依赖 store/router/Element Plus 全局的组件**（如 Gallery）挂载时需通过 `global.plugins` 注入，留到后续按需覆盖；PhotoViewer 无 store/router 依赖，`vi.mock('@/services/api')` 后即可直接挂载（见 [src/components/PhotoViewer.spec.ts](../frontend/src/components/PhotoViewer.spec.ts)）。

## 验证

```bash
cd frontend
npm run test          # 全绿
npm run test:coverage # 产出 coverage/index.html
npm run build         # vue-tsc 类型检查仍通过（spec 文件也被类型检查）
```
