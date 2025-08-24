use super::{Renderer, RendererError};

/// Public control surface for the renderer. Keeps UI/controllers decoupled.
pub trait RendererControl {
    fn set_theme_by_name(&mut self, theme: &str) -> Result<(), RendererError>;
    fn set_pattern_by_id(&mut self, pattern_id: &str) -> Result<(), RendererError>;
    fn update_params_from_str(&mut self, params_csv: &str) -> Result<(), RendererError>;
}

impl RendererControl for Renderer {
    #[inline]
    fn set_theme_by_name(&mut self, theme: &str) -> Result<(), RendererError> {
        Renderer::set_theme_by_name(self, theme)
    }

    #[inline]
    fn set_pattern_by_id(&mut self, pattern_id: &str) -> Result<(), RendererError> {
        Renderer::set_pattern_by_id(self, pattern_id)
    }

    #[inline]
    fn update_params_from_str(&mut self, params_csv: &str) -> Result<(), RendererError> {
        Renderer::update_params_from_str(self, params_csv)
    }
}


