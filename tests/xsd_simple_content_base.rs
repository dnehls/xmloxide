//! The text of an element whose type derives by simpleContent from another
//! complex type is checked against the simple type at the end of that chain
//! (XSD 1.0 Part 1, 3.4.2, {content type}; 3.4.4, Element Locally Valid
//! (Complex Type) clause 2.2). `gml:AreaType` extends `gml:MeasureType`
//! which extends `double`: both modes used to look only for a simple base
//! type and skipped the value otherwise, so `<area>viel</area>` passed.
//! xmllint (libxml2) is the reference for every expectation, except for the
//! cyclic schema, which it rejects.
use xmloxide::validation::xsd::{
    parse_xsd, parse_xsd_with_options, validate_xsd, validate_xsd_strict, XsdParseOptions,
    XsdSchema,
};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">
  <xs:simpleType name="Doubles"><xs:list itemType="xs:double"/></xs:simpleType>
  <xs:complexType name="Measure">
    <xs:simpleContent>
      <xs:extension base="xs:double"><xs:attribute name="uom" type="xs:string"/></xs:extension>
    </xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Area">
    <xs:simpleContent><xs:extension base="t:Measure"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Area2">
    <xs:simpleContent><xs:extension base="t:Area"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Restricted">
    <xs:simpleContent><xs:restriction base="t:Measure"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Int">
    <xs:simpleContent><xs:extension base="xs:integer"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Int2">
    <xs:simpleContent><xs:extension base="t:Int"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Date">
    <xs:simpleContent><xs:extension base="xs:date"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Date2">
    <xs:simpleContent><xs:extension base="t:Date"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Coords">
    <xs:simpleContent><xs:extension base="t:Doubles"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Coords2">
    <xs:simpleContent><xs:extension base="t:Coords"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Text">
    <xs:simpleContent><xs:extension base="xs:string"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Text2">
    <xs:simpleContent><xs:extension base="t:Text"/></xs:simpleContent>
  </xs:complexType>
  <xs:element name="measure" type="t:Measure"/>
  <xs:element name="area" type="t:Area"/>
  <xs:element name="area2" type="t:Area2"/>
  <xs:element name="restricted" type="t:Restricted"/>
  <xs:element name="anon">
    <xs:complexType>
      <xs:simpleContent><xs:extension base="t:Measure"/></xs:simpleContent>
    </xs:complexType>
  </xs:element>
  <xs:element name="int2" type="t:Int2"/>
  <xs:element name="date2" type="t:Date2"/>
  <xs:element name="coords2" type="t:Coords2"/>
  <xs:element name="text2" type="t:Text2"/>
  <xs:element name="nil" type="t:Area" nillable="true"/>
  <xs:element name="head" type="t:Area" abstract="true"/>
  <xs:element name="member" type="t:Area" substitutionGroup="t:head"/>
  <xs:element name="root">
    <xs:complexType>
      <xs:sequence><xs:element ref="t:head" maxOccurs="unbounded"/></xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

/// Errors of `xml` against `schema`, lax and strict.
#[allow(clippy::unwrap_used)]
fn errors(schema: &XsdSchema, xml: &str) -> [Vec<String>; 2] {
    let doc = Document::parse_str(xml).unwrap();
    [
        validate_xsd(&doc, schema).errors,
        validate_xsd_strict(&doc, schema).errors,
    ]
    .map(|errs| errs.into_iter().map(|e| e.message).collect())
}

fn assert_valid_in(schema: &XsdSchema, xml: &str) {
    for (mode, errs) in ["lax", "strict"].iter().zip(errors(schema, xml)) {
        assert!(errs.is_empty(), "{mode} {xml}: {errs:?}");
    }
}

/// `xml` fails in both modes, with one error that names `expected`.
fn assert_invalid_in(schema: &XsdSchema, xml: &str, expected: &str) {
    for (mode, errs) in ["lax", "strict"].iter().zip(errors(schema, xml)) {
        assert_eq!(errs.len(), 1, "{mode} {xml}: {errs:?}");
        assert!(errs[0].contains(expected), "{mode} {xml}: {errs:?}");
    }
}

/// `<name>text</name>` in the target namespace of [`XSD`].
fn element(name: &str, text: &str) -> String {
    format!(r#"<{name} xmlns="urn:t">{text}</{name}>"#)
}

#[allow(clippy::unwrap_used)]
fn assert_valid(name: &str, text: &str) {
    assert_valid_in(&parse_xsd(XSD).unwrap(), &element(name, text));
}

#[allow(clippy::unwrap_used)]
fn assert_invalid(name: &str, text: &str, expected: &str) {
    assert_invalid_in(&parse_xsd(XSD).unwrap(), &element(name, text), expected);
}

/// Control: a simple base type was always checked.
#[test]
fn one_step_checks_the_value() {
    assert_valid("measure", "1.5");
    assert_invalid("measure", "viel", "not a valid double");
}

#[test]
fn two_steps_check_the_value() {
    assert_valid("area", "1.5");
    assert_invalid("area", "viel", "not a valid double");
    assert_invalid("area", "", "not a valid double");
}

#[test]
fn three_steps_check_the_value() {
    assert_valid("area2", "1.5");
    assert_invalid("area2", "viel", "not a valid double");
}

#[test]
fn anonymous_type_checks_the_value() {
    assert_valid("anon", "1.5");
    assert_invalid("anon", "viel", "not a valid double");
}

#[test]
fn restriction_of_a_complex_base_checks_the_base_value() {
    assert_valid("restricted", "1.5");
    assert_invalid("restricted", "viel", "not a valid double");
}

#[test]
fn other_simple_types_are_checked() {
    assert_valid("int2", "-7");
    assert_invalid("int2", "1.5", "not a valid integer");
    assert_valid("date2", "2026-11-24");
    assert_invalid("date2", "24.11.2026", "not a valid date");
    assert_valid("coords2", "1 2.5 -3e2");
    assert_invalid("coords2", "1 viel", "not a valid double");
}

#[test]
fn value_is_normalized_and_string_stays_free() {
    assert_valid("area", " 1.5 ");
    assert_valid("area", "\n\t1.5\n");
    assert_valid("area", "INF");
    assert_valid("text2", "viel");
    assert_valid("text2", "");
}

#[test]
fn substitution_member_checks_the_value() {
    let root = |text: &str| format!(r#"<root xmlns="urn:t"><member>{text}</member></root>"#);
    #[allow(clippy::unwrap_used)]
    let schema = parse_xsd(XSD).unwrap();
    assert_valid_in(&schema, &root("1.5"));
    assert_invalid_in(&schema, &root("viel"), "not a valid double");
}

#[test]
fn nilled_element_has_no_value_to_check() {
    #[allow(clippy::unwrap_used)]
    let schema = parse_xsd(XSD).unwrap();
    let xsi = r#"xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#;
    assert_valid_in(
        &schema,
        &format!(r#"<nil xmlns="urn:t" {xsi} xsi:nil="true"/>"#),
    );
}

/// The base `QName` is resolved where it is written. The importing schema
/// binds the prefix `x` to `urn:c`, the imported one to its own `urn:b`,
/// and both namespaces and the root's define a `Measure`.
const MAIN: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:m="urn:main" xmlns:bb="urn:b" xmlns:x="urn:c"
    targetNamespace="urn:main" elementFormDefault="qualified">
  <xs:import namespace="urn:b" schemaLocation="b.xsd"/>
  <xs:import namespace="urn:c" schemaLocation="c.xsd"/>
  <xs:complexType name="Measure">
    <xs:simpleContent><xs:extension base="xs:boolean"/></xs:simpleContent>
  </xs:complexType>
  <xs:element name="prefixed" type="bb:Prefixed"/>
  <xs:element name="chameleon" type="bb:Chameleon"/>
  <xs:element name="foreign">
    <xs:complexType>
      <xs:simpleContent><xs:extension base="x:Measure"/></xs:simpleContent>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

const B: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:x="urn:b" targetNamespace="urn:b">
  <xs:include schemaLocation="chameleon.xsd"/>
  <xs:complexType name="Measure">
    <xs:simpleContent><xs:extension base="xs:double"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="Prefixed">
    <xs:simpleContent><xs:extension base="x:Measure"/></xs:simpleContent>
  </xs:complexType>
</xs:schema>"#;

/// No target namespace: takes `urn:b`, and so does its unprefixed base.
const CHAMELEON: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="Chameleon">
    <xs:simpleContent><xs:extension base="Measure"/></xs:simpleContent>
  </xs:complexType>
</xs:schema>"#;

/// The default namespace is the XSD one, as in the GML schemas.
const C: &str = r#"<schema xmlns="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:c">
  <complexType name="Measure">
    <simpleContent><extension base="date"/></simpleContent>
  </complexType>
</schema>"#;

#[allow(clippy::unwrap_used)]
fn imported_schema() -> XsdSchema {
    let resolver = |location: &str, _base: Option<&str>| {
        match location {
            "b.xsd" => Some(B),
            "c.xsd" => Some(C),
            "chameleon.xsd" => Some(CHAMELEON),
            _ => None,
        }
        .map(String::from)
    };
    parse_xsd_with_options(
        MAIN,
        &XsdParseOptions {
            resolver: Some(&resolver),
            base_uri: None,
        },
    )
    .unwrap()
}

/// `<name>text</name>` in the target namespace of [`MAIN`].
fn main_element(name: &str, text: &str) -> String {
    format!(r#"<{name} xmlns="urn:main">{text}</{name}>"#)
}

#[test]
fn imported_base_is_resolved_in_its_own_schema() {
    let schema = imported_schema();
    for name in ["prefixed", "chameleon"] {
        assert_valid_in(&schema, &main_element(name, "1.5"));
        assert_invalid_in(&schema, &main_element(name, "viel"), "not a valid double");
        assert_invalid_in(
            &schema,
            &main_element(name, "2026-11-24"),
            "not a valid double",
        );
    }
}

#[test]
fn imported_base_is_resolved_by_its_namespace() {
    let schema = imported_schema();
    assert_valid_in(&schema, &main_element("foreign", "2026-11-24"));
    assert_invalid_in(&schema, &main_element("foreign", "1.5"), "not a valid date");
}

/// A circular derivation is a schema error (3.4.6, `ct-props-correct`.3)
/// the parser does not report; validation must still terminate.
#[test]
fn circular_base_terminates() {
    #[allow(clippy::unwrap_used)]
    let schema = parse_xsd(
        r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="A">
    <xs:simpleContent><xs:extension base="B"/></xs:simpleContent>
  </xs:complexType>
  <xs:complexType name="B">
    <xs:simpleContent><xs:extension base="A"/></xs:simpleContent>
  </xs:complexType>
  <xs:element name="a" type="A"/>
</xs:schema>"#,
    )
    .unwrap();
    assert_valid_in(&schema, "<a>viel</a>");
}
