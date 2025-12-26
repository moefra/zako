use crate::version_extractor::extract_from_string;
use semver::Version;

#[test]
fn test_extract_from_string() {
    // Normal semver
    assert_eq!(extract_from_string("1.2.3"), Some(Version::new(1, 2, 3)));
    assert_eq!(extract_from_string("v1.2.3"), Some(Version::new(1, 2, 3)));

    // With prefix/suffix
    assert_eq!(extract_from_string("cargo 1.75.0 (ad0f143 2023-12-28)"), Some(Version::new(1, 75, 0)));
    assert_eq!(extract_from_string("git version 2.43.0"), Some(Version::new(2, 43, 0)));

    // Lenient parsing (e.g. 1.75 instead of 1.75.0)
    assert_eq!(extract_from_string("rustc 1.75"), Some(Version::new(1, 75, 0)));

    // None
    assert_eq!(extract_from_string("no version here"), None);
    assert_eq!(extract_from_string(""), None);
}

