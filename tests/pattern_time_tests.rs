use chromacat::pattern::config::{CommonParams, PatternConfig, PatternParams};
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
            correct_aspect: true,
            aspect_ratio: 0.5,
            theme_name: Some("test".to_string()),
        },
        params: PatternParams::Plasma(PlasmaParams::default()),
    };

    let mut engine = PatternEngine::new(Box::new(gradient), config, 100, 100);

    // Test smooth progression with more lenient threshold and better debugging
    println!("\nTesting smooth progression:");
    engine.set_time(0.0);
    let mut last_value = engine.get_value_at(50, 50).unwrap();
    let mut max_delta = 0.0;
    let mut max_delta_time = 0.0;

    // Test smaller time increments for smoother animation
    let time_step = 0.016; // ~60fps
    for i in 1..100 {
        let time = i as f64 * time_step;
        engine.update(time_step);
        let value = engine.get_value_at(50, 50).unwrap();
        let delta = (value - last_value).abs();

        // Track maximum change
        if delta > max_delta {
            max_delta = delta;
            max_delta_time = time;
        }

        // Print values when change is significant
        if delta > 0.1 {
            println!("Large value change at time {time:.3}:");
            println!("  Previous value: {last_value:.6}");
            println!("  Current value:  {value:.6}");
            println!("  Delta:          {delta:.6}");
        }

        // Ensure changes between frames are not too drastic
        assert!(
            delta < 0.15,
            "Value change too large at time {time}: {delta} (prev: {last_value}, curr: {value})"
        );

        last_value = value;
    }

    println!("\nMaximum value change:");
    println!("  Delta: {max_delta:.6}");
    println!("  Time:  {max_delta_time:.3}");
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
            correct_aspect: true,
            aspect_ratio: 0.5,
            theme_name: Some("test".to_string()),
        },
        params: PatternParams::Plasma(PlasmaParams::default()),
    };

    let mut engine = PatternEngine::new(Box::new(gradient), config, 100, 100);
    let x = 50;
    let y = 50;

    // Record time progression
    let delta = 1.0 / 60.0; // 60fps
    let periods = 5;
    let steps_per_period = 60; // 1 second worth of frames

    println!("\nTesting animation speed consistency:");
    println!("Delta time: {delta:.6} seconds");
    println!(
        "Testing {periods} periods of {steps_per_period} steps each"
    );

    // Track time progression instead of value changes
    let mut times = Vec::new();
    let mut values = Vec::new();

    for period in 0..periods {
        println!("\nPeriod {period}:");
        let start_value = engine.get_value_at(x, y).unwrap();
        println!("Initial value: {start_value:.6}");

        let mut period_times = Vec::new();
        let mut period_values = Vec::new();

        for _ in 0..steps_per_period {
            period_times.push(engine.time());
            period_values.push(engine.get_value_at(x, y).unwrap());
            engine.update(delta);
        }

        let time_diff = period_times.last().unwrap() - period_times.first().unwrap();
        println!("Period {period} time progression: {time_diff:.6}");

        times.push(period_times);
        values.push(period_values);
    }

    // Compare time progression between periods
    println!("\nComparing time progression between periods:");
    for i in 1..times.len() {
        let current_diff = times[i].last().unwrap() - times[i].first().unwrap();
        let prev_diff = times[i - 1].last().unwrap() - times[i - 1].first().unwrap();
        let ratio = current_diff / prev_diff;

        println!(
            "Period {}/{} time ratio: {:.6} ({:.6} / {:.6})",
            i - 1,
            i,
            ratio,
            prev_diff,
            current_diff
        );

        // Time progression should be very consistent
        assert!(
            (ratio - 1.0).abs() < 0.001,
            "Time progression should be consistent between periods\n\
             Period {}/{} ratio: {:.6} exceeds threshold",
            i - 1,
            i,
            ratio
        );
    }

    // Verify that values are changing
    for period_values in values {
        let mut has_change = false;
        for i in 1..period_values.len() {
            if (period_values[i] - period_values[i - 1]).abs() > 0.001 {
                has_change = true;
                break;
            }
        }
        assert!(has_change, "Pattern values should change during animation");
    }
}
