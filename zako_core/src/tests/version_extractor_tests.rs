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
    let cargo = extract_from_string("cargo 1.94.0-nightly (3861f60f6 2025-12-19)").unwrap();
    assert_eq!(cargo.major, 1);
    assert_eq!(cargo.minor, 94);
    assert_eq!(cargo.patch, 0);
    assert_eq!(cargo.pre.as_str(), "nightly");
    assert_eq!(
        extract_from_string("git version 2.50.1 (Apple Git-155)"),
        Some(Version::new(2, 50, 1))
    );
    let msbuild = extract_from_string("17.8.3+195e7f5a3").unwrap();
    assert_eq!(msbuild.major, 17);
    assert_eq!(msbuild.minor, 8);
    assert_eq!(msbuild.patch, 3);
    assert_eq!(msbuild.build.as_str(), "195e7f5a3");
    let msvc = extract_from_string("19.00.24215.1").unwrap();
    assert_eq!(msvc.major, 19);
    assert_eq!(msvc.minor, 0);
    assert_eq!(msvc.patch, 24215);
    assert_eq!(msvc.build.as_str(), "1");
    let win_sdk = extract_from_string("10.0.22621.0").unwrap();
    assert_eq!(win_sdk.major, 10);
    assert_eq!(win_sdk.minor, 0);
    assert_eq!(win_sdk.patch, 22621);
    assert_eq!(win_sdk.build.as_str(), "0");
    assert_eq!(
        extract_from_string("Apple clang version 17.0.0 (clang-1700.4.4.1)"),
        Some(Version::new(17, 0, 0))
    );
    assert_eq!(
        extract_from_string("xcode-select version 2416."),
        Some(Version::new(2416, 0, 0))
    );
    let git_win = extract_from_string("git version 2.43.0.windows.1").unwrap();
    assert_eq!(git_win.major, 2);
    assert_eq!(git_win.minor, 43);
    assert_eq!(git_win.patch, 0);
    assert_eq!(git_win.pre.as_str(), "windows.1");
    let full = extract_from_string("1.2.3-alpha.1+sha.ebf2012").unwrap();
    assert_eq!(full.major, 1);
    assert_eq!(full.minor, 2);
    assert_eq!(full.patch, 3);
    assert_eq!(full.pre.as_str(), "alpha.1");
    assert_eq!(full.build.as_str(), "sha.ebf2012");

    // None
    assert_eq!(extract_from_string("no version here"), None);
    assert_eq!(extract_from_string(""), None);
}
