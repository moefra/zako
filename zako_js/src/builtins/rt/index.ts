/**
 * This file is must for zako.
 *
 * It contains things that caused by zako internally.
 */

export class ZakoInternalError extends Error {
    constructor(message: string) {
        super(
            `This is a zako internal error and should be reported as a bug:${message}`,
        );
        this.name = "ZakoInternalError";
    }
}
