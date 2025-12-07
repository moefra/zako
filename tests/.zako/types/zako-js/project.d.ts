import * as core from "zako:core";
export interface Project extends core.ProjectMeta {
    workspaces?: core.Pattern;
    rules?: core.Pattern;
}
export interface ProjectBuilder extends Project {
    addWorkspace(workspace: core.Pattern): void;
    addRule(rule: core.Pattern): void;
}
export declare function project(options: Project): ProjectBuilder;
