
import * as core from "zako:core";
import * as rt from "zako:rt";
import * as semver from "zako:semver";
import * as contextSyscalls from "zako:context";

/**
 * @internal
 */
export interface ProjectSyscalls extends contextSyscalls.ContextSyscall{
    syscall_package_group():string
    syscall_package_artifact():string
    syscall_package_version():string
    syscall_package_config():string | boolean | number | undefined
}

/**
 * @internal
 */
export const syscalls:ProjectSyscalls = contextSyscalls.syscalls as any as ProjectSyscalls;

export interface Project extends core.ProjectMeta {
    description?: string;
    license?: string;
    authors?: core.Author[];
    builds?: core.Pattern;
    rules?: core.Pattern;
    toolchains?: core.Pattern;
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

export function newProject(options: Project): ProjectBuilder{
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

let ver : semver.SemVer|null = semver.parse(syscalls.syscall_package_version(), false, false);

if(ver == null){
    throw new rt.ZakoInternalError(`The version "${ver}" is not valid semver.` +
        `From project "${syscalls.syscall_package_group}:${syscalls.syscall_package_artifact}"`);
}

export const project:core.ProjectMeta = {
    group: syscalls.syscall_package_group(),
    artifact: syscalls.syscall_package_artifact(),
    version: ver,
};

export default project;
