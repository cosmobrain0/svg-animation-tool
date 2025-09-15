use std::f64;

use cosmo_svg_animation_lib::ease_in_out_cubic;
use cosmo_svg_animation_lib::events;
use cosmo_svg_animation_lib::interpolate;
use cosmo_svg_animation_lib::signals;
use cosmo_svg_animation_lib::time_since;
use cosmo_svg_animation_lib::Event;
use cosmo_svg_animation_lib::Scene;
use cosmo_svg_animation_lib::SceneManager;
use leptos::{logging::log, prelude::*};

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    view! {
        <svg viewBox="-50 -50 100 100" style="width: 100vw; height: 100vh;">
            <SceneManager sources=vec![Box::new(SceneTwo)] />
        </svg>
    }
}

struct SceneOne;
impl Scene for SceneOne {
    fn start(&mut self, scene_complete: Box<dyn Fn()>, time: Signal<f64>) -> AnyView {
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
}

struct SceneTwo;
impl Scene for SceneTwo {
    fn start(&mut self, scene_complete: Box<dyn Fn()>, time: Signal<f64>) -> AnyView {
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
}
