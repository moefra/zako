use ::deno_core::v8::{self, Local, ObjectTemplate, PinScope};

deno_core::extension!(
    zako_global,
    deps = [zako_syscall],
    esm_entry_point = "zako:global",
    esm = ["zako:global" = "../dist/builtins/global.js"],
    global_template_middleware = remove_api,
    docs = "The extension that provide necessary global for zako",
);

fn remove_api<'s, 'i>(
    scope: &mut PinScope<'s, 'i, ()>,
    template: Local<'s, ObjectTemplate>,
) -> Local<'s, ObjectTemplate> {
    let undefined = v8::undefined(scope);

    let banned_apis = [
        "Date",
        "Intl",
        "performance",
        "setTimeout",
        "setInterval",
        "Crypto",
        "FinalizationRegistry",
        "WeakRef",
        "SharedArrayBuffer",
        "Atomics",
        "CryptoKey",
        "CryptoKeyPair",
    ];

    for api_name in banned_apis {
        let key = v8::String::new(scope, api_name).unwrap();
        template.set(key.into(), undefined.into());
    }

    template
}
