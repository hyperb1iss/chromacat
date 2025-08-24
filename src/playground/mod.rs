// Playground UI scaffold (feature-gated). For now, this is a placeholder that
// can later be replaced with a ratatui-driven overlay and layout.

#[cfg(feature = "playground-ui")]
pub mod ui {
    use crate::renderer::Renderer;
    use crate::error::Result;

    /// Runs the playground loop using the existing renderer.
    /// Placeholder: delegates to the normal animation loop at the app layer.
    pub fn run_with_renderer(_renderer: &mut Renderer, _content: &str) -> Result<()> {
        // Real ratatui integration will live here.
        Ok(())
    }
}


