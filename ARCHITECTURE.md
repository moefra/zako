# zako 架构设计文档 (ARCHITECTURE.md)

此文档旨在为熟悉 Rust 且对构建系统有一定了解的开发者提供 `zako` 项目的深度架构解析。

## 1. 项目愿景与设计理念

`zako` 是一个旨在解决多语言、大规模、分布式构建问题的现代构建系统。它的设计深受 Bazel、Buck2 及智能合约引擎的启发，核心理念包括：

*   **绝对确定性 (Determinism)**：同样的输入（代码、配置、环境）必须产生同样的输出。
*   **封闭性 (Hermeticity)**：构建过程在沙盒中运行，严格控制对文件系统和网络的访问。
*   **内容寻址存储 (CAS)**：所有构建产物和输入均通过内容哈希进行标识，原生支持远程缓存。
*   **混合执行模型**：使用 Rust 编写高性能内核，使用 TypeScript/V8 编写灵活且可验证的构建逻辑。

---

## 2. 核心组件总览

项目采用多 Crate 架构，各司其职：

| Crate | 职责 |
| :--- | :--- |
| `zako_core` | **系统内核**。负责 V8 运行时管理、模块加载、CAS 存储协议、沙盒执行及项目解析。 |
| `hone` | **任务调度引擎**。一个通用的递归式计算引擎，将构建任务建模为有向无环图 (DAG)，支持增量计算。 |
| `zako_cli` | **命令行界面**。用户交互入口，负责初始化构建环境并触发引擎计算。 |
| `zako_digest` | **哈希与序列化**。定义了系统通用的 Protobuf 协议和高性能哈希逻辑。 |
| `zako_js` | **脚本内置库**。为构建脚本提供的 TypeScript 类型定义及 API 实现（如 `zako:core`）。 |

---

## 3. 计算模型：Hone 引擎

`zako` 的计算核心是 `hone`。在 `zako` 中，每一个构建步骤都被抽象为一个 `Key` 到 `Value` 的映射过程。

*   **ZakoKey**：描述一个计算任务。例如 `ResolveProject { path }` 或 `TranspileTs { code }`。
*   **ZakoValue**：计算的结果。
*   **Computer**：Rust 侧实现的执行逻辑。

这种模型使得 `zako` 能够天然支持：
1.  **并行化**：不相关的 Key 可以并行计算。
2.  **记忆化**：计算结果会被缓存，如果输入 Key 不变且依赖项未更新，则直接返回缓存值。

---

## 4. 脚本运行时：确定性 V8

`zako` 选择 TypeScript 作为构建语言，但为了保证确定性，我们对 V8 环境进行了深度定制：

1.  **禁用副作用 API**：移除或模拟了 `Date.now()`、`Math.random()` 等会导致非确定性结果的 API。
2.  **多 Isolate 隔离**：每个项目的解析运行在独立的 V8 Isolate 中，确保互不干扰并支持高效并行。
3.  **分层权限体系**：
    *   **定义层 (`zako.ts`)**：声明项目元数据，只能访问 `zako:project`。
    *   **逻辑层 (`BUILD.ts`)**：定义具体的构建目标（Target），禁止任何 IO。
    *   **工具链层 (`*.toolchain.ts`)**：唯一允许受控 IO 的层级，用于探测系统环境或执行本地命令。

---

## 5. 值得关注的 API 与示例

### 5.1 Rust 侧：项目元数据 (`crate::project::Project`)

在 Rust 内核中，`Project` 结构体承载了项目的核心定义。

```rust
// 位于 zako_core/src/project.rs
pub struct Project {
    pub group: String,           // 包组名，如 "moe.fra"
    pub artifact: SmolStr,       // 项目名，如 "zako"
    pub version: String,        // 符合 SemVer 的版本号
    pub builds: Option<Pattern>, // 构建脚本所在的路径模式
    pub dependencies: Option<HashMap<SmolStr, PackageSource>>, // 外部依赖
    pub config: Option<HashMap<SmolStr, Config>>,             // 项目自定义配置
}
```

### 5.2 TypeScript 侧：定义项目 (`zako.ts`)

用户通过 TypeScript 导出一个 `Project` 实例来定义项目：

```typescript
import { project } from "zako:project";
import * as core from "zako:core";

// 这是一个 zako.ts 示例
const p = project({
    group: "com.example",
    artifact: "hello-world",
    version: "1.0.0",
    config: {
        "enable_feature_x": true
    }
});

// 可以动态添加构建路径
if (core.os === "linux") {
    p.addBuild("./linux_specific/**/*");
}

export default p;
```

### 5.3 TypeScript 侧：定义目标 (`BUILD.ts`)

在 `BUILD.ts` 中，用户利用规则创建具体的构建目标：

```typescript
import { ccBinary, ccLibrary } from "zako:rule";

const lib = ccLibrary({
    name: "utils",
    srcs: ["src/utils.cpp"],
    hdrs: ["include/utils.h"],
});

export default ccBinary({
    name: "app",
    srcs: ["src/main.cpp"],
    deps: [lib],
});
```

---

## 6. 当前项目状态

目前 `zako` 处于积极开发阶段：

*   ✅ **已完成**：
    *   Rust 内核与 V8 运行时的集成。
    *   基础的 `hone` 计算引擎实现。
    *   TypeScript 到 JavaScript 的实时转译（基于 `oxc`）。
    *   基于内容的路径实习（String Interning）与标识符解析。
*   🚧 **进行中**：
    *   完善内置规则（C++, Rust, JS 等）。
    *   实现完整的 CAS 远程同步协议。
    *   沙盒环境（Sandbox）的跨平台加固。
*   📅 **计划中**：
    *   IDE 支持 (BSP 协议)。
    *   分布式集群调度算法优化。

---

## 7. 开发者指南

如果你想为 `zako` 贡献代码，建议按照以下顺序阅读：
1.  `zako_core/src/lib.rs`：了解模块组织结构。
2.  `hone/src/lib.rs`：理解核心调度逻辑。
3.  `zako_core/src/worker/`：了解 Rust 如何驱动 V8 执行脚本。
4.  `zako_js/src/builtins/`：查看为用户提供的 API 是如何定义的。

---
*Stay zako, build better.*
