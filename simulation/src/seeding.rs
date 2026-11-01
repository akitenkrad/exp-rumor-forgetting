use crate::config::{ForgetMode, TrialSpec};
use socsim_core::derive_seed;

const NETWORK_STREAM: u64 = 0x006e_6574_776f_726b;
const INITIAL_STREAM: u64 = 0x0069_6e69_7469_616c;
const DYNAMICS_STREAM: u64 = 0x6479_6e61_6d69_6373;

pub fn network_seed(seed: u64) -> u64 {
    derive_seed(seed, &[NETWORK_STREAM])
}

pub fn initial_seed(seed: u64) -> u64 {
    derive_seed(seed, &[INITIAL_STREAM])
}

pub fn dynamics_seed(spec: &TrialSpec) -> u64 {
    derive_seed(
        spec.seed,
        &[
            DYNAMICS_STREAM,
            spec.lambda.to_bits(),
            spec.sigma.to_bits(),
            spec.delta.to_bits(),
            match spec.forget {
                ForgetMode::Both => 0,
                ForgetMode::StiflerOnly => 1,
            },
        ],
    )
}
