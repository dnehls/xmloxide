//! `<xs:group ref>` resolves against every named model group of the
//! schema: one from an included document, one declared further down the
//! same document, and from inside an anonymous complex type (XSD 1.0
//! §3.7.2, §4.2.1). The expected verdicts are xmllint's (libxml2).
use xmloxide::validation::xsd::{
    parse_xsd_with_options, validate_xsd, validate_xsd_strict, XsdParseOptions,
};
use xmloxide::Document;

const INCLUDED: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">
  <xs:group name="Position">
    <xs:choice>
      <xs:element name="pos" type="xs:string"/>
      <xs:element name="point" type="xs:string"/>
    </xs:choice>
  </xs:group>
</xs:schema>"#;

const ROOT: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">
  <xs:include schemaLocation="inc.xsd"/>
  <xs:group name="Outer">
    <xs:sequence>
      <xs:group ref="t:Inner"/>
    </xs:sequence>
  </xs:group>
  <xs:group name="Inner">
    <xs:sequence>
      <xs:element name="a" type="xs:string"/>
      <xs:element name="b" type="xs:string"/>
    </xs:sequence>
  </xs:group>
  <xs:complexType name="CurveType">
    <xs:sequence>
      <xs:group ref="t:Position" minOccurs="2" maxOccurs="unbounded"/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="ForwardType">
    <xs:sequence>
      <xs:group ref="t:Outer"/>
    </xs:sequence>
  </xs:complexType>
  <xs:element name="Curve" type="t:CurveType"/>
  <xs:element name="Forward" type="t:ForwardType"/>
  <xs:element name="Anonymous">
    <xs:complexType>
      <xs:sequence>
        <xs:group ref="t:Inner"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

fn errors(xml: &str, strict: bool) -> Vec<String> {
    let resolver = |location: &str, _base: Option<&str>| -> Option<String> {
        (location == "inc.xsd").then(|| INCLUDED.to_string())
    };
    let options = XsdParseOptions {
        resolver: Some(&resolver),
        base_uri: None,
    };
    let schema = parse_xsd_with_options(ROOT, &options).unwrap();
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
fn group_ref_from_included_schema_resolves() {
    assert_valid(r#"<Curve xmlns="urn:t"><pos>1</pos><point>2</point><pos>3</pos></Curve>"#);
    assert_invalid(r#"<Curve xmlns="urn:t"><pos>1</pos></Curve>"#);
}

#[test]
fn forward_group_ref_in_group_resolves() {
    assert_valid(r#"<Forward xmlns="urn:t"><a>1</a><b>2</b></Forward>"#);
    assert_invalid(r#"<Forward xmlns="urn:t"><b>2</b></Forward>"#);
}

#[test]
fn group_ref_in_anonymous_type_resolves() {
    assert_valid(r#"<Anonymous xmlns="urn:t"><a>1</a><b>2</b></Anonymous>"#);
    assert_invalid(r#"<Anonymous xmlns="urn:t"><b>2</b></Anonymous>"#);
}
