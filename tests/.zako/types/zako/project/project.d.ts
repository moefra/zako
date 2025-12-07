/// <reference path="../global.d.ts" />
import * as core from "zako:core";
export interface Project extends core.ProjectMeta {
    workspaces?: core.Pattern;
    rules?: core.Pattern;
    toolchain?: core.Pattern;
}
export interface ProjectBuilder extends Project {
    addBuild(workspace: core.Pattern | string): void;
    addRule(rule: core.Pattern | string): void;
    addToolchain(toolchain: core.Pattern | string): void;
}
export declare function project(options: Project): ProjectBuilder;
