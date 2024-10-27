use std::any::Any;
use std::fmt::Debug;

/// Represents a parameter value type
#[derive(Debug, Clone)]
pub enum ParamType {
    /// Numeric value with optional range
    Number { min: f64, max: f64 },
    /// Boolean value
    Boolean,
    /// Enumerated value with possible options
    Enum { options: &'static [&'static str] },
    /// Composite type containing multiple parameters
    Composite,
}

/// Trait for pattern parameters that can be configured via CLI
pub trait PatternParam: Debug + Any {
    /// Name of the parameter for CLI help text
    fn name(&self) -> &'static str;
    
    /// Description of the parameter for CLI help text
    fn description(&self) -> &'static str;
    
    /// Type of the parameter
    fn param_type(&self) -> ParamType;
    
    /// Default value as a string
    fn default_value(&self) -> String;
    
    /// Validate a string value for this parameter
    fn validate(&self, value: &str) -> Result<(), String>;
    
    /// Parse a string value into the appropriate type
    fn parse(&self, value: &str) -> Result<Box<dyn PatternParam>, String>;
    
    /// List of sub-parameters if this is a composite type
    fn sub_params(&self) -> Vec<Box<dyn PatternParam>> {
        Vec::new()
    }

    /// Clone implementation for trait object
    fn clone_param(&self) -> Box<dyn PatternParam>;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Macro for defining pattern parameters
#[macro_export]
macro_rules! define_param {
    // For numeric parameters
    ($(#[$meta:meta])* num $pattern:ident, $name:ident, $desc:expr, $min:expr, $max:expr, $default:expr) => {
        paste::paste! {
            $(#[$meta])*
            #[derive(Debug, Clone)]
            struct [<$pattern $name>];

            impl $crate::pattern::params::PatternParam for [<$pattern $name>] {
                fn name(&self) -> &'static str { stringify!($name) }
                fn description(&self) -> &'static str { $desc }
                fn param_type(&self) -> $crate::pattern::params::ParamType { 
                    $crate::pattern::params::ParamType::Number { min: $min, max: $max } 
                }
                fn default_value(&self) -> String { $default.to_string() }
                
                fn validate(&self, value: &str) -> Result<(), String> {
                    let val = value.parse::<f64>().map_err(|_| "Invalid number".to_string())?;
                    if !($min..=$max).contains(&val) {
                        return Err(format!("{} must be between {} and {}", self.name(), $min, $max));
                    }
                    Ok(())
                }
                
                fn parse(&self, _: &str) -> Result<Box<dyn $crate::pattern::params::PatternParam>, String> {
                    unimplemented!("Individual parameters don't support parsing")
                }
                fn clone_param(&self) -> Box<dyn $crate::pattern::params::PatternParam> {
                    Box::new(self.clone())
                }
                fn as_any(&self) -> &dyn std::any::Any { self }
            }
        }
    };

    // For boolean parameters
    ($(#[$meta:meta])* bool $pattern:ident, $name:ident, $desc:expr, $default:expr) => {
        paste::paste! {
            $(#[$meta])*
            #[derive(Debug, Clone)]
            struct [<$pattern $name>];

            impl $crate::pattern::params::PatternParam for [<$pattern $name>] {
                fn name(&self) -> &'static str { stringify!($name) }
                fn description(&self) -> &'static str { $desc }
                fn param_type(&self) -> $crate::pattern::params::ParamType { 
                    $crate::pattern::params::ParamType::Boolean 
                }
                fn default_value(&self) -> String { $default.to_string() }
                
                fn validate(&self, value: &str) -> Result<(), String> {
                    match value {
                        "true" | "false" => Ok(()),
                        _ => Err(format!("{} must be true or false", self.name())),
                    }
                }
                
                fn parse(&self, _: &str) -> Result<Box<dyn $crate::pattern::params::PatternParam>, String> {
                    unimplemented!("Individual parameters don't support parsing")
                }
                fn clone_param(&self) -> Box<dyn $crate::pattern::params::PatternParam> {
                    Box::new(self.clone())
                }
                fn as_any(&self) -> &dyn std::any::Any { self }
            }
        }
    };

    // For enum parameters
    ($(#[$meta:meta])* enum $pattern:ident, $name:ident, $desc:expr, $options:expr, $default:expr) => {
        paste::paste! {
            $(#[$meta])*
            #[derive(Debug, Clone)]
            struct [<$pattern $name>];

            impl $crate::pattern::params::PatternParam for [<$pattern $name>] {
                fn name(&self) -> &'static str { stringify!($name) }
                fn description(&self) -> &'static str { $desc }
                fn param_type(&self) -> $crate::pattern::params::ParamType { 
                    $crate::pattern::params::ParamType::Enum { options: $options } 
                }
                fn default_value(&self) -> String { $default.to_string() }
                
                fn validate(&self, value: &str) -> Result<(), String> {
                    if $options.contains(&value) {
                        Ok(())
                    } else {
                        Err(format!("{} must be one of: {:?}", self.name(), $options))
                    }
                }
                
                fn parse(&self, _: &str) -> Result<Box<dyn $crate::pattern::params::PatternParam>, String> {
                    unimplemented!("Individual parameters don't support parsing")
                }
                fn clone_param(&self) -> Box<dyn $crate::pattern::params::PatternParam> {
                    Box::new(self.clone())
                }
                fn as_any(&self) -> &dyn std::any::Any { self }
            }
        }
    };
}

pub use define_param as define_param_type;
