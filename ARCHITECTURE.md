# ZMake Architecture & Context

> 此文件定义了 ZMake 项目的核心架构、技术栈约束及设计哲学。AI 助手在生成代码时必须严格遵循此文档。
> 此文档也供人类阅读。

## 项目概览
*   **名称**: ZMake
*   **定位**: 适用于**多语言**、**分布式**的**严肃**构建工具。
*   **地址**: https://github.com/moefra/zmake
*   **作者**: Moe Godot (me@kawayi.moe)
*   **项目官方id**: `moe.kawayi`
*   **项目官方jsr scope**: `@zmake`

## 技术栈约束 (Tech Stack Constraints)

### Rust 内核
*   **版本**: Rust Stable (Latest), Edition 2024。拒绝支持旧版本。
*   **架构**:
    *   `zmake_lib`: 核心逻辑库。
    *   `zmake_cli`: 用户交互界面 (UI)。
*   **关键库**:
    *   **JS 引擎**: `rusty_v8` (直接绑定 V8 C++ API)。
    *   **异步**: `tokio` (Runtime), `tracing` (日志/遥测)。
    *   **错误处理**: `thiserror` (Lib层), `eyre` (App层)。
    *   **序列化**: `serde` (JSON), `prost` (Protobuf)。
    *   **网络**: `tonic` (gRPC), `reqwest` (HTTP)。
    *   **哈希**: `xxhash-rust` (快速非加密), `sha2` (安全加密)。
    *   **CLI**: `clap` (v4+)。

### 脚本运行时 (Script Runtime)
*   **策略**: **Sidecar 模式**。
    *   **不使用** `deno_core` 或 `deno_runtime` crate 嵌入代码。
    *   **嵌入二进制**: 在 `zmake` 二进制中内嵌最新的 `deno` 可执行文件。
    *   **运行方式**: 运行时释放 `deno` 到缓存目录，并通过子进程调用执行运维脚本。

## 设计哲学 (Philosophy)

### 1. 去中心化 (Decentralization)
*   无 `std` 或 `core` 等特殊保留字。
*   官方规则包使用 `moe.kawayi:rules_cc` 等反向域名格式，与社区包平权。

### 2. 封闭性与可复现性 (Hermeticity)
*   **输入即定义**: 构建结果仅取决于 Hash 和 Config。
*   **沙箱化**: 严格限制文件访问权限。
*   **Lockfile**: 外部依赖必须锁定哈希。

### 3. 分层运行时 (Layered Runtime)
文件命名与权限有着严格的对应关系：

1.  **定义层 (`ZPROJECT.ts`)**
    *   **职责**: 项目根配置，声明 Workspace 成员。
    *   **权限**: 纯声明式，无 IO，允许 Glob。
2.  **逻辑层 (`ZBUILD.ts`)**
    *   **职责**: 定义构建目标 (Target)。
    *   **权限**: 纯计算，生成构建图，禁止 IO。
3.  **规则层 (`*.zmake.ts`)**
    *   **职责**: 定义 Rule 和 Toolchain。
    *   **权限**: 分析阶段禁止 IO；探测阶段允许访问系统 (需写入 Config)。
4.  **脚本层 (`*.zscript.ts`)**
    *   **职责**: 运维、部署、胶水代码。
    *   **权限**: **全功能 Deno 环境** (通过嵌入的 deno 执行)。

## 互操作性 (Interoperability)
*   **包管理**: 使用 `jsr` 管理 TS 依赖。
*   **远程协议**: 基于 gRPC/Protobuf，目标是兼容 Bazel REAPI。
*   **IDE**: 计划支持 BSP 协议。
*   **CI/CD**: 计划集成 GitHub Actions、GitLab CI 等主流平台。
