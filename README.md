# LRC Editor

一个基于 **Tauri 2 + Vue 3 + TypeScript** 的桌面端 LRC 歌词打点编辑工具。加载本地 MP3，通过「打点」的方式为每行歌词标记播放时间戳，支持实时预览、自动滚动，并导出标准 LRC 文件。

> 当前版本：v0.1.0 ｜ 目标平台：macOS（可扩展至 Windows / Linux）

---

## 功能特性

### 🎵 音频播放
- 通过系统原生对话框打开本地 MP3 文件
- 播放 / 暂停切换（同一按钮，状态联动显示 ▶️ / ⏸️）
- 可拖动进度条跳转播放进度（进度条自动占满剩余宽度，拖动手感顺滑）
- 实时显示当前播放时间 / 总时长（`mm:ss.xx` 格式）

### 📌 打点与自动换行
- 点击「打点并下一行」：为当前选中行写入当前播放时间戳，并**自动跳到下一行**（该行为固定开启，不提供取消开关）
- 同一行可多次打点，时间戳自动升序排序
- 自动跳转时，若行底部超出可视区，自动滚动跟随（靠近列表末尾时改为贴底滚动，不越界）

### ✏️ 歌词行编辑
- 行内直接编辑歌词文本
- 点击任意行切换选中状态
- 在选中行**上方 / 下方插入**空行
- **删除当前行 / 删除所有行**（高危操作带原生确认弹窗）
- **上移 / 下移**调整行顺序
- **清除本行时间戳** / **清除全部时间戳**（批量清除需确认）

### 📋 LRC 元信息
- 标题 `ti`、歌手 `ar`、专辑 `al` 三个独立输入框
- 导入时自动解析填充，导出时写入文件头部

### 🔍 歌词预览模式
- 顶部切换按钮（普通模式显示「🔍预览」，预览模式显示「✏️编辑」）
- 进入预览后：**打点、插入、删除、移动、文本编辑等全部禁用**，仅保留播放 / 暂停 / 进度拖动
- 高亮匹配规则：在全部歌词行的时间戳中，取 **≤ 当前播放时间的最大时间戳**，高亮其对应行；同一时间戳对应多行时，**只高亮下标最大（靠后）的那一行**，始终仅单行高亮
- 播放时间持续变化时实时重新匹配；拖拽进度条跳转同样即时刷新
- 高亮行变化时，**自动平滑滚动至歌词列表垂直居中**（预览期间强制跟随，不受手动滚动干扰）
- 预览高亮与编辑选中状态**完全独立**：预览不修改编辑态选中行，退出预览后高亮立即清除、编辑功能全部恢复

### 📥 导入 / 📤 导出
- **粘贴歌词**：弹窗粘贴 LRC 文本，解析后载入编辑器
- **导入 LRC 文件**：系统对话框选择本地 `.lrc` 文件读取（UTF-8）
- **保存导出**：生成标准 LRC 文件并保存为 `output.lrc`

### 🎨 界面风格
- Material Design 扁平化按钮：统一 `hover / active / disabled` 四态反馈，Emoji 与文字垂直居中
- 全局字体统一为 macOS 系统无衬线字体（苹方 / San Francisco），输入框、标签、歌词区保持一致，不再出现衬线字
- 时间戳列使用等宽字体，排版对齐
- 预览高亮行使用蓝色描边 + 极淡底色，与编辑选中行的浅蓝背景清晰区分

---

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2（Rust 后端 + WebView 前端） |
| 前端框架 | Vue 3（Composition API，`<script setup>`）+ TypeScript |
| 构建工具 | Vite |
| Tauri API | `@tauri-apps/api`（core / dialog / fs） |
| 系统交互 | `@tauri-apps/plugin-dialog`（打开文件、确认弹窗）、`@tauri-apps/plugin-fs`（读取文件） |
| 后端命令 | Rust 侧封装 LRC 解析 / 导出 / MP3 文件选择，前端通过 `invoke` 调用 |

---

## 项目结构（示意）

```
lrc-editor/
├── src/
│   ├── main.ts                  # Vue 入口
│   ├── App.vue                  # 主界面：播放器、打点、编辑、预览全部逻辑
│   ├── types.ts                 # LyricLine 等类型定义
│   └── api/
│       └── backend.ts           # invoke 封装：parseLrcText / exportLrcText / selectMp3File
├── src-tauri/
│   ├── Cargo.toml               # Rust crate（name = lrceditor，控制 macOS 菜单栏名）
│   ├── tauri.conf.json          # productName / identifier / window / bundle.icon
│   ├── icons/
│   │   └── icon.icns            # macOS 应用图标（tauri icon 生成）
│   └── src/
│       ├── main.rs
│       └── lib.rs               # LRC 解析 / 导出等命令实现
├── package.json
└── vite.config.ts
```

---

## 开发环境要求

- macOS（当前打包目标平台）
- Node.js 18+ 与 npm
- Rust toolchain（stable）
- Xcode Command Line Tools（macOS 打包需要）

---

## 快速开始

```bash
# 1. 安装前端依赖
npm install

# 2. 启动开发模式（自动拉起 Rust 编译 + Vite 热更新）
npm run tauri dev

# 3. 打包发布（产出 .app / .dmg）
npm run tauri build
```

打包产物位于：`src-tauri/target/release/bundle/macos/`

---

## 使用流程示例

1. **打开 MP3**：点击「🎵打开MP3」选择音频文件
2. **准备歌词**：点击「📋粘贴歌词」或「📁导入LRC」，也可直接编辑初始空行
3. **打点**：播放音频，选中歌词行后点击「⏱️计时并下一行」，写入时间戳并自动切到下一行，逐行完成
4. **微调文本与顺序**：行内编辑文字，使用插入 / 删除 / 上移 / 下移调整
5. **预览检查**：点击「🔍预览」进入预览模式，播放音频，当前行蓝色边框高亮并自动居中滚动，确认节奏是否对齐
6. **导出**：点击「✏️编辑」退出预览，填写 ti / ar / al 元信息，点击「💾保存到下载目录 output.lrc」

---

## 构建与打包注意事项（macOS）

### 应用名称
- **macOS 顶部菜单栏名称**：`src-tauri/Cargo.toml` → `[package].name`（如 `lrceditor`，不含空格）
- **About 弹窗标题 / .app 包名**：`src-tauri/tauri.conf.json` → `package.productName`（如 `LRC Editor`，可含空格）
- **窗口标题栏**：`tauri.conf.json` → `app.windows[0].title`
- **Bundle Identifier**：`package.identifier`（如 `com.yourname.lrceditor`，不含空格）

### 应用图标
- 使用 `tauri icon <原图.png>` 生成全套图标（原图建议 1024×1024 带透明通道 PNG）
- **About 弹窗图标只读取 `icons/icon.icns`**，需确认 `bundle.icon` 数组包含 `icons/icon.icns`
- ⚠️ **开发模式（`tauri dev`）下 About 弹窗永远显示默认文件夹图标**，这是 Tauri 2 的正常行为；必须 `tauri build` 打包成 `.app` 后，About 才会显示真实图标
- macOS 有图标缓存：测试新包前将旧 `.app` 移入废纸篓，避免误看旧图标

### 清理缓存
```bash
# 常规清理（改了 Rust / tauri 配置后）
# 先彻底退出正在运行的 tauri dev 进程
cd src-tauri && cargo clean && cd ..
rm -rf .vite
npm run tauri dev

# 彻底全量重建（仅依赖损坏等异常时才用）
cd src-tauri && cargo clean && cd ..
rm -rf node_modules .vite dist
npm install
npm run tauri dev
```

---

## 已知限制

| 限制 | 说明 |
|---|---|
| 导入编码 | LRC 导入按 UTF-8 处理，GBK 编码文件可能出现乱码（编码自动检测尚未实现） |
| 导出文件名 | 固定为 `output.lrc`，暂不跟随歌曲标题自动命名 |
| 预览滚动 | 预览期间强制高亮行居中，用户手动滚动不会打断自动跟随 |
