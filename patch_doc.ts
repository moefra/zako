import { $ } from "bun";
import { join } from "node:path";
import { existsSync, readdirSync } from "node:fs";

/**
 * 配置项：通过环境变量读取，如果不提供则跳过对应步骤
 */
const CONFIG = {
  DOC_DIR: process.env.DOC_DIR || "./public",
  CSS_FILE: process.env.CUSTOM_CSS || "./theme/custom.css",
  JS_FILE: process.env.CUSTOM_JS || "./theme/custom.js",
  LOGO_FILE: process.env.CUSTOM_LOGO || "./theme/logo.svg",
};

async function patchDocs() {
  console.log("🛠️ 开始应用样式补丁...");

  if (!existsSync(CONFIG.DOC_DIR)) {
    console.error(`❌ 错误: 找不到文档目录 "${CONFIG.DOC_DIR}"`);
    process.exit(1);
  }

  // 1. 寻找 static.files 目录 (rustdoc 存放全局静态资源的文件夹)
  const staticFilesDir = readdirSync(CONFIG.DOC_DIR).find((f) =>
    f.startsWith("static.files")
  );

  if (!staticFilesDir) {
    console.error("❌ 错误: 在文档目录中未找到 static.files 文件夹。");
    process.exit(1);
  }

  const staticPath = join(CONFIG.DOC_DIR, staticFilesDir);
  console.log(`📂 目标静态目录: ${staticPath}`);

  // 2. 注入自定义 CSS
  if (existsSync(CONFIG.CSS_FILE)) {
    const customCss = await Bun.file(CONFIG.CSS_FILE).text();
    const targetCss = join(staticPath, "rustdoc.css");
    if (existsSync(targetCss)) {
      const originalCss = await Bun.file(targetCss).text();
      await Bun.write(targetCss, originalCss + "\n/* Custom Patch */\n" + customCss);
      console.log("✅ 已注入 CSS 样式");
    }
  }

  // 3. 注入自定义 JS
  if (existsSync(CONFIG.JS_FILE)) {
    const customJs = await Bun.file(CONFIG.JS_FILE).text();
    // main.js 是 rustdoc 的主逻辑
    const targetJs = join(staticPath, "main.js");
    if (existsSync(targetJs)) {
      const originalJs = await Bun.file(targetJs).text();
      await Bun.write(targetJs, originalJs + "\n/* Custom Patch */\n" + customJs);
      console.log("✅ 已注入 JS 脚本");
    }
  }

  // 4. 替换 Logo
  if (existsSync(CONFIG.LOGO_FILE)) {
    // Rustdoc 通常生成多个 logo 文件名，包含 rust-logo-xxxx.svg
    const files = readdirSync(staticPath);
    const logoFiles = files.filter(
      (f) => f.startsWith("rust-logo-") && f.endsWith(".svg")
    );

    for (const logoFile of logoFiles) {
      const targetPath = join(staticPath, logoFile);
      await $`cp ${CONFIG.LOGO_FILE} ${targetPath}`;
      console.log(`✅ 已替换 Logo: ${logoFile}`);
    }

    // 同时也替换 favicon (可选)
    const favicon = files.find(f => f.startsWith("favicon-") && f.endsWith(".svg"));
    if (favicon) {
      await $`cp ${CONFIG.LOGO_FILE} ${join(staticPath, favicon)}`;
    }
  }

  console.log("✨ 所有补丁应用完成！");
}

patchDocs();
