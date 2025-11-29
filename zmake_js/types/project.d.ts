import * as core from "zmake:core";

declare module "zmake:project" {
    export interface Project extends core.ProjectMeta {
        workspaces?: core.Pattern;
        rules?: core.Pattern;
    }

    export interface ProjectBuilder extends Project {
        addWorkspace(workspace: core.Pattern): void;
        addRule(rule: core.Pattern): void;
    }

    export function project(options: Project): ProjectBuilder;
}
