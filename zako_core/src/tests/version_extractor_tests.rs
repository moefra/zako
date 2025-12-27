use crate::version_extractor::extract_from_string;
use semver::Version;

#[test]
fn test_extract_from_string() {
    // Normal semver
    assert_eq!(extract_from_string("1.2.3"), Some(Version::new(1, 2, 3)));
    assert_eq!(extract_from_string("v1.2.3"), Some(Version::new(1, 2, 3)));

    // With prefix/suffix
    assert_eq!(
        extract_from_string("Python 3.13.7"),
        Some(Version::new(3, 13, 7))
    );
    assert_eq!(
        extract_from_string("cargo 1.94.0-nightly (3861f60f6 2025-12-19)"),
        Some(Version::new(1, 94, 0))
    );
    assert_eq!(
        extract_from_string("git version 2.50.1 (Apple Git-155)"),
        Some(Version::new(2, 50, 1))
    );
    assert_eq!(
        extract_from_string("Apple clang version 17.0.0 (clang-1700.4.4.1)"),
        Some(Version::new(17, 0, 0))
    );
    assert_eq!(
        extract_from_string("xcode-select version 2416."),
        Some(Version::new(2416, 0, 0))
    );

    // None
    assert_eq!(extract_from_string("no version here"), None);
    assert_eq!(extract_from_string(""), None);
}
