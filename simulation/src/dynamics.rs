use rand::Rng;
use socsim_core::SimRng;

use crate::state::RumorState;
use crate::world::{PendingEvent, RumorWorld};

#[derive(Clone, Copy, Debug)]
pub struct RumorParameters {
    pub lambda: f64,
    pub sigma: f64,
    pub delta: f64,
}

pub fn apply_conversation_draw(world: &mut RumorWorld, speaker: usize, partner: usize, draw: f64, lambda: f64, sigma: f64) {
    match world.state(partner) {
        RumorState::Ignorant if draw < lambda => world.transition(partner, RumorState::Spreader),
        RumorState::Spreader | RumorState::Stifler if draw < sigma => {
            world.transition(speaker, RumorState::Stifler);
        }
        _ => {}
    }
}

#[inline]
fn execute_event(world: &mut RumorWorld, parameters: RumorParameters, rng: &mut SimRng) {
    let spreaders = world.spreader_count() as f64;
    let forgetting_rate = parameters.delta * world.forgettable_count() as f64;
    if rng.gen::<f64>() * (spreaders + forgetting_rate) < spreaders {
        let speaker = world.sample_spreader(rng).expect("positive spreader rate");
        world.conversations += 1;
        if let Some(partner) = world.network.sample_neighbor(speaker, rng) {
            let draw = rng.gen::<f64>();
            apply_conversation_draw(world, speaker, partner, draw, parameters.lambda, parameters.sigma);
        }
    } else {
        let person = world.sample_forgettable(rng).expect("positive forgetting rate");
        world.transition(person, RumorState::Ignorant);
    }
}

pub fn advance_to(world: &mut RumorWorld, target: f64, parameters: RumorParameters, rng: &mut SimRng) {
    while world.continuous_time < target && !world.spreaders.is_empty() {
        if world.pending.is_none() {
            let total_rate = world.spreader_count() as f64 + parameters.delta * world.forgettable_count() as f64;
            if total_rate == 0.0 {
                break;
            }
            let uniform = rng.gen::<f64>().max(f64::MIN_POSITIVE);
            world.pending = Some(PendingEvent {
                at: world.continuous_time - uniform.ln() / total_rate,
            });
        }
        let event_time = world.pending.expect("pending event was just created").at;
        if event_time > target {
            world.spreader_time += world.spreader_count() as f64 * (target - world.continuous_time);
            world.continuous_time = target;
            break;
        }
        world.spreader_time += world.spreader_count() as f64 * (event_time - world.continuous_time);
        world.continuous_time = event_time;
        world.pending = None;
        execute_event(world, parameters, rng);
        debug_assert_eq!(world.counts().0 + world.counts().1 + world.counts().2, world.len());
        if world.spreaders.is_empty() {
            world.extinction_time = Some(world.continuous_time);
        }
    }
    if world.continuous_time < target && !world.spreaders.is_empty() {
        world.spreader_time += world.spreader_count() as f64 * (target - world.continuous_time);
        world.continuous_time = target;
    }
}
