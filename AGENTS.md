# Zako 仓库上下文（给新建 AI 对话用）

> 用法：把这份 `Context.md` 贴到新对话开头，帮助 AI 快速理解仓库结构、关键约束与常用入口。

## 一句话概览

Zako是一个“后现代”的构建工具：核心引擎与 CLI 用 Rust 实现；构建/配置脚本使用受限的 JS/TS（嵌入 `deno_core`/V8），目标是尽可能 **hermetic（可封闭/可复现）**、可缓存、可并行，并支持远程构建/缓存等能力（部分仍在开发中）。

## 仓库结构（你通常要找的地方）

- `Cargo.toml`：Rust workspace 根配置（edition 2024、nightly、workspace 级 lints）。
- `zako_core/`：核心库（构建引擎、模块加载、V8/deno_core 运行时封装、CAS/worker 池等）。
  - `zako_core/src/lib.rs`：最重要的“系统说明书”，包含脚本文件类型与权限模型。
  - `zako_core/src/engine.rs`：基于 `deno_core::JsRuntime` 的引擎封装。
  - `zako_core/src/builtin/extension/`：内置 `zako:*` 模块的 Deno Extension 注册点（JS 实现在 `dist/builtins/*.js`）。
- `zako_cli/`：CLI 二进制（bin 名：`zako`），入口在 `zako_cli/src/main.rs`。
- `hone/`：通用的增量/依赖图计算引擎（Zako 用它来 `resolve` 计算节点）。
- `zako_*` 其他 crates（大多是基础能力）：
  - `zako_cancel/`：取消/中断（token）。
  - `zako_digest/`：digest / protobuf（被 core 使用）。
  - `zako_interner/`：字符串/ID interner（带持久化相关实现）。
  - `zako_resource/`、`zako_shared/`、`zako_id/`：资源池、共享结构、ID/命名解析等。
  - `zako_kgp/`：Kitty Graphics Protocol（CLI 有把图输出到终端的计划/宣传）。
- `zako_js/`：Zako 的内置 JS/TS 模块与类型声明的“源码包”（Bun 工具链）。
  - 产物输出到仓库根的 `dist/`（被 Rust 扩展引用/内嵌）。
- `dist/`（生成/同步文件夹）：内置模块 JS、TS 类型声明与 tsconfig 模板。
  - `dist/builtins/*.js`：`zako:*` JS 模块实现（由 `zako_js/script/build.ts` 生成）。
  - `dist/types/`：对编辑器/TS 类型检查友好的 `.d.ts` 打包产物（以及一个 npm-style `package.json`）。
  - `dist/template/tsconfig.*.json`：按“文件类型”划分的 tsconfig 模板（用于限制不同脚本的可用模块）。
- `tests/`：示例/用例工程（含 `zako.toml`、`zako.ts`、`BUILD.ts` 等）与 tsconfig 模板副本。
- `ApiNote.md`：JS 运行时禁用的 ECMAScript 标准 API 列表（以 hermetic 为目标）。
- Git submodules（见 `.gitmodules`）：`zako_camino/`、`zako_email_address/` 以及 `zako_js/*` 下的子模块。

## Rust 侧关键约束

- 工具链：`rust-toolchain.toml` 固定 `nightly`（workspace `edition = "2024"`）。
- `zako_core/build.rs` 使用 `tonic_prost_build` 编译 proto：本机构建需要可用的 `protoc`。
- `zako_cli/build.rs` 会在构建时从 GitHub 下载并打包 Bun 可执行文件（写入 `OUT_DIR` 再 `include_bytes!`）：
  - 没有网络时会导致 `zako-cli` 构建失败（除非已命中构建缓存/已有产物）。
  - 环境变量 `ZAKO_FORCE_REDOWNLOAD=true` 可强制重新下载。
  - 代码里也预留了下载 Deno 的逻辑，但目前是注释掉的。
- Workspace lints（根 `Cargo.toml`）：
  - clippy：`unwrap_used/expect_used/panic` 默认 `deny`（`clippy.toml` 允许 tests 中 unwrap）。
  - rustdoc：缺文档/坏链接等为 `deny`（倾向“写代码就要写文档”）。
- 风格：`.editorconfig` 规定 4 空格缩进；代码/注释/文档倾向英文（`zako_core` crate docs 与 `.github/copilot-instructions.md`）.

## CLI（`zako`）速览

入口：`zako_cli/src/main.rs`（Clap）。

主要子命令（当前版本）：

- `make`：构建/解析一个 package（默认 DB：`./.zako/cache.db`），内部会调用 `hone.resolve(ZakoKey::ResolvePackage(...))`。
- `information`：打印构建信息，并输出内置 TypeScript 导出（目前相关导出生成仍在完善中）。
- `generate-complete`：生成 shell completion。
- `v8snapshot`：生成 V8 snapshot（与 feature `v8snapshot` 逻辑有关）。
- `bun` / `bunx`：解压并运行内嵌 Bun（注意：当前实现末尾存在 `panic!("test panic")`，可能是临时调试代码）。

CLI 运行特性：

- 支持 `@argfile`（通过 `argfile` crate 展开参数）。
- 有 “shebang relay to bun” 的分支：若检测到参数里出现 `#!...`，会把参数透传给 bun（同样会触发上面的 panic 问题）。

## JS/TS 构建脚本模型（最重要的概念）

Zako 把不同用途的 TS 文件分为多类，并对可导入的模块/权限做限制（详见 `zako_core/src/lib.rs` 顶部文档与 `zako_core/src/consts.rs` 常量）：

- **library file**：`*.ts`，可被复用；只能 import 其他 library 与核心内置模块（如 `zako:core`）。
- **script file**：`*.script.ts` 或 `scripts/**` 下的脚本；“不受 Zako 处理”，可以访问系统/`node:*`/`Bun`/`Deno`，但 **不能访问 `zako:` 内置模块**。
- **package manifest**：`zako.toml`（项目元数据），不允许 import。
- **package root**：`zako.ts`（定义项目，导出 build/rule/toolchain 等），可 import library、核心内置模块与 `zako:project`（规划中/仍在接入）。
- **build file**：`BUILD.ts`（定义 build targets），可 import library、核心内置模块与 `zako:build`（规划中/仍在接入）。
- **rule file**：`*.rule.ts`（定义规则：处理源文件集合/配置，抽象访问 toolchain），可 import library、核心内置模块与 `zako:rule`（规划中/仍在接入）。
- **toolchain file**：`*.toolchain.ts`（定义工具链：可访问系统，但输入应来自 rule），可 import library、核心内置模块与 `zako:toolchain`（规划中/仍在接入）。
- **config file**：`*.config.ts`（定义配置），可访问系统与 `zako:config`。

TS 类型检查支持：

- `zako_js/script/build.ts` 会生成 `dist/template/tsconfig.*.json`，按上述文件类型划分 include/exclude 并配置 `paths`（把 `zako:*` 指向 `.zako/types`）。
- `zako_js/script/updateTestsTemplate.ts` 会把模板与 `dist/types` 复制到 `tests/` 下，方便示例工程获得类型提示。

## 内置模块（`zako:*`）现状

`zako_core/src/builtin/extension/*.rs` 使用 `deno_core::extension!` 注册内置模块，并从 `dist/builtins/*.js` 作为 ESM 源码：

- 已接入（Rust Extension 有注册）：`zako:rt`、`zako:syscall`、`zako:global`、`zako:semver`、`zako:core`、`zako:console`、`zako:context`、`zako:package`、`zako:config`。
- `ApiNote.md` + `dist/builtins/global.js` 体现了 “禁用非 hermetic API” 的策略（例如：禁用 `Math.random`、`Promise.race`、`Symbol()` 构造器、部分 locale 相关字符串 API 等）。
- `dist/builtins/build.js`、`dist/builtins/rule.js`、`dist/builtins/toolchain.js` 等目前是占位实现（`export default undefined`），且 Rust 侧暂未看到对应 Extension 注册（意味着相关模块能力仍在开发中）。

## 测试/用例（当前更像是示例驱动）

- `tests/` 下每个用例目录通常包含 `argfile`（写入子命令，例如 `make`）与项目文件（如 `zako.toml`、`zako.ts`、`BUILD.ts`）。
- `test.ts`（Deno）与 `tests.ps1`（PowerShell）会遍历 `tests/*` 并执行类似：
  - `zako -C tests/<case> @argfile`
  - 注意：部分脚本/CI 文案里仍有旧命令（如 `zmake` 或 `zako deno ...`）的痕迹，需要以实际 CLI 子命令为准。

## 常用命令（本仓库内）

Rust（需要 nightly + protoc）：

- 构建 CLI：`cargo +nightly build -p zako-cli`
- 运行 CLI：`cargo +nightly run -p zako-cli -- <args>`
- 跑测试：`cargo +nightly test -p zako-core` 或 `cargo +nightly test --workspace`

仓库自带 Bun 脚本（方便开发）：

- `./cli.ts`：`cargo +nightly build -p zako-cli`
- `./run.ts`：`cargo +nightly run -p zako-cli -- ...`
- `./tests.ts` / `./tests_all.ts`：跑 `zako-core` / workspace tests
- `./release.ts`：release 构建（使用 `--artifact-dir=release` 等 nightly/unstable 选项）
- `./doc.ts`：生成 rustdoc 到 `./public`（并提示可用 `ENABLE_RELEASE=TRUE` 改域名）
- `./patch_doc.ts`：给 rustdoc 输出注入自定义 CSS/JS/Logo（读 env：`DOC_DIR/CUSTOM_CSS/CUSTOM_JS/CUSTOM_LOGO`）
- `./export_bindings.ts`：重生成 `zako_core/bindings/*.d.ts`（通过 cargo test `export_bindings`）
- `./fmt.sh`：格式化生成的 bindings（`bun fmt ...`）

JS/TS 内置模块与类型（生成根目录 `dist/`）：

- `cd zako_js && bun run build`
- `cd zako_js && bun run fullBuild`（会额外更新 `tests/` 模板/类型）

