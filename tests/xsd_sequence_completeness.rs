//! A sequence is complete only when every particle it did not match can be
//! empty (XSD 1.0 §3.8.4, a sequence group of `{particles}` each matched
//! `{min occurs}`..`{max occurs}` times). Strict mode must check this for a
//! top-level sequence, for a sequence group chosen by a choice and for each
//! element's own `minOccurs`; lax mode already does. The expected verdicts
//! are xmllint's.
use std::fmt::Write;

use xmloxide::validation::xsd::{parse_xsd, validate_xsd, validate_xsd_strict};
use xmloxide::Document;

const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    targetNamespace="urn:t" xmlns="urn:t" elementFormDefault="qualified">
  <xs:group name="AB">
    <xs:sequence>
      <xs:element name="a" type="xs:string"/>
      <xs:element name="b" type="xs:string"/>
    </xs:sequence>
  </xs:group>
  <xs:element name="RefUnb">
    <xs:complexType>
      <xs:sequence>
        <xs:choice maxOccurs="unbounded">
          <xs:group ref="AB"/>
          <xs:element name="c" type="xs:string"/>
        </xs:choice>
        <xs:element name="b" type="xs:string" minOccurs="0"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
  <xs:element name="InlUnb">
    <xs:complexType>
      <xs:sequence>
        <xs:choice maxOccurs="unbounded">
          <xs:sequence>
            <xs:element name="a" type="xs:string"/>
            <xs:element name="b" type="xs:string"/>
          </xs:sequence>
          <xs:element name="c" type="xs:string"/>
        </xs:choice>
        <xs:element name="b" type="xs:string" minOccurs="0"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
  <xs:element name="InlOne">
    <xs:complexType>
      <xs:sequence>
        <xs:choice>
          <xs:sequence>
            <xs:element name="a" type="xs:string"/>
            <xs:element name="b" type="xs:string"/>
          </xs:sequence>
          <xs:element name="c" type="xs:string"/>
        </xs:choice>
        <xs:element name="b" type="xs:string" minOccurs="0"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
  <xs:element name="SeqTop">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="a" type="xs:string"/>
        <xs:element name="b" type="xs:string"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
  <xs:element name="Min2">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="a" type="xs:string" minOccurs="2" maxOccurs="3"/>
        <xs:element name="c" type="xs:string"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
  <xs:element name="SkipGrp">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="e" type="xs:string" minOccurs="0"/>
        <xs:group ref="AB"/>
        <xs:element name="c" type="xs:string"/>
      </xs:sequence>
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

fn doc(root: &str, children: &str) -> String {
    let mut body = String::new();
    for c in children.chars() {
        write!(body, "<{c}>1</{c}>").unwrap();
    }
    format!(r#"<{root} xmlns="urn:t">{body}</{root}>"#)
}

fn assert_valid(root: &str, children: &str) {
    let xml = doc(root, children);
    assert_eq!(errors(&xml, false), Vec::<String>::new(), "lax: {xml}");
    assert_eq!(errors(&xml, true), Vec::<String>::new(), "strict: {xml}");
}

fn assert_invalid(root: &str, children: &str) {
    let xml = doc(root, children);
    assert!(!errors(&xml, false).is_empty(), "lax accepted {xml}");
    assert!(!errors(&xml, true).is_empty(), "strict accepted {xml}");
}

/// `<a/><c/>` starts the group alternative `AB` and leaves it without `b`.
#[test]
fn choice_group_alternative_incomplete_ac() {
    assert_invalid("RefUnb", "ac");
}

/// The incomplete group alternative is the last round of the choice.
#[test]
fn choice_group_alternative_incomplete_last_round_ca() {
    assert_invalid("RefUnb", "ca");
}

#[test]
fn choice_inline_sequence_incomplete_a() {
    assert_invalid("InlUnb", "a");
    assert_invalid("InlOne", "a");
}

#[test]
fn top_level_sequence_truncated_a() {
    assert_invalid("SeqTop", "a");
}

#[test]
fn sequence_min_occurs_shortfall() {
    assert_invalid("Min2", "ac");
    assert_valid("Min2", "aac");
    assert_valid("Min2", "aaac");
}

/// A required group between an optional element and `c` cannot be skipped;
/// strict reports the missing group once.
#[test]
fn required_group_not_skippable() {
    assert_invalid("SkipGrp", "c");
    let strict = errors(&doc("SkipGrp", "c"), true);
    assert_eq!(strict.len(), 1, "strict: {strict:?}");
    assert_valid("SkipGrp", "abc");
    assert_valid("SkipGrp", "eabc");
}

#[test]
fn complete_sequences_stay_valid() {
    for root in ["RefUnb", "InlUnb"] {
        for children in ["ab", "abc", "cab", "c", "cb"] {
            assert_valid(root, children);
        }
    }
    assert_valid("InlOne", "ab");
    assert_valid("InlOne", "cb");
    assert_valid("SeqTop", "ab");
}
