use std::sync::Arc;

use rand::seq::SliceRandom;
use socsim_core::{AgentId, SimClock, SimRng, WorldState};

use crate::config::ForgetMode;
use crate::network::{ContactNetwork, SharedNetwork};
use crate::state::{IndexedSet, RumorState};

#[derive(Clone, Copy, Debug)]
pub struct PendingEvent {
    pub at: f64,
}

pub struct RumorWorld {
    clock: SimClock,
    pub(crate) states: Vec<RumorState>,
    pub(crate) spreaders: IndexedSet,
    pub(crate) nonignorant: IndexedSet,
    pub(crate) stiflers: IndexedSet,
    ever_heard: Vec<bool>,
    ever_count: usize,
    pub network: SharedNetwork,
    pub forget: ForgetMode,
    pub continuous_time: f64,
    pub pending: Option<PendingEvent>,
    pub conversations: u64,
    pub spreader_time: f64,
    pub extinction_time: Option<f64>,
}

impl RumorWorld {
    pub fn new(n: usize, t_max: u64, network: ContactNetwork, forget: ForgetMode, rng: &mut SimRng) -> Self {
        let mut states = vec![RumorState::Ignorant; n];
        let mut order: Vec<usize> = (0..n).collect();
        order.shuffle(rng);
        let initially_spreading = n.div_ceil(100);
        for &index in &order[..initially_spreading] {
            states[index] = RumorState::Spreader;
        }
        Self::from_parts(states, Arc::new(network), forget, t_max)
    }

    pub fn new_shared(n: usize, t_max: u64, network: SharedNetwork, forget: ForgetMode, rng: &mut SimRng) -> Self {
        let mut states = vec![RumorState::Ignorant; n];
        let mut order: Vec<usize> = (0..n).collect();
        order.shuffle(rng);
        for &index in &order[..n.div_ceil(100)] {
            states[index] = RumorState::Spreader;
        }
        Self::from_parts(states, network, forget, t_max)
    }

    pub fn from_states(states: Vec<RumorState>, network: ContactNetwork, forget: ForgetMode) -> Self {
        Self::from_parts(states, Arc::new(network), forget, u64::MAX)
    }

    fn from_parts(states: Vec<RumorState>, network: SharedNetwork, forget: ForgetMode, t_max: u64) -> Self {
        let n = states.len();
        assert_eq!(network.n(), n);
        let mut spreaders = IndexedSet::new(n);
        let mut nonignorant = IndexedSet::new(n);
        let mut stiflers = IndexedSet::new(n);
        let mut ever_heard = vec![false; n];
        let mut ever_count = 0;
        for (index, &state) in states.iter().enumerate() {
            match state {
                RumorState::Ignorant => {}
                RumorState::Spreader => {
                    spreaders.insert(index);
                    nonignorant.insert(index);
                    ever_heard[index] = true;
                    ever_count += 1;
                }
                RumorState::Stifler => {
                    stiflers.insert(index);
                    nonignorant.insert(index);
                    ever_heard[index] = true;
                    ever_count += 1;
                }
            }
        }
        Self {
            clock: SimClock::new(t_max),
            states,
            spreaders,
            nonignorant,
            stiflers,
            ever_heard,
            ever_count,
            network,
            forget,
            continuous_time: 0.0,
            pending: None,
            conversations: 0,
            spreader_time: 0.0,
            extinction_time: None,
        }
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
    pub fn state(&self, index: usize) -> RumorState {
        self.states[index]
    }
    pub fn counts(&self) -> (usize, usize, usize) {
        let s = self.spreaders.len();
        let r = self.stiflers.len();
        (self.len() - s - r, s, r)
    }
    pub fn ever_count(&self) -> usize {
        self.ever_count
    }
    pub fn spreader_count(&self) -> usize {
        self.spreaders.len()
    }
    pub fn stifler_count(&self) -> usize {
        self.stiflers.len()
    }
    pub fn forgettable_count(&self) -> usize {
        match self.forget {
            ForgetMode::Both => self.nonignorant.len(),
            ForgetMode::StiflerOnly => self.stiflers.len(),
        }
    }
    pub fn sample_spreader(&self, rng: &mut SimRng) -> Option<usize> {
        self.spreaders.sample(rng)
    }
    pub fn sample_forgettable(&self, rng: &mut SimRng) -> Option<usize> {
        match self.forget {
            ForgetMode::Both => self.nonignorant.sample(rng),
            ForgetMode::StiflerOnly => self.stiflers.sample(rng),
        }
    }

    pub fn transition(&mut self, index: usize, next: RumorState) {
        let previous = self.states[index];
        if previous == next {
            return;
        }
        match previous {
            RumorState::Ignorant => {}
            RumorState::Spreader => {
                self.spreaders.remove(index);
                self.nonignorant.remove(index);
            }
            RumorState::Stifler => {
                self.stiflers.remove(index);
                self.nonignorant.remove(index);
            }
        }
        match next {
            RumorState::Ignorant => {}
            RumorState::Spreader => {
                self.spreaders.insert(index);
                self.nonignorant.insert(index);
            }
            RumorState::Stifler => {
                self.stiflers.insert(index);
                self.nonignorant.insert(index);
            }
        }
        if next != RumorState::Ignorant && !self.ever_heard[index] {
            self.ever_heard[index] = true;
            self.ever_count += 1;
        }
        self.states[index] = next;
    }

    pub fn assert_invariants(&self) -> Result<(), String> {
        let (i, s, r) = self.counts();
        if i + s + r != self.len() {
            return Err("state counts do not sum to N".into());
        }
        for (index, state) in self.states.iter().enumerate() {
            if self.spreaders.contains(index) != (*state == RumorState::Spreader)
                || self.stiflers.contains(index) != (*state == RumorState::Stifler)
                || self.nonignorant.contains(index) != (*state != RumorState::Ignorant)
            {
                return Err(format!("indexed set mismatch at agent {index}"));
            }
            if *state != RumorState::Ignorant && !self.ever_heard[index] {
                return Err(format!("ever-heard flag missing at agent {index}"));
            }
        }
        if self.ever_heard.iter().filter(|flag| **flag).count() != self.ever_count {
            return Err("ever-heard count mismatch".into());
        }
        Ok(())
    }
}

impl WorldState for RumorWorld {
    fn agent_ids(&self) -> Vec<AgentId> {
        (0..self.len() as u64).map(AgentId).collect()
    }
    fn clock(&self) -> &SimClock {
        &self.clock
    }
    fn clock_mut(&mut self) -> &mut SimClock {
        &mut self.clock
    }
}
