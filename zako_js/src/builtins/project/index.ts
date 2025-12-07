
import * as core from "zako:core";

export interface Project extends core.ProjectMeta {
    builds?: core.Pattern;
    rules?: core.Pattern;
    toolchain?: core.Pattern;
    options?: core.OptionsDeclaration[];
}

/** The created means that the project's options is finalized and cannot be changed.
 * This is prevent some nt user writing code look like:
 * ```ts
 * let proj = project({...});
 * if(config.os == "windows"){ proj.addOptions("use_unicode_api"); }
 * ```
 * The options should always be defined and just throw an error instead of changing options in different platform.
 *
 * It should still be return to runtime as default to registered.
 * In another way, it is not ***registered project***.
 **/
export interface CreatedProject extends core.ProjectMeta {
    builds?: core.Pattern;
    rules?: core.Pattern;
    toolchain?: core.Pattern;
    readonly options?: core.OptionsDeclaration[];
}

export interface ProjectBuilder extends CreatedProject {
    addBuild(workspace: core.Pattern | string): void;
    addRule(rule: core.Pattern | string): void;
    addToolchain(toolchain: core.Pattern | string): void;
}

export function project(options: Project): ProjectBuilder{
    let proj:ProjectBuilder = {
        ...options,
        addBuild(workspace: core.Pattern | string): void {
            if (typeof workspace === "string") {
                workspace = [workspace];
            }
            this.builds = core.appendPattern(this.builds,workspace);
        },
        addRule(rule: core.Pattern | string): void {
            if (typeof rule === "string") {
                rule = [rule];
            }
            this.rules = core.appendPattern(this.rules,rule);
        },
        addToolchain(toolchain: core.Pattern | string): void {
            if (typeof toolchain === "string") {
                toolchain = [toolchain];
            }
            this.toolchain = core.appendPattern(this.toolchain,toolchain);
        }
    };
    Object.defineProperty(proj, "options", {
        value: proj.options,
        writable: false
    });
    return proj;
}
