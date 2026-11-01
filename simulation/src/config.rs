use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkKind {
    Complete,
    Er,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgetMode {
    Both,
    StiflerOnly,
}

fn default_forget() -> ForgetMode {
    ForgetMode::Both
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub name: String,
    pub network: NetworkKind,
    pub n: usize,
    pub mean_degree: Option<f64>,
    pub sigma: f64,
    pub lambdas: Vec<f64>,
    pub deltas: Vec<f64>,
    pub t_max: u64,
    pub seeds: Vec<u64>,
    #[serde(default = "default_forget")]
    pub forget: ForgetMode,
}

impl ExperimentConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let config: Self = toml::from_str(&fs::read_to_string(path)?)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.name.is_empty() || self.n < 2 || self.t_max == 0 || self.seeds.is_empty() {
            return Err("name, n >= 2, t_max > 0, and at least one seed are required".into());
        }
        if !(0.0..=1.0).contains(&self.sigma)
            || self.lambdas.iter().any(|value| !(0.0..=1.0).contains(value))
            || self.deltas.iter().any(|value| *value < 0.0 || !value.is_finite())
        {
            return Err("sigma/lambda must be in [0,1] and delta must be finite and non-negative".into());
        }
        if self.network == NetworkKind::Er && self.mean_degree.is_none() {
            return Err("mean_degree is required for an ER network".into());
        }
        Ok(())
    }

    pub fn trial_specs(&self, selected_seeds: Option<&[u64]>) -> Result<Vec<TrialSpec>, Box<dyn Error>> {
        let seeds: Vec<u64> = match selected_seeds {
            Some(requested) => {
                let allowed: BTreeSet<_> = self.seeds.iter().copied().collect();
                if requested.iter().any(|seed| !allowed.contains(seed)) {
                    return Err("--seeds contains a seed not declared by the config".into());
                }
                requested.to_vec()
            }
            None => self.seeds.clone(),
        };
        let mut specs = Vec::with_capacity(self.lambdas.len() * self.deltas.len() * seeds.len());
        for &lambda in &self.lambdas {
            for &delta in &self.deltas {
                for &seed in &seeds {
                    specs.push(TrialSpec::new(
                        self.n,
                        self.network,
                        self.mean_degree.unwrap_or(0.0),
                        lambda,
                        self.sigma,
                        delta,
                        self.t_max,
                        self.forget,
                        seed,
                    ));
                }
            }
        }
        Ok(specs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrialSpec {
    pub n: usize,
    pub network: NetworkKind,
    pub mean_degree: f64,
    pub lambda: f64,
    pub sigma: f64,
    pub delta: f64,
    pub t_max: u64,
    pub forget: ForgetMode,
    pub seed: u64,
}

impl TrialSpec {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        n: usize,
        network: NetworkKind,
        mean_degree: f64,
        lambda: f64,
        sigma: f64,
        delta: f64,
        t_max: u64,
        forget: ForgetMode,
        seed: u64,
    ) -> Self {
        Self {
            n,
            network,
            mean_degree,
            lambda,
            sigma,
            delta,
            t_max,
            forget,
            seed,
        }
    }
}

pub fn parse_seed_filter(value: &str) -> Result<Vec<u64>, Box<dyn Error>> {
    value.split(',').map(|part| Ok(part.trim().parse()?)).collect()
}
