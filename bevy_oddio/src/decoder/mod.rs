#[cfg(feature = "mp3")]
mod mp3;
#[cfg(feature = "mp3")]
pub use mp3::*;
#[cfg(feature = "flac")]
mod flac;
#[cfg(feature = "flac")]
pub use flac::*;

use bevy_reflect::TypeUuid;
use bevy_utils::Duration;
use oddio::Frames;
use std::sync::Arc;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f40c2d6a-d2ad-42cc-8f86-0147d3ddd68c"]
pub struct AudioSource {
    pub(crate) stereo: Option<Arc<Frames<[f32; 2]>>>,
    pub(crate) mono: Arc<Frames<f32>>,
    pub(crate) duration: Duration,
}

impl AudioSource {
    pub fn duration(&self) -> Duration {
        self.duration
    }
}
