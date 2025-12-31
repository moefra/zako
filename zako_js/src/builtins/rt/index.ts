/**
 * This file is must for zmake.
 *
 * It contains things that back zmake(like ZMake specified error)
 */

export class ZakoInternalError extends Error {
    constructor(message: string) {
        super(
            `This is a zako internal error and should be reported as a bug:${message}`,
        );
        this.name = "ZakoInternalError";
    }
}
