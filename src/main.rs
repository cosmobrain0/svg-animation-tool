use cosmo_svg_animation_lib::pausable_timer;
use std::f64;

use cosmo_svg_animation_lib::ease_in_out_cubic;
use cosmo_svg_animation_lib::events;
use cosmo_svg_animation_lib::interpolate;
use cosmo_svg_animation_lib::signals;
use cosmo_svg_animation_lib::tick_iterate;
use cosmo_svg_animation_lib::Event;
use cosmo_svg_animation_lib::SceneManager;
use leptos::{logging::log, prelude::*};

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    view! {
        <svg viewBox="-50 -50 100 100" style="width: calc(min(100vw, 100vh)); height: calc(min(100vw, 100vh));">
            <SceneManager sources=vec![Box::new(pause_timer_test)] />
        </svg>
    }
}

fn scene_one(_scene_complete: &(dyn Fn() + 'static), time: Signal<f64>) -> AnyView {
    events!(time, started expanded particles_spawned shrunk);
    let ease_radius_expand = interpolate(ease_in_out_cubic, (0., 1000.), (0., 10.), true);
    let ease_radius_shrink = interpolate(ease_in_out_cubic, (0., 1000.), (10., 0.), true);
    let logarithmic_easing = |x: f64| (1. + (f64::consts::E - 1.) * x).ln();

    const HOTPINK: (f64, f64, f64) = (255., 105., 180.);

    signals![radius 0.0, main_colour HOTPINK, particle_offset 0.0, particle_radius 0.0];
    let rgb = Signal::derive(move || {
        format!(
            "rgb({}, {}, {})",
            main_colour.with(|t| t.0).to_string(),
            main_colour.with(|t| t.1).to_string(),
            main_colour.with(|t| t.2).to_string()
        )
    });

    started.after(
        move |t| {
            radius.set(ease_radius_expand(t));
            if t >= 1000.0 {
                expanded.trigger_once();
                particles_spawned.trigger();
            }
        },
        vec![expanded],
    );
    expanded.after(
        move |t| {
            radius.set(ease_radius_shrink(t));
            if t >= 1000.0 {
                shrunk.trigger_once();
            }
        },
        vec![shrunk],
    );
    let ease_colour = interpolate(logarithmic_easing, (0.0, 200.0), (1.0, 0.0), true);
    expanded.after(
        move |t| {
            let x = ease_colour(t);
            main_colour.set((HOTPINK.0 * x, HOTPINK.1 * x, HOTPINK.2 * x));
        },
        vec![shrunk],
    );
    let ease_particle_offset =
        interpolate(|x| 1. - (1. - x).powi(3), (0.0, 500.0), (8.0, 20.0), true);
    let ease_particle_radius = interpolate(|x| x, (400.0, 500.0), (2.0, 0.0), true);
    particles_spawned.after(
        move |t| {
            particle_offset.set(ease_particle_offset(t));
            particle_radius.set(ease_particle_radius(t));
        },
        vec![shrunk],
    );
    // shrunk.after(move |_| scene_complete(), vec![]);
    started.trigger();

    let particle_count = 10;
    let particle_index_to_angle = interpolate(
        |x| x,
        (0.0, particle_count as f64),
        (0.0, f64::consts::TAU),
        true,
    );
    let particles = move || {
        (0..particle_count)
            .map(|i| particle_index_to_angle(i as f64))
            .map(|angle| (angle.cos(), angle.sin()))
            .map(|(norm_x, norm_y)| {
                (
                    norm_x * particle_offset.get(),
                    norm_y * particle_offset.get(),
                )
            })
            .map(|(x, y)| view! { <circle fill="hotpink" cx=x cy=y r=particle_radius /> })
            .collect_view()
    };

    view! {
            {particles}
        <circle r=radius cx=0 cy=0 fill=rgb />
    }
    .into_any()
}

fn scene_two(scene_complete: &(dyn Fn() + 'static), time: Signal<f64>) -> AnyView {
    events!(time, particles_spawned);

    signals![particle_offset 0.0, particle_radius 0.0];

    let ease_particle_offset =
        interpolate(|x| 1. - (1. - x).powi(3), (0.0, 500.0), (8.0, 20.0), true);
    let ease_particle_radius = interpolate(|x| x, (400.0, 500.0), (2.0, 0.0), true);
    particles_spawned.after(
        move |t| {
            particle_offset.set(ease_particle_offset(t));
            particle_radius.set(ease_particle_radius(t));
            if t >= 1000.0 {
                particles_spawned.trigger();
            }
        },
        vec![],
    );
    particles_spawned.trigger();

    let particle_count = 10;
    let particle_index_to_angle = interpolate(
        |x| x,
        (0.0, particle_count as f64),
        (0.0, f64::consts::TAU),
        true,
    );
    let particles = move || {
        (0..particle_count)
            .map(|i| particle_index_to_angle(i as f64))
            .map(|angle| (angle.cos(), angle.sin()))
            .map(|(norm_x, norm_y)| {
                (
                    norm_x * particle_offset.get(),
                    norm_y * particle_offset.get(),
                )
            })
            .map(|(x, y)| view! { <circle fill="hotpink" cx=x cy=y r=particle_radius /> })
            .collect_view()
    };

    view! {
            {particles}
    }
    .into_any()
}

fn scene_three(scene_complete: &(dyn Fn() + 'static), time: Signal<f64>) -> AnyView {
    view! {
        <g transform="translate(-25, 0)">
            {scene_one(scene_complete, time)}
        </g>
        <g transform="translate(25, 0)">
            {scene_two(scene_complete, time)}
        </g>
    }
    .into_any()
}

fn ticker_test(scene_complete: &(dyn Fn() + 'static), time: Signal<f64>) -> AnyView {
    let counter = tick_iterate(1000.0, 1000.0, 0usize..=5).unwrap();
    (0..=5)
        .map(|i| Signal::derive(move || if i <= counter.get() { "#000" } else { "#555" }))
        .enumerate()
        .map(|(i, signal)| view! { <circle cx={i*5} cy=0 r=2 fill=signal /> })
        .collect_view()
        .into_any()
}

fn spritesheet_test(_scene_complete: &(dyn Fn() + 'static), time: Signal<f64>) -> AnyView {
    let spritesheet_width = 32 * 18;
    let sprite_count = 18;
    let sprite_index = tick_iterate(100.0, 0.0, 0..sprite_count).unwrap();

    let angles = RwSignal::new(vec![]);
    let particle_offset = RwSignal::new(0.);
    let ease_particle_offset =
        interpolate(|x| 3. * x * x - 2. * x * x * x, (0., 200.), (0., 1.), true);
    let ease_particle_radius = interpolate(|x| x, (150., 200.), (1., 0.), true);
    let particle_radius = RwSignal::new(0.);
    let start_enlarging_particles = Event::new(time);
    start_enlarging_particles.after(
        move |t| {
            particle_offset.set(ease_particle_offset(t));
            particle_radius.set(ease_particle_radius(t));
        },
        vec![],
    );

    let spawn_particles =
        Event::from_trigger(Signal::derive(move || sprite_index.get() == 9), time);
    spawn_particles.on(move |_, _| {
        let generate_float = || {
            (getrandom::u32().expect("should be able to generate a random number!")) as f64
                / u32::MAX as f64
        };
        let generate_angle = || generate_float() * f64::consts::TAU;
        let generate_max_offset = || generate_float() * 5. + 7.;
        angles.set(
            (1..100)
                .map(|_| generate_angle())
                .map(|angle| {
                    let max_offset = generate_max_offset();
                    (angle.cos() * max_offset, angle.sin() * max_offset)
                })
                .collect(),
        );
        start_enlarging_particles.trigger();
    });

    let particle_view = {
        let particle_count = Memo::new(move |_| angles.with(|x| x.len()));
        move || {
            (0..particle_count.get()).map(|i| Signal::derive(move || {
            let (norm_x, norm_y) = angles.with(|angles| angles[i]);
            let radius = particle_radius.get();
            (norm_x * particle_offset.get(), norm_y * particle_offset.get(), radius)
        })).map(|particle_data| view! { <circle cx={particle_data.get().0} cy={particle_data.get().1} r={particle_data.get().2} fill="#00000022" /> }).collect_view()
        }
    };

    view! {
        <Sprite spritesheet_path={String::from("./bomb.png")} spritesheet_width sprite_width=32 location=((-8.0).into(), (-8.0).into()) sprite_size=(16.0.into(), 16.0.into()) sprite_index id=0 />
        {particle_view}
    }.into_any()
}

#[component]
fn Sprite(
    spritesheet_path: String,
    spritesheet_width: usize,
    sprite_width: usize,
    location: (Signal<f64>, Signal<f64>),
    sprite_size: (Signal<f64>, Signal<f64>),
    sprite_index: Signal<usize>,
    id: usize,
) -> impl IntoView {
    let clip_path_id = format!("clippath-def-{id}");
    let clip_x = Signal::derive(move || sprite_index.get() as f64 * sprite_size.0.get());
    let clip_y = 0.;
    let sprite_height = sprite_size.1;
    // NOTE: this pixel-art solution -moz-crisp-edges only works in firefox!
    view! {
        <defs>
            <clipPath id={clip_path_id.clone()}>
                <rect x={location.0} y={location.1} width={sprite_size.0} height={sprite_size.1} />
            </clipPath>
        </defs>
        <image x={move || location.0.get() - clip_x.get()} y={move || location.1.get() - clip_y} clip-path={format!("url(#{clip_path_id})")} image-rendering="-moz-crisp-edges" href=spritesheet_path width={spritesheet_width as f64 * sprite_size.0.get() / sprite_width as f64} height={sprite_size.1}></image>
    }
}

fn pause_timer_test(scene_complete: &(dyn Fn() + 'static), time: Signal<f64>) -> AnyView {
    let (pausable_timer, timer_paused) = pausable_timer(time);
    let toggle_timer = move || {
        timer_paused.update(|x| *x = !*x);
    };
    let circle_fill = Memo::new(move |_| {
        if timer_paused.get() {
            "hotpink"
        } else {
            "black"
        }
    });
    view! {
        <circle on:click={move |_| toggle_timer()} fill=circle_fill cx=0 cy=0 r=10 />
        <text fill="black" x=10>{pausable_timer}</text>
    }
    .into_any()
}
