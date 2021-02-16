use std::io::Cursor;

use anyhow::Result;
use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy_utils::{BoxedFuture, Duration};
use cpal::Sample;
use oddio::Frames;

use super::AudioSource;

/// Loads mp3 files as [AudioSource] [Assets](bevy_asset::Assets)
#[derive(Default)]
pub struct Mp3Loader;

impl AssetLoader for Mp3Loader {
    fn load(&self, bytes: &[u8], load_context: &mut LoadContext) -> BoxedFuture<Result<()>> {
        let mut decoder = minimp3::Decoder::new(Cursor::new(bytes));

        let mut base_sample_rate = 0u32;
        let mut stereo = vec![];
        let mut mono = vec![];

        while let Ok(minimp3::Frame {
            data,
            sample_rate,
            channels,
            ..
        }) = decoder.next_frame()
        {
            base_sample_rate = sample_rate as u32;

            if channels == 1 {
                mono.extend(data.iter().map(Sample::to_f32));
            } else if channels == 2 {
                stereo.extend(data.chunks_exact(2).map(|s| [s[0].to_f32(), s[1].to_f32()]));
                mono.extend(
                    data.chunks_exact(2)
                        .map(|s| (s[0].to_f32() + s[1].to_f32()) / 2.),
                );
            } else {
                panic!(
                    "Expected a mono or stereo file, found file with {} channels!",
                    channels
                )
            }
        }

        let stereo = if stereo.is_empty() {
            None
        } else {
            Some(Frames::from_slice(base_sample_rate, &stereo))
        };
        let mono = Frames::from_slice(base_sample_rate, &mono);
        let duration = Duration::from_secs_f32(mono.len() as f32 / mono.rate() as f32);
        load_context.set_default_asset(LoadedAsset::new(AudioSource {
            stereo,
            mono,
            duration,
        }));
        Box::pin(async move { Ok(()) })
    }

    fn extensions(&self) -> &[&str] {
        &["mp3"]
    }
}
