use rumor_simulation::checks::{run_checks_with_scale, CheckScale};

#[test]
fn small_validation_suite_covers_v1_through_v7() {
    let results = run_checks_with_scale(CheckScale::Small).unwrap();
    assert_eq!(results.len(), 7);
    assert!(results.iter().all(|result| result.passed), "{results:#?}");
}
