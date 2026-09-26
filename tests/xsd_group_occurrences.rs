//! A sequence group and a `<xs:group ref>` repeat as a whole,
//! `{min occurs}`..`{max occurs}` times (XSD 1.0 §3.8.4 for model groups,
//! §3.7.2 for the particle a group reference contributes). The expected
//! verdicts are xmllint's (libxml2) for the same schema and instance.
use xmloxide::validation::xsd::{parse_xsd, validate_xsd, validate_xsd_strict};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">
  <xs:group name="AB">
    <xs:sequence>
      <xs:element name="a" type="xs:string"/>
      <xs:element name="b" type="xs:string"/>
    </xs:sequence>
  </xs:group>
  <xs:complexType name="SeqRepType">
    <xs:sequence maxOccurs="unbounded">
      <xs:element name="a" type="xs:string"/>
      <xs:element name="b" type="xs:string"/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="SeqTwoType">
    <xs:sequence minOccurs="2" maxOccurs="2">
      <xs:element name="a" type="xs:string"/>
      <xs:element name="b" type="xs:string"/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="GrpRepType">
    <xs:sequence>
      <xs:group ref="t:AB" maxOccurs="unbounded"/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="GrpTwoType">
    <xs:sequence>
      <xs:group ref="t:AB" minOccurs="2" maxOccurs="2"/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="GrpOptType">
    <xs:sequence>
      <xs:group ref="t:AB" minOccurs="0"/>
      <xs:element name="c" type="xs:string"/>
    </xs:sequence>
  </xs:complexType>
  <xs:complexType name="GrpReqType">
    <xs:sequence>
      <xs:group ref="t:AB"/>
      <xs:element name="c" type="xs:string"/>
    </xs:sequence>
  </xs:complexType>
  <xs:element name="SeqRep" type="t:SeqRepType"/>
  <xs:element name="SeqTwo" type="t:SeqTwoType"/>
  <xs:element name="GrpRep" type="t:GrpRepType"/>
  <xs:element name="GrpTwo" type="t:GrpTwoType"/>
  <xs:element name="GrpOpt" type="t:GrpOptType"/>
  <xs:element name="GrpReq" type="t:GrpReqType"/>
</xs:schema>"#;

const THREE_ROUNDS: &str = "<a>1</a><b>2</b><a>3</a><b>4</b><a>5</a><b>6</b>";

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
fn sequence_unbounded_two_particles_three_rounds_valid() {
    assert_valid(&format!(r#"<SeqRep xmlns="urn:t">{THREE_ROUNDS}</SeqRep>"#));
}

#[test]
fn sequence_min_two_one_round_invalid() {
    assert_invalid(r#"<SeqTwo xmlns="urn:t"><a>1</a><b>2</b></SeqTwo>"#);
}

#[test]
fn group_ref_unbounded_three_rounds_valid() {
    assert_valid(&format!(r#"<GrpRep xmlns="urn:t">{THREE_ROUNDS}</GrpRep>"#));
}

#[test]
fn group_ref_min_two_one_round_invalid() {
    assert_invalid(r#"<GrpTwo xmlns="urn:t"><a>1</a><b>2</b></GrpTwo>"#);
}

#[test]
fn optional_group_ref_absent_valid() {
    assert_valid(r#"<GrpOpt xmlns="urn:t"><c>1</c></GrpOpt>"#);
}

#[test]
fn required_group_ref_missing_invalid() {
    assert_invalid(r#"<GrpReq xmlns="urn:t"><c>1</c></GrpReq>"#);
}
