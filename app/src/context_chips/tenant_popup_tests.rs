use super::*;

#[test]
fn parses_section_names_only() {
    let contents = "\
[default]
api_key = secret

[acme-prod]
username = bob
password = hunter2

  [indented-is-a-continuation]
[ spaced ]
[]
; [commented]
";
    assert_eq!(
        parse_tenant_labels(contents),
        vec!["default", "acme-prod", "spaced"]
    );
}

#[test]
fn empty_file_has_no_tenants() {
    assert!(parse_tenant_labels("").is_empty());
}
