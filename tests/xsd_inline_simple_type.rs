//! Anonymous base and item types (REC-xmlschema-2-20041028 4.1.2): a
//! `<restriction>` names its base either in `base` or as a `<simpleType>`
//! child, a `<list>` its item type either in `itemType` or as a
//! `<simpleType>` child. The child used to be ignored and the type fell back
//! to `string`, so any value passed. xmllint (libxml2) is the reference for
//! every expectation.
use xmloxide::validation::xsd::{parse_xsd, validate_xsd, validate_xsd_strict};
use xmloxide::Document;

/// A restriction of an anonymous `xs:int` restriction, capped at 5.
const RESTRICTION: &str = r#"<xs:simpleType name="R"><xs:restriction>
    <xs:simpleType><xs:restriction base="xs:int"/></xs:simpleType>
    <xs:maxInclusive value="5"/>
  </xs:restriction></xs:simpleType>"#;

/// A list of an anonymous `xs:int` restriction.
const LIST: &str = r#"<xs:simpleType name="L"><xs:list>
    <xs:simpleType><xs:restriction base="xs:int"/></xs:simpleType>
  </xs:list></xs:simpleType>"#;

/// Errors of `<v>{value}</v>` (element) against a schema declaring `v`
/// with the simple type `ty`, both lax and strict.
#[allow(clippy::unwrap_used)]
fn errors(types: &str, ty: &str, value: &str) -> [Vec<String>; 2] {
    let xsd = format!(
        r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  {types}
  <xs:element name="v" type="{ty}"/>
</xs:schema>"#
    );
    let schema = parse_xsd(&xsd).unwrap();
    let doc = Document::parse_str(&format!("<v>{value}</v>")).unwrap();
    [
        validate_xsd(&doc, &schema).errors,
        validate_xsd_strict(&doc, &schema).errors,
    ]
    .map(|errs| errs.into_iter().map(|e| e.message).collect())
}

fn assert_valid(types: &str, ty: &str, value: &str) {
    for (mode, errs) in ["lax", "strict"].iter().zip(errors(types, ty, value)) {
        assert!(errs.is_empty(), "{mode} {value:?}: {errs:?}");
    }
}

fn assert_invalid(types: &str, ty: &str, value: &str) {
    for (mode, errs) in ["lax", "strict"].iter().zip(errors(types, ty, value)) {
        assert!(!errs.is_empty(), "{mode} {value:?} passed");
    }
}

#[test]
fn restriction_inline_base_rejects_non_int() {
    assert_invalid(RESTRICTION, "R", "x");
}

#[test]
fn restriction_inline_base_keeps_facet() {
    assert_invalid(RESTRICTION, "R", "9");
}

#[test]
fn restriction_inline_base_accepts_int() {
    assert_valid(RESTRICTION, "R", "3");
}

/// The whiteSpace in force is the anonymous base's `collapse`, not the
/// `preserve` of the old `string` fallback.
#[test]
fn restriction_inline_base_whitespace_from_inline() {
    assert_valid(RESTRICTION, "R", " 3 ");
}

#[test]
fn list_inline_item_rejects_bad_item() {
    assert_invalid(LIST, "L", "1 x");
}

#[test]
fn list_inline_item_accepts_ints() {
    assert_valid(LIST, "L", "1 2");
}
