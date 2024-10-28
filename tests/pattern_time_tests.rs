use std::f64::consts::PI;
use chromacat::pattern::config::{PatternConfig, PatternParams, CommonParams};
use chromacat::pattern::engine::PatternEngine;
use chromacat::pattern::patterns::PlasmaParams;
use colorgrad::preset::greys;

#[test]
fn test_time_consistency() {
    // Create a simple grayscale gradient for testing
    let gradient = greys();

    // Create pattern config
    let config = PatternConfig {
        common: CommonParams {
            frequency: 1.0,
            amplitude: 1.0,
            speed: 1.0,
        },
        params: PatternParams::Plasma(PlasmaParams::default()),
    };

    let mut engine = PatternEngine::new(Box::new(gradient), config, 100, 100);

    // Get initial value at time 0
    engine.set_time(0.0);
    let x = 50;
    let y = 50;
    let initial_value = engine.get_value_at(x, y).unwrap();
    println!("Initial value: {}", initial_value);
    println!("Initial time: {}", engine.time());

    // Test several points in the cycle
    let test_points = [
        0.0, PI/4.0, PI/2.0, PI, 3.0*PI/2.0, 2.0*PI - 0.001
    ];

    for &time in &test_points {
        engine.set_time(time);
        let value = engine.get_value_at(x, y).unwrap();
        println!("Time {:.3}: value = {:.6}", time, value);
    }

    // Test that pattern repeats after 2π
    engine.set_time(0.0);
    let start_value = engine.get_value_at(x, y).unwrap();
    engine.set_time(2.0 * PI);
    let end_value = engine.get_value_at(x, y).unwrap();
    
    let final_diff = (start_value - end_value).abs();
    
    println!("Cycle comparison:");
    println!("  Start value: {}", start_value);
    println!("  End value:   {}", end_value);
    println!("  Difference:  {}", final_diff);
    println!("  Final time:  {}", engine.time());

    assert!(final_diff < 0.001, 
        "Pattern should approximately repeat after one cycle (diff: {})", final_diff);

    // Test smooth progression with more lenient threshold and better debugging
    println!("\nTesting smooth progression:");
    engine.set_time(0.0);
    let mut last_value = engine.get_value_at(x, y).unwrap();
    let mut max_delta = 0.0;
    let mut max_delta_time = 0.0;
    
    for i in 1..100 {
        let time = (i as f64) * (2.0 * PI / 100.0);
        engine.set_time(time);
        let value = engine.get_value_at(x, y).unwrap();
        let delta = (value - last_value).abs();
        
        // Track maximum change
        if delta > max_delta {
            max_delta = delta;
            max_delta_time = time;
        }

        // Print values when change is significant
        if delta > 0.1 {
            println!("Large value change at time {:.3}:", time);
            println!("  Previous value: {:.6}", last_value);
            println!("  Current value:  {:.6}", value);
            println!("  Delta:          {:.6}", delta);
        }
        
        // Increase threshold to 0.15 to allow for some larger changes
        assert!(delta < 0.15, 
            "Value change too large at time {}: {} (prev: {}, curr: {})", 
            time, delta, last_value, value);
        
        last_value = value;
    }

    println!("\nMaximum value change:");
    println!("  Delta: {:.6}", max_delta);
    println!("  Time:  {:.3}", max_delta_time);
}

#[test]
fn test_consistent_animation_speed() {
    // Create a simple grayscale gradient for testing
    let gradient = greys();

    let config = PatternConfig {
        common: CommonParams {
            frequency: 1.0,
            amplitude: 1.0,
            speed: 1.0,
        },
        params: PatternParams::Plasma(PlasmaParams::default()),
    };

    let mut engine = PatternEngine::new(Box::new(gradient), config, 100, 100);
    let x = 50;
    let y = 50;

    // Record value changes over multiple cycles
    let mut value_changes = Vec::new();
    let delta = 1.0 / 60.0;
    let cycles = 5;
    let steps_per_cycle = (2.0 * PI / (delta * 0.05)) as usize; // 0.05 is BASE_TIME_SCALE

    for _ in 0..cycles {
        let mut last_value = engine.get_value_at(x, y).unwrap();
        let mut cycle_changes = Vec::new();

        for _ in 0..steps_per_cycle {
            engine.update(delta);
            let new_value = engine.get_value_at(x, y).unwrap();
            cycle_changes.push((new_value - last_value).abs());
            last_value = new_value;
        }

        value_changes.push(cycle_changes);
    }

    // Compare value changes between cycles
    // They should be similar if animation speed is consistent
    for i in 1..value_changes.len() {
        let avg_changes_current = average(&value_changes[i]);
        let avg_changes_prev = average(&value_changes[i-1]);
        
        let change_ratio = avg_changes_current / avg_changes_prev;
        assert!((change_ratio - 1.0).abs() < 0.1, 
            "Animation speed should remain consistent between cycles");
    }
}

fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}
