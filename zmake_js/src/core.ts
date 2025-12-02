import * as syscall from "zmake:syscall";
import * as semver from "zmake:semver";

export class ZMakeRuntimeError extends Error {
    constructor(message: string) {
        super(`zmake runtime error:${message}`);
        this.name = "ZMakeRuntimeError";
    }
}

/**
 * core version string of semver 2.0
 *
 * Including major.minor.patch only.
 */
export type VersionCore = `${number}.${number}.${number}`;

type PreRelease = `-${string}`;
type BuildVersion = `+${string}`;

/**
 * full version string of semver 2.0
 *
 * including version core, prerelease and build.
 */
export type Version = `${VersionCore}${PreRelease | ""}${BuildVersion | ""}`;

export type GroupId = `${string}`;
export type ArtifactId = `${GroupId}:${string}`;
export type QualifiedArtifactId = `${ArtifactId}@${Version}`;

type Id<Str extends string> = `${QualifiedArtifactId}#${Str}::${string}`;

/**
 * zmake Id with type `target`
 */
export type Target = Id<"target">;

/**
 * zmake Id with type `target_type`
 */
export type TargetType = Id<"target_type">;

/**
 * zmake Id with type `architecture`
 */
export type Architecture = Id<"architecture">;

/**
 * zmake Id with type `os`
 */
export type Os = Id<"os">;

/**
 * zmake Id with type `tool_type`
 */
export type ToolType = Id<"tool_type">;

/**
 * zmake Id with type `tool_name`
 */
export type ToolName = Id<"tool_name">;

export const version: semver.SemVer = new semver.SemVer(
    syscall.version as Version,
    false,
);

export function requireZMakeVersion(requiredVersion: string): void {
    if (!semver.satisfies(version, requiredVersion, false)) {
        throw new ZMakeRuntimeError(
            `zmake version ${version} is required but current version ${version} is not satisified`,
        );
    }
}

export type visibility = "public" | "private" | string[];

export type transitiveLevel = "public" | "private" | "interface";

/**
 * git style author sign
 */
type Author = `${string} <${string}@${string}>`;

/**
 * project.zmake
 */
export interface ProjectMeta {
    group: GroupId;
    artifact: string;
    version: Version;
    description?: string;
    license?: string;
    authors?: Author[];
}

/**
 * A pattern to include and exclude files.
 *
 * If there is a string array, it will be treated as a list of files to include.
 */
export type Pattern =
    | {
          include?: string[];
          exclude?: string[];
      }
    | string[];

export function trace(message: string): void {
    return syscall.log("trace", message);
}
export function debug(message: string): void {
    return syscall.log("debug", message);
}
export function info(message: string): void {
    return syscall.log("info", message);
}
export function warn(message: string): void {
    return syscall.log("warn", message);
}
export function error(message: string): void {
    return syscall.log("error", message);
}
