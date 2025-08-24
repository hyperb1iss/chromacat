use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub enum ModSourceKind {
    Lfo,
}

#[derive(Debug, Clone, Copy)]
pub struct Lfo {
    pub frequency_hz: f64,
    pub phase: f64,
    pub min: f64,
    pub max: f64,
}

impl Lfo {
    pub fn new(frequency_hz: f64, min: f64, max: f64) -> Self {
        Self { frequency_hz, phase: 0.0, min, max }
    }

    pub fn advance(&mut self, dt: f64) {
        self.phase = (self.phase + dt * self.frequency_hz) % 1.0;
    }

    pub fn value(&self) -> f64 {
        // Sine in 0..1 range mapped to min..max
        let s = ((self.phase * TAU).sin() + 1.0) * 0.5;
        self.min + s * (self.max - self.min)
    }
}

#[derive(Debug, Clone)]
pub struct ModRoute {
    /// Target parameter name in the current pattern
    pub target_param: String,
    /// Index into the `lfos` vector
    pub source_index: usize,
    /// Depth multiplier applied to the source value relative to its range
    pub depth: f64,
}

#[derive(Debug, Default, Clone)]
pub struct Modulator {
    pub lfos: Vec<Lfo>,
    pub routes: Vec<ModRoute>,
}

impl Modulator {
    pub fn new() -> Self { Self { lfos: Vec::new(), routes: Vec::new() } }

    pub fn add_lfo(&mut self, lfo: Lfo) -> usize {
        let idx = self.lfos.len();
        self.lfos.push(lfo);
        idx
    }

    pub fn add_route(&mut self, target_param: String, source_index: usize, depth: f64) {
        self.routes.push(ModRoute { target_param, source_index, depth });
    }

    /// Advances sources and returns parameter updates as (name, value)
    pub fn advance(&mut self, dt: f64) -> Vec<(String, f64)> {
        for l in &mut self.lfos { l.advance(dt); }
        let mut updates = Vec::new();
        for r in &self.routes {
            if let Some(src) = self.lfos.get(r.source_index) {
                let center = (src.min + src.max) * 0.5;
                let centered = src.value() - center;
                let value = center + centered * r.depth;
                updates.push((r.target_param.clone(), value));
            }
        }
        updates
    }
}


