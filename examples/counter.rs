use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    prelude::*,
    time::common_conditions::on_timer,
};
use bevy_rat::{ratatui_error_handler, ratatui_plugin, RatatuiEvent, RatatuiResource};
use crossterm::event;
use ratatui::{prelude::Stylize, widgets::Paragraph};
use std::io::Result;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 15.0,
            ))),
        )
        .add_plugins(ratatui_plugin)
        .add_systems(Startup, count_setup)
        .add_systems(
            Update,
            count_update.run_if(on_timer(Duration::from_secs(1))),
        )
        .add_systems(Update, ratatui_update.pipe(ratatui_error_handler))
        .add_systems(Update, q_to_quit.pipe(ratatui_error_handler))
        .run();
}

fn ratatui_update(mut rat: ResMut<RatatuiResource>, count: Res<Count>) -> Result<()> {
    rat.terminal.draw(|frame| {
        let message = format!("count: {} ('q' to quit)", count.count);
        let area = frame.size();
        frame.render_widget(Paragraph::new(message).white().on_blue(), area);
    })?;

    Ok(())
}

fn q_to_quit(
    mut exit: EventWriter<AppExit>,
    mut rat_events: EventReader<RatatuiEvent>,
) -> Result<()> {
    for ev in rat_events.read() {
        if let RatatuiEvent(event::Event::Key(key_event)) = ev {
            if key_event.kind == event::KeyEventKind::Press
                && key_event.code == event::KeyCode::Char('q')
            {
                exit.send(AppExit);
            }
        }
    }

    Ok(())
}

#[derive(Resource)]
struct Count {
    count: u32,
}

fn count_setup(mut commands: Commands) {
    commands.insert_resource(Count { count: 0 });
}

fn count_update(mut count: ResMut<Count>) {
    count.count += 1;
}
