use crate::blob_range::*;

#[test]
fn test_blob_range_new() {
    let r = BlobRange::new(10, Some(20)).unwrap();
    assert_eq!(r.start(), 10);
    assert_eq!(r.length(), Some(20));
    assert_eq!(r.end(), Some(30));

    let r_full = BlobRange::new(0, None).unwrap();
    assert_eq!(r_full.start(), 0);
    assert_eq!(r_full.length(), None);
    assert_eq!(r_full.end(), None);

    assert!(BlobRange::new(10, Some(0)).is_err());
}

#[test]
fn test_blob_range_is_out_of() {
    let r = BlobRange::new(10, Some(20)).unwrap();
    assert!(!r.is_out_of(30));
    assert!(r.is_out_of(29));
    assert!(r.is_out_of(5));

    let r_full = BlobRange::full();
    assert!(!r_full.is_out_of(100));
    let r_from = BlobRange::new(50, None).unwrap();
    assert!(!r_from.is_out_of(100));
    assert!(r_from.is_out_of(40));
}

#[test]
fn test_blob_range_from_ops() {
    let r1: BlobRange = (100..).into();
    assert_eq!(r1.start(), 100);
    assert_eq!(r1.length(), None);

    let r2: BlobRange = (..).into();
    assert_eq!(r2.start(), 0);
    assert_eq!(r2.length(), None);
}
