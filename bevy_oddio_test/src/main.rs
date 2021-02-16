use bevy::{asset::LoadState, prelude::*};
use bevy_oddio::{AudioSource, GlobalSpeaker, OddioPlugin};

#[derive(Copy, Clone)]
enum AppState {
    Loading,
    Main,
}

struct Handles {
    mp3: Handle<AudioSource>,
}

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(OddioPlugin)
        .insert_resource(State::new(AppState::Loading))
        .add_system_set(State::<AppState>::make_driver())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(State::on_enter(AppState::Loading))
                .with_system(initial_load.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(State::on_update(AppState::Loading))
                .with_system(load_poll.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(State::on_enter(AppState::Main))
                .with_system(setup.system()),
        );
    app.run();
}

fn initial_load(commands: &mut Commands, assets: Res<AssetServer>) {
    println!("initial load");
    commands.insert_resource(Handles {
        mp3: assets.load("test_audio.mp3"),
    });
}

fn load_poll(handles: Res<Handles>, assets: Res<AssetServer>, mut state: ResMut<State<AppState>>) {
    println!("load poll");
    if assets.get_group_load_state([&handles.mp3].iter().map(|h| h.id)) == LoadState::Loaded {
        state.set_next(AppState::Main).unwrap();
    }
}

fn setup(commands: &mut Commands, handles: Res<Handles>) {
    println!("playing now");
    commands
        .spawn(())
        .with(GlobalSpeaker::new(handles.mp3.clone()));
}
