//! liminality-engine
//!
//! This crate implements simulation modes and state evaluation for liminality.
//! It depends on liminality-model.
//! It must support debug tick and predictive modes eventually.

use liminality_model::WorldModel;

pub fn simulate_furnace_line(start_state: &WorldModel, seconds: u64) -> WorldModel {
    let mut state = start_state.clone();

    // The logic: 1 iron ore and 1/8 coal consumed every 10 seconds.
    // That means every 10 seconds, it produces 1 iron ingot.
    // In other words, every 80 seconds, it produces 8 iron ingots, consuming 8 iron ore and 1 coal.
    // We can simulate this simply by counting how many 10-second ticks have occurred.

    let ticks = seconds / 10;

    let max_ingots_from_ore = state.ore_storage;
    let max_ingots_from_coal = state.coal_storage * 8; // 1 coal can smelt 8 items

    // capacity limit:
    let output_capacity: u32 = 64;
    let available_capacity = output_capacity.saturating_sub(state.output_storage);

    // Number of ingots we can actually produce is the minimum of ticks, max_from_ore, max_from_coal, and available_capacity
    let ingots_produced = ticks
        .min(max_ingots_from_ore as u64)
        .min(max_ingots_from_coal as u64)
        .min(available_capacity as u64) as u32;

    if ingots_produced > 0 {
        state.ore_storage -= ingots_produced;
        // Coal consumption: 1 coal per 8 ingots.
        // We use integer division with ceiling equivalent to figure out consumed coal,
        // Wait, if 1 coal smelts 8 items, then if we produce `ingots_produced` items, how many coal are consumed?
        // Actually, let's just model the exact consumption.
        // It consumes 1/8 coal every 10 seconds.
        // This is easiest to model by consuming 1 coal when 8 ingots are produced.
        // But what if it produces 1 ingot? The prompt says "1/8 coal being consumed every 10 seconds".
        // In the prompt expectations:
        // "query at 600s returns 60 iron_ingot, 68 iron_ore, 24 coal"
        // Let's check:
        // 600 seconds = 60 ticks.
        // 60 ticks = 60 iron ingots produced.
        // iron_ore: 128 - 60 = 68.
        // coal: 60 ingots / 8 = 7.5 coal. Wait.
        // 32 - 7.5 = 24.5. The prompt says 24 coal.
        // This implies ceiling for coal consumed: (60 + 7) / 8 = 8 coal consumed.
        // 32 - 8 = 24 coal remaining.

        // Wait, query at 640s:
        // 640 seconds = 64 ticks.
        // 64 ingots produced.
        // iron_ore: 128 - 64 = 64.
        // coal consumed: (64 + 7) / 8 = 8 coal consumed.
        // 32 - 8 = 24 coal remaining.

        let coal_consumed = ingots_produced.div_ceil(8);
        state.coal_storage -= coal_consumed;
        state.output_storage += ingots_produced;
    }

    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_600s() {
        let state = WorldModel::furnace_line_demo();
        let final_state = simulate_furnace_line(&state, 600);
        assert_eq!(final_state.output_storage, 60);
        assert_eq!(final_state.ore_storage, 68);
        assert_eq!(final_state.coal_storage, 24);
    }

    #[test]
    fn it_works_640s() {
        let state = WorldModel::furnace_line_demo();
        let final_state = simulate_furnace_line(&state, 640);
        assert_eq!(final_state.output_storage, 64);
        assert_eq!(final_state.ore_storage, 64);
        assert_eq!(final_state.coal_storage, 24);
    }
}
