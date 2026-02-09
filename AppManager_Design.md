# AppManager 桌面应用设计文档

## 1. 需求概述
### 1.1 目标
打造一款小巧、精致的 Windows 桌面工具，自动分析并可视化展示电脑上“正规渠道安装软件”的实际空间占用，重点解决 C 盘空间被应用数据（AppData/ProgramData）暗中吃掉的问题。

### 1.2 核心功能
- **软件清单识别**：从系统注册表和 UWP 应用商店获取已安装软件列表。
- **深度空间归因**：不只计算安装目录，还需通过智能匹配算法关联其在 `AppData` 和 `ProgramData` 下的存储目录。
- **可视化列表**：像手机存储管理一样，按占用大小排序，支持搜索和查看明细。
- **极速扫描**：利用 Rust 的并发能力，在数秒内完成 C 盘几十万个文件的扫描与统计。
- **数据可视化**：通过饼图与堆叠条形图直观展示存储构成与高占用应用。

---

## 2. 技术选型
- **框架**: [Tauri](https://tauri.app/) (v2)
- **后端 (Core)**: Rust
  - `jwalk`: 高性能并发目录遍历。
  - `sysinfo` / `winreg`: 获取系统信息与注册表操作。
  - `serde`: 数据序列化。
- **前端 (UI)**: React + TypeScript
  - `Tailwind CSS`: 实现精致、现代化的 UI。
  - `Lucide React`: 图标库。
  - `Framer Motion`: 实现流畅的动效。

---

## 3. 架构设计 (Architecture)

项目采用 **Tauri v2** 框架，实现了前后端逻辑的深度解耦与模块化。

### 3.1 核心分层
- **后端 (Rust Core)**: 负责系统级操作（注册表、文件系统遍历、并发计算）。
- **通信层 (Tauri IPC)**: 通过 `Commands` (请求-响应) 和 `Events` (流式通知) 进行交互。
- **前端 (React UI)**: 负责状态管理、交互逻辑与数据可视化。

### 3.2 后端架构 (src-tauri)
后端逻辑通过功能模块进行划分，确保代码的可维护性：
- **`commands.rs`**: 统一管理所有对外暴露的 Tauri Command，作为 API 入口。
- **`apps` 模块**: 核心业务逻辑层。
  - **`windows` 子模块**: 封装 Windows 特有实现（注册表读取、AppData 归因、路径映射）。
    - `roots.rs`: 根目录枚举与缓存。
    - `uninstall.rs`: 注册表卸载信息提取。
    - `matching.rs`: 软件与文件夹的归因算法。
    - `sizing.rs`: 高性能目录大小计算（支持缓存）。
    - `audit.rs`: 系统存储占用审计逻辑。

### 3.3 前端架构 (src)
前端遵循 **Feature-based** 结构，按功能模块组织代码：
- **`types/`**: 集中管理所有 TypeScript 类型定义，与 Rust 端结构严格对应。
- **`lib/tauri/`**: 封装对 Tauri API 的调用，提供类型安全的异步函数。
- **`features/apps/`**: 应用管理核心功能。
  - `hooks/`: `useScanApps` (扫描流处理), `useAudit` (审计逻辑)。
  - `components/`: `AppsList` (列表渲染), `AuditPanel` (审计看板)。
- **`App.tsx`**: 作为应用入口，负责高层级的组合与布局。

### 3.4 数据归因逻辑
软件占用空间 = **安装目录 (InstallLocation)** + **用户数据目录 (AppData)** + **机器数据目录 (ProgramData)**。
1. **直接路径**：优先使用注册表中的 `InstallLocation`。
2. **启发式匹配**：基于 `DisplayName` 和 `Publisher` 生成特征 Token，与 `AppData` 目录名进行加权匹配。
3. **性能保证**：通过并发扫描与目录大小缓存，避免重复计算。

---

## 4. 技术实现细节

### 4.1 异步扫描流
扫描过程采用事件驱动模型：
1. 前端调用 `scan_apps` Command。
2. 后端启动扫描，并通过 `emit` 发送 `scan_progress` 和 `scan_result` 事件。
3. 前端 `useScanApps` Hook 监听事件并实时更新 UI 状态，实现“边扫描边展示”。

### 4.2 目录大小统计
利用 Rust 的并发特性，结合 `jwalk` 进行高性能遍历。同时引入了目录级缓存，在一次会话中对同一个 AppData 目录仅计算一次大小，显著提升二次扫描速度。

---

## 5. 项目目录结构
```text
AppManager/
├── src/                # 前端源代码
│   ├── features/       # 功能模块 (Apps, Audit)
│   ├── lib/            # 工具库 (Tauri API 封装)
│   ├── types/          # 类型定义
│   └── App.tsx         # 入口组件
├── src-tauri/          # 后端源代码
│   ├── src/
│   │   ├── apps/       # 核心业务逻辑 (模块化实现)
│   │   ├── commands.rs # Tauri 命令定义
│   │   ├── lib.rs      # 应用入口与生命周期
│   │   └── main.rs     # 程序启动点
│   └── Cargo.toml      # Rust 依赖配置
└── tauri.conf.json     # Tauri 配置文件
```

---

## 6. 开发路线图 (Roadmap)
- [x] **Phase 1**: 环境配置与项目初始化。
- [x] **Phase 2**: 实现 Rust 端基础扫描引擎（注册表读取 + 目录遍历）。
- [x] **Phase 3**: 实现归因算法与模块化重构。
- [x] **Phase 4**: 前端 UI 开发与 Hook 逻辑封装。
- [ ] **Phase 5**: 数据可视化仪表盘（饼图、堆叠图）。
- [ ] **Phase 6**: 性能深度优化（如索引持久化）与兼容性测试。
