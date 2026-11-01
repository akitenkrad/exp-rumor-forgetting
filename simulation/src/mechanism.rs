use socsim_core::{AgentId, Mechanism, Phase, Result, Scheduler, SimRng, StepContext};

use crate::dynamics::{advance_to, RumorParameters};
use crate::world::RumorWorld;

pub struct RumorGillespie {
    parameters: RumorParameters,
}

pub struct EventDrivenScheduler;

impl Scheduler<RumorWorld> for EventDrivenScheduler {
    fn activation_order(&mut self, _world: &RumorWorld, _rng: &mut SimRng) -> Vec<AgentId> {
        Vec::new()
    }
}

impl RumorGillespie {
    pub fn new(lambda: f64, sigma: f64, delta: f64) -> Self {
        Self {
            parameters: RumorParameters { lambda, sigma, delta },
        }
    }
}

impl Mechanism<RumorWorld> for RumorGillespie {
    fn name(&self) -> &str {
        "rumor_gillespie"
    }
    fn phases(&self) -> &'static [Phase] {
        &[Phase::Interaction]
    }

    fn apply(&mut self, phase: Phase, ctx: &mut StepContext<'_, RumorWorld>) -> Result<()> {
        debug_assert_eq!(phase, Phase::Interaction);
        advance_to(ctx.world, ctx.clock.t() as f64, self.parameters, ctx.rng);
        if ctx.world.spreader_count() == 0 {
            ctx.request_stop();
        }
        Ok(())
    }
}
