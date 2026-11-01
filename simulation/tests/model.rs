use rumor_simulation::config::{ForgetMode, NetworkKind, TrialSpec};
use rumor_simulation::dynamics::apply_conversation_draw;
use rumor_simulation::dynamics::{advance_to, RumorParameters};
use rumor_simulation::network::ContactNetwork;
use rumor_simulation::state::{IndexedSet, RumorState};
use rumor_simulation::world::RumorWorld;
use socsim_core::SimRng;

#[test]
fn indexed_set_swap_remove_keeps_membership_consistent() {
    let mut set = IndexedSet::new(5);
    assert!(set.insert(1));
    assert!(set.insert(4));
    assert!(!set.insert(1));
    assert!(set.remove(1));
    assert!(!set.contains(1));
    assert!(set.contains(4));
    assert_eq!(set.len(), 1);
}

#[test]
fn initial_state_uses_ceiling_one_percent_and_marks_ever_heard() {
    let net = ContactNetwork::Complete { n: 101 };
    let world = RumorWorld::new(101, 10, net, ForgetMode::Both, &mut SimRng::from_seed(7));
    assert_eq!(world.counts(), (99, 2, 0));
    assert_eq!(world.ever_count(), 2);
    world.assert_invariants().unwrap();
}

#[test]
fn conversation_changes_only_the_specified_party() {
    let mut world = RumorWorld::from_states(
        vec![RumorState::Spreader, RumorState::Ignorant, RumorState::Stifler],
        ContactNetwork::Complete { n: 3 },
        ForgetMode::Both,
    );
    apply_conversation_draw(&mut world, 0, 1, 0.0, 1.0, 1.0);
    assert_eq!(world.state(0), RumorState::Spreader);
    assert_eq!(world.state(1), RumorState::Spreader);
    apply_conversation_draw(&mut world, 0, 2, 0.0, 1.0, 1.0);
    assert_eq!(world.state(0), RumorState::Stifler);
    assert_eq!(world.state(2), RumorState::Stifler);
    world.assert_invariants().unwrap();
}

#[test]
fn complete_partner_is_uniform_over_everyone_except_self() {
    let net = ContactNetwork::Complete { n: 2 };
    let mut rng = SimRng::from_seed(3);
    for _ in 0..20 {
        assert_eq!(net.sample_neighbor(0, &mut rng), Some(1));
        assert_eq!(net.sample_neighbor(1, &mut rng), Some(0));
    }
}

#[test]
fn isolated_spreader_conversation_has_no_state_effect() {
    let spec = TrialSpec::new(2, NetworkKind::Er, 0.0, 1.0, 1.0, 0.0, 10, ForgetMode::Both, 0);
    let result = rumor_simulation::runner::run_trial(&spec).unwrap();
    assert_eq!(result.final_s, 1);
    assert_eq!(result.isolated_fraction, 1.0);
}

#[test]
fn carrying_an_event_across_unit_steps_preserves_the_trajectory() {
    let states = vec![RumorState::Spreader, RumorState::Ignorant, RumorState::Stifler];
    let mut stepped = RumorWorld::from_states(states.clone(), ContactNetwork::Complete { n: 3 }, ForgetMode::Both);
    let mut unbroken = RumorWorld::from_states(states, ContactNetwork::Complete { n: 3 }, ForgetMode::Both);
    let parameters = RumorParameters {
        lambda: 0.7,
        sigma: 0.8,
        delta: 0.2,
    };
    let mut stepped_rng = SimRng::from_seed(999);
    let mut unbroken_rng = SimRng::from_seed(999);
    advance_to(&mut stepped, 1.0, parameters, &mut stepped_rng);
    advance_to(&mut stepped, 2.0, parameters, &mut stepped_rng);
    advance_to(&mut unbroken, 2.0, parameters, &mut unbroken_rng);
    assert_eq!(stepped.counts(), unbroken.counts());
    assert_eq!(stepped.ever_count(), unbroken.ever_count());
    assert_eq!(stepped.conversations, unbroken.conversations);
    assert_eq!(stepped.spreader_time, unbroken.spreader_time);
}

#[test]
fn stifler_only_forgetting_excludes_spreaders() {
    let world = RumorWorld::from_states(
        vec![RumorState::Spreader, RumorState::Spreader, RumorState::Stifler],
        ContactNetwork::Complete { n: 3 },
        ForgetMode::StiflerOnly,
    );
    assert_eq!(world.forgettable_count(), 1);
}
