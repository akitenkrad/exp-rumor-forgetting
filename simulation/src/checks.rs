use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::{ForgetMode, NetworkKind, TrialSpec};
use crate::runner::{run_specs_parallel, run_trial, TrialResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckScale {
    Small,
    Full,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub passed: bool,
    pub observed: String,
    pub expected: String,
}

#[derive(Deserialize)]
struct ReferenceRow {
    target_id: String,
    value: f64,
}

fn references() -> Result<BTreeMap<String, f64>, Box<dyn Error + Send + Sync>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../reference.csv");
    let mut reader = csv::Reader::from_path(path)?;
    let mut values = BTreeMap::new();
    for row in reader.deserialize::<ReferenceRow>() {
        let row = row?;
        values.insert(row.target_id, row.value);
    }
    Ok(values)
}

fn complete_specs(n: usize, lambda: f64, sigma: f64, delta: f64, t_max: u64, seeds: usize) -> Vec<TrialSpec> {
    (0..seeds as u64)
        .map(|seed| TrialSpec::new(n, NetworkKind::Complete, 0.0, lambda, sigma, delta, t_max, ForgetMode::Both, seed))
        .collect()
}

fn mean(trials: &[TrialResult], select: fn(&TrialResult) -> f64) -> f64 {
    trials.iter().map(select).sum::<f64>() / trials.len() as f64
}

fn result(id: &str, passed: bool, observed: impl ToString, expected: impl ToString) -> CheckResult {
    CheckResult {
        id: id.to_owned(),
        passed,
        observed: observed.to_string(),
        expected: expected.to_string(),
    }
}

pub fn run_checks_with_scale(scale: CheckScale) -> Result<Vec<CheckResult>, Box<dyn Error + Send + Sync>> {
    let refs = references()?;
    let (n, final_seeds, endemic_seeds, endemic_t, final_tol, relative_tol, rate_tol) = match scale {
        CheckScale::Small => (2_000, 20, 3, 300, 0.04, 0.20, 0.05),
        CheckScale::Full => (10_000, 100, 5, 1_000, 0.01, 0.05, 0.01),
    };
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let mut checks = Vec::new();

    let v1 = run_specs_parallel(&complete_specs(n, 1.0, 1.0, 0.0, 1_000, final_seeds), threads)?;
    let v1_observed = mean(&v1, |trial| trial.final_ignorant);
    let v1_expected = refs["v1.ode.l1p00_s1p00"];
    checks.push(result(
        "v1",
        (v1_observed - v1_expected).abs() <= final_tol,
        format!("mean_i_final={v1_observed:.8}"),
        format!("ode={v1_expected:.8}, abs_tol={final_tol}"),
    ));

    let mut v2_details = Vec::new();
    let mut v2_passed = true;
    for (lambda, sigma, target) in [(0.5, 1.0, "v2.ode.l0p50_s1p00"), (1.0, 0.5, "v2.ode.l1p00_s0p50")] {
        let trials = run_specs_parallel(&complete_specs(n, lambda, sigma, 0.0, 1_000, final_seeds), threads)?;
        let observed = mean(&trials, |trial| trial.final_ignorant);
        let expected = refs[target];
        v2_passed &= (observed - expected).abs() <= final_tol;
        v2_details.push(format!("l={lambda:.1},s={sigma:.1}: {observed:.8}/{expected:.8}"));
    }
    checks.push(result("v2", v2_passed, v2_details.join("; "), format!("ODE abs_tol={final_tol}")));

    let mut v3_details = Vec::new();
    let mut v3_passed = true;
    for (lambda, delta) in [(0.8, 0.2), (0.6, 0.3), (1.0, 0.5)] {
        let trials = run_specs_parallel(&complete_specs(n, lambda, 1.0, delta, endemic_t, endemic_seeds), threads)?;
        let survivors: Vec<_> = trials.iter().filter(|trial| trial.survived).collect();
        if survivors.is_empty() {
            v3_passed = false;
            v3_details.push(format!("l={lambda},d={delta}: no survivors"));
            continue;
        }
        let observed = [
            survivors.iter().map(|trial| trial.i_qs.unwrap()).sum::<f64>() / survivors.len() as f64,
            survivors.iter().map(|trial| trial.s_qs.unwrap()).sum::<f64>() / survivors.len() as f64,
            survivors.iter().map(|trial| trial.known_qs.unwrap()).sum::<f64>() / survivors.len() as f64,
        ];
        let label = |value: f64| format!("{value:.2}").replace('.', "p");
        let expected = ["i", "s", "known"].map(|quantity| refs[&format!("a3.{quantity}.l{}_d{}", label(lambda), label(delta))]);
        let errors = [0, 1, 2].map(|j| (observed[j] - expected[j]).abs() / expected[j].abs());
        v3_passed &= errors.iter().all(|error| *error <= relative_tol);
        v3_details.push(format!("l={lambda:.1},d={delta:.1}: obs={observed:?}, ref={expected:?}, rel={errors:?}"));
    }
    checks.push(result("v3", v3_passed, v3_details.join("; "), format!("relative_tol={relative_tol}")));

    let a1 = refs["a1.l1p00_s1p00"];
    let equation_residual = 2.0 * (1.0 - a1) + a1.ln();
    let mut v4_passed = equation_residual.abs() < 1e-10;
    for (lambda, delta) in [(0.8, 0.2), (0.6, 0.3), (1.0, 0.5)] {
        let label = |value: f64| format!("{value:.2}").replace('.', "p");
        let prefix = format!("l{}_d{}", label(lambda), label(delta));
        let expected = [
            (1.0 + delta) / (lambda + 1.0),
            delta * (lambda - delta) / (lambda * (1.0 + delta)),
            (lambda - delta) / (lambda + 1.0),
        ];
        for (quantity, analytic) in ["i", "s", "known"].into_iter().zip(expected) {
            v4_passed &= (refs[&format!("a3.{quantity}.{prefix}")] - analytic).abs() < 1e-6;
        }
    }
    checks.push(result(
        "v4",
        v4_passed,
        format!("A1 residual={equation_residual:.3e}; A3 CSV checked"),
        "residual < 1e-10; A3 abs error < 1e-6",
    ));

    let v5 = run_specs_parallel(
        &complete_specs(n, 0.4, 1.0, 0.5, endemic_t, if scale == CheckScale::Full { 20 } else { 8 }),
        threads,
    )?;
    let extinct = v5.iter().filter(|trial| !trial.survived && trial.extinction_time.is_some()).count();
    checks.push(result(
        "v5",
        extinct == v5.len(),
        format!("extinct={extinct}/{}", v5.len()),
        "all extinct before T",
    ));

    let v6_specs = complete_specs(300, 0.8, 1.0, 0.2, 50, 8);
    let one = run_specs_parallel(&v6_specs, 1)?;
    let eight = run_specs_parallel(&v6_specs, 8)?;
    let bytes_one = serde_json::to_vec(&one)?;
    let bytes_eight = serde_json::to_vec(&eight)?;
    checks.push(result(
        "v6",
        bytes_one == bytes_eight && run_trial(&v6_specs[0])? == run_trial(&v6_specs[0])?,
        format!("bytes_equal={}, invariant_checks=passed", bytes_one == bytes_eight),
        "byte-identical at 1 and 8 threads",
    ));

    let v7 = run_trial(&TrialSpec::new(
        n,
        NetworkKind::Complete,
        0.0,
        0.8,
        1.0,
        0.2,
        endemic_t,
        ForgetMode::Both,
        70_007,
    ))?;
    let rate_error = (v7.conversations as f64 - v7.spreader_time).abs() / v7.spreader_time;
    checks.push(result(
        "v7",
        rate_error <= rate_tol,
        format!(
            "conversations={}, integral_s_dt={:.6}, rel_error={rate_error:.6}",
            v7.conversations, v7.spreader_time
        ),
        format!("relative_error <= {rate_tol}"),
    ));
    Ok(checks)
}
