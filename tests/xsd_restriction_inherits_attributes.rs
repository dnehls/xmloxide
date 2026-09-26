//! A type derived by `complexContent/restriction` keeps the attribute uses
//! of its base: local declarations replace the inherited ones by name,
//! `use="prohibited"` removes them. The content model is not inherited.
use xmloxide::validation::xsd::{parse_xsd, validate_xsd_strict};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">
  <xs:complexType name="B">
    <xs:sequence><xs:element name="only_b" type="xs:string" minOccurs="0"/></xs:sequence>
    <xs:attribute name="a" type="xs:string" fixed="x"/>
    <xs:attribute name="n" type="xs:integer"/>
    <xs:attribute name="k" type="xs:string"/>
  </xs:complexType>
  <xs:complexType name="R">
    <xs:complexContent>
      <xs:restriction base="t:B">
        <xs:sequence><xs:element name="only_r" type="xs:string" minOccurs="0"/></xs:sequence>
        <xs:attribute name="n" type="xs:integer" fixed="1"/>
        <xs:attribute name="k" type="xs:string" use="prohibited"/>
      </xs:restriction>
    </xs:complexContent>
  </xs:complexType>
  <xs:complexType name="C">
    <xs:complexContent>
      <xs:extension base="t:R"/>
    </xs:complexContent>
  </xs:complexType>
  <xs:element name="r" type="t:R"/>
  <xs:element name="c" type="t:C"/>
</xs:schema>"#;

fn strict_errors(xml: &str) -> Vec<String> {
    let schema = parse_xsd(XSD).unwrap();
    let doc = Document::parse_str(xml).unwrap();
    validate_xsd_strict(&doc, &schema)
        .errors
        .into_iter()
        .map(|e| e.message)
        .collect()
}

#[test]
fn test_complex_content_restriction_inherits_base_attributes() {
    assert_eq!(
        strict_errors(r#"<r xmlns="urn:t" a="x" n="1"/>"#),
        Vec::<String>::new()
    );
    assert_eq!(
        strict_errors(r#"<c xmlns="urn:t" a="x" n="1"/>"#),
        Vec::<String>::new()
    );

    let prohibited = strict_errors(r#"<r xmlns="urn:t" k="v"/>"#);
    assert!(
        prohibited.iter().any(|e| e.contains("not declared")),
        "{prohibited:?}"
    );

    let fixed = strict_errors(r#"<r xmlns="urn:t" n="2"/>"#);
    assert!(fixed.iter().any(|e| e.contains("fixed")), "{fixed:?}");

    let content = strict_errors(r#"<r xmlns="urn:t"><only_b>v</only_b></r>"#);
    assert!(!content.is_empty(), "content model must not be inherited");
}
