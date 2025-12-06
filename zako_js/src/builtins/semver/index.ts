/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

import * as semver from "semver"

export const SemVer = semver.SemVer;

export const valid = semver.valid;
export const parse = semver.parse;
export const clean: typeof semver.clean = semver.clean;

export const satisfies = semver.satisfies;
export const gt = semver.gt;
export const lt = semver.lt;
export const eq = semver.eq;

export default semver;
