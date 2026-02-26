# 视频缩略图旋转问题修复计划

## 背景

部分视频存在旋转元数据（如手机竖屏拍摄的视频），当前生成的缩略图没有正确处理旋转角度，导致显示方向错误。

**测试视频**：
- `/home/songer/Projects/LatteAlbum/photos/VID_20251005_234621.mp4` - 90度旋转
- `/home/songer/Projects/LatteAlbum/photos/VID_20251004_210235.mp4` - -90度旋转

---

## 问题1：DisplayMatrix 解析错误（已修复 ✓）

### 之前的实现问题

1. **DisplayMatrix 格式错误**: 代码假设是 72 字节 (9 × float64)，但实际是 **36 字节 (9 × int32_t)**
2. **数据类型错误**: 使用 `&[f64]` 解析，但应该是 `&[i32]` 并从固定点数转换

### DisplayMatrix 格式（FFmpeg 官方文档）

- **大小**: 36 字节 (9 × 4 字节 int32_t)
- **格式**: 16.16 固定点数表示
- **布局**: [a, b, u, c, d, v, x, y, w] 代表 3x3 矩阵

### 旋转角度计算公式

```c
// 从固定点数 (16.16) 转换为双精度浮点数
#define CONV_FP(x) ((double) (x)) / (1 << 16)

// 计算缩放因子
scale[0] = hypot(CONV_FP(matrix[0]), CONV_FP(matrix[3]));
scale[1] = hypot(CONV_FP(matrix[1]), CONV_FP(matrix[4]));

// 计算旋转角度 (逆时针为正)
rotation = -atan2(matrix[1]/scale[1], matrix[0]/scale[0]) * 180 / M_PI;
```

---

## 问题2：旋转方向错误（已修复 ✓）

### 问题描述

使用 `image::imageops::rotate90` 后，VID_20251005_234621.mp4 (90°) 出现上下反转。

### 根本原因

**FFmpeg DisplayMatrix 的旋转角度定义**：
- 使用数学标准：逆时针旋转为正角度
- 90° = 逆时针旋转90度

**image crate 的旋转函数**：
- `rotate90` = 顺时针旋转90度
- `rotate270` = 逆时针旋转90度

### 解决方案

交换旋转逻辑：
- DisplayMatrix **90°** → 逆时针90° → 使用 `rotate270`
- DisplayMatrix **270°** (或 -90°) → 顺时针90° → 使用 `rotate90`

---

## 修复文件

### 1. `rust/examples/video_thumbnail_with_rotation.rs` (第 271-288 行)

修改前：
```rust
let final_image = match normalized_rotation {
    Some(90) | Some(270) => {
        image::imageops::rotate90(&rgb_image)
    }
    Some(180) => {
        image::imageops::rotate180(&rgb_image)
    }
    Some(0) | None => {
        rgb_image
    }
    _ => {
        rgb_image
    }
};
```

修改后：
```rust
let final_image = match normalized_rotation {
    Some(90) => {
        // DisplayMatrix 90° = 逆时针90° = rotate270
        image::imageops::rotate270(&rgb_image)
    }
    Some(270) => {
        // DisplayMatrix 270° (-90°) = 顺时针90° = rotate90
        image::imageops::rotate90(&rgb_image)
    }
    Some(180) => {
        image::imageops::rotate180(&rgb_image)
    }
    Some(0) | None => {
        rgb_image
    }
    _ => {
        rgb_image
    }
};
```

### 2. `rust/src/processors/video_processor.rs` (第 377-391 行)

同步修改相同的旋转逻辑。

---

## 验证方法

1. 运行示例程序生成两个有旋转视频的缩略图：
   ```bash
   ./cargo-with-vendor.sh run --example video_thumbnail_with_rotation -- /home/songer/Projects/LatteAlbum/photos/VID_20251005_234621.mp4
   ./cargo-with-vendor.sh run --example video_thumbnail_with_rotation -- /home/songer/Projects/LatteAlbum/photos/VID_20251004_210235.mp4
   ```

2. 检查输出图像：
   - VID_20251005_234621.jpg 应该显示正常竖屏（不是倒立）
   - VID_20251004_210235.jpg 应该显示正常竖屏

3. 对比原视频和缩略图的方向一致性

## 验证结果

| 视频 | DisplayMatrix | 应用旋转 | 结果 |
|------|---------------|----------|------|
| VID_20251005_234621.mp4 | 90° | 270° (逆时针) | ✓ 正常竖屏 |
| VID_20251004_210235.mp4 | -90°/270° | 90° (顺时针) | ✓ 正常竖屏 |

---

## 相关提交

- `92a74f3` - parse DisplayMatrix to correctly rotate the video thumbnail
- `53833d3` - video thumbnail test version
