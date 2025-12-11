# zako Architecture & Context

此文件定义了 zako 项目的核心架构、技术栈约束及设计哲学。AI 助手在生成代码时必须严格遵循此文档。

此文档也供人类阅读。

## 项目概览

* 名称: zako
* 定位: 适用于多语言、分布式的严肃构建工具。
* 地址: <https://github.com/moefra/zako>
* 作者: Moe Godot (<me@kawayi.moe>)
* id: `moe.fra:zako`
* jsr scope: `@zako`
* javascript virtual module prefix:`zako:`

## 技术栈约束

### Rust 内核

* 版本: Rust Stable (Latest), Edition 2024。拒绝支持旧版本。
* 禁止使用`unwrap()`和`expect()`等可能引发 panic 的方法。
* 架构:
  * `zako_core`: 核心逻辑库。
  * `zako_digest`: 哈希和内容寻址存储的protobuf定义。
  * `hone`: 内部依赖图和任务调度引擎。
  * `zako_js`: 核心JS/TS库，还包含类型定义等文件。
  * `zako_cli`: 用户交互界面 (UI)。
* 关键Crates:
  * JS 引擎: `deno_core` (底层是`v8`)。
  * TS 转换: `oxc` 系列 (transformer, parser, codegen, semantic, allocator, span)。
  * Glob引擎：`ignore` crate。
  * 异步: `tokio` (Runtime), `tracing` (日志/遥测)。
  * 错误处理: `thiserror` (Lib层), `eyre` + `color-eyre` (App层)。
  * 序列化: `serde` (JSON), `prost` (Protobuf), `json5` (JSON5)。
  * 网络: `tonic` (gRPC), `reqwest` (HTTP)。
  * 哈希: `xxhash-rust` (快速非加密), `sha2` (安全加密)。
  * 并发: `rayon` (数据并行), `crossbeam` (并发原语), `dashmap` (并发HashMap), `parking_lot` (锁)。
  * 字符串池: `lasso` (多线程字符串intern)。
  * CLI: `clap` (v4+)。
  * 内存分配器: `mimalloc`。

### 脚本运行时

* 构建脚本策略：`v8`支持的快速且强类型可检查的构建脚本
  * 使用`typescript`作为构建脚本，复用现有生态。
  * 使用`v8`，并原生支持多线程构建（通过创建多个`isolate`）
  * 灵感源自区块链/智能合约引擎：禁用了一些来自标准ECMAScript的无关紧要且会造成确定性影响的API来保证js/ts脚本执行结果的可复现性。见`ApiNote.md`。
* 运维脚本策略: Sidecar 模式。
  * 嵌入二进制: 在 `zako` 二进制中内嵌最新的 `bun` 可执行文件，也提供unbundle版本。
  * 运行方式: 运行时释放`bun` 到缓存目录，并通过子进程调用执行运维脚本。

## 设计哲学

### 1. 封闭性与可复现性

* 输入即定义: 构建结果仅取决于 Hash 和 Config。
* 沙箱化: 严格限制文件访问权限。
* Lockfile: 外部依赖必须锁定哈希。
* 信息最小原则：禁止对系统环境变量等信息直接访问，只能添加所需的变量。
* 支持软件工程BOM，可进行license和供应链审查，可获取编译产物可信级别。

### 2. 可伸缩性

* 使用CAS，原生支持远程缓存。
* 原生支持远程构建，本地构建是一种远程构建的特例。

### 3. 分层运行时

文件命名与权限有对应关系：

1. 核心层(`*.ts`)
    * 职责: 提供可在各个层共享的工具代码。
    * 权限: 只能访问核心API，如`zako:core`。
2. 定义层 (`zako.json5` + `zako.ts`)
    * 职责: `zako.json5`:项目根配置（JSON5格式），声明构建选项。`zako.ts`可根据构建选项动态进行添加子项目等操作。
    * 权限: 纯声明式，无 IO，只能提供文件列表或者用于glob的字符串。
3. 逻辑层 (`BUILD.ts`)
    * 职责: 定义构建目标 (Target)。
    * 权限: 纯计算，生成构建图，禁止 IO。可导入`*.rule.ts`使用构建规则。
    * 后续可以支持动态构建图，创建一个动态构建图引擎。
4. 规则层 (`*.rule.ts`)
    * 职责: 定义 Rule。
    * 权限: 禁止 IO。分阶段运行。只能获取抽象的构建工具，提供抽象的构建选项。
5. 工具链层 (`*.toolchain.ts`)
    * 职责: 定义 Toolchain。
    * 权限: 允许使用IO；探测阶段允许访问系统 (访问需写入Config，并遵循最小信息原则)。不能直接访问target，只能获得`rule`提供的构建参数。
6. 脚本层 (`*.script.ts`)
    * 职责: 运维、部署、胶水代码。不在沙箱内执行。
    * 权限: 全功能 Bun 环境 (通过嵌入的 bun 执行)。

Also see `zako_core/lib.rs` file for more docs.

### 4. 去中心化

* 无 `std` 或 `core` 等特殊保留字。
* 官方规则包使用 `moe.fra:xxxx` 等反向域名格式，与社区包平权。
* Battery included，内置一些官方编写的基本规则。

### 5. 标识符与命名规范

#### 包标识符 (Package ID)

格式: `<group>:<artifact_name>`

* `group`: 反向域名格式，由 `.` 分隔的标识符，例如 `moe.fra`, `com.example`
* `artifact_name`: 包名，单个标识符，例如 `zako`, `curl`

示例: `moe.fra:zako`, `com.google:guava`

#### 包版本

遵循 **SemVer 2.0.0** 规范，例如 `1.0.0`, `2.3.1-beta.1`

#### 目标标识符 (InternedId)

格式: `@<package_ref>//<path>:<target>`

* `@<package_ref>`: 包引用，可选，省略表示当前包
  * 格式: `@` + 包的 artifact_name（不含 group）
  * 示例: `@zako`, `@curl`, `@openssl`
  * 当引用当前包时，可省略 `@<package_ref>` 部分

* `<path>`: 目标所在的路径，可为空表示包根路径
  * 由 `/` 分隔的标识符序列
  * 示例: `src/ui/button`, `core`, `` (空字符串表示根路径)
  * 不允许包含 `.` 或 `..`

* `<target>`: 目标名称，必需
  * 单个标识符
  * 示例: `main`, `lib-utils`, `test_suite`

**完整示例:**

* `//:main` - 当前包根路径下的 main 目标
* `//src:main` - 当前包 src 路径下的 main 目标
* `@curl//:main` - curl 包根路径下的 main 目标
* `@curl//src:lib` - curl 包 src 路径下的 lib 目标
* `@openssl//crypto:core` - openssl 包 crypto 路径下的 core 目标

#### 标识符规则

所有标识符（atom, path segment, target name）必须符合以下规则：

* 首字符: Unicode XID_Start 或 `_`
* 后续字符: Unicode XID_Continue 或 `-`
* 不能为空
* 支持多语言（Unicode）标识符

这套规则既保证了与主流编程语言的兼容性，又允许使用 kebab-case 风格的命名。

## 互操作性

* 包管理: 使用 `bun` 管理 TS 依赖。同时不提供`node` API（只提供一些诸如`console.log`等简单的全局确定性API），并且使用lock机制，确保包的可复现性。
* 远程协议: 基于 gRPC/Protobuf，目标是兼容 Bazel REAPI。
* IDE: 计划支持 BSP 协议。计划支持输出`compile_commands.json`。
* 计划支持V8 debugger对构建脚本进行debug。
* CI/CD: 计划集成 GitHub Actions、GitLab CI 等主流平台。
