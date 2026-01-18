use camino::Utf8Path;

use crate::intern::*;
use crate::tests::TEST_INTERNER;

#[test]
fn test_interned_absolute_path() {
    let interner = &TEST_INTERNER;
    // Using a path that is absolute on most systems for test
    #[cfg(unix)]
    let abs_path = Utf8Path::new("/usr/bin/cargo");
    #[cfg(windows)]
    let abs_path = Utf8Path::new(r"C:\Windows\System32\cmd.exe");

    let rel_path = Utf8Path::new("src/main.rs");

    // Test new
    let p1 = InternedAbsolutePath::new(abs_path, interner);
    assert!(p1.is_ok());
    let p1 = p1.unwrap();

    let p2 = InternedAbsolutePath::new(rel_path, interner);
    assert!(p2.is_err());

    // Test from_interned
    let p3 = InternedAbsolutePath::from_interned(*p1.as_ref(), interner).unwrap();
    assert!(p3.is_some());
    assert_eq!(p3.unwrap(), p1);
}
