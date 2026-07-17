# Known Issues

Known issues and workarounds for Latte Album.

## iPhone mov Video Seek Failure

**Problem**: iPhone mov videos fail to seek to middle/end positions.

**Root cause**: iPhone mov files have moov atom at file end (HEVC/H.265 common practice). Range requests return data without moov metadata.

**Solution**: Use full file request for mov format, or detect moov position server-side.

## 灯箱 ESC 导致标签页变 about:blank（仅自动化环境，dogfood ISSUE-002）

**Problem**: 在 agent-browser 自动启动的无头 Chrome for Testing（`webdriver=true` 自动化配置）中，灯箱查看大图后按 CDP 注入的 ESC 键，会间歇性导致**渲染进程崩溃**，标签页以 `about:blank` 恢复（DOM 清空、`history.length===1`、无 beforeunload、无控制台错误）。

**Root cause**: Chromium 渲染器在「原生 ESC 停止加载行为 + 大图图片管线拆除」竞态下崩溃。已验证：
- 应用源码中无任何页面导航逻辑；所有 JS 导航钩子均未被触发。
- 仅在 agent-browser 自启动的自动化浏览器中复现（多会话 10+/15+ 次）；手动启动的同版本 Chrome（有头 6 轮、无头 7 轮、相同启动参数）及合成 JS 按键均 0 复现。
- 普通用户环境（有头浏览器、物理按键）未观察到该问题，判断为自动化测试环境特有的浏览器缺陷。

**Mitigation**（已合入，降低风险但无法在自动化环境 100% 消除）:
- `PhotoViewer.vue` 大图不再经 `axios + createObjectURL` 加载 Blob，改用缩略图直链 URL + 离屏 `Image` 预加载 full 后替换，消除 blob 生命周期竞态并大幅降低内存占用。
- ESC 分支增加 `e.preventDefault()`，抑制浏览器原生「停止加载」行为（查看器自行处理关闭）。

**注意**: 在该自动化环境中验证修复时，「翻页 + 停留 15 秒 + CDP ESC」仍可能偶发崩溃；这不代表真实用户风险。

## Timezone Handling

**Design**: Frontend shows time literal without conversion. Backend sorts by literal time. Timezone label shown only when different from user's local timezone.

**Known issue**: Photos from different timezones may be out of order. Not currently fixed.
