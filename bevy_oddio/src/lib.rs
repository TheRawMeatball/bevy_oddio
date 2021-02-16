mod decoder;

use bevy_app::{AppBuilder, Plugin};
use bevy_asset::{AddAsset, Assets, Handle};
use bevy_ecs::{IntoSystem, Query, Res, ResMut};
use bevy_math::Vec3;
use bevy_transform::components::Transform;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use oddio::{
    FramesSignal, Gain, Handle as OddioHandle, Mixer, MonoToStereo, Signal, Spatial, SpatialScene,
    Stop,
};

pub struct OddioPlugin;
pub use decoder::AudioSource;

pub struct OddioContext {
    pub mixer: OddioHandle<Mixer<[f32; 2]>>,
    pub spatial: OddioHandle<SpatialScene>,
}

impl Plugin for OddioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let sample_rate = device.default_output_config().unwrap().sample_rate();
        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let (mut root_mixer_handle, root_mixer) = oddio::split(Mixer::new());
        let (scene_handle, scene) = oddio::split(SpatialScene::new(sample_rate.0, 0.1));
        root_mixer_handle.control().play(scene);

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let frames = oddio::frame_stereo(data);
                    oddio::run(&root_mixer, sample_rate.0, frames);
                },
                move |err| {
                    eprintln!("{}", err);
                },
            )
            .unwrap();
        stream.play().unwrap();

        app.insert_non_send_resource(stream)
            .insert_resource(OddioContext {
                mixer: root_mixer_handle,
                spatial: scene_handle,
            })
            .add_asset::<AudioSource>()
            .add_system(spatial_audio_system.system())
            .add_system(nonspatial_audio_system.system());

        #[cfg(feature = "mp3")]
        app.add_asset_loader(decoder::Mp3Loader);
        #[cfg(feature = "flac")]
        app.add_asset_loader(decoder::FlacLoader);
    }
}

pub struct SpatialSpeaker {
    asset_handle: Option<Handle<AudioSource>>,
    handle_dirty: bool,
    volume: f32,
    paused: bool,
    control_handle: Option<OddioHandle<Spatial<Stop<Gain<FramesSignal<f32>>>>>>,
}

pub struct GlobalSpeaker {
    asset_handle: Option<Handle<AudioSource>>,
    handle_dirty: bool,
    volume: f32,
    paused: bool,
    control_handle: Option<OptionalStereoHandle>,
}

enum OptionalStereoHandle {
    Stereo(OddioHandle<Stop<Gain<FramesSignal<[f32; 2]>>>>),
    Mono(OddioHandle<Stop<Gain<MonoToStereo<FramesSignal<f32>>>>>),
}

impl Default for SpatialSpeaker {
    fn default() -> Self {
        Self {
            asset_handle: None,
            handle_dirty: false,
            volume: 1.,
            paused: false,
            control_handle: None,
        }
    }
}

impl Default for GlobalSpeaker {
    fn default() -> Self {
        Self {
            asset_handle: None,
            handle_dirty: true,
            volume: 1.,
            paused: false,
            control_handle: None,
        }
    }
}

impl SpatialSpeaker {
    pub fn play(&mut self) {
        self.paused = false;
        if let Some(handle) = &mut self.control_handle {
            handle.control::<Stop<_>, _>().resume()
        }
    }
    pub fn pause(&mut self) {
        self.paused = true;
        if let Some(handle) = &mut self.control_handle {
            handle.control::<Stop<_>, _>().pause()
        }
    }
    #[inline(always)]
    pub fn paused(&self) -> bool {
        self.paused
    }
    #[inline(always)]
    pub fn volume(&self) -> f32 {
        self.volume
    }
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v;
        if let Some(handle) = &mut self.control_handle {
            handle.control::<Gain<_>, _>().set_gain(v)
        }
    }
    #[inline]
    pub fn set_track(&mut self, track: Handle<AudioSource>) {
        self.asset_handle = Some(track);
        self.handle_dirty = true;
    }
    #[inline]
    pub fn clear_track(&mut self) {
        self.asset_handle = None;
        self.handle_dirty = true;
    }
    #[inline(always)]
    pub fn get_track(&self) -> &Option<Handle<AudioSource>> {
        &self.asset_handle
    }
    pub fn new(track: Handle<AudioSource>) -> Self {
        Self {
            asset_handle: Some(track),
            ..Default::default()
        }
    }
}

impl GlobalSpeaker {
    pub fn play(&mut self) {
        self.paused = false;
        if let Some(handle) = &mut self.control_handle {
            match handle {
                OptionalStereoHandle::Stereo(handle) => handle.control::<Stop<_>, _>().resume(),
                OptionalStereoHandle::Mono(handle) => handle.control::<Stop<_>, _>().resume(),
            }
        }
    }
    pub fn pause(&mut self) {
        self.paused = true;
        if let Some(handle) = &mut self.control_handle {
            match handle {
                OptionalStereoHandle::Stereo(handle) => handle.control::<Stop<_>, _>().pause(),
                OptionalStereoHandle::Mono(handle) => handle.control::<Stop<_>, _>().pause(),
            }
        }
    }
    #[inline(always)]
    pub fn paused(&self) -> bool {
        self.paused
    }
    #[inline(always)]
    pub fn volume(&self) -> f32 {
        self.volume
    }
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v;
        if let Some(handle) = &mut self.control_handle {
            match handle {
                OptionalStereoHandle::Stereo(handle) => handle.control::<Gain<_>, _>().set_gain(v),
                OptionalStereoHandle::Mono(handle) => handle.control::<Gain<_>, _>().set_gain(v),
            }
        }
    }
    #[inline]
    pub fn set_track(&mut self, track: Handle<AudioSource>) {
        self.asset_handle = Some(track);
        self.handle_dirty = true;
    }
    #[inline]
    pub fn clear_track(&mut self) {
        self.asset_handle = None;
        self.handle_dirty = true;
    }
    #[inline(always)]
    pub fn get_track(&self) -> &Option<Handle<AudioSource>> {
        &self.asset_handle
    }
    pub fn new(track: Handle<AudioSource>) -> Self {
        Self {
            asset_handle: Some(track),
            ..Default::default()
        }
    }
}

pub struct AudioVelocity(Vec3);

pub struct Listener;

fn spatial_audio_system(
    mut player: ResMut<OddioContext>,
    audio: Res<Assets<AudioSource>>,
    mut spatial: Query<(&mut SpatialSpeaker, &Transform, Option<&AudioVelocity>)>,
) {
    let player = &mut player.spatial;
    for (mut speaker, transform, velocity) in spatial.iter_mut() {
        let velocity = velocity.map(|v| v.0).unwrap_or_else(Vec3::zero);
        let position = transform.translation;

        let velocity = [velocity.x, velocity.y, velocity.z].into();
        let position = [position.x, position.y, position.z].into();

        if speaker.handle_dirty {
            if let Some(ref asset) = &speaker.asset_handle {
                let signal =
                    FramesSignal::new(audio.get(asset).unwrap().mono.clone(), 0.).with_gain();
                let mut handle = player.control().play(
                    signal, position, velocity,
                    500., /* I don't know what I'm doing, fine tune this magic number */
                );
                handle.control::<Gain<_>, _>().set_gain(speaker.volume);
                if speaker.paused {
                    handle.control::<Stop<_>, _>().pause();
                } else {
                    handle.control::<Stop<_>, _>().resume();
                }
                speaker.control_handle = Some(handle);
            }
            speaker.handle_dirty = false;
        }

        if let Some(handle) = &mut speaker.control_handle {
            handle
                .control::<Spatial<_>, _>()
                .set_motion(position, velocity);
        }
    }
}

fn nonspatial_audio_system(
    mut player: ResMut<OddioContext>,
    audio: Res<Assets<AudioSource>>,
    mut spatial: Query<&mut GlobalSpeaker>,
) {
    let player = &mut player.mixer;
    for mut speaker in spatial.iter_mut() {
        if speaker.handle_dirty {
            if let Some(ref asset) = &speaker.asset_handle {
                let buffers = audio.get(asset).unwrap();
                let mut handle = if let Some(stereo) = &buffers.stereo {
                    let signal = FramesSignal::new(stereo.clone(), 0.);
                    OptionalStereoHandle::Stereo(player.control().play(signal.with_gain()))
                } else {
                    let signal = FramesSignal::new(buffers.mono.clone(), 0.);
                    OptionalStereoHandle::Mono(player.control().play(signal.into_stereo().with_gain()))
                };
                match &mut handle {
                    OptionalStereoHandle::Stereo(ref mut handle) => {
                        handle.control::<Gain<_>, _>().set_gain(speaker.volume);
                        if speaker.paused {
                            handle.control::<Stop<_>, _>().pause();
                        } else {
                            handle.control::<Stop<_>, _>().resume();
                        }
                    }
                    OptionalStereoHandle::Mono(ref mut handle) => {
                        handle.control::<Gain<_>, _>().set_gain(speaker.volume);
                        if speaker.paused {
                            handle.control::<Stop<_>, _>().pause();
                        } else {
                            handle.control::<Stop<_>, _>().resume();
                        }
                    }
                }

                speaker.control_handle = Some(handle);
            }
            speaker.handle_dirty = false;
        }
    }
}
