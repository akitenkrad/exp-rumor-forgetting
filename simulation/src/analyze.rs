use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::runner::TrialResult;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionAggregate {
    pub lambda: f64,
    pub delta: f64,
    pub trials: usize,
    pub surviving_trials: usize,
    pub p_survive: f64,
    pub i_qs: Option<f64>,
    pub s_qs: Option<f64>,
    pub known_qs: Option<f64>,
    pub ever: f64,
    pub ignorant_final: f64,
}

pub fn aggregate_conditions(trials: &[TrialResult]) -> Vec<ConditionAggregate> {
    let mut groups: BTreeMap<(u64, u64), Vec<&TrialResult>> = BTreeMap::new();
    for trial in trials {
        groups.entry((trial.lambda.to_bits(), trial.delta.to_bits())).or_default().push(trial);
    }
    groups
        .into_values()
        .map(|group| {
            let count = group.len();
            let survivors: Vec<_> = group.iter().copied().filter(|trial| trial.survived).collect();
            let mean_all = |select: fn(&TrialResult) -> f64| group.iter().map(|trial| select(trial)).sum::<f64>() / count as f64;
            let mean_survivors = |select: fn(&TrialResult) -> Option<f64>| {
                (!survivors.is_empty()).then(|| {
                    survivors
                        .iter()
                        .map(|trial| select(trial).expect("survivors have QS samples"))
                        .sum::<f64>()
                        / survivors.len() as f64
                })
            };
            ConditionAggregate {
                lambda: group[0].lambda,
                delta: group[0].delta,
                trials: count,
                surviving_trials: survivors.len(),
                p_survive: survivors.len() as f64 / count as f64,
                i_qs: mean_survivors(|trial| trial.i_qs),
                s_qs: mean_survivors(|trial| trial.s_qs),
                known_qs: mean_survivors(|trial| trial.known_qs),
                ever: mean_all(|trial| trial.ever_fraction),
                ignorant_final: mean_all(|trial| trial.final_ignorant),
            }
        })
        .collect()
}

pub fn condition_label(lambda: f64, delta: f64) -> String {
    let number = |value: f64| format!("{value:.2}").replace('.', "p");
    format!("l{}_d{}", number(lambda), number(delta))
}
