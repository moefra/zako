use crate::path::NeutralPath;
use crate::path::PathError;

#[test]
fn test_neutral_path_normalization() {
    // Basic normalization
    assert_eq!(
        NeutralPath::from_path("a/b/c").unwrap().as_ref() as &str,
        "a/b/c"
    );
    assert_eq!(
        NeutralPath::from_path("a//b/c").unwrap().as_ref() as &str,
        "a/b/c"
    );
    assert_eq!(
        NeutralPath::from_path("a/./b").unwrap().as_ref() as &str,
        "a/b"
    );
    assert_eq!(
        NeutralPath::from_path("a/b/..").unwrap().as_ref() as &str,
        "a"
    );
    assert_eq!(
        NeutralPath::from_path("a/../b").unwrap().as_ref() as &str,
        "b"
    );

    // Windows separators
    assert_eq!(
        NeutralPath::from_path("a\\b\\c").unwrap().as_ref() as &str,
        "a/b/c"
    );

    // Current directory
    assert_eq!(NeutralPath::from_path(".").unwrap().as_ref() as &str, ".");
    assert_eq!(NeutralPath::from_path("./.").unwrap().as_ref() as &str, ".");

    // Parent directory at start
    assert_eq!(NeutralPath::from_path("..").unwrap().as_ref() as &str, "..");
    assert_eq!(
        NeutralPath::from_path("../a").unwrap().as_ref() as &str,
        "../a"
    );
    assert_eq!(
        NeutralPath::from_path("../../a").unwrap().as_ref() as &str,
        "../../a"
    );
}

#[test]
fn test_neutral_path_absolute_rejection() {
    // Unix absolute
    assert!(matches!(
        NeutralPath::from_path("/a/b"),
        Err(PathError::PathIsAbsolute())
    ));

    // Windows absolute
    assert!(matches!(
        NeutralPath::from_path("C:\\a"),
        Err(PathError::PathIsAbsolute())
    ));
    assert!(matches!(
        NeutralPath::from_path("\\\\server\\share"),
        Err(PathError::PathIsAbsolute())
    ));
}

#[test]
fn test_neutral_path_invalid_names() {
    // Reserved names (Windows)
    assert!(NeutralPath::from_path("con").is_err());
    assert!(NeutralPath::from_path("aux.txt").is_err());
    assert!(NeutralPath::from_path("com1").is_err());

    // Invalid characters
    assert!(NeutralPath::from_path("a<b").is_err());
    assert!(NeutralPath::from_path("a>b").is_err());
    assert!(NeutralPath::from_path("a:b").is_err());
    assert!(NeutralPath::from_path("a|b").is_err());
    assert!(NeutralPath::from_path("a?b").is_err());
    assert!(NeutralPath::from_path("a*b").is_err());

    // Spaces or dots at the end
    assert!(NeutralPath::from_path("abc ").is_err());
    assert!(NeutralPath::from_path("abc.").is_err());
}

#[test]
fn test_neutral_path_join() {
    let p = NeutralPath::from_path("a/b").unwrap();
    assert_eq!(p.join("c").unwrap().as_ref() as &str, "a/b/c");
    assert_eq!(p.join("../c").unwrap().as_ref() as &str, "a/c");
    assert_eq!(p.join("../../c").unwrap().as_ref() as &str, "c");
    assert_eq!(p.join("../../../c").unwrap().as_ref() as &str, "../c");

    // Joining absolute path should fail
    assert!(p.join("/absolute").is_err());
}

#[test]
fn test_neutral_path_parent_filename_ext() {
    let p = NeutralPath::from_path("a/b/file.txt").unwrap();
    assert_eq!(p.parent().as_ref() as &str, "a/b");
    assert_eq!(p.filename().unwrap(), "file.txt");
    assert_eq!(p.extname().unwrap(), "txt");

    let p2 = NeutralPath::from_path("noext").unwrap();
    assert_eq!(p2.extname(), None);

    let p3 = NeutralPath::from_path(".").unwrap();
    assert_eq!(p3.filename(), None);
}

#[test]
fn test_neutral_path_relative_to() {
    let a = NeutralPath::from_path("src/main.rs").unwrap();
    let b = NeutralPath::from_path("src/utils/helper.rs").unwrap();

    assert_eq!(
        a.get_relative_path_to(&b).unwrap().as_ref() as &str,
        "../utils/helper.rs"
    );
    assert_eq!(
        b.get_relative_path_to(&a).unwrap().as_ref() as &str,
        "../../main.rs"
    );

    let c = NeutralPath::from_path("tests/test.rs").unwrap();
    assert_eq!(
        a.get_relative_path_to(&c).unwrap().as_ref() as &str,
        "../../tests/test.rs"
    );
}

#[test]
fn test_neutral_path_is_in_dir() {
    let dir = NeutralPath::from_path("src").unwrap();
    let file = NeutralPath::from_path("src/main.rs").unwrap();
    let outside = NeutralPath::from_path("tests/test.rs").unwrap();

    assert!(file.is_in_dir(&dir));
    assert!(!outside.is_in_dir(&dir));
    assert!(dir.is_in_dir(&dir)); // A directory is in itself
}
