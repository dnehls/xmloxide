//! Union types (REC-xmlschema-2-20041028 2.5.1.3): the member types are
//! those named in `memberTypes` followed by the anonymous `<simpleType>`
//! children, in document order, and a value is valid when at least one
//! member accepts it. Anonymous members used to be dropped, so a union made
//! only of them accepted everything. xmllint (libxml2) is the reference for
//! every expectation.
use xmloxide::validation::xsd::{parse_xsd, validate_xsd, validate_xsd_strict};
use xmloxide::Document;

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
fn anonymous_only_union_rejects_non_member() {
    let types = r#"<xs:simpleType name="U"><xs:union>
    <xs:simpleType><xs:restriction base="xs:string"><xs:enumeration value="a"/></xs:restriction></xs:simpleType>
    <xs:simpleType><xs:restriction base="xs:string"><xs:enumeration value="b"/></xs:restriction></xs:simpleType>
  </xs:union></xs:simpleType>"#;
    assert_valid(types, "U", "a");
    assert_valid(types, "U", "b");
    assert_invalid(types, "U", "c");
}

#[test]
fn mixed_union_accepts_anonymous_member() {
    let types = r#"<xs:simpleType name="Enum"><xs:restriction base="xs:string">
    <xs:enumeration value="missing"/><xs:enumeration value="unknown"/>
  </xs:restriction></xs:simpleType>
  <xs:simpleType name="U"><xs:union memberTypes="Enum">
    <xs:simpleType><xs:restriction base="xs:string"><xs:pattern value="other:\w{2,}"/></xs:restriction></xs:simpleType>
  </xs:union></xs:simpleType>"#;
    assert_valid(types, "U", "missing");
    assert_valid(types, "U", "other:ab");
    assert_invalid(types, "U", "other:a");
    assert_invalid(types, "U", "foo");
}

/// `gml:NilReasonEnumeration` (ISO 19136-1:2020 8.2.3.1, SCHEMA
/// basicTypes.xsd:36) is the union of two anonymous simple types.
#[test]
fn nil_reason_enumeration_shape() {
    let types = r#"<xs:simpleType name="NilReasonEnumeration"><xs:union>
    <xs:simpleType><xs:restriction base="xs:string">
      <xs:enumeration value="inapplicable"/><xs:enumeration value="missing"/>
      <xs:enumeration value="template"/><xs:enumeration value="unknown"/>
      <xs:enumeration value="withheld"/>
    </xs:restriction></xs:simpleType>
    <xs:simpleType><xs:restriction base="xs:string"><xs:pattern value="other:\w{2,}"/></xs:restriction></xs:simpleType>
  </xs:union></xs:simpleType>"#;
    assert_valid(types, "NilReasonEnumeration", "withheld");
    assert_valid(types, "NilReasonEnumeration", "other:xy");
    assert_invalid(types, "NilReasonEnumeration", "other:x");
    assert_invalid(types, "NilReasonEnumeration", "bogus");
}

/// Every builtin but `string` and `normalizedString` has whiteSpace
/// `collapse` (REC-xmlschema-2-20041028 4.3.6), also as a union member.
#[test]
fn builtin_int_collapses_whitespace() {
    let types = r#"<xs:simpleType name="U"><xs:union memberTypes="xs:int"/></xs:simpleType>"#;
    assert_valid(types, "xs:int", " 5");
    assert_valid(types, "xs:int", "5\n");
    assert_invalid(types, "xs:int", "5 5");
    assert_valid(types, "U", " 5");
    assert_invalid(types, "U", " x");
}
