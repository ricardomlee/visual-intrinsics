# Visual Intrinsics — SIMD 寄存器可视化工具

一个基于浏览器的交互式 x86 SSE/AVX 风格 128/256/512 位 SIMD 寄存器可视化工具。计算逻辑由 **Rust** 编写，通过 [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) 编译为 **WebAssembly**，前端使用 webpack 打包，存放于 `www/` 目录。

> 📖 参考资料：[Intel Intrinsics Guide（英特尔内置函数指南）](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html)

## 功能介绍

| 功能 | 说明 |
|---|---|
| **三种寄存器宽度** | 可在 XMM（`__m128i` 128 位）、YMM（`__m256i` 256 位）和 ZMM（`__m512i` 512 位）之间切换 |
| **十六进制输入** | 输入任意值（可带或不带 `0x` 前缀），按 **Set** 确认 |
| **位网格** | 4 行 × 32 列实时位单元格，按通道（lane）着色 |
| **多种通道视图** | 可在 `epi8`、`epu8`、`epi16`、`epu16`、`epi32`、`epu32`、`epi64` 以及原始位视图之间切换 |
| **可编辑通道值** | 点击 **✏️ Edit Lanes** 后，可直接在 A/B 面板的任意通道单元格中输入数值并立即生效 |
| **位运算** | AND、OR、XOR、ANDNOT（A op B），以及 NOT A / NOT B |
| **打包算术** | 加法、减法、饱和加/减、乘法（低位 / 高位） |
| **绝对值** | `abs_epi8/16/32` |
| **比较与选择** | `cmpeq`、`cmpgt`，以及有符号/无符号的 `max`/`min` |
| **移位** | 每通道逻辑/算术移位；全寄存器逻辑移位 |
| **重排列** | `unpacklo/hi`、`packs`/`packus`、`shuffle_epi8/32`、`alignr`、`blendv` |
| **水平运算** | `hadd`/`hsub` epi16/32 |
| **复制** | A → B、B → A、Result → A、Result → B |
| **结果显示** | 计算结果的十六进制值 + 通道表 + 位网格 |

## 快速开始

### 环境依赖

* [Rust 工具链](https://rustup.rs/)（stable，1.70+）
* [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)  
  ```
  cargo install wasm-pack
  ```
* [Node.js](https://nodejs.org/) ≥ 18 + npm

### 构建 WASM 包

```
wasm-pack build
```

### 安装前端依赖并运行

```
cd www
npm install --legacy-peer-deps
npm start        # webpack 开发服务器，地址：http://localhost:8080
```

### 生产构建

```
cd www
npm run build    # 输出到 www/dist/
```

## 项目结构

```
src/
  lib.rs      — M128i/M256i/M512i 结构体及所有 Rust WASM API
  utils.rs    — panic hook 辅助工具
pkg/          — 由 wasm-pack 生成（已 git-ignore）
www/
  index.html  — 单页 UI
  index.js    — JS 与 WASM API 的绑定逻辑
  bootstrap.js
  webpack.config.js
  package.json
tests/
  web.rs      — wasm-bindgen 浏览器测试
```

## 运行浏览器测试

```
wasm-pack test --headless --chrome
```

## 许可证

可在 [Apache-2.0](LICENSE_APACHE) 或 [MIT](LICENSE_MIT) 两者之一下使用，自行选择。
