# Banned Ecmascript Standard API and API note

Based on `ES2025`.

No `node:` API provide,only `zako:` can use. The api from `zako:` should be limited and hermetic.

Below ecmascript standard APIs are banned.

- whole Date: `Date` will provide unsealing,zako provide a hermetic and cacheable alternative(`zako:timestamp`) for hermetic timestamp and datetime.
- whole Intl: reason same as `Date`
- whole performance: it is useless,we use open telemetry to monitor performance,and it cause script unsealed
- Math.random(): is is useless and cause script unsealed. For hermetic and unique ID and temporary file,provide utility in `zako:unique`.
- whole Crypto - it use random. and the version of non-random crypto is **not-secure**. Use `zako:hash` and `zako:crypto` instead.
- whole FinalizationRegistry: it is useless and cause script unsealed
- whole WeakRef: reason same as `FinalizationRegistry`
- setTimeout: it is useless and cause script unsealed
- setInterval: it is useless and cause script unsealed
- whole SharedArrayBuffer: we do not support `Worker` and we needless to communicate between threads in js(the parallel is processed by rust)
- whole Atomics: reason same as `SharedArrayBuffer`
- Promise.race(): it is useless in building script and cause script unsealed
- constructor of Symbol: when used as map key,it cause unsealing.but other api like `Symbol.for()` and `Symbol.iterator` is not banned. For hermetic and unique ID and temporary file,provide utility in `zako:unique`.

Below APIs should be rewritten or banned:

- String.prototype.localeCompare: rewrite locale-insensitive version
- String.prototype.toLocaleLowerCase: rewrite locale-insensitive version

TODO: rewrite Error.prototype.stack to provide machine-insensitive information

Above APIs are banned in runtime.

TODO: write a lint to apply more rules in script?(provided as **style recommendation**)
