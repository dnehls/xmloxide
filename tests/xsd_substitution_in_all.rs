//! A substitution-group member inside an `all` group counts for its head
//! and is validated against its own declaration.
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
  <xs:element name="Or" type="t:BinType" substitutionGroup="t:ops"/>
  <xs:element name="Bag">
    <xs:complexType><xs:all><xs:element ref="t:ops"/></xs:all></xs:complexType>
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

fn any_contains(errs: &[String], needle: &str) -> bool {
    errs.iter().any(|m| m.contains(needle))
}

#[test]
fn substitution_member_in_all_counts_for_head() {
    let xml = r#"<Bag xmlns="urn:t"><And><v>a</v><v>b</v></And></Bag>"#;
    for strict in [false, true] {
        assert_eq!(errors(xml, strict), Vec::<String>::new(), "strict={strict}");
    }
}

#[test]
fn substitution_member_in_all_still_checks_member_content() {
    let xml = r#"<Bag xmlns="urn:t"><And><w/></And></Bag>"#;
    for strict in [false, true] {
        let errs = errors(xml, strict);
        assert!(any_contains(&errs, "<w>"), "strict={strict}: {errs:?}");
    }
}

#[test]
fn two_members_in_all_exceed_head_max_occurs() {
    let xml = r#"<Bag xmlns="urn:t"><And><v>a</v></And><Or><v>b</v></Or></Bag>"#;
    for strict in [false, true] {
        let errs = errors(xml, strict);
        assert!(
            any_contains(&errs, "appears more than 1 time(s) in all group"),
            "strict={strict}: {errs:?}"
        );
    }
}

#[test]
fn missing_head_in_all_still_fails() {
    let xml = r#"<Bag xmlns="urn:t"/>"#;
    for strict in [false, true] {
        let errs = errors(xml, strict);
        assert!(
            any_contains(&errs, "requires at least 1 occurrence(s) of <ops>"),
            "strict={strict}: {errs:?}"
        );
    }
}
