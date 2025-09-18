use leptos::prelude::*;
use leptos_use::use_timestamp;

struct SceneHandler {
    scene: Box<dyn Send + 'static + Fn(&(dyn Fn() + 'static), Signal<f64>) -> AnyView>,
}
impl SceneHandler {
    fn new(
        scene: Box<dyn Send + 'static + Fn(&(dyn Fn() + 'static), Signal<f64>) -> AnyView>,
    ) -> Self {
        Self { scene }
    }

    fn activate(&mut self, active_scene: RwSignal<usize>, last_scene: usize) -> AnyView {
        let id = active_scene.get_untracked();
        let timestamp = use_timestamp();
        let start = timestamp.get_untracked();
        let changed_scene = RwSignal::new(false);
        let stop_time = RwSignal::new(None);
        let time = Signal::derive(move || stop_time.get().unwrap_or(timestamp.get() - start));
        let next_scene = move || {
            if !changed_scene.get_untracked() {
                changed_scene.set(true);
                stop_time.set(Some(time.get_untracked()));
                if active_scene.get_untracked() == id && id != last_scene {
                    active_scene.set(id + 1);
                }
            }
        };

        (self.scene)(&next_scene, time)
    }
}

pub fn time_since(trigger: Signal<bool>) -> leptos::prelude::Signal<f64> {
    let is_triggered = Memo::new(move |x| x.is_some_and(|x| *x) || trigger.get());
    let start = RwSignal::new(None);
    let timestamp = use_timestamp();
    Effect::new(move || {
        if is_triggered.get() {
            start.set(Some(timestamp.get_untracked()));
        }
    });
    Signal::derive(move || match start.get() {
        None => 0.0,
        Some(start) => timestamp.get() - start,
    })
}

#[component]
pub fn SceneManager(
    sources: Vec<Box<dyn Send + 'static + Fn(&(dyn Fn() + 'static), Signal<f64>) -> AnyView>>,
) -> impl IntoView {
    let active_scene = RwSignal::new(0usize);
    let mut sources = sources
        .into_iter()
        .map(SceneHandler::new)
        .collect::<Vec<_>>();
    let last_scene = sources.len() - 1;

    move || {
        sources
            .get_mut(active_scene.get())
            .map(|x| x.activate(active_scene, last_scene))
    }
}

pub fn interpolate(
    easing: impl 'static + Send + Sync + Fn(f64) -> f64,
    input_range: (f64, f64),
    output_range: (f64, f64),
    clamp: bool,
) -> impl Fn(f64) -> f64 {
    let (min_in, max_in) = input_range;
    let (min_out, max_out) = output_range;
    move |input: f64| {
        let clamped = if clamp {
            input.max(min_in).min(max_in)
        } else {
            input
        };
        let normalised = (clamped - min_in) / (max_in - min_in);
        let eased = easing(normalised);
        eased * (max_out - min_out) + min_out
    }
}

pub fn ease_in_out_cubic(x: f64) -> f64 {
    3.0 * x * x - 2.0 * x * x * x
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    trigger_time: RwSignal<Option<f64>>,
    global_time: Signal<f64>,
}
impl Event {
    pub fn new(global_time: Signal<f64>) -> Self {
        Self {
            trigger_time: RwSignal::new(None),
            global_time,
        }
    }

    pub fn from_trigger(trigger: Signal<bool>, global_time: Signal<f64>) -> Self {
        let trigger_time = RwSignal::new(None);
        Effect::new(move |was_set: Option<bool>| {
            if trigger.get() && was_set.is_none_or(|x| !x) {
                trigger_time.set(Some(global_time.get_untracked()));
            }
            trigger.get()
        });
        Self {
            trigger_time,
            global_time,
        }
    }

    pub fn triggered(&self) -> bool {
        self.trigger_time.get().is_some()
    }

    pub fn triggered_untracked(&self) -> bool {
        self.trigger_time.get_untracked().is_some()
    }

    pub fn time(&self) -> f64 {
        self.trigger_time
            .get()
            .map(|t| self.global_time.get() - t)
            .unwrap_or(0.)
    }

    pub fn on(&self, response: impl Fn(f64, usize) + 'static) {
        let initial_trigger_time = self.trigger_time.get_untracked();
        let trigger_time = self.trigger_time;
        Effect::new(move |count: Option<usize>| {
            if trigger_time.get() != initial_trigger_time {
                response(trigger_time.get().unwrap(), count.unwrap_or_default());
                count.unwrap_or_default() + 1
            } else {
                count.unwrap_or_default()
            }
        });
    }

    pub fn time_untracked(&self) -> f64 {
        self.trigger_time
            .get_untracked()
            .map(|t| self.global_time.get_untracked() - t)
            .unwrap_or(0.)
    }

    pub fn after(&self, f: impl Fn(f64) + 'static, before: Vec<Event>) {
        let trigger_time = self.trigger_time;
        let global_time = self.global_time;
        let time_since_trigger =
            Signal::derive(move || trigger_time.get().map(|t| global_time.get() - t));
        Effect::new(move || {
            if let Some(t) = time_since_trigger.get()
                && !before.iter().any(|e| e.triggered())
            {
                f(t);
            }
        });
    }

    pub fn trigger(&self) {
        self.trigger_time
            .set(Some(self.global_time.get_untracked()));
    }

    pub fn trigger_once(&self) {
        if !self.triggered() {
            self.trigger();
        }
    }
}

#[macro_export]
macro_rules! events {
    ($time: expr, $($event: ident)+) => {
        $(
            let $event = Event::new($time);
        )+
    }
}

#[macro_export]
macro_rules! signals {
    ($($name: ident $value: expr),*) => {
        $(let $name = RwSignal::new($value);)*
    }
}

pub fn tick(ms_between_ticks: f64, ms_before_first_tick: f64) -> Signal<u64> {
    let timestamp = use_timestamp();
    let first_tick = ms_before_first_tick + timestamp.get_untracked();
    Memo::new(move |_| ((timestamp.get() - first_tick).max(0.0) / ms_between_ticks).floor() as u64)
        .into()
}

pub fn tick_bool(ms_between_ticks: f64, ms_before_first_tick: f64) -> Signal<bool> {
    let ticker = tick(ms_between_ticks, ms_before_first_tick);
    Signal::derive(move || ticker.get().is_multiple_of(2))
}

pub fn tick_iterate<T: Send + Sync + Clone + 'static>(
    ms_between_ticks: f64,
    ms_before_first_tick: f64,
    iterator: impl IntoIterator<Item = T>,
) -> Option<Signal<T>> {
    let values: Vec<_> = iterator.into_iter().collect();
    if values.len() == 0 {
        return None;
    }
    let ticker = tick(ms_between_ticks, ms_before_first_tick);
    Some(Signal::derive(move || {
        values[ticker.get() as usize % values.len()].clone()
    }))
}

pub fn pausable_timer(global_time: Signal<f64>) -> (Signal<f64>, RwSignal<bool>) {
    // we imaging that the timer was created
    // then immediately paused and restarted
    // so that we don't have to consider
    // a timer which has never been paused before
    let time_at_last_pause = RwSignal::new(0.);
    let paused = RwSignal::new(false);
    let paused_memo = Memo::new(move |_| paused.get());
    let time_at_last_resume = RwSignal::new(global_time.get_untracked());
    let accumulated_time = Signal::derive(move || {
        time_at_last_pause.get()
            + if paused.get() {
                0.0
            } else {
                global_time.get() - time_at_last_resume.get()
            }
    });
    Effect::new(move |_| {
        if paused_memo.get() {
            // paused_memo was false, but is now true: the timer was just paused!
            leptos::logging::log!("just paused!");
            time_at_last_pause.set(
                time_at_last_pause.get_untracked() + global_time.get_untracked()
                    - time_at_last_resume.get_untracked(),
            );
            leptos::logging::log!("last pause: {last_pause} | last_resume: {last_resume} | global_time: {global_time}", last_pause = time_at_last_pause.get_untracked().floor(), last_resume = time_at_last_resume.get_untracked().floor(), global_time = global_time.get_untracked());
        } else {
            // paused_memo was true, but is now false: the timer was just unpaused!
            leptos::logging::log!("just UN-paused!");
            time_at_last_resume.set(global_time.get_untracked());
        }
    });
    (accumulated_time, paused)
}
