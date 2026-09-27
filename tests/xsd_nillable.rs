//! An element with `xsi:nil="true"` whose declaration is nillable has no
//! content to match against its type (XSD 1.0 §3.3.4, Element Locally
//! Valid (Element) clause 3.2): its content model is not checked, and it
//! must have neither character nor element children. Its attributes are
//! still checked. The expected verdicts are xmllint's.
use xmloxide::validation::xsd::{parse_xsd, validate_xsd, validate_xsd_strict};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    targetNamespace="urn:t" xmlns="urn:t" elementFormDefault="qualified">
  <xs:element name="Nil" nillable="true">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="a" type="xs:string"/>
      </xs:sequence>
      <xs:attribute name="id" type="xs:string"/>
    </xs:complexType>
  </xs:element>
  <xs:element name="NotNil">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="a" type="xs:string"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
  <xs:element name="bbox" nillable="true">
    <xs:complexType>
      <xs:choice>
        <xs:element name="env" type="xs:string"/>
        <xs:element name="null" type="xs:string"/>
      </xs:choice>
    </xs:complexType>
  </xs:element>
  <xs:element name="Feature">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="id" type="xs:string"/>
        <xs:element ref="bbox"/>
        <xs:element name="n" type="xs:int" nillable="true"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

const NS: &str = r#"xmlns="urn:t" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#;

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

fn assert_valid(xml: &str) {
    assert_eq!(errors(xml, false), Vec::<String>::new(), "lax: {xml}");
    assert_eq!(errors(xml, true), Vec::<String>::new(), "strict: {xml}");
}

fn assert_invalid(xml: &str) {
    assert!(!errors(xml, false).is_empty(), "lax accepted {xml}");
    assert!(!errors(xml, true).is_empty(), "strict accepted {xml}");
}

#[test]
fn nil_element_with_required_content() {
    assert_valid(&format!(r#"<Nil {NS} xsi:nil="true"/>"#));
    assert_valid(&format!(r#"<Nil {NS} xsi:nil="true" id="x"/>"#));
}

/// The shape of `<gml:boundedBy xsi:nil="true"/>`: a nillable global
/// element with a required choice, referenced from a sequence, next to a
/// nillable simple-typed local element.
#[test]
fn nil_referenced_element_and_simple_type() {
    assert_valid(&format!(
        r#"<Feature {NS}><id>1</id><bbox xsi:nil="true"/><n xsi:nil="true"/></Feature>"#
    ));
    assert_invalid(&format!(
        r"<Feature {NS}><id>1</id><bbox/><n>1</n></Feature>"
    ));
}

#[test]
fn nil_element_must_be_empty() {
    assert_invalid(&format!(r#"<Nil {NS} xsi:nil="true"><a>1</a></Nil>"#));
    assert_invalid(&format!(r#"<Nil {NS} xsi:nil="true">x</Nil>"#));
    assert_invalid(&format!(r#"<Nil {NS} xsi:nil="true"> </Nil>"#));
}

/// `xsi:nil` only takes effect on a nillable declaration, and
/// `xsi:nil="false"` never does: the content model applies.
#[test]
fn nil_without_effect_checks_content() {
    assert_invalid(&format!(r#"<NotNil {NS} xsi:nil="true"/>"#));
    assert_invalid(&format!(r#"<Nil {NS} xsi:nil="false"/>"#));
}

#[test]
fn nil_element_attributes_still_checked() {
    let xml = format!(r#"<Nil {NS} xsi:nil="true" undeclared="x"/>"#);
    assert!(!errors(&xml, true).is_empty(), "strict accepted {xml}");
}
