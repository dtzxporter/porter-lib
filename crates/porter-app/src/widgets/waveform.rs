use std::marker::PhantomData;
use std::time::Duration;
use std::time::Instant;

use iced::advanced;
use iced::advanced::Widget;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse::Cursor;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree;

use iced::border::radius;
use iced::gradient::Linear;

use iced::Background;
use iced::Border;
use iced::Element;
use iced::Event;
use iced::Gradient;
use iced::Length;
use iced::Rectangle;
use iced::Size;
use iced::window;

use crate::palette;

/// Maximum safe floating point integer.
const MAX_SAFE_FLOAT: f32 = (1 << f32::MANTISSA_DIGITS) as f32;
/// Maximum height that the waveform block can be contained in.
const MAX_HEIGHT: f32 = 650.0;

/// Maximum number of waveform lines.
const WAVEFORM_COUNT: usize = 25;
/// Spacing between waveform lines.
const WAVEFORM_SPACING: f32 = 8.0;

/// Refresh interval.
const REFRESH_INTERVAL: Duration = Duration::from_millis(33);
/// Update interval.
const UPDATE_INTERVAL: Duration = REFRESH_INTERVAL
    .checked_mul(2)
    .expect("Update interval has failed");

/// A waveform effect animated container widget.
pub struct Waveform<'a, Message, Theme, Renderer> {
    width: Length,
    height: Length,
    is_playing: bool,
    seed: u64,
    on_update: Message,
    _phantom: PhantomData<&'a (Message, Theme, Renderer)>,
}

/// State for the waveform widget.
struct State {
    start: Instant,
    delta: f32,
    delta_last: f32,
    is_playing: bool,
    seed: u64,
    values: Vec<f32>,
}

impl<Message, Theme, Renderer> Waveform<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: advanced::Renderer,
{
    /// Constructs a new isntance of [`Waveform`].
    pub fn new(is_playing: bool, seed: u64, on_update: Message) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            is_playing,
            seed,
            on_update,
            _phantom: PhantomData,
        }
    }

    /// Sets the width of the [`Waveform`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Waveform`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Waveform<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let mut state = State {
            start: Instant::now(),
            delta: 0.0,
            delta_last: 0.0,
            is_playing: self.is_playing,
            seed: self.seed,
            values: Vec::with_capacity(WAVEFORM_COUNT),
        };

        state.calculate_waveforms();

        tree::State::new(state)
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: advanced::Layout<'_>,
        _cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if state.is_playing != self.is_playing {
            state.is_playing = self.is_playing;
            shell.request_redraw();
        }

        if state.seed != self.seed {
            state.seed = self.seed;
            state.calculate_waveforms();
        }

        if !self.is_playing {
            return;
        }

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let distance = now.duration_since(state.start);
            let distance_f32 = distance.as_secs_f32();

            if distance_f32 > MAX_SAFE_FLOAT {
                state.start = *now;
                state.delta = 0.0;
                state.delta_last = 0.0;
            } else {
                state.delta = distance_f32;
            }

            if state.delta - state.delta_last > UPDATE_INTERVAL.as_secs_f32() {
                state.delta_last = state.delta;
                shell.publish(self.on_update.clone());
            }

            shell.request_redraw_at(*now + REFRESH_INTERVAL);
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let bounds = layout.bounds();
        let bounds_center_y = bounds.center_y();

        let height = bounds.height.min(MAX_HEIGHT) / 2.0;

        let bar_count = state.values.len();
        let bar_width = bar_count as f32 * WAVEFORM_SPACING;

        let start_x = bounds.x + (bounds.width - bar_width) / 2.0;

        let background = Background::Gradient(Gradient::Linear(
            Linear::new(0.0)
                .add_stop(0.0, palette::PRIMARY_COLOR_DARK_250)
                .add_stop(0.5, palette::PRIMARY_COLOR)
                .add_stop(1.0, palette::PRIMARY_COLOR_DARK_250),
        ));

        for (i, &base) in state.values.iter().enumerate() {
            let x = start_x + i as f32 * WAVEFORM_SPACING + WAVEFORM_SPACING / 2.0;
            let anim = ((state.delta * 4.0) + i as f32 * 0.3).sin() * 0.5 + 0.5;
            let h = if state.is_playing {
                base * anim * height * 0.2
            } else {
                2.0
            };

            renderer.fill_quad(
                advanced::renderer::Quad {
                    bounds: Rectangle {
                        x: x - 2.0,
                        y: bounds_center_y - h,
                        width: 4.0,
                        height: h * 2.0,
                    },
                    border: Border {
                        radius: radius(4.0),
                        ..Default::default()
                    },
                    shadow: Default::default(),
                },
                background,
            );
        }
    }
}

impl State {
    /// Calculates new waveforms based on the current seed.
    fn calculate_waveforms(&mut self) {
        self.values.clear();

        for i in 0..WAVEFORM_COUNT {
            let mut x = self
                .seed
                .wrapping_add(i as u64)
                .wrapping_mul(0x5851F42D4C957F2D)
                .wrapping_add(1);

            x ^= x >> 33;
            x = x.wrapping_mul(0xFF51AFD7ED558CCD);
            x ^= x >> 33;
            x = x.wrapping_mul(0xFF51AFD7ED558CCD);
            x ^= x >> 33;

            let sample = (x as f64 / u64::MAX as f64) as f32;

            self.values.push(0.3 + sample * 0.7);
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Waveform<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(
        value: Waveform<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(value)
    }
}
