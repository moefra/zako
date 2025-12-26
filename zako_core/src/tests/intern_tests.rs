use crate::intern::*;
use crate::tests::TEST_INTERNER;

#[test]
fn test_interned_absolute_path() {
    let interner = &TEST_INTERNER;
    // Using a path that is absolute on most systems for test
    #[cfg(unix)]
    let abs_path = "/usr/bin/cargo";
    #[cfg(windows)]
    let abs_path = r"C:\Windows\System32\cmd.exe";

    let rel_path = "src/main.rs";

    // Test new
    let p1 = InternedAbsolutePath::new(abs_path, interner).unwrap();
    assert!(p1.is_some());

    let p2 = InternedAbsolutePath::new(rel_path, interner).unwrap();
    assert!(p2.is_none());

    // Test from_interned
    if let Some(p) = p1 {
        let p3 = InternedAbsolutePath::from_interned(p.interned, interner).unwrap();
        assert!(p3.is_some());
        assert_eq!(p3.unwrap().interned, p.interned);
    }
}
