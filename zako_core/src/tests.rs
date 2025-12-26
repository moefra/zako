cfg_if::cfg_if! {
    if #[cfg(test)] {
        use std::sync::Arc;
        use static_init::dynamic;

        #[dynamic(lazy)]
        pub static TEST_INTERNER: Arc<::zako_interner::ThreadedInterner> = {
            Arc::new(::zako_interner::ThreadedInterner::new().unwrap())
        };
    }
}
