// @ts-ignore
import * as semver from "semver"

export type SemVer = semver.SemVer;
export type ReleaseType = semver.ReleaseType;
export type Range = semver.Range;
export type Comparator = semver.Comparator;
export type Options = semver.Options;

export const SEMVER_SPEC_VERSION: typeof semver.SEMVER_SPEC_VERSION = semver.SEMVER_SPEC_VERSION;

export const parse: typeof semver.parse = semver.parse;
export const valid: typeof semver.valid = semver.valid;
export const clean: typeof semver.clean = semver.clean;
export const inc: typeof semver.inc = semver.inc;
export const diff: typeof semver.diff = semver.diff;
export const major: typeof semver.major = semver.major;
export const minor: typeof semver.minor = semver.minor;
export const patch: typeof semver.patch = semver.patch;
export const prerelease: typeof semver.prerelease = semver.prerelease;
export const compare: typeof semver.compare = semver.compare;
export const rcompare: typeof semver.rcompare = semver.rcompare;
export const compareLoose: typeof semver.compareLoose = semver.compareLoose;
export const compareBuild: typeof semver.compareBuild = semver.compareBuild;
export const sort: typeof semver.sort = semver.sort;
export const rsort: typeof semver.rsort = semver.rsort;
export const gt: typeof semver.gt = semver.gt;
export const lt: typeof semver.lt = semver.lt;
export const eq: typeof semver.eq = semver.eq;
export const neq: typeof semver.neq = semver.neq;
export const gte: typeof semver.gte = semver.gte;
export const lte: typeof semver.lte = semver.lte;
export const cmp: typeof semver.cmp = semver.cmp;
export const coerce: typeof semver.coerce = semver.coerce;
export const satisfies: typeof semver.satisfies = semver.satisfies;
export const toComparators: typeof semver.toComparators = semver.toComparators;
export const maxSatisfying: typeof semver.maxSatisfying = semver.maxSatisfying;
export const minSatisfying: typeof semver.minSatisfying = semver.minSatisfying;
export const minVersion: typeof semver.minVersion = semver.minVersion;
export const validRange: typeof semver.validRange = semver.validRange;
export const outside: typeof semver.outside = semver.outside;
export const gtr: typeof semver.gtr = semver.gtr;
export const ltr: typeof semver.ltr = semver.ltr;
export const intersects: typeof semver.intersects = semver.intersects;
export const simplifyRange: typeof semver.simplifyRange = semver.simplifyRange;
export const subset: typeof semver.subset = semver.subset;
