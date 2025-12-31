use crate::id::*;
use crate::tests::TEST_INTERNER;

#[test]
fn test_is_ident() {
    assert!(!is_ident(""));
    assert!(!is_ident("123"));
    assert!(!is_ident("my-var"));
    assert!(is_ident("main"));
    assert!(is_ident("my_var"));
    assert!(is_ident("_my_var"));
    assert!(!is_ident("file."));
    assert!(!is_ident("."));
    assert!(!is_ident(".."));
    assert!(!is_ident("..."));
    assert!(!is_ident("file.txt"));
    assert!(!is_ident("file...txt"));
    assert!(!is_ident(".file"));
    assert!(!is_ident(".f...il...e"));
    assert!(!is_ident(".file."));
}

#[test]
fn test_is_loose_ident() {
    assert!(!is_loose_ident(""));
    assert!(!is_loose_ident("123"));
    assert!(is_loose_ident("my-var"));
    assert!(is_loose_ident("main"));
    assert!(is_loose_ident("my_var"));
    assert!(is_loose_ident("_my_var"));
    assert!(!is_loose_ident("file."));
    assert!(!is_loose_ident("."));
    assert!(!is_loose_ident(".."));
    assert!(!is_loose_ident("..."));
    assert!(!is_loose_ident("file.txt"));
    assert!(!is_loose_ident("file...txt"));
    assert!(!is_loose_ident(".file"));
    assert!(!is_loose_ident(".f...il...e"));
    assert!(!is_loose_ident(".file."));
}

#[test]
fn test_is_more_loose_ident() {
    assert!(!is_more_loose_ident(""));
    assert!(!is_more_loose_ident("123"));
    assert!(is_more_loose_ident("my-var"));
    assert!(is_more_loose_ident("main"));
    assert!(is_more_loose_ident("my_var"));
    assert!(is_more_loose_ident("_my_var"));
    assert!(!is_more_loose_ident("file."));
    assert!(!is_more_loose_ident("."));
    assert!(!is_more_loose_ident(".."));
    assert!(!is_more_loose_ident("..."));
    assert!(is_more_loose_ident("file.txt"));
    assert!(is_more_loose_ident("file...txt"));
    assert!(is_more_loose_ident(".file"));
    assert!(is_more_loose_ident(".f...il...e"));
    assert!(!is_more_loose_ident(".file."));
}

#[test]
fn test_interned_atom() {
    let interner = &TEST_INTERNER;
    assert!(InternedAtom::try_parse("main", interner).is_ok());
    assert!(InternedAtom::try_parse("lib-utils", interner).is_ok());
    assert!(InternedAtom::try_parse("_internal", interner).is_ok());
    assert!(InternedAtom::try_parse("", interner).is_err());
    assert!(InternedAtom::try_parse("123", interner).is_err());
}

#[test]
fn test_interned_path() {
    let interner = &TEST_INTERNER;
    assert!(InternedPath::try_parse("src/ui/button", interner).is_ok());
    assert!(InternedPath::try_parse("core", interner).is_ok());
    assert!(InternedPath::try_parse("", interner).is_ok());
    assert!(InternedPath::try_parse("src/./button", interner).is_err());
    assert!(InternedPath::try_parse("src/../button", interner).is_err());
    assert!(InternedPath::try_parse("src//button", interner).is_err());
    assert!(InternedPath::try_parse("/", interner).is_ok());
    assert!(InternedPath::try_parse("//", interner).is_ok());
    assert!(InternedPath::try_parse("/////", interner).is_ok());
    assert!(InternedPath::try_parse(".", interner).is_err());
    assert!(InternedPath::try_parse("..", interner).is_err());
}

#[test]
fn test_interned_package_ref() {
    let interner = &TEST_INTERNER;
    assert!(InternedPackageRef::try_parse("@zako", interner).is_ok());
    assert!(InternedPackageRef::try_parse("", interner).is_ok());
    assert!(InternedPackageRef::try_parse("@", interner).is_err());
    assert!(InternedPackageRef::try_parse("zako", interner).is_err());
    assert!(InternedPackageRef::try_parse("@zako-core", interner).is_ok());
}

#[test]
fn test_label_parsing() {
    let interner = &TEST_INTERNER;

    let l1 = Label::try_parse("//:main", interner).unwrap();
    assert_eq!(interner.resolve(&l1.package_ref).unwrap(), "");
    assert_eq!(interner.resolve(&l1.path).unwrap(), "");
    assert_eq!(interner.resolve(&l1.target).unwrap(), "main");

    let l2 = Label::try_parse("//src", interner).unwrap();
    assert_eq!(interner.resolve(&l2.path).unwrap(), "src");
    assert_eq!(interner.resolve(&l2.target).unwrap(), "src");

    let l3 = Label::try_parse("@curl//src:lib", interner).unwrap();
    assert_eq!(interner.resolve(&l3.package_ref).unwrap(), "curl");
    assert_eq!(interner.resolve(&l3.path).unwrap(), "src");
    assert_eq!(interner.resolve(&l3.target).unwrap(), "lib");

    let l4 = Label::try_parse("@curl//crypto", interner).unwrap();
    assert_eq!(interner.resolve(&l4.path).unwrap(), "crypto");
    assert_eq!(interner.resolve(&l4.target).unwrap(), "crypto");
}

#[test]
fn test_label_resolved() {
    let interner = &TEST_INTERNER;
    let l = Label::try_parse("@curl//src:lib", interner).unwrap();
    assert_eq!(l.resolved(interner).unwrap(), "curl//src:lib");
}
