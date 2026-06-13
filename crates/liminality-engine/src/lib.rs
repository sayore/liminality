//! liminality-engine
//!
//! This crate implements simulation modes and state evaluation for liminality.
//! It depends on liminality-model.
//! It must support debug tick and predictive modes eventually.

use std::collections::HashMap;

use liminality_model::WorldModel;
use thiserror::Error;

const COAL_ID: &str = "coal";
const ORE_ID: &str = "iron_ore";
const INGOT_ID: &str = "iron_ingot";
const FURNACE_ID: &str = "furnace_1";

#[derive(Debug, Clone, PartialEq)]
pub enum SimMode {
    DebugTick(DebugTickConfig),
    Predictive(PredictiveConfig),
    Hybrid(HybridConfig),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DebugTickConfig {
    pub tick_duration_secs: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PredictiveConfig {}

#[derive(Debug, Clone, PartialEq)]
pub struct HybridConfig {}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedState {
    pub time: u64,
    pub resources: HashMap<String, u32>,
    pub transformer_states: HashMap<String, TransformerState>,
    pub stop_reason: Option<StopReason>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PredictedEvent {
    pub time: u64,
    pub reason: StopReason,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateSegment {
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub affected_nodes: Vec<String>,
    pub formula: SegmentFormula,
    pub stop_reason: Option<StopReason>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SegmentFormula {
    Constant,
    LinearTransformer {
        ops_per_sec: f64,
        input_id: String,
        input_qty: u32,
        fuel_id: String,
        fuel_ops: u32,
        output_id: String,
        output_qty: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    OutputFull,
    InputEmpty,
    FuelEmpty,
}

#[derive(Debug, Error, PartialEq)]
pub enum SimulationError {
    #[error("Target time is in the past")]
    TimeInPast,
    #[error("Simulation stopped")]
    Stopped,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transformer {
    pub id: String,
    pub input_id: String,
    pub input_qty: u32,
    pub fuel_id: String,
    pub fuel_ops: u32,
    pub output_id: String,
    pub output_qty: u32,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransformerState {
    pub progress_secs: u64,
    pub fuel_ops_left: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseState {
    pub time: u64,
    pub resources: HashMap<String, u32>,
    pub capacities: HashMap<String, u32>,
    pub transformers: Vec<Transformer>,
    pub transformer_states: HashMap<String, TransformerState>,
}

pub struct SimulationEngine {
    pub base_state: BaseState,
    pub mode: SimMode,
}

impl SimulationEngine {
    pub fn new(base_state: BaseState, mode: SimMode) -> Self {
        Self { base_state, mode }
    }

    pub fn state_at(&self, target_time: u64) -> Result<ComputedState, SimulationError> {
        if target_time < self.base_state.time {
            return Err(SimulationError::TimeInPast);
        }

        match &self.mode {
            SimMode::DebugTick(config) => {
                let mut current_state = self.base_state.clone();
                let mut current_time = current_state.time;
                let mut stop_reason = None;

                while current_time < target_time && stop_reason.is_none() {
                    let next_time =
                        std::cmp::min(current_time + config.tick_duration_secs, target_time);
                    let delta = next_time - current_time;
                    let result = self.step_debug(&current_state, delta);
                    current_state = result.0;
                    stop_reason = result.1;
                    current_time = next_time;
                }

                Ok(ComputedState {
                    time: current_time,
                    resources: current_state.resources,
                    transformer_states: current_state.transformer_states,
                    stop_reason,
                })
            }
            SimMode::Predictive(_) => self.predict_until(target_time),
            SimMode::Hybrid(_) => self.predict_until(target_time),
        }
    }

    pub fn step_debug(&self, state: &BaseState, delta: u64) -> (BaseState, Option<StopReason>) {
        let mut next_state = state.clone();
        let mut stop_reason = None;

        for transformer in &self.base_state.transformers {
            let mut t_state = next_state
                .transformer_states
                .remove(&transformer.id)
                .unwrap_or(TransformerState {
                    progress_secs: 0,
                    fuel_ops_left: 0,
                });

            for _ in 0..delta {
                if t_state.fuel_ops_left == 0 {
                    let fuel_available = next_state
                        .resources
                        .get(&transformer.fuel_id)
                        .copied()
                        .unwrap_or(0);
                    if fuel_available > 0 {
                        *next_state
                            .resources
                            .entry(transformer.fuel_id.clone())
                            .or_insert(0) -= 1;
                        t_state.fuel_ops_left = transformer.fuel_ops;
                    } else {
                        stop_reason = Some(StopReason::FuelEmpty);
                        break;
                    }
                }

                let input_available = next_state
                    .resources
                    .get(&transformer.input_id)
                    .copied()
                    .unwrap_or(0);
                if input_available < transformer.input_qty {
                    stop_reason = Some(StopReason::InputEmpty);
                    break;
                }

                let current_output = next_state
                    .resources
                    .get(&transformer.output_id)
                    .copied()
                    .unwrap_or(0);
                let max_output = self
                    .base_state
                    .capacities
                    .get(&transformer.output_id)
                    .copied()
                    .unwrap_or(u32::MAX);
                if current_output + transformer.output_qty > max_output {
                    stop_reason = Some(StopReason::OutputFull);
                    break;
                }

                t_state.progress_secs += 1;

                if t_state.progress_secs >= transformer.duration_secs {
                    *next_state
                        .resources
                        .entry(transformer.input_id.clone())
                        .or_insert(0) -= transformer.input_qty;
                    *next_state
                        .resources
                        .entry(transformer.output_id.clone())
                        .or_insert(0) += transformer.output_qty;

                    t_state.progress_secs = 0;
                    t_state.fuel_ops_left -= 1;
                }
            }

            next_state
                .transformer_states
                .insert(transformer.id.clone(), t_state);

            if stop_reason.is_some() {
                break;
            }
        }

        next_state.time += delta;
        (next_state, stop_reason)
    }

    pub fn predict_until(&self, target_time: u64) -> Result<ComputedState, SimulationError> {
        let delta = target_time.saturating_sub(self.base_state.time);

        let mut computed_resources = self.base_state.resources.clone();
        let mut computed_t_states = self.base_state.transformer_states.clone();
        let mut final_stop_reason = None;
        let mut actual_delta = delta;

        if let Some(event) = self.next_event_after(self.base_state.time)
            && event.time <= target_time
        {
            actual_delta = event.time - self.base_state.time;
            final_stop_reason = Some(event.reason);
        }

        for transformer in &self.base_state.transformers {
            let mut t_state =
                computed_t_states
                    .remove(&transformer.id)
                    .unwrap_or(TransformerState {
                        progress_secs: 0,
                        fuel_ops_left: 0,
                    });

            let total_progress = t_state.progress_secs + actual_delta;
            let ops = (total_progress / transformer.duration_secs) as u32;
            t_state.progress_secs = total_progress % transformer.duration_secs;

            if ops > 0 {
                if let Some(input) = computed_resources.get_mut(&transformer.input_id) {
                    *input = input.saturating_sub(ops * transformer.input_qty);
                }

                let total_ops_to_cover = ops.saturating_sub(t_state.fuel_ops_left);
                let new_fuel_needed = if total_ops_to_cover > 0 {
                    total_ops_to_cover.div_ceil(transformer.fuel_ops)
                } else if t_state.fuel_ops_left == 0 {
                    1
                } else {
                    0
                };

                if new_fuel_needed > 0
                    && let Some(fuel) = computed_resources.get_mut(&transformer.fuel_id)
                {
                    *fuel = fuel.saturating_sub(new_fuel_needed);
                }

                let total_fuel_ops = new_fuel_needed * transformer.fuel_ops + t_state.fuel_ops_left;
                t_state.fuel_ops_left = total_fuel_ops.saturating_sub(ops);

                if let Some(output) = computed_resources.get_mut(&transformer.output_id) {
                    *output = output.saturating_add(ops * transformer.output_qty);
                } else {
                    computed_resources
                        .insert(transformer.output_id.clone(), ops * transformer.output_qty);
                }
            }

            computed_t_states.insert(transformer.id.clone(), t_state);
        }

        Ok(ComputedState {
            time: self.base_state.time + actual_delta,
            resources: computed_resources,
            transformer_states: computed_t_states,
            stop_reason: final_stop_reason,
        })
    }

    pub fn next_event_after(&self, _time: u64) -> Option<PredictedEvent> {
        let mut earliest_event: Option<PredictedEvent> = None;

        for transformer in &self.base_state.transformers {
            let t_state = self
                .base_state
                .transformer_states
                .get(&transformer.id)
                .cloned()
                .unwrap_or(TransformerState {
                    progress_secs: 0,
                    fuel_ops_left: 0,
                });

            let input_available = self
                .base_state
                .resources
                .get(&transformer.input_id)
                .copied()
                .unwrap_or(0);
            let fuel_available = self
                .base_state
                .resources
                .get(&transformer.fuel_id)
                .copied()
                .unwrap_or(0);
            let current_output = self
                .base_state
                .resources
                .get(&transformer.output_id)
                .copied()
                .unwrap_or(0);
            let max_output = self
                .base_state
                .capacities
                .get(&transformer.output_id)
                .copied()
                .unwrap_or(u32::MAX);

            let output_space = max_output.saturating_sub(current_output);
            let max_ops_input = input_available / transformer.input_qty;
            let max_ops_fuel = fuel_available * transformer.fuel_ops + t_state.fuel_ops_left;
            let max_ops_output = output_space / transformer.output_qty;
            let limiting_ops = max_ops_input.min(max_ops_fuel).min(max_ops_output);
            let time_to_stop =
                (limiting_ops as u64) * transformer.duration_secs - t_state.progress_secs;
            let event_time = self.base_state.time + time_to_stop;

            let reason = if limiting_ops == max_ops_output {
                StopReason::OutputFull
            } else if limiting_ops == max_ops_input {
                StopReason::InputEmpty
            } else {
                StopReason::FuelEmpty
            };

            if let Some(current_earliest) = &earliest_event {
                if event_time < current_earliest.time {
                    earliest_event = Some(PredictedEvent {
                        time: event_time,
                        reason,
                    });
                }
            } else {
                earliest_event = Some(PredictedEvent {
                    time: event_time,
                    reason,
                });
            }
        }

        earliest_event
    }

    pub fn materialize_at(&mut self, target_time: u64) -> Result<(), SimulationError> {
        let computed_state = self.state_at(target_time)?;
        self.base_state.time = computed_state.time;
        self.base_state.resources = computed_state.resources;
        self.base_state.transformer_states = computed_state.transformer_states;
        Ok(())
    }
}

pub fn simulate_furnace_line(start_state: &WorldModel, seconds: u64) -> WorldModel {
    let engine = SimulationEngine::new(
        furnace_line_base_state(start_state),
        SimMode::Predictive(PredictiveConfig {}),
    );
    let computed_state = engine
        .state_at(seconds)
        .expect("predictive furnace-line simulation should be valid");
    world_model_from_computed_state(&computed_state)
}

fn furnace_line_base_state(start_state: &WorldModel) -> BaseState {
    let mut resources = HashMap::new();
    resources.insert(COAL_ID.to_string(), start_state.coal_storage);
    resources.insert(ORE_ID.to_string(), start_state.ore_storage);
    resources.insert(INGOT_ID.to_string(), start_state.output_storage);

    let mut capacities = HashMap::new();
    capacities.insert(INGOT_ID.to_string(), 64);

    let transformer = Transformer {
        id: FURNACE_ID.to_string(),
        input_id: ORE_ID.to_string(),
        input_qty: 1,
        fuel_id: COAL_ID.to_string(),
        fuel_ops: 8,
        output_id: INGOT_ID.to_string(),
        output_qty: 1,
        duration_secs: 10,
    };

    BaseState {
        time: 0,
        resources,
        capacities,
        transformers: vec![transformer],
        transformer_states: HashMap::new(),
    }
}

fn world_model_from_computed_state(state: &ComputedState) -> WorldModel {
    WorldModel {
        coal_storage: state.resources.get(COAL_ID).copied().unwrap_or(0),
        ore_storage: state.resources.get(ORE_ID).copied().unwrap_or(0),
        output_storage: state.resources.get(INGOT_ID).copied().unwrap_or(0),
    }
}

#[cfg(test)]
fn create_test_base_state() -> BaseState {
    furnace_line_base_state(&WorldModel::furnace_line_demo())
}

#[cfg(test)]
fn create_low_capacity_state(output_capacity: u32) -> BaseState {
    let mut state = create_test_base_state();
    state.capacities.insert(INGOT_ID.to_string(), output_capacity);
    state
}

#[cfg(test)]
fn create_sparse_input_state(ore: u32) -> BaseState {
    let mut state = create_test_base_state();
    state.resources.insert(ORE_ID.to_string(), ore);
    state
}

#[cfg(test)]
fn assert_world_model(state: &WorldModel, ingots: u32, ore: u32, coal: u32) {
    assert_eq!(state.output_storage, ingots);
    assert_eq!(state.ore_storage, ore);
    assert_eq!(state.coal_storage, coal);
}

#[cfg(test)]
fn assert_computed_resources(state: &ComputedState, ingots: u32, ore: u32, coal: u32) {
    assert_eq!(state.resources.get(INGOT_ID), Some(&ingots));
    assert_eq!(state.resources.get(ORE_ID), Some(&ore));
    assert_eq!(state.resources.get(COAL_ID), Some(&coal));
}

#[cfg(test)]
fn assert_stop_reason(state: &ComputedState, reason: Option<StopReason>) {
    assert_eq!(state.stop_reason, reason);
}

#[cfg(test)]
fn assert_state_time(state: &ComputedState, time: u64) {
    assert_eq!(state.time, time);
}

#[cfg(test)]
fn predictive_engine() -> SimulationEngine {
    SimulationEngine::new(create_test_base_state(), SimMode::Predictive(PredictiveConfig {}))
}

#[cfg(test)]
fn debug_engine(tick_duration_secs: u64) -> SimulationEngine {
    SimulationEngine::new(
        create_test_base_state(),
        SimMode::DebugTick(DebugTickConfig { tick_duration_secs }),
    )
}

#[cfg(test)]
fn low_capacity_debug_engine(output_capacity: u32, tick_duration_secs: u64) -> SimulationEngine {
    SimulationEngine::new(
        create_low_capacity_state(output_capacity),
        SimMode::DebugTick(DebugTickConfig { tick_duration_secs }),
    )
}

#[cfg(test)]
fn sparse_input_predictive_engine(ore: u32) -> SimulationEngine {
    SimulationEngine::new(
        create_sparse_input_state(ore),
        SimMode::Predictive(PredictiveConfig {}),
    )
}

#[cfg(test)]
fn simulated_world(seconds: u64) -> WorldModel {
    simulate_furnace_line(&WorldModel::furnace_line_demo(), seconds)
}

#[cfg(test)]
fn assert_simulated_world(seconds: u64, ingots: u32, ore: u32, coal: u32) {
    let state = simulated_world(seconds);
    assert_world_model(&state, ingots, ore, coal);
}

#[cfg(test)]
fn assert_prediction_matches_debug(seconds: u64) {
    let state_debug = debug_engine(1).state_at(seconds).unwrap();
    let state_pred = predictive_engine().state_at(seconds).unwrap();

    assert_eq!(state_debug.time, state_pred.time);
    assert_eq!(state_debug.resources, state_pred.resources);
    assert_eq!(state_debug.stop_reason, state_pred.stop_reason);
}

#[cfg(test)]
fn assert_materialized_state(seconds: u64, ingots: u32, coal: u32) {
    let mut engine = predictive_engine();
    engine.materialize_at(seconds).unwrap();

    assert_eq!(engine.base_state.time, seconds);
    assert_eq!(engine.base_state.resources.get(INGOT_ID), Some(&ingots));
    assert_eq!(engine.base_state.resources.get(COAL_ID), Some(&coal));
}

#[cfg(test)]
fn assert_sparse_input_stop(ore: u32, seconds: u64, stop_time: u64, ingots: u32) {
    let state = sparse_input_predictive_engine(ore).state_at(seconds).unwrap();

    assert_state_time(&state, stop_time);
    assert_eq!(state.resources.get(INGOT_ID), Some(&ingots));
    assert_eq!(state.resources.get(ORE_ID), Some(&0));
    assert_stop_reason(&state, Some(StopReason::InputEmpty));
}

#[cfg(test)]
fn assert_low_capacity_stop(output_capacity: u32, seconds: u64, stop_time: u64) {
    let state = low_capacity_debug_engine(output_capacity, 10)
        .state_at(seconds)
        .unwrap();

    assert_state_time(&state, stop_time);
    assert_eq!(state.resources.get(INGOT_ID), Some(&output_capacity));
    assert_stop_reason(&state, Some(StopReason::OutputFull));
}

#[cfg(test)]
fn assert_predictive_state(seconds: u64, time: u64, ingots: u32, ore: u32, coal: u32, reason: Option<StopReason>) {
    let state = predictive_engine().state_at(seconds).unwrap();
    assert_state_time(&state, time);
    assert_computed_resources(&state, ingots, ore, coal);
    assert_stop_reason(&state, reason);
}

#[cfg(test)]
fn assert_debug_state(seconds: u64, ingots: u32, ore: u32, coal: u32) {
    let state = debug_engine(10).state_at(seconds).unwrap();
    assert_state_time(&state, seconds);
    assert_computed_resources(&state, ingots, ore, coal);
    assert_stop_reason(&state, None);
}

#[cfg(test)]
fn output_full_reason() -> Option<StopReason> {
    Some(StopReason::OutputFull)
}

#[cfg(test)]
fn no_stop_reason() -> Option<StopReason> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_600s() {
        assert_simulated_world(600, 60, 68, 24);
    }

    #[test]
    fn it_works_640s() {
        assert_simulated_world(640, 64, 64, 24);
    }

    #[test]
    fn test_furnace_line_debug_tick_600s() {
        assert_debug_state(600, 60, 68, 24);
    }

    #[test]
    fn test_furnace_line_predictive_600s() {
        assert_predictive_state(600, 600, 60, 68, 24, no_stop_reason());
    }

    #[test]
    fn test_furnace_line_predictive_640s_output_full() {
        assert_predictive_state(640, 640, 64, 64, 24, output_full_reason());
        assert_predictive_state(700, 640, 64, 64, 24, output_full_reason());
    }

    #[test]
    fn test_predictive_matches_debug_tick() {
        assert_prediction_matches_debug(600);
    }

    #[test]
    fn test_materialize_at_commits_state() {
        assert_materialized_state(600, 60, 24);
    }

    #[test]
    fn test_no_negative_resources() {
        assert_sparse_input_stop(1, 600, 10, 1);
    }

    #[test]
    fn test_system_stops_when_output_full() {
        assert_low_capacity_stop(5, 600, 60);
    }
}
