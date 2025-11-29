const result = await Deno.bundle({
    entrypoints: [`${import.meta.dirname}/rt.ts`],
    outputDir: `${import.meta.dirname}/../dist`,
    platform: "deno",
    minify: true,
    sourceMap: "inline",
    inlineImports: true,
    write: true,
    format: "esm",
});

console.log(result);
