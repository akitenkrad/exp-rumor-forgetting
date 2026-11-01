use std::sync::Arc;

use rand::Rng;
use socsim_core::{AgentId, SimRng};
use socsim_net::SocialNetwork;

use crate::config::{NetworkKind, TrialSpec};
use crate::seeding::network_seed;

#[derive(Clone, Debug)]
pub enum ContactNetwork {
    Complete {
        n: usize,
    },
    Csr {
        n: usize,
        offsets: Vec<usize>,
        neighbors: Vec<usize>,
        isolated: usize,
    },
}

impl ContactNetwork {
    pub fn generate(spec: &TrialSpec) -> Self {
        match spec.network {
            NetworkKind::Complete => Self::Complete { n: spec.n },
            NetworkKind::Er => {
                let ids: Vec<_> = (0..spec.n as u64).map(AgentId).collect();
                let p = spec.mean_degree / (spec.n - 1) as f64;
                let mut rng = SimRng::from_seed(network_seed(spec.seed));
                let network = SocialNetwork::erdos_renyi(&ids, p, &mut rng);
                let mut offsets = Vec::with_capacity(spec.n + 1);
                let mut neighbors = Vec::new();
                let mut isolated = 0;
                offsets.push(0);
                for id in ids {
                    // This allocation happens exactly once per vertex while creating the CSR.
                    // The Gillespie loop only reads the CSR slices below.
                    let row = network.neighbors(id);
                    if row.is_empty() {
                        isolated += 1;
                    }
                    neighbors.extend(row.into_iter().map(|neighbor| neighbor.0 as usize));
                    offsets.push(neighbors.len());
                }
                Self::Csr {
                    n: spec.n,
                    offsets,
                    neighbors,
                    isolated,
                }
            }
        }
    }

    pub fn n(&self) -> usize {
        match self {
            Self::Complete { n } | Self::Csr { n, .. } => *n,
        }
    }

    pub fn isolated_fraction(&self) -> f64 {
        match self {
            Self::Complete { .. } => 0.0,
            Self::Csr { n, isolated, .. } => *isolated as f64 / *n as f64,
        }
    }

    #[inline]
    pub fn sample_neighbor(&self, speaker: usize, rng: &mut SimRng) -> Option<usize> {
        match self {
            Self::Complete { n } => {
                if *n < 2 {
                    return None;
                }
                let draw = rng.gen_range(0..(*n - 1));
                Some(if draw >= speaker { draw + 1 } else { draw })
            }
            Self::Csr { offsets, neighbors, .. } => {
                let row = &neighbors[offsets[speaker]..offsets[speaker + 1]];
                (!row.is_empty()).then(|| row[rng.gen_range(0..row.len())])
            }
        }
    }
}

pub type SharedNetwork = Arc<ContactNetwork>;
