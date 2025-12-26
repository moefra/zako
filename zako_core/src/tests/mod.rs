use static_init::dynamic;
use std::sync::Arc;

#[dynamic(lazy)]
pub static TEST_INTERNER: Arc<::zako_interner::ThreadedInterner> =
    { Arc::new(::zako_interner::ThreadedInterner::new().unwrap()) };

pub mod id_tests;
pub mod package_tests;
