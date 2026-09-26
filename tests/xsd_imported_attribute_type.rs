//! Attribute types named by a prefixed `QName` resolve like element types:
//! through the root prefix map, the target namespace and the imported
//! namespaces. An unresolved name used to fall through to the built-in
//! check, which accepts every unknown type name. Pattern facets the matcher
//! cannot evaluate (groups, alternation, counted quantifiers) count as not
//! checkable instead of violated; xmllint (libxml2) is the reference.
use xmloxide::validation::xsd::{
    parse_xsd_with_options, validate_xsd, validate_xsd_strict, XsdParseOptions,
};
use xmloxide::Document;

const IMPORTED: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t">
  <xs:simpleType name="Mode">
    <xs:restriction base="xs:string">
      <xs:enumeration value="a"/><xs:enumeration value="b"/>
    </xs:restriction>
  </xs:simpleType>
  <xs:simpleType name="Star">
    <xs:restriction base="xs:string"><xs:enumeration value="*"/></xs:restriction>
  </xs:simpleType>
  <xs:simpleType name="UomSymbol">
    <xs:restriction base="xs:string"><xs:pattern value="[^: \n\r\t]+"/></xs:restriction>
  </xs:simpleType>
  <xs:simpleType name="UomURI">
    <xs:restriction base="xs:anyURI">
      <xs:pattern value="([a-zA-Z][a-zA-Z0-9\-\+\.]*:|\.\./|\./|#).*"/>
    </xs:restriction>
  </xs:simpleType>
  <xs:simpleType name="UomIdentifier">
    <xs:union memberTypes="t:UomSymbol t:UomURI"/>
  </xs:simpleType>
</xs:schema>"#;

const ROOT: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" xmlns:r="urn:r" targetNamespace="urn:r" elementFormDefault="qualified">
  <xs:import namespace="urn:t" schemaLocation="t.xsd"/>
  <xs:simpleType name="StarOrCount">
    <xs:union memberTypes="xs:positiveInteger t:Star"/>
  </xs:simpleType>
  <xs:element name="e">
    <xs:complexType>
      <xs:attribute name="m" type="t:Mode"/>
      <xs:attribute name="s" type="r:StarOrCount"/>
      <xs:attribute name="uom" type="t:UomIdentifier"/>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

fn errors(attrs: &str, strict: bool) -> Vec<String> {
    let resolver = |location: &str, _base: Option<&str>| -> Option<String> {
        (location == "t.xsd").then(|| IMPORTED.to_string())
    };
    let opts = XsdParseOptions {
        resolver: Some(&resolver),
        base_uri: None,
    };
    let schema = parse_xsd_with_options(ROOT, &opts).unwrap();
    let doc = Document::parse_str(&format!(r#"<e xmlns="urn:r" {attrs}/>"#)).unwrap();
    let report = if strict {
        validate_xsd_strict(&doc, &schema)
    } else {
        validate_xsd(&doc, &schema)
    };
    report.errors.into_iter().map(|e| e.message).collect()
}

#[test]
fn attribute_type_from_imported_namespace_is_validated() {
    for strict in [false, true] {
        let errs = errors(r#"m="c""#, strict);
        assert!(
            errs.iter().any(|m| m.contains("not in the enumeration")),
            "strict={strict}: {errs:?}"
        );
        assert_eq!(errors(r#"m="a""#, strict), Vec::<String>::new());
    }
}

#[test]
fn union_member_from_imported_namespace_is_validated() {
    for strict in [false, true] {
        assert_eq!(errors(r#"s="*""#, strict), Vec::<String>::new());
        assert_eq!(errors(r#"s="3""#, strict), Vec::<String>::new());
        let errs = errors(r#"s="x""#, strict);
        assert!(
            errs.iter()
                .any(|m| m.contains("does not match any member type")),
            "strict={strict}: {errs:?}"
        );
    }
}

#[test]
fn unsupported_pattern_is_not_a_violation() {
    for strict in [false, true] {
        for uom in ["urn:adv:uom:rad", "m", "#deg"] {
            assert_eq!(
                errors(&format!(r#"uom="{uom}""#), strict),
                Vec::<String>::new(),
                "strict={strict} uom={uom}"
            );
        }
    }
}
