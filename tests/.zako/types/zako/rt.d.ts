/// <reference path="./global.d.ts" />
/**
 * This file is must for zmake.
 *
 * It contains things that back zmake(like ZMake specified error)
 */
export declare class ZakoInternalError extends Error {
    constructor(message: string);
}
