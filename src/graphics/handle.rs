use std::fmt;

use crate::graphics::backend::BackgroundBackend;

#[allow(dead_code)]
pub struct FbOverlayHandle {
    pub backend_name: String,
    backend: Option<Box<dyn BackgroundBackend>>,
}

impl fmt::Debug for FbOverlayHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FbOverlayHandle")
            .field("backend_name", &self.backend_name)
            .field("has_backend", &self.backend.is_some())
            .finish()
    }
}

impl FbOverlayHandle {
    #[allow(dead_code)]
    pub fn new(backend_name: String, backend: Box<dyn BackgroundBackend>) -> Self {
        Self {
            backend_name,
            backend: Some(backend),
        }
    }
}

impl Drop for FbOverlayHandle {
    fn drop(&mut self) {
        if let Some(mut backend) = self.backend.take() {
            if let Err(e) = backend.clear() {
                tracing::warn!("Failed to clear backend {}: {}", self.backend_name, e);
            }
        }
    }
}
