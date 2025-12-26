use crate::author::*;
use std::str::FromStr;

#[test]
fn test_author_new() {
    let a = Author::new("Alice", "alice@example.com").unwrap();
    assert_eq!(a.author(), "Alice");
    assert_eq!(a.email(), "alice@example.com");
    assert_eq!(a.get_output_format(), "Alice <alice@example.com>");
}

#[test]
fn test_author_invalid_name() {
    assert!(Author::new("Alice <", "alice@example.com").is_err());
    assert!(Author::new("Alice >", "alice@example.com").is_err());
}

#[test]
fn test_author_from_str() {
    let a = Author::from_str("Bob <bob@example.com>").unwrap();
    assert_eq!(a.author(), "Bob");
    assert_eq!(a.email(), "bob@example.com");

    assert!(Author::from_str("Bob bob@example.com").is_err());
}

#[test]
fn test_author_ord() {
    let a1 = Author::new("Alice", "alice@example.com").unwrap();
    let a2 = Author::new("Bob", "bob@example.com").unwrap();
    let a3 = Author::new("Alice", "z@example.com").unwrap();

    assert!(a1 < a2);
    assert!(a1 < a3);
}
