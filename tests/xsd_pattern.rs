//! Pattern facets follow the regular expressions of XML Schema Part 2
//! (REC-xmlschema-2-20041028, Appendix F): the whole value must match,
//! `^` and `$` are literals, `.` is `[^\n\r]`, `\d` is `\p{Nd}`, `\w`
//! excludes punctuation (`_` included), separators and others, and
//! quantifiers count characters, not bytes. Patterns with groups,
//! alternation, counted quantifiers or escaped sets used to count as
//! matched. xmllint (libxml2) is the reference for every expectation.
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

fn one_pattern(pattern: &str) -> String {
    format!(
        r#"<xs:simpleType name="T">
    <xs:restriction base="xs:string"><xs:pattern value="{pattern}"/></xs:restriction>
  </xs:simpleType>"#
    )
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

const UOM: &str = r#"<xs:simpleType name="UomSymbol">
    <xs:restriction base="xs:string"><xs:pattern value="[^: \n\r\t]+"/></xs:restriction>
  </xs:simpleType>
  <xs:simpleType name="UomURI">
    <xs:restriction base="xs:anyURI">
      <xs:pattern value="([a-zA-Z][a-zA-Z0-9\-\+\.]*:|\.\./|\./|#).*"/>
    </xs:restriction>
  </xs:simpleType>
  <xs:simpleType name="UomIdentifier">
    <xs:union memberTypes="UomSymbol UomURI"/>
  </xs:simpleType>"#;

/// `gml:UomIdentifier` (ISO 19136-1:2020 8.2.3.6) is the union of `UomSymbol`
/// and `UomURI`; a value with a space fits neither.
#[test]
fn uom_identifier_union_rejects_space() {
    for ok in ["m", "m/s", "a:b", "#m", "urn:adv:uom:rad"] {
        assert_valid(UOM, "UomIdentifier", ok);
    }
    for bad in ["a b", ""] {
        assert_invalid(UOM, "UomIdentifier", bad);
    }
}

#[test]
fn uom_symbol_set_escapes() {
    assert_valid(UOM, "UomSymbol", "m");
    assert_invalid(UOM, "UomSymbol", "a:b");
    assert_invalid(UOM, "UomSymbol", "a&#9;b");
}

#[test]
fn counted_quantifier_word() {
    let t = one_pattern(r"other:\w{2,}");
    for ok in ["other:äö", "other:12"] {
        assert_valid(&t, "T", ok);
    }
    for bad in ["other:a", "other:a_b", "other:a-b"] {
        assert_invalid(&t, "T", bad);
    }
}

#[test]
fn mime_group_alternation() {
    let t =
        one_pattern(r"(application|audio|image|text|video|message|multipart|model)/.+(;\s*.+=.+)*");
    for ok in ["text/xml", "text/xml; subtype=gml/3.2"] {
        assert_valid(&t, "T", ok);
    }
    for bad in ["foo/bar", "text/", "TEXT/xml"] {
        assert_invalid(&t, "T", bad);
    }
}

#[test]
fn axis_direction_escaped_set() {
    let t = one_pattern(r"[\+\-][1-9][0-9]*");
    for ok in ["+1", "-2"] {
        assert_valid(&t, "T", ok);
    }
    for bad in ["+0", "1", "+01"] {
        assert_invalid(&t, "T", bad);
    }
}

#[test]
fn dot_excludes_newline_counts_chars() {
    let t = one_pattern("a.b");
    assert_valid(&t, "T", "aäb");
    assert_invalid(&t, "T", "a&#10;b");
}

/// The value is linear in size for the regex engine; a backtracking
/// matcher would not finish on `(;\s*.+=.+)*`. No wall-clock assertion:
/// the test only has to end.
#[test]
fn mime_pattern_long_value_terminates() {
    let t =
        one_pattern(r"(application|audio|image|text|video|message|multipart|model)/.+(;\s*.+=.+)*");
    let value = format!("text/xml{}", ";a=a".repeat(20_000));
    assert_valid(&t, "T", &value);
}

/// Several pattern facets in one restriction step are alternatives
/// (XSD Part 2, 4.3.4.3); patterns of different derivation steps all apply.
#[test]
fn patterns_in_one_step_are_ored() {
    let t = r#"<xs:simpleType name="T">
    <xs:restriction base="xs:string">
      <xs:pattern value="a+"/><xs:pattern value="b+"/>
    </xs:restriction>
  </xs:simpleType>
  <xs:simpleType name="U">
    <xs:restriction base="T"><xs:pattern value="a*"/></xs:restriction>
  </xs:simpleType>"#;
    for ok in ["aa", "bb"] {
        assert_valid(t, "T", ok);
    }
    assert_invalid(t, "T", "ab");
    assert_valid(t, "U", "aa");
    assert_invalid(t, "U", "bb");
}
