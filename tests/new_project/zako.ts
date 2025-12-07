
import { project } from "zako:project";
import def from "./zako.json" with { type: "json" };

project(def as any);

const myProject = project({
    group: "fra.moe",
    artifact: "zako_test",
    version: "1.0.0",
    description: "An example zako project",
    license: "MIT",
    authors: ["MoeGodot <me@kawayi.moe>"],
    workspaces: ["src/**","tests/**"],
});

myProject.addBuild("src/**");
myProject.addRule({
    include: ["rules/**/*.ts"],
    exclude: ["rules/test/*.ts"],
});
