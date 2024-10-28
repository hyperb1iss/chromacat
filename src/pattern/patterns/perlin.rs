use std::any::Any;
use crate::pattern::params::{PatternParam, ParamType};
use crate::pattern::utils::PatternUtils;
use crate::define_param;

// Define parameters with proper CLI names and bounds
define_param!(num Perlin, OctavesParam, "octaves", "Number of noise layers", 1.0, 8.0, 4.0);
define_param!(num Perlin, PersistenceParam, "persistence", "How quickly amplitudes diminish", 0.0, 1.0, 0.5);
define_param!(num Perlin, ScaleParam, "scale", "Scale of the noise", 0.1, 5.0, 1.0);
define_param!(num Perlin, SeedParam, "seed", "Random seed", 0.0, 4294967295.0, 0.0);

/// Parameters for configuring Perlin noise pattern effects
#[derive(Debug, Clone)]
pub struct PerlinParams {
    /// Number of noise layers (1-8)
    pub octaves: u32,
    /// How quickly amplitudes diminish (0.0-1.0)
    pub persistence: f64,
    /// Scale of the noise (0.1-5.0)
    pub scale: f64,
    /// Random seed for noise generation
    pub seed: u32,
}

impl PerlinParams {
    const OCTAVES_PARAM: PerlinOctavesParam = PerlinOctavesParam;
    const PERSISTENCE_PARAM: PerlinPersistenceParam = PerlinPersistenceParam;
    const SCALE_PARAM: PerlinScaleParam = PerlinScaleParam;
    const SEED_PARAM: PerlinSeedParam = PerlinSeedParam;
}

impl Default for PerlinParams {
    fn default() -> Self {
        Self {
            octaves: 4,
            persistence: 0.5,
            scale: 1.0,
            seed: 0,
        }
    }
}

// Use the validate macro to implement validation
define_param!(validate PerlinParams,
    OCTAVES_PARAM: PerlinOctavesParam,
    PERSISTENCE_PARAM: PerlinPersistenceParam,
    SCALE_PARAM: PerlinScaleParam,
    SEED_PARAM: PerlinSeedParam
);

impl PatternParam for PerlinParams {
    fn name(&self) -> &'static str {
        "perlin"
    }

    fn description(&self) -> &'static str {
        "Perlin noise-based pattern with multiple octaves"
    }

    fn param_type(&self) -> ParamType {
        ParamType::Composite
    }

    fn default_value(&self) -> String {
        format!(
            "octaves={},persistence={},scale={},seed={}",
            self.octaves, self.persistence, self.scale, self.seed
        )
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        self.validate_params(value)
    }

    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String> {
        let mut params = PerlinParams::default();
        
        for part in value.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() != 2 {
                continue;
            }
            
            match kv[0] {
                "octaves" => {
                    Self::OCTAVES_PARAM.validate(kv[1])?;
                    params.octaves = kv[1].parse().unwrap();
                }
                "persistence" => {
                    Self::PERSISTENCE_PARAM.validate(kv[1])?;
                    params.persistence = kv[1].parse().unwrap();
                }
                "scale" => {
                    Self::SCALE_PARAM.validate(kv[1])?;
                    params.scale = kv[1].parse().unwrap();
                }
                "seed" => {
                    Self::SEED_PARAM.validate(kv[1])?;
                    params.seed = kv[1].parse().unwrap();
                }
                invalid_param => {
                    return Err(format!("Invalid parameter name: {}", invalid_param));
                }
            }
        }
        
        Ok(Box::new(params))
    }

    fn sub_params(&self) -> Vec<Box<dyn PatternParam>> {
        vec![
            Box::new(Self::OCTAVES_PARAM),
            Box::new(Self::PERSISTENCE_PARAM),
            Box::new(Self::SCALE_PARAM),
            Box::new(Self::SEED_PARAM),
        ]
    }

    fn clone_param(&self) -> Box<dyn PatternParam> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl super::Patterns {
    /// Generates a Perlin noise pattern with multiple octaves
    pub fn perlin(&self, x: usize, y: usize, params: PerlinParams) -> f64 {
        let x_norm = x as f64 / self.width as f64;
        let y_norm = y as f64 / self.height as f64;

        let mut total = 0.0;
        let mut frequency = params.scale;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..params.octaves {
            total += self.perlin_noise(
                x_norm * frequency + self.time,
                y_norm * frequency + self.time,
            ) * amplitude;

            max_value += amplitude;
            amplitude *= params.persistence;
            frequency *= 2.0;
        }

        (total / max_value + 1.0) / 2.0
    }

    /// Calculates a single octave of Perlin noise
    fn perlin_noise(&self, x: f64, y: f64) -> f64 {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let dx = x - x0 as f64;
        let dy = y - y0 as f64;

        let sx = PatternUtils::smoothstep(dx);
        let sy = PatternUtils::smoothstep(dy);

        let n00 = self.gradient_dot(self.utils.hash(x0, y0), x0, y0, dx, dy);
        let n10 = self.gradient_dot(self.utils.hash(x1, y0), x1, y0, dx - 1.0, dy);
        let n01 = self.gradient_dot(self.utils.hash(x0, y1), x0, y1, dx, dy - 1.0);
        let n11 = self.gradient_dot(self.utils.hash(x1, y1), x1, y1, dx - 1.0, dy - 1.0);

        let nx0 = PatternUtils::lerp(n00, n10, sx);
        let nx1 = PatternUtils::lerp(n01, n11, sx);
        PatternUtils::lerp(nx0, nx1, sy)
    }

    /// Calculates dot product between gradient vector and distance vector
    fn gradient_dot(&self, hash: u8, _x: i32, _y: i32, dx: f64, dy: f64) -> f64 {
        match hash & 3 {
            0 => dx + dy,
            1 => -dx + dy,
            2 => dx - dy,
            _ => -dx - dy,
        }
    }
}
