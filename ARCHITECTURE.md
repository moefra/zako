# zako Architecture & Context

This file defines the core architecture, technology stack constraints, and design philosophy of the zako project. AI assistants must strictly adhere to this document when generating code.

This document is also intended for human reading.

## Project Overview

* Name: zako
* Positioning: A serious build tool suitable for multi-language and distributed environments.
* URL: <https://github.com/moefra/zako>
* Author: Moe Godot (<me@kawayi.moe>)
* id: `moe.fra:zako`
* jsr scope: `@zako`
* javascript virtual module prefix: `zako:`

## Tech Stack Constraints

### Rust Kernel

* Version: Rust Stable (Latest), Edition 2024. Old versions are not supported.
* Usage of methods that may cause panics, such as `unwrap()` and `expect()`, is prohibited.
* Architecture:
  * `zako_core`: Core logic library.
  * `zako_js`: Core JS/TS library, also containing type definitions and other files.
  * `zako_cli`: User Interface (UI).
* Key Crates:
  * JS Engine: `deno_core` (backed by `v8`).
  * Glob Engine: `ignore` crate.
  * Async: `tokio` (Runtime), `tracing` (Logging/Telemetry).
  * Error Handling: `thiserror` (Lib layer), `eyre` (App layer).
  * Serialization: `serde` (JSON), `prost` (Protobuf).
  * Network: `tonic` (gRPC), `reqwest` (HTTP).
  * Hashing: `xxhash-rust` (Fast non-cryptographic), `sha2` (Secure cryptographic).
  * CLI: `clap` (v4+).

### Script Runtime

* Build Script Strategy: Fast and strongly-typed checkable build scripts supported by `v8`.
  * Use `typescript` for build scripts to reuse the existing ecosystem.
  * Use `v8` with native support for multi-threaded builds (by creating multiple `isolate`s).
  * Inspired by blockchain/smart contract engines: disabled some insignificant APIs from standard ECMAScript that would cause deterministic issues to ensure reproducibility of JS/TS script execution results. See `ApiNote.md`.
* Operations Script Strategy: Sidecar pattern.
  * Embedded Binary: Embed the latest `bun` executable in the `zako` binary; an unbundled version is also provided.
  * Execution Method: Extract `bun` to the cache directory at runtime and execute operations scripts via subprocess calls.

## Design Philosophy

### 1. Hermeticity & Reproducibility

* Input is Definition: Build results depend only on Hash and Config.
* Sandboxing: Strict restrictions on file access permissions.
* Lockfile: External dependencies must have their hashes locked.
* Principle of Least Information: Direct access to system environment variables and similar information is prohibited; only required variables can be added.
* Supports Software Engineering BOM (SBOM), allowing for license and supply chain audits, and obtaining trust levels of compilation artifacts.

### 2. Scalability

* Uses CAS (Content Addressable Storage), natively supports remote caching.
* Natively supports remote builds; local builds are a special case of remote builds.

### 3. Layered Runtime

File naming corresponds to permissions:

1. Core Layer (`*.ts`)
    * Responsibility: Provides utility code sharable across layers.
    * Permissions: Can only access core APIs, such as `zako:core`.
2. Definition Layer (`zako.json` + `zako.ts`)
    * Responsibility: `zako.json`: Project root configuration, declares build options. `zako.ts`: Can dynamically perform operations like adding sub-projects based on build options.
    * Permissions: Purely declarative, no IO, can only provide file lists or strings used for globs.
3. Logic Layer (`BUILD.ts`)
    * Responsibility: Defines Targets.
    * Permissions: Pure computation, generates the build graph, IO prohibited. Can import `*.rule.ts` to use build rules.
    * A dynamic build graph engine can be supported later to create dynamic build graphs.
4. Rule Layer (`*.rule.ts`)
    * Responsibility: Defines Rules.
    * Permissions: IO prohibited. Runs in stages. Can only acquire abstract build tools and provide abstract build options.
5. Toolchain Layer (`*.toolchain.ts`)
    * Responsibility: Defines Toolchains.
    * Permissions: IO allowed; System access allowed during the probe stage (access requires writing to Config and follows the principle of least information). Cannot access targets directly, can only obtain build parameters provided by `rule`.
6. Script Layer (`*.zscript.ts`)
    * Responsibility: Operations, deployment, glue code. Not executed within the sandbox.
    * Permissions: Full-featured Deno/Bun environment (executed via the embedded deno/bun).

Also see `zako_core/lib.rs` file for more docs.

### 4. Decentralization

* No special reserved words like `std` or `core`.
* Official rule packages use reverse domain name format like `moe.fra:xxxx`, equal in rights to community packages.
* Batteries included, built-in with some basic rules written officially.

## Interoperability

* Package Management: Use `npm` to manage TS dependencies. Does not provide `node` APIs (only provides simple global deterministic APIs like `console.log`), and uses a locking mechanism to ensure reproducibility of packages.
* Remote Protocols: Based on gRPC/Protobuf, aiming to be compatible with Bazel REAPI.
* IDE: Planned support for BSP (Build Server Protocol). Planned support for outputting `compile_commands.json`.
* Debugger: Planned support for V8 debugger to debug build scripts.
* CI/CD: Planned integration with mainstream platforms like GitHub Actions, GitLab CI, etc.
