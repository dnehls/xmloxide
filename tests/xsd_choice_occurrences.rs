//! A choice validates every child it consumes and honours its own
//! `minOccurs`/`maxOccurs` (XSD 1.0 §3.8.4, a choice group repeated
//! `{min occurs}`..`{max occurs}` times).
use xmloxide::validation::xsd::{parse_xsd, validate_xsd, validate_xsd_strict};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">
  <xs:element name="Two">
    <xs:complexType>
      <xs:choice minOccurs="2" maxOccurs="2">
        <xs:element name="a" type="xs:string"/>
        <xs:element name="b" type="xs:string"/>
      </xs:choice>
    </xs:complexType>
  </xs:element>
  <xs:element name="UpToTwo">
    <xs:complexType>
      <xs:choice maxOccurs="2">
        <xs:element name="a" type="xs:string"/>
        <xs:element name="b" type="xs:string"/>
      </xs:choice>
    </xs:complexType>
  </xs:element>
  <xs:element name="One">
    <xs:complexType>
      <xs:choice>
        <xs:element name="a" type="xs:string"/>
        <xs:element name="b" type="xs:string"/>
      </xs:choice>
    </xs:complexType>
  </xs:element>
  <xs:element name="Seg">
    <xs:complexType>
      <xs:choice>
        <xs:choice minOccurs="2" maxOccurs="unbounded">
          <xs:element name="pos" type="xs:string"/>
          <xs:element name="pointProperty" type="xs:string"/>
        </xs:choice>
        <xs:element name="posList" type="xs:string"/>
      </xs:choice>
    </xs:complexType>
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

fn assert_valid(xml: &str) {
    assert_eq!(errors(xml, false), Vec::<String>::new(), "lax: {xml}");
    assert_eq!(errors(xml, true), Vec::<String>::new(), "strict: {xml}");
}

fn assert_invalid(xml: &str) {
    assert!(!errors(xml, false).is_empty(), "lax accepted {xml}");
    assert!(!errors(xml, true).is_empty(), "strict accepted {xml}");
}

#[test]
fn choice_min_two_rejects_single_child() {
    assert_invalid(r#"<Two xmlns="urn:t"><a>x</a></Two>"#);
}

#[test]
fn choice_min_two_accepts_two_children() {
    assert_valid(r#"<Two xmlns="urn:t"><a>x</a><b>y</b></Two>"#);
}

#[test]
fn choice_max_two_rejects_third_child() {
    assert_valid(r#"<UpToTwo xmlns="urn:t"><a>x</a><b>y</b></UpToTwo>"#);
    assert_invalid(r#"<UpToTwo xmlns="urn:t"><a>x</a><b>y</b><a>z</a></UpToTwo>"#);
}

#[test]
fn choice_rejects_unknown_second_child() {
    assert_invalid(r#"<One xmlns="urn:t"><a>x</a><c>y</c></One>"#);
}

#[test]
fn repeated_choice_element_counts_rounds() {
    assert_valid(r#"<Seg xmlns="urn:t"><pos>1</pos><pos>2</pos><pos>3</pos><pos>4</pos></Seg>"#);
    assert_valid(r#"<Seg xmlns="urn:t"><posList>1 2 3 4</posList></Seg>"#);
    assert_invalid(r#"<Seg xmlns="urn:t"><pos>1</pos></Seg>"#);
}
