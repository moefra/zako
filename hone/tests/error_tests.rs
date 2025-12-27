use hone::assert as hone_assert;
use hone::error::HoneError;

#[test]
fn test_assert_macro() {
    fn check_assert(val: bool) -> Result<(), HoneError> {
        hone_assert!("value must be true", val);
        Ok(())
    }

    assert!(check_assert(true).is_ok());
    let err = check_assert(false);
    assert!(err.is_err());
    if let Err(HoneError::AssertionFailed(msg, cond)) = err {
        assert_eq!(msg, "value must be true");
        assert_eq!(cond, "val");
    } else {
        panic!("Expected AssertionFailed error");
    }
}
