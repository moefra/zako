use crate::package::*;
use crate::tests::TEST_INTERNER;

#[test]
fn test_interned_version() {
    let interner = &TEST_INTERNER;
    assert!(InternedVersion::try_parse("1.0.0", interner).is_ok());
    assert!(InternedVersion::try_parse("1.2.3-alpha.1", interner).is_ok());
    assert!(InternedVersion::try_parse("invalid", interner).is_err());
}

#[test]
fn test_interned_group() {
    let interner = &TEST_INTERNER;
    assert!(InternedGroup::try_parse("moe.fra", interner).is_ok());
    assert!(InternedGroup::try_parse("com.example.project", interner).is_ok());
    assert!(InternedGroup::try_parse("my-group", interner).is_ok());
    assert!(InternedGroup::try_parse("", interner).is_err());
    assert!(InternedGroup::try_parse("123.abc", interner).is_err());
}

#[test]
fn test_interned_artifact_id() {
    let interner = &TEST_INTERNER;
    let aid = InternedArtifactId::try_parse("moe.fra:zako", interner).unwrap();
    assert_eq!(interner.resolve(&aid.group.0).unwrap(), "moe.fra");
    assert_eq!(interner.resolve(&aid.name.0).unwrap(), "zako");

    assert!(InternedArtifactId::try_parse("zako", interner).is_err());
    assert!(InternedArtifactId::try_parse("moe.fra:", interner).is_err());
}

#[test]
fn test_interned_package_id() {
    let interner = &TEST_INTERNER;
    let pid = InternedPackageId::try_parse("moe.fra:zako@1.0.0", interner).unwrap();
    assert_eq!(pid.resolved(interner).unwrap(), "moe.fra:zako@1.0.0");

    assert!(InternedPackageId::try_parse("moe.fra:zako", interner).is_err());
    assert!(InternedPackageId::try_parse("moe.fra:zako@invalid", interner).is_err());
}

