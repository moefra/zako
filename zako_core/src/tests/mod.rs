use static_init::dynamic;
use std::sync::Arc;

#[dynamic(lazy)]
pub static TEST_INTERNER: Arc<::zako_interner::ThreadedInterner> =
    Arc::new(::zako_interner::ThreadedInterner::new().unwrap());

pub mod author_tests;
pub mod blob_range_tests;
pub mod config_value_tests;
pub mod id_tests;
pub mod intern_tests;
pub mod neutral_path_tests;
pub mod package_tests;
pub mod version_extractor_tests;
