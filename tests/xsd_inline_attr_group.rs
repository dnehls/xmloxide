//! An attributeGroup referenced from the anonymous type of an element is
//! expanded like one referenced from a named type.
use xmloxide::validation::xsd::{parse_xsd, validate_xsd_strict};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">
  <xs:attributeGroup name="Resolve">
    <xs:attribute name="resolve" type="xs:string"/>
  </xs:attributeGroup>
  <xs:element name="PropertyName">
    <xs:complexType><xs:simpleContent><xs:extension base="xs:string">
      <xs:attributeGroup ref="t:Resolve"/>
    </xs:extension></xs:simpleContent></xs:complexType>
  </xs:element>
  <xs:element name="Query">
    <xs:complexType><xs:sequence>
      <xs:element name="Name">
        <xs:complexType><xs:simpleContent><xs:extension base="xs:string">
          <xs:attributeGroup ref="t:Resolve"/>
        </xs:extension></xs:simpleContent></xs:complexType>
      </xs:element>
    </xs:sequence></xs:complexType>
  </xs:element>
</xs:schema>"#;

fn errors(xml: &str) -> Vec<String> {
    let schema = parse_xsd(XSD).unwrap();
    let doc = Document::parse_str(xml).unwrap();
    let report = validate_xsd_strict(&doc, &schema);
    report.errors.into_iter().map(|e| e.message).collect()
}

#[test]
fn attribute_group_in_global_anonymous_type_is_expanded() {
    let xml = r#"<PropertyName xmlns="urn:t" resolve="local">x</PropertyName>"#;
    assert_eq!(errors(xml), Vec::<String>::new());
    let bad = r#"<PropertyName xmlns="urn:t" other="1">x</PropertyName>"#;
    assert!(!errors(bad).is_empty());
}

#[test]
fn attribute_group_in_local_anonymous_type_is_expanded() {
    let xml = r#"<Query xmlns="urn:t"><Name resolve="local">x</Name></Query>"#;
    assert_eq!(errors(xml), Vec::<String>::new());
}
