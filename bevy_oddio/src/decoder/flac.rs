use std::io::Cursor;

use anyhow::Result;
use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy_utils::{BoxedFuture, Duration};
use claxon::metadata::StreamInfo;
use cpal::Sample;
use oddio::Frames;

use super::AudioSource;

/// Loads flac files as [AudioSource] [Assets](bevy_asset::Assets)
#[derive(Default)]
pub struct FlacLoader;

impl AssetLoader for FlacLoader {
    fn load(&self, bytes: &[u8], load_context: &mut LoadContext) -> BoxedFuture<Result<()>> {
        let mut decoder = claxon::FlacReader::new(Cursor::new(bytes)).unwrap();

        let StreamInfo {
            sample_rate,
            channels,
            bits_per_sample,
            ..
        } = decoder.streaminfo();
        let mut stereo = vec![];
        let mut mono = vec![];

        let f32_samples = decoder
            .samples()
            .map(Result::unwrap)
            .map(|raw| {
                if bits_per_sample == 16 {
                    raw as i16
                } else if bits_per_sample < 16 {
                    (raw << (16 - bits_per_sample)) as i16
                } else {
                    (raw >> (bits_per_sample - 16)) as i16
                }
            })
            .map(|v| Sample::to_f32(&v))
            .collect::<Vec<_>>();

        if channels == 1 {
            stereo.extend(f32_samples.iter().map(|&s| [s, s]));
            mono.extend(f32_samples.iter());
        } else if channels == 2 {
            stereo.extend(f32_samples.chunks_exact(2).map(|s| [s[0], s[1]]));
            mono.extend(
                f32_samples
                    .chunks_exact(2)
                    .map(|s| (s[0].to_f32() + s[1].to_f32()) / 2.),
            );
        } else {
            panic!(
                "Expected a mono or stereo file, found file with {} channels!",
                channels
            )
        }

        let stereo = if stereo.is_empty() {
            None
        } else {
            Some(Frames::from_slice(sample_rate, &stereo))
        };
        let mono = Frames::from_slice(sample_rate, &mono);
        let duration = Duration::from_secs_f32(mono.len() as f32 / mono.rate() as f32);
        load_context.set_default_asset(LoadedAsset::new(AudioSource {
            stereo,
            mono,
            duration,
        }));
        Box::pin(async move { Ok(()) })
    }

    fn extensions(&self) -> &[&str] {
        &["flac"]
    }
}
