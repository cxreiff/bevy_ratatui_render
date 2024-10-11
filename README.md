# bevy_ratatui_render

Bevy inside the terminal!

Uses bevy headless rendering, [ratatui](https://github.com/ratatui-org/ratatui), and
[ratatui_image](https://github.com/benjajaja/ratatui-image) to print the rendered output
of your bevy application to the terminal using unicode halfblocks.

<p float="left">
<img src="https://assets.cxreiff.com/github/cube.gif" width="30%" alt="cube">
<img src="https://assets.cxreiff.com/github/foxes.gif" width="30%" alt="foxes">
<img src="https://assets.cxreiff.com/github/sponza.gif" width="30%" alt="sponza test scene">
<p>

> examples/cube.rs, bevy many_foxes example, sponza test scene

Use [bevy_ratatui](https://github.com/joshka/bevy_ratatui/tree/main) for setting ratatui up
and receiving terminal events (keyboard, focus, mouse, paste, resize) inside bevy.

## getting started

`cargo add bevy_ratatui_render bevy_ratatui`

```rust
fn main() {
    App::new()
        .add_plugins((
            // disable WinitPlugin as it panics in environments without a display server
            // disable LogPlugin as it interferes with terminal output.
            DefaultPlugins.build()
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),

            // create windowless loop and set its frame rate
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),

            // set up the Ratatui context and forward input events
            RatatuiPlugins::default(),

            // connect a bevy camera target to a ratatui widget
            RatatuiRenderPlugin::new("main", (256, 256)),
        ))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .run();
}

fn setup_scene_system(
    mut commands: Commands,
    ratatui_render: Res<RatatuiRenderContext>,
) {
    // spawn objects into your scene

    ...

    commands.spawn(Camera3dBundle {
        camera: Camera {
            target: ratatui_render.target("main").unwrap(),
            ..default()
        },
        ..default()
    });
}

fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    ratatui_render: Res<RatatuiRenderContext>,
) -> io::Result<()> {
    ratatui.draw(|frame| {
        frame.render_widget(ratatui_render.widget("main").unwrap(), frame.size());
    })?;

    Ok(())
}
```

As shown above, `RatatuiRenderPlugin` makes a `RatatuiRenderContext` resource available that has two
primary methods:

- `target(id)`: Provides a bevy `RenderTarget` that can be set as the `target` of a normal bevy camera.
- `widget(id)`: Provides a ratatui widget that prints the latest render made to the corresponding camera,
as unicode half-blocks.

There is a convenience function if you do not need access to the ratatui draw loop and just would
like the render to print to the full terminal (for the above example, use this instead of adding the
`draw_scene_system`):

```rust
RatatuiRenderPlugin::new("main", (256, 256)).print_full_terminal()
```

There is another convenience function for autoresizing the render texture to match the terminal
dimensions, during startup and when the terminal is resized:

```rust
RatatuiRenderPlugin::new("main", (1, 1)).autoresize()
```

To customize how the texture dimensions are calculated from the terminal dimensions, provide a callback
to `autoresize_conversion_fn`:

```rust
RatatuiRenderPlugin::new("main", (1, 1))
    .autoresize()
    .autoresize_conversion_fn(|(width, height)| (width * 4, height * 3))
```

## multiple renders

`RatatuiRenderPlugin` can be added to bevy multiple times. To access the correct render, use the same
string id you passed into `RatatuiRenderPlugin::new(id, dimensions)` to call the `target(id)` and
`widget(id)` methods on the `RatatuiRenderContext` resource.

## supported terminals

Printing to terminal relies on the terminal supporting 24-bit color. I've personally tested and confirmed
that the following terminals display correctly:

- Alacritty
- Kitty
- iTerm
- WezTerm

...but any terminal with 24-bit color support should work fine, if its performance is adequate.

## compatibility

| bevy  | bevy_ratatui_render |
|-------|---------------------|
| 0.14  | 0.5                 |
| 0.13  | 0.4                 |

## credits

* Headless rendering code adapted from bevy's
[headless_render](https://github.com/bevyengine/bevy/blob/main/examples/app/headless_renderer.rs)
example (@bugsweeper, @alice-i-cecile, @mockersf).
* bevy's [many_foxes](https://github.com/bevyengine/bevy/blob/main/examples/stress_tests/many_foxes.rs)
example used for example gif.
* [bevy_sponza_scene](https://github.com/DGriffin91/bevy_sponza_scene) used for example gif.
