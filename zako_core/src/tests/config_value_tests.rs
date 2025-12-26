use crate::config_value::*;
use crate::id::Label;
use crate::tests::TEST_INTERNER;

#[test]
fn test_resolved_config_value_resolve() {
    let interner = &TEST_INTERNER;

    // Label
    let label = Label::try_parse("//src:lib", interner).unwrap();
    let rcv_label = ResolvedConfigValue::Label(label);
    let cv_label = rcv_label.resolve(interner).unwrap();
    assert!(matches!(cv_label.r#type, ConfigType::Label));
    if let Some(ConfigDefault::Label(s)) = cv_label.default {
        assert_eq!(s, "//src:lib");
    } else {
        panic!("Expected Label default");
    }

    // String
    let rcv_str = ResolvedConfigValue::String("hello".into());
    let cv_str = rcv_str.resolve(interner).unwrap();
    assert!(matches!(cv_str.r#type, ConfigType::String));
    if let Some(ConfigDefault::String { string }) = cv_str.default {
        assert_eq!(string, "hello");
    } else {
        panic!("Expected String default");
    }

    // Boolean
    let rcv_bool = ResolvedConfigValue::Boolean(true);
    let cv_bool = rcv_bool.resolve(interner).unwrap();
    assert!(matches!(cv_bool.r#type, ConfigType::Boolean));
    assert_eq!(cv_bool.default, Some(ConfigDefault::Boolean(true)));

    // Number
    let rcv_num = ResolvedConfigValue::Number(42);
    let cv_num = rcv_num.resolve(interner).unwrap();
    assert!(matches!(cv_num.r#type, ConfigType::Number));
    assert_eq!(cv_num.default, Some(ConfigDefault::Number(42)));
}

