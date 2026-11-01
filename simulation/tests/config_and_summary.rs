use rumor_simulation::analyze::aggregate_conditions;
use rumor_simulation::config::{ExperimentConfig, ForgetMode, NetworkKind, TrialSpec};
use rumor_simulation::runner::{run_specs_parallel, run_trial};

#[test]
fn main_config_has_the_declared_420_conditions() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../configs/main.toml");
    let cfg = ExperimentConfig::load(path).unwrap();
    assert_eq!(cfg.network, NetworkKind::Er);
    assert_eq!(cfg.n, 10_000);
    assert_eq!(cfg.lambdas.len(), 20);
    assert_eq!(cfg.deltas.len(), 21);
    assert_eq!(cfg.seeds.len(), 10);
    assert_eq!(cfg.trial_specs(None).unwrap().len(), 4_200);
}

#[test]
fn same_trial_is_byte_reproducible_and_parallelism_independent() {
    let specs: Vec<_> = (0..8)
        .map(|seed| TrialSpec::new(80, NetworkKind::Complete, 0.0, 0.8, 1.0, 0.2, 30, ForgetMode::Both, seed))
        .collect();
    let a = run_specs_parallel(&specs, 1).unwrap();
    let b = run_specs_parallel(&specs, 8).unwrap();
    assert_eq!(serde_json::to_vec(&a).unwrap(), serde_json::to_vec(&b).unwrap());
    assert_eq!(run_trial(&specs[0]).unwrap(), run_trial(&specs[0]).unwrap());
}

#[test]
fn aggregate_uses_only_surviving_trials_for_qs_means() {
    let specs = [
        TrialSpec::new(60, NetworkKind::Complete, 0.0, 1.0, 1.0, 0.2, 20, ForgetMode::Both, 1),
        TrialSpec::new(60, NetworkKind::Complete, 0.0, 0.1, 1.0, 1.0, 20, ForgetMode::Both, 2),
    ];
    let trials: Vec<_> = specs.iter().map(|s| run_trial(s).unwrap()).collect();
    let aggregates = aggregate_conditions(&trials);
    assert_eq!(aggregates.len(), 2);
    for aggregate in aggregates {
        if aggregate.surviving_trials == 0 {
            assert!(aggregate.s_qs.is_none());
            assert!(aggregate.known_qs.is_none());
        }
    }
}
