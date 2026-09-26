//! A substitution-group member reached through a choice is
//! validated against its own declaration, not against the abstract head.
use xmloxide::validation::xsd::{parse_xsd, validate_xsd, validate_xsd_strict};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">
  <xs:element name="ops" type="t:OpsType" abstract="true"/>
  <xs:complexType name="OpsType" abstract="true"/>
  <xs:complexType name="BinType">
    <xs:complexContent>
      <xs:extension base="t:OpsType">
        <xs:sequence><xs:element name="v" type="xs:string" maxOccurs="2"/></xs:sequence>
      </xs:extension>
    </xs:complexContent>
  </xs:complexType>
  <xs:element name="And" type="t:BinType" substitutionGroup="t:ops"/>
  <xs:element name="Filter">
    <xs:complexType><xs:choice><xs:element ref="t:ops"/></xs:choice></xs:complexType>
  </xs:element>
</xs:schema>"#;

fn errors(xml: &str, strict: bool) -> Vec<String> {
    let schema = parse_xsd(XSD).unwrap();
    let doc = Document::parse_str(xml).unwrap();
    let report = if strict {
        validate_xsd_strict(&doc, &schema)
    } else {
        validate_xsd(&doc, &schema)
    };
    report.errors.into_iter().map(|e| e.message).collect()
}

#[test]
fn substitution_member_in_choice_uses_member_type() {
    let xml = r#"<Filter xmlns="urn:t"><And><v>a</v><v>b</v></And></Filter>"#;
    assert_eq!(errors(xml, false), Vec::<String>::new());
    assert_eq!(errors(xml, true), Vec::<String>::new());
}

#[test]
fn substitution_member_in_choice_still_checks_member_content() {
    let xml = r#"<Filter xmlns="urn:t"><And><w/></And></Filter>"#;
    assert!(!errors(xml, false).is_empty());
}
