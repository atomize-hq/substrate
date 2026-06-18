#![allow(unused_crate_dependencies)]

mod support;

use support::{
    sorted_entry_names, ObjectiveAcceptanceCorpus, OBJECTIVE_ACCEPTANCE_FAMILIES,
    OBJECTIVE_ACCEPTANCE_FAMILY_README,
};

#[test]
fn objective_acceptance_fixture_root_contains_only_readme_and_family_dirs() {
    let corpus = ObjectiveAcceptanceCorpus::load();
    let mut expected_root_entries = OBJECTIVE_ACCEPTANCE_FAMILIES
        .iter()
        .map(|family| family.dir_name.to_owned())
        .collect::<Vec<_>>();
    expected_root_entries.push("README.md".to_owned());
    expected_root_entries.sort();

    assert_eq!(
        sorted_entry_names(corpus.root().as_std_path()),
        expected_root_entries,
        "Packet SO-4.1 must keep the objective acceptance root bounded to README.md plus the committed family directories"
    );
}

#[test]
fn objective_acceptance_family_dirs_exist_as_placeholder_only_until_seed_cases_land() {
    let corpus = ObjectiveAcceptanceCorpus::load();
    let expected_family_entries = vec![OBJECTIVE_ACCEPTANCE_FAMILY_README.to_owned()];

    for family in OBJECTIVE_ACCEPTANCE_FAMILIES {
        let family_dir = corpus.family_dir(family);
        assert_eq!(
            sorted_entry_names(family_dir.as_std_path()),
            expected_family_entries,
            "objective acceptance family {} must stay placeholder-only until SO-5 seeds commit real cases",
            family.dir_name
        );
        assert!(
            corpus.case_paths(family).is_empty(),
            "objective acceptance family {} must not report committed case directories before SO-5",
            family.dir_name
        );
    }
}

#[test]
fn objective_acceptance_readme_documents_the_family_contract() {
    let readme = std::fs::read_to_string(
        ObjectiveAcceptanceCorpus::load()
            .root()
            .join("README.md")
            .as_std_path(),
    )
    .expect("read objective acceptance README");

    for family in OBJECTIVE_ACCEPTANCE_FAMILIES {
        assert!(
            readme.contains(family.dir_name),
            "objective acceptance README must document family {}",
            family.dir_name
        );
    }
}
