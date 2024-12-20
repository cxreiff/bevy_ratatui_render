use std::io;

use bevy::prelude::*;
use bevy::utils::error;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::Widget,
};

use crate::{RatatuiCamera, RatatuiCameraWidget};

pub struct RatatuiCameraAutoprintPlugin;

impl Plugin for RatatuiCameraAutoprintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, autoprint_system.map(error));
    }
}

fn autoprint_system(
    mut ratatui: ResMut<RatatuiContext>,
    ratatui_camera_widgets: Query<(&RatatuiCamera, &RatatuiCameraWidget)>,
) -> io::Result<()> {
    let widgets = ratatui_camera_widgets
        .iter()
        .filter(|(camera, _)| camera.autoprint)
        .collect::<Vec<_>>();

    if !widgets.is_empty() {
        ratatui.draw(|frame| {
            let layout = Layout::new(
                Direction::Horizontal,
                vec![Constraint::Fill(1); widgets.len()],
            )
            .split(frame.area());

            for (i, (_, widget)) in widgets.iter().enumerate() {
                widget.render(layout[i], frame.buffer_mut());
            }
        })?;
    }

    Ok(())
}
