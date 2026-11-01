use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use serde::{Deserialize, Serialize};
use socsim_core::SimRng;
use socsim_engine::SimulationBuilder;

use crate::config::{NetworkKind, TrialSpec};
use crate::mechanism::{EventDrivenScheduler, RumorGillespie};
use crate::network::{ContactNetwork, SharedNetwork};
use crate::seeding::{dynamics_seed, initial_seed};
use crate::summary::Sample;
use crate::world::RumorWorld;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrialResult {
    pub seed: u64,
    pub lambda: f64,
    pub sigma: f64,
    pub delta: f64,
    pub survived: bool,
    pub extinction_time: Option<f64>,
    pub i_qs: Option<f64>,
    pub s_qs: Option<f64>,
    pub known_qs: Option<f64>,
    pub ever_fraction: f64,
    pub final_ignorant: f64,
    pub final_s: usize,
    pub conversations: u64,
    pub spreader_time: f64,
    pub isolated_fraction: f64,
    #[serde(skip_serializing)]
    pub history: Vec<Sample>,
}

fn representative(spec: &TrialSpec) -> bool {
    if spec.seed != 0 {
        return false;
    }
    let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
    (spec.network == NetworkKind::Complete
        && close(spec.delta, 0.0)
        && [(1.0, 1.0), (0.5, 1.0), (1.0, 0.5)]
            .iter()
            .any(|&(l, s)| close(spec.lambda, l) && close(spec.sigma, s)))
        || (spec.network == NetworkKind::Complete
            && [(0.8, 0.2), (0.6, 0.3), (1.0, 0.5)]
                .iter()
                .any(|&(l, d)| close(spec.lambda, l) && close(spec.delta, d)))
        || (spec.network == NetworkKind::Er
            && [(0.5, 0.1), (0.5, 0.3), (0.3, 0.3)]
                .iter()
                .any(|&(l, d)| close(spec.lambda, l) && close(spec.delta, d)))
}

pub fn run_trial(spec: &TrialSpec) -> Result<TrialResult, Box<dyn Error + Send + Sync>> {
    run_trial_with_network(spec, Arc::new(ContactNetwork::generate(spec)))
}

fn run_trial_with_network(spec: &TrialSpec, network: SharedNetwork) -> Result<TrialResult, Box<dyn Error + Send + Sync>> {
    let isolated_fraction = network.isolated_fraction();
    let mut init_rng = SimRng::from_seed(initial_seed(spec.seed));
    let world = RumorWorld::new_shared(spec.n, spec.t_max, network, spec.forget, &mut init_rng);
    let keep_history = representative(spec);
    let mut history = if keep_history {
        vec![Sample::from_counts(0, spec.n, world.counts(), world.ever_count())]
    } else {
        Vec::new()
    };
    let mut qs = Vec::new();
    let mut simulation = SimulationBuilder::new(world)
        .seed(dynamics_seed(spec))
        .scheduler(Box::new(EventDrivenScheduler))
        .add_mechanism(Box::new(RumorGillespie::new(spec.lambda, spec.sigma, spec.delta)))
        .build();
    simulation.run_observed(|report| {
        let sample = Sample::from_counts(report.t, spec.n, report.world.counts(), report.world.ever_count());
        if report.t >= spec.t_max / 2 {
            qs.push(sample.clone());
        }
        if keep_history {
            history.push(sample);
        }
    })?;
    let world = simulation.world();
    world
        .assert_invariants()
        .map_err(|message| -> Box<dyn Error + Send + Sync> { message.into() })?;
    let counts = world.counts();
    let survived = counts.1 > 0 && world.continuous_time >= spec.t_max as f64;
    let qs_mean = |select: fn(&Sample) -> f64| survived.then(|| qs.iter().map(select).sum::<f64>() / qs.len() as f64);
    Ok(TrialResult {
        seed: spec.seed,
        lambda: spec.lambda,
        sigma: spec.sigma,
        delta: spec.delta,
        survived,
        extinction_time: world.extinction_time,
        i_qs: qs_mean(|sample| sample.i),
        s_qs: qs_mean(|sample| sample.s),
        known_qs: qs_mean(|sample| sample.s + sample.r),
        ever_fraction: world.ever_count() as f64 / spec.n as f64,
        final_ignorant: counts.0 as f64 / spec.n as f64,
        final_s: counts.1,
        conversations: world.conversations,
        spreader_time: world.spreader_time,
        isolated_fraction,
        history,
    })
}

fn network_key(spec: &TrialSpec) -> (NetworkKind, usize, u64, u64) {
    (spec.network, spec.n, spec.mean_degree.to_bits(), spec.seed)
}

pub fn run_specs_parallel(specs: &[TrialSpec], threads: usize) -> Result<Vec<TrialResult>, Box<dyn Error + Send + Sync>> {
    let mut networks = BTreeMap::new();
    for spec in specs {
        networks
            .entry(network_key(spec))
            .or_insert_with(|| Arc::new(ContactNetwork::generate(spec)));
    }
    let pool = ThreadPoolBuilder::new().num_threads(threads.max(1)).build()?;
    pool.install(|| {
        specs
            .par_iter()
            .map(|spec| run_trial_with_network(spec, Arc::clone(&networks[&network_key(spec)])))
            .collect()
    })
}
