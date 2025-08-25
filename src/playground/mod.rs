// Playground UI scaffold (feature-gated). For now, this is a placeholder that
// can later be replaced with a ratatui-driven overlay and layout.

pub mod ui {
    use crate::error::Result;
    use crate::renderer::Renderer;

    /// Runs the playground loop using the existing renderer.
    /// Placeholder: delegates to the normal animation loop at the app layer.
    pub fn run_with_renderer(_renderer: &mut Renderer, _content: &str) -> Result<()> {
        // Real ratatui integration will live here.
        Ok(())
    }
}
