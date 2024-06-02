# bevy_ratatui_render

Bevy inside the terminal!

Uses bevy headless rendering, [ratatui](https://github.com/ratatui-org/ratatui), and
[ratatui_image](https://github.com/benjajaja/ratatui-image) to print the rendered output
of your bevy application to the terminal using unicode halfblocks.

![cube example](https://assets.cxreiff.com/github/cube.gif)![foxes](https://assets.cxreiff.com/github/foxes.gif)![sponza test scene](https://assets.cxreiff.com/github/sponza.gif)

> examples/cube.rs, bevy many_foxes example, sponza test scene

## getting started

```rust
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RatatuiPlugin,
            RatatuiRenderPlugin::new(256, 256),
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, draw_scene.map(error))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ratatui_render: Res<RatatuiRenderContext>,
) {
    // spawn objects into your scene

    ...

    commands.spawn(Camera3dBundle {
        camera: Camera {
            target: ratatui_render.target(),
            ..default()
        },
        ..default()
    });
}

fn draw_scene(
    mut ratatui: ResMut<RatatuiContext>,
    ratatui_render: Res<RatatuiRenderContext>,
) -> io::Result<()> {
    ratatui.draw(|frame| {
        frame.render_widget(ratatui_render.widget(), frame.size());
    })?;

    Ok(())
}
```

There is a convenience function if you do not need access to the ratatui draw loop and just would like
the render to print to the full terminal (use instead of adding the `draw_scene` system above):

```rust
RatatuiRenderPlugin::new(256, 256).print_full_terminal()
```

I also recommend telling bevy you don't need a window or anti-aliasing:

```rust
DefaultPlugins
    .set(ImagePlugin::default_nearest())
    .set(WindowPlugin {
        primary_window: None,
        exit_condition: ExitCondition::DontExit,
        close_when_requested: false,
    })
```

## supported terminals

This relies on the terminal supporting 24-bit color. I've personality tested and confirmed that the following terminals display color correctly:

- Alacritty
- Kitty
- iTerm

## what's next?

This package currently contains both the headless rendering functionality and a layer of integration between
ratatui and bevy. I plan to scoop out the integration layer and contribute to the
[bevy_ratatui](https://github.com/joshka/bevy_ratatui/tree/main) package instead.

Additionally, this package is currently set up for a single camera. It shouldn't take much more code to allow
creating multiple render targets.

## credits

* Headless rendering code adapted from bevy's [headless_render](https://github.com/bevyengine/bevy/blob/main/examples/app/headless_renderer.rs)
example (@bugsweeper, @alice-i-cecile, @mockersf).
* bevy's [many_foxes](https://github.com/bevyengine/bevy/blob/main/examples/stress_tests/many_foxes.rs) example used for example gif.
* [bevy_sponza_scene](https://github.com/DGriffin91/bevy_sponza_scene) used for example gif.
