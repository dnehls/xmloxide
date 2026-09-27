//! Length facets count in the unit of the type (XML Schema Part 2,
//! REC-xmlschema-2-20041028, 4.3.1-4.3.3): characters for strings, items
//! for lists, octets for hexBinary and base64Binary. They used to count
//! UTF-8 bytes of the lexical form. xmllint (libxml2) is the reference
//! for every expectation.
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

fn restricted(base: &str, facet: &str) -> String {
    format!(
        r#"<xs:simpleType name="T">
    <xs:restriction base="{base}">{facet}</xs:restriction>
  </xs:simpleType>"#
    )
}

#[test]
fn string_length_counts_chars() {
    let max = restricted("xs:string", r#"<xs:maxLength value="3"/>"#);
    assert_valid(&max, "T", "äää");
    assert_invalid(&max, "T", "ääää");
    let min = restricted("xs:string", r#"<xs:minLength value="3"/>"#);
    assert_invalid(&min, "T", "ää");
    assert_valid(&min, "T", "äää");
    let len = restricted("xs:string", r#"<xs:length value="2"/>"#);
    assert_valid(&len, "T", "äö");
}

/// `swe:IntegerPair` (SWE Common 2.0 `basic_types.xsd`) restricts an
/// anonymous list of integers to length 2 without a `base` attribute.
#[test]
fn list_length_counts_items() {
    let named = r#"<xs:simpleType name="L"><xs:list itemType="xs:string"/></xs:simpleType>
  <xs:simpleType name="T">
    <xs:restriction base="L"><xs:length value="2"/></xs:restriction>
  </xs:simpleType>"#;
    assert_valid(named, "T", "ab cd");
    assert_invalid(named, "T", "abc");
    let pair = r#"<xs:simpleType name="IntegerPair">
    <xs:restriction>
      <xs:simpleType><xs:list itemType="xs:integer"/></xs:simpleType>
      <xs:length value="2"/>
    </xs:restriction>
  </xs:simpleType>"#;
    assert_valid(pair, "IntegerPair", "1 2");
    assert_invalid(pair, "IntegerPair", "1");
}

#[test]
fn hex_binary_length_counts_octets() {
    let t = restricted("xs:hexBinary", r#"<xs:length value="2"/>"#);
    assert_valid(&t, "T", "0AFF");
    assert_invalid(&t, "T", "0AFF00");
}

#[test]
fn base64_length_counts_octets() {
    let t = restricted("xs:base64Binary", r#"<xs:length value="2"/>"#);
    assert_valid(&t, "T", "AAA=");
    assert_invalid(&t, "T", "AAAA");
}
