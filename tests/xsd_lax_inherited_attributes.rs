//! Attributes a simpleContent extension inherits from its base type
//! (XSD 1.0 Part 1, 3.4.2, {attribute uses}) are checked in lax mode as
//! well as in strict mode. `gml:AngleType` extends `gml:MeasureType` whose
//! `uom` is required: lax used to check only the attributes declared on the
//! element's own type, so a missing or malformed inherited `uom` passed.
//! xmllint (libxml2) is the reference for every expectation.
use xmloxide::validation::xsd::{parse_xsd, validate_xsd, validate_xsd_strict};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:simpleType name="UomSymbol">
    <xs:restriction base="xs:string"><xs:pattern value="[^: \n\r\t]+"/></xs:restriction>
  </xs:simpleType>
  <xs:complexType name="Measure">
    <xs:simpleContent>
      <xs:extension base="xs:double">
        <xs:attribute name="uom" type="UomSymbol" use="required"/>
      </xs:extension>
    </xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Angle">
    <xs:simpleContent><xs:extension base="Measure"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Angle2">
    <xs:simpleContent><xs:extension base="Angle"/></xs:simpleContent>
  </xs:complexType>
  <xs:attributeGroup name="UomGroup">
    <xs:attribute name="uom" type="UomSymbol" use="required"/>
  </xs:attributeGroup>
  <xs:complexType name="GroupMeasure">
    <xs:simpleContent>
      <xs:extension base="xs:double"><xs:attributeGroup ref="UomGroup"/></xs:extension>
    </xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="GroupDerived">
    <xs:simpleContent><xs:extension base="GroupMeasure"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Base">
    <xs:sequence/>
    <xs:attribute name="uom" type="UomSymbol" use="required"/>
  </xs:complexType>
  <xs:complexType name="Ext">
    <xs:complexContent><xs:extension base="Base"/></xs:complexContent>
  </xs:complexType>
  <xs:element name="sc1" type="Angle"/>
  <xs:element name="sc2" type="Angle2"/>
  <xs:element name="gder" type="GroupDerived"/>
  <xs:element name="anon">
    <xs:complexType>
      <xs:simpleContent><xs:extension base="Measure"/></xs:simpleContent>
    </xs:complexType>
  </xs:element>
  <xs:element name="sc1nil" type="Angle" nillable="true"/>
  <xs:element name="ccext" type="Ext"/>
</xs:schema>"#;

/// Errors of `xml` against [`XSD`], lax and strict.
#[allow(clippy::unwrap_used)]
fn errors(xml: &str) -> [Vec<String>; 2] {
    let schema = parse_xsd(XSD).unwrap();
    let doc = Document::parse_str(xml).unwrap();
    [
        validate_xsd(&doc, &schema).errors,
        validate_xsd_strict(&doc, &schema).errors,
    ]
    .map(|errs| errs.into_iter().map(|e| e.message).collect())
}

fn assert_valid(xml: &str) {
    for (mode, errs) in ["lax", "strict"].iter().zip(errors(xml)) {
        assert!(errs.is_empty(), "{mode} {xml}: {errs:?}");
    }
}

fn assert_invalid(xml: &str) {
    for (mode, errs) in ["lax", "strict"].iter().zip(errors(xml)) {
        assert!(!errs.is_empty(), "{mode} {xml} passed");
    }
}

const XSI: &str = r#"xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#;

#[test]
fn simple_content_one_step_checks_inherited_uom() {
    assert_valid(r#"<sc1 uom="deg">1.5</sc1>"#);
    assert_invalid("<sc1>1.5</sc1>");
    assert_invalid(r#"<sc1 uom="a b">1.5</sc1>"#);
}

#[test]
fn simple_content_two_steps_checks_inherited_uom() {
    assert_valid(r#"<sc2 uom="deg">1.5</sc2>"#);
    assert_invalid("<sc2>1.5</sc2>");
    assert_invalid(r#"<sc2 uom="a b">1.5</sc2>"#);
}

#[test]
fn simple_content_base_with_attribute_group_checks_uom() {
    assert_valid(r#"<gder uom="deg">1.5</gder>"#);
    assert_invalid("<gder>1.5</gder>");
    assert_invalid(r#"<gder uom="a b">1.5</gder>"#);
}

#[test]
fn anonymous_simple_content_type_checks_inherited_uom() {
    assert_valid(r#"<anon uom="deg">1.5</anon>"#);
    assert_invalid("<anon>1.5</anon>");
    assert_invalid(r#"<anon uom="a b">1.5</anon>"#);
}

#[test]
fn nilled_simple_content_element_checks_inherited_uom() {
    assert_valid(&format!(r#"<sc1nil {XSI} xsi:nil="true" uom="deg"/>"#));
    assert_invalid(&format!(r#"<sc1nil {XSI} xsi:nil="true"/>"#));
    assert_invalid(&format!(r#"<sc1nil {XSI} xsi:nil="true" uom="a b"/>"#));
}

/// Control: complexContent extension already merges base attributes when
/// the schema is loaded.
#[test]
fn complex_content_extension_checks_inherited_uom() {
    assert_valid(r#"<ccext uom="deg"/>"#);
    assert_invalid("<ccext/>");
    assert_invalid(r#"<ccext uom="a b"/>"#);
}
