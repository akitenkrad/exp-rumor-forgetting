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
    // Two decimals by default (`0p50`); a third one only when it is not zero (`0p001`),
    // so the small-delta grid does not collapse onto `0p00`.
    let number = |value: f64| {
        let mut text = format!("{value:.3}");
        if text.ends_with('0') {
            text.pop();
        }
        text.replace('.', "p")
    };
    format!("l{}_d{}", number(lambda), number(delta))
}

#[cfg(test)]
mod label_tests {
    use super::condition_label;

    #[test]
    fn small_deltas_get_distinct_labels() {
        assert_eq!(condition_label(0.5, 0.2), "l0p50_d0p20");
        assert_eq!(condition_label(0.25, 0.001), "l0p25_d0p001");
        assert_ne!(condition_label(0.25, 0.001), condition_label(0.25, 0.002));
        assert_eq!(condition_label(1.0, 0.0), "l1p00_d0p00");
    }
}
