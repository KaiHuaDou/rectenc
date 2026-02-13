# Rectenc

`rectenc` 是一个使用 Rust 编写的通过矩形来有损编解码视频的工具。

灵感来源：<https://www.youtube.com/watch?v=V9KJpbzNvKw>

## 使用方法

```text
A video encoding/decoding utility using rectangles

Usage: rectenc.exe <COMMAND>

Commands:
  encode
  decode
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version```
```

```text
Usage: rectenc.exe encode [OPTIONS] <INPUT> <OUTPUT>

Arguments:
  <INPUT>
  <OUTPUT>

Options:
  -r, --rect-count <RECT_COUNT>  [default: 1]
  -p, --preview
  -a, --adaptive
  -h, --help                     Print help
```

```text
Usage: rectenc.exe decode <INPUT> <OUTPUT>

Arguments:
  <INPUT>
  <OUTPUT>

Options:
  -h, --help  Print help
```

## 原理

### 编码

1. 读取输入视频并提取元数据（FPS、宽度、高度）
2. 将元数据作为第一个矩形存储在输出中
3. 创建全黑参考帧
4. 对于每个后续帧：
   - 计算当前帧与参考帧的差异权重
   - (若 N > 2) 计算后续每一帧与参考帧的差异并叠加到差异权重上
   - 确定能弥补差异权重的最佳矩形
      - 修正权重使得 0 元素具有负数值惩罚
      - 通过 Kadane 算法分别确定正负值最大的矩形
   - 将矩形储存叠加在参考帧之上
5. 将所有矩形序列化并保存

### 解码

- 从第一个矩形中获取元数据
- 逐帧逐个叠加矩形
