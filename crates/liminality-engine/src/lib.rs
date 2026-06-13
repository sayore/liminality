//! liminality-engine
//!
//! This crate implements simulation modes and state evaluation for liminality.
//! It depends on liminality-model.
//! It must support debug tick and predictive modes eventually.

use std::collections::HashMap;
use thiserror::Error;

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

                    // Apply one step
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
            SimMode::Hybrid(_) => unimplemented!(),
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

            // Iterate second by second to handle fine-grained state updates correctly
            for _ in 0..delta {
                // Check if we need fuel
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

                // Check input
                let input_available = next_state
                    .resources
                    .get(&transformer.input_id)
                    .copied()
                    .unwrap_or(0);
                if input_available < transformer.input_qty {
                    stop_reason = Some(StopReason::InputEmpty);
                    break;
                }

                // Check output capacity
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
                    // Consume input
                    *next_state
                        .resources
                        .entry(transformer.input_id.clone())
                        .or_insert(0) -= transformer.input_qty;
                    // Produce output
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

        // Find the earliest stopping event
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
                } else {
                    // Check if we need initial fuel at all
                    if t_state.fuel_ops_left == 0 { 1 } else { 0 }
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

            // If limiting ops is 0, we are blocked immediately.
            // Actually, if we have 0 input but we haven't completed any progress, we are blocked at progress=0.
            if limiting_ops == 0 {
                // If we are already mid-progress we'd be blocked when we try to start the NEXT one,
                // but we might not even be able to start the first one.
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_base_state() -> BaseState {
        let mut resources = HashMap::new();
        resources.insert("coal".to_string(), 32);
        resources.insert("iron_ore".to_string(), 128);
        resources.insert("iron_ingot".to_string(), 0);

        let mut capacities = HashMap::new();
        capacities.insert("iron_ingot".to_string(), 64);

        let transformer = Transformer {
            id: "furnace_1".to_string(),
            input_id: "iron_ore".to_string(),
            input_qty: 1,
            fuel_id: "coal".to_string(),
            fuel_ops: 8,
            output_id: "iron_ingot".to_string(),
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

    #[test]
    fn test_furnace_line_debug_tick_600s() {
        let base_state = create_test_base_state();
        let engine = SimulationEngine::new(
            base_state,
            SimMode::DebugTick(DebugTickConfig {
                tick_duration_secs: 10,
            }),
        );

        let state = engine.state_at(600).unwrap();
        assert_eq!(state.time, 600);
        assert_eq!(state.resources.get("iron_ingot"), Some(&60));
        assert_eq!(state.resources.get("iron_ore"), Some(&68));
        assert_eq!(state.resources.get("coal"), Some(&24)); // 60 ops / 8 ops_per_coal = 7.5 -> 8 coal consumed, 32 - 8 = 24
        assert_eq!(state.stop_reason, None);
    }

    #[test]
    fn test_furnace_line_predictive_600s() {
        let base_state = create_test_base_state();
        let engine = SimulationEngine::new(base_state, SimMode::Predictive(PredictiveConfig {}));

        let state = engine.state_at(600).unwrap();
        assert_eq!(state.time, 600);
        assert_eq!(state.resources.get("iron_ingot"), Some(&60));
        assert_eq!(state.resources.get("iron_ore"), Some(&68));
        assert_eq!(state.resources.get("coal"), Some(&24));
        assert_eq!(state.stop_reason, None);
    }

    #[test]
    fn test_furnace_line_predictive_640s_output_full() {
        let base_state = create_test_base_state();
        let engine = SimulationEngine::new(base_state, SimMode::Predictive(PredictiveConfig {}));

        // At 640s, we try to do 64 ops. Output capacity is 64.
        let state = engine.state_at(640).unwrap();
        assert_eq!(state.time, 640);
        assert_eq!(state.resources.get("iron_ingot"), Some(&64));
        assert_eq!(state.resources.get("iron_ore"), Some(&64));
        assert_eq!(state.resources.get("coal"), Some(&24)); // 64 ops / 8 = 8 coal consumed. 32-8 = 24.
        assert_eq!(state.stop_reason, Some(StopReason::OutputFull));

        // Let's try 700s to make sure it stops at 640s
        let state2 = engine.state_at(700).unwrap();
        assert_eq!(state2.time, 640);
        assert_eq!(state2.stop_reason, Some(StopReason::OutputFull));
    }

    #[test]
    fn test_predictive_matches_debug_tick() {
        let base_state = create_test_base_state();
        let engine_debug = SimulationEngine::new(
            base_state.clone(),
            SimMode::DebugTick(DebugTickConfig {
                tick_duration_secs: 1,
            }),
        );
        let engine_pred =
            SimulationEngine::new(base_state, SimMode::Predictive(PredictiveConfig {}));

        let state_debug = engine_debug.state_at(600).unwrap();
        let state_pred = engine_pred.state_at(600).unwrap();

        assert_eq!(state_debug.time, state_pred.time);
        assert_eq!(state_debug.resources, state_pred.resources);
        assert_eq!(state_debug.stop_reason, state_pred.stop_reason);
    }

    #[test]
    fn test_materialize_at_commits_state() {
        let base_state = create_test_base_state();
        let mut engine =
            SimulationEngine::new(base_state, SimMode::Predictive(PredictiveConfig {}));

        engine.materialize_at(600).unwrap();

        assert_eq!(engine.base_state.time, 600);
        assert_eq!(engine.base_state.resources.get("iron_ingot"), Some(&60));
        assert_eq!(engine.base_state.resources.get("coal"), Some(&24));
    }

    #[test]
    fn test_no_negative_resources() {
        let mut base_state = create_test_base_state();
        // Start with only 1 ore
        base_state.resources.insert("iron_ore".to_string(), 1);

        let engine = SimulationEngine::new(base_state, SimMode::Predictive(PredictiveConfig {}));
        let state = engine.state_at(600).unwrap();

        // Should stop early due to empty input
        assert_eq!(state.time, 10);
        assert_eq!(state.resources.get("iron_ingot"), Some(&1));
        assert_eq!(state.resources.get("iron_ore"), Some(&0));
        assert_eq!(state.stop_reason, Some(StopReason::InputEmpty));
    }

    #[test]
    fn test_system_stops_when_output_full() {
        let mut base_state = create_test_base_state();
        // Set capacity very low
        base_state.capacities.insert("iron_ingot".to_string(), 5);

        let engine = SimulationEngine::new(
            base_state,
            SimMode::DebugTick(DebugTickConfig {
                tick_duration_secs: 10,
            }),
        );

        // Try to run for 600s
        let state = engine.state_at(600).unwrap();

        // It should stop as soon as output hits 5
        // Since duration is 10s and it outputs 1 per 10s, it outputs 5 at 50s.
        // But the debug loop checks current_time < target_time. If it hits an error at 50s,
        // the step_debug returned 50s and an error, next tick it will add 10s and report 60s and an error.
        assert_eq!(state.time, 60);
        assert_eq!(state.resources.get("iron_ingot"), Some(&5));
        assert_eq!(state.stop_reason, Some(StopReason::OutputFull));
    }
}
