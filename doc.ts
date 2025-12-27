#! /usr/bin/env bun

import { $ } from "bun";
import { join } from "node:path";
import { existsSync } from "node:fs";
import { rm, mkdir } from "node:fs/promises";

// === 配置区域 ===
const DOMAIN = process.env.ENABLE_RELEASE === "TRUE" ? "https://zako.fra.moe/docs" : "file://" + import.meta.dir + "/public";
const OUTPUT_DIR = "./public";
const PROJECT_DIR = "./target/doc";

async function buildDocs() {
  try {
    console.log(`⚠️ Use "ENABLE_RELEASE=TRUE" to modify the target domain name`)
    console.log(`🚀 Start building documentation, target domain name: ${DOMAIN}`);

    console.log(`🧹 Clean directory: ${OUTPUT_DIR}`);
    await rm(OUTPUT_DIR, { recursive: true, force: true });
    await mkdir(OUTPUT_DIR, { recursive: true });

    console.log("🛠️  Running cargo doc...");

    await $`cargo +nightly doc --color always --release --no-deps --package zako-core`;

    const entryHtml = `
<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Docs - ${DOMAIN}</title></head>
<body>
    <h1>Documentation Root</h1>
    <ul>
        <li><a href="./std/index.html">Standard Library (std)</a></li>
        <li><a href="./core/index.html">Core Library</a></li>
        <li>查看项目中的具体 Crate 目录。</li>
    </ul>
</body>
</html>`;
    // TODO: Add index.html
    // Issue URL: https://github.com/moefra/zako/issues/27
    // await Bun.write(join(OUTPUT_DIR, "index.html"), entryHtml);

    console.log(`\n✅ Build successful!`);
    console.log(`👉 Please deploy the content in the '${OUTPUT_DIR}' directory to ${DOMAIN}`);
  } catch (error) {
    console.error("❌ Error during build:", error);
    process.exit(1);
  }
}

buildDocs();
