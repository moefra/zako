

import * as syscall from "zako:syscall";
import * as semver from "zako:semver";

export class ZakoRuntimeError extends Error {
    constructor(message: string) {
        super(`zako runtime error:${message}`);
        this.name = "ZakoRuntimeError";
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
 * Id with type `target`
 */
export type Target = Id<"target">;

/**
 * Id with type `target_type`
 */
export type TargetType = Id<"target_type">;

/**
 * Id with type `architecture`
 */
export type Architecture = Id<"architecture">;

/**
 * Id with type `os`
 */
export type Os = Id<"os">;

/**
 * Id with type `tool_type`
 */
export type ToolType = Id<"tool_type">;

/**
 * Id with type `tool_name`
 */
export type ToolName = Id<"tool_name">;

let parseVersion = semver.parse(syscall.version,false);

if(parseVersion === null){
    throw new ZakoRuntimeError(`invalid zako version string from syscall.version: ${syscall.version}`);
}

export const version:semver.SemVer = parseVersion;

export function requireZakoVersion(requiredVersion: string | semver.Range): void {
    if (!semver.satisfies(version, requiredVersion)) {
        throw new ZakoRuntimeError(
            `zako version ${version} is required but current version ${version} is not satisified`,
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
 * ZPROJECT.ts
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
