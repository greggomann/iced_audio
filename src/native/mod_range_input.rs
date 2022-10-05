//! Display an interactive dot that controls an [`NormalParam`]
//!
//! [`NormalParam`]: ../core/normal_param/struct.NormalParam.html

use std::fmt::Debug;

use iced_native::widget::tree::{self, Tree};
use iced_native::{
    event, keyboard, layout, mouse, touch, Clipboard, Element, Event, Layout,
    Length, Point, Rectangle, Shell, Size, Widget,
};

use crate::core::{Normal, NormalParam};

static DEFAULT_SIZE: u16 = 10;
static DEFAULT_SCALAR: f32 = 0.00385 / 2.0;
static DEFAULT_WHEEL_SCALAR: f32 = 0.01 / 2.0;
static DEFAULT_MODIFIER_SCALAR: f32 = 0.02;

/// An interactive dot that controls an [`NormalParam`]
///
/// [`NormalParam`]: ../core/normal_param/struct.NormalParam.html
#[allow(missing_debug_implementations)]
pub struct ModRangeInput<'a, Message, Renderer: self::Renderer> {
    normal_param: NormalParam,
    size: Length,
    on_change: Box<dyn Fn(Normal) -> Message + 'a>,
    scalar: f32,
    wheel_scalar: f32,
    modifier_scalar: f32,
    modifier_keys: keyboard::Modifiers,
    style: Renderer::Style,
}

impl<'a, Message, Renderer> ModRangeInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer,
{
    /// Creates a new [`ModRangeInput`].
    ///
    /// It expects:
    ///   * the [`NormalParam`] of the [`ModRangeInput`]
    ///   * a function that will be called when the [`ModRangeInput`] is turned.
    ///
    /// [`NormalParam`]: struct.NormalParam.html
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn new<F>(normal_param: NormalParam, on_change: F) -> Self
    where
        F: 'a + Fn(Normal) -> Message,
    {
        ModRangeInput {
            normal_param,
            size: Length::from(Length::Units(DEFAULT_SIZE)),
            on_change: Box::new(on_change),
            scalar: DEFAULT_SCALAR,
            wheel_scalar: DEFAULT_WHEEL_SCALAR,
            modifier_scalar: DEFAULT_MODIFIER_SCALAR,
            modifier_keys: keyboard::Modifiers::CTRL,
            style: Renderer::Style::default(),
        }
    }

    /// Sets the diameter of the [`ModRangeInput`]. The default size is
    /// `Length::from(Length::Units(31))`.
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn size(mut self, size: Length) -> Self {
        self.size = size;
        self
    }

    /// Sets the style of the [`ModRangeInput`].
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets how much the [`Normal`] value will change for the [`ModRangeInput`] per `y`
    /// pixel movement of the mouse.
    ///
    /// The default value is `0.001925`
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    /// [`Normal`]: ../../core/struct.Normal.html
    pub fn scalar(mut self, scalar: f32) -> Self {
        self.scalar = scalar;
        self
    }

    /// Sets how much the [`Normal`] value will change for the [`ModRangeInput`] per line scrolled
    /// by the mouse wheel.
    ///
    /// This can be set to `0.0` to disable the scroll wheel from moving the parameter.
    ///
    /// The default value is `0.005`
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    /// [`Normal`]: ../../core/struct.Normal.html
    pub fn wheel_scalar(mut self, wheel_scalar: f32) -> Self {
        self.wheel_scalar = wheel_scalar;
        self
    }

    /// Sets the modifier keys of the [`ModRangeInput`].
    ///
    /// The default modifier key is `Ctrl`.
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn modifier_keys(mut self, modifier_keys: keyboard::Modifiers) -> Self {
        self.modifier_keys = modifier_keys;
        self
    }

    /// Sets the scalar to use when the user drags the ModRangeInputs while holding down
    /// the modifier key. This is multiplied to the value set by
    /// `ModRangeInput::scalar()` (which the default is `0.001925`).
    ///
    /// For example, a `modifier_scalar` of `0.5` will cause the ModRangeInput to turn
    /// half as fast when the modifier key is down.
    ///
    /// The default `modifier_scalar` is `0.02`, and the default modifier key
    /// is `Ctrl`.
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn modifier_scalar(mut self, scalar: f32) -> Self {
        self.modifier_scalar = scalar;
        self
    }

    fn move_virtual_slider(
        &mut self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
        mut normal_delta: f32,
    ) {
        if normal_delta.abs() < f32::EPSILON {
            return;
        }

        if state.pressed_modifiers.contains(self.modifier_keys) {
            normal_delta *= self.modifier_scalar;
        }

        self.normal_param.value =
            Normal::new(state.continuous_normal - normal_delta);
        state.continuous_normal = self.normal_param.value.as_f32();

        shell.publish((self.on_change)(self.normal_param.value));
    }
}

/// The local state of an [`ModRangeInput`].
///
/// [`ModRangeInput`]: struct.ModRangeInput.html
#[derive(Debug, Copy, Clone)]
struct State {
    is_dragging: bool,
    prev_drag_y: f32,
    continuous_normal: f32,
    pressed_modifiers: keyboard::Modifiers,
    last_click: Option<mouse::Click>,
}

impl State {
    /// Creates a new [`ModRangeInput`] state.
    ///
    /// It expects:
    /// * current [`Normal`] value for the [`ModRangeInput`]
    ///
    /// [`Normal`]: ../../core/normal/struct.Normal.html
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    fn new(normal: Normal) -> Self {
        Self {
            is_dragging: false,
            prev_drag_y: 0.0,
            continuous_normal: normal.as_f32(),
            pressed_modifiers: Default::default(),
            last_click: None,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for ModRangeInput<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: self::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.normal_param.value))
    }

    fn width(&self) -> Length {
        self.size
    }

    fn height(&self) -> Length {
        self.size
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.size).height(self.size);

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let state = state.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if state.is_dragging {
                    let normal_delta =
                        (cursor_position.y - state.prev_drag_y) * self.scalar;

                    state.prev_drag_y = cursor_position.y;

                    self.move_virtual_slider(state, shell, normal_delta);

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if self.wheel_scalar == 0.0 {
                    return event::Status::Ignored;
                }

                if layout.bounds().contains(cursor_position) {
                    let lines = match delta {
                        iced_native::mouse::ScrollDelta::Lines {
                            y, ..
                        } => y,
                        iced_native::mouse::ScrollDelta::Pixels {
                            y, ..
                        } => {
                            if y > 0.0 {
                                1.0
                            } else if y < 0.0 {
                                -1.0
                            } else {
                                0.0
                            }
                        }
                    };

                    if lines != 0.0 {
                        let normal_delta = -lines * self.wheel_scalar;

                        self.move_virtual_slider(state, shell, normal_delta);

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    let click =
                        mouse::Click::new(cursor_position, state.last_click);

                    match click.kind() {
                        mouse::click::Kind::Single => {
                            state.is_dragging = true;
                            state.prev_drag_y = cursor_position.y;
                            state.continuous_normal =
                                self.normal_param.value.as_f32();
                        }
                        _ => {
                            state.is_dragging = false;

                            self.normal_param.value = self.normal_param.default;
                            state.continuous_normal =
                                self.normal_param.default.as_f32();

                            shell.publish((self.on_change)(
                                self.normal_param.value,
                            ));
                        }
                    }

                    state.last_click = Some(click);

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    state.continuous_normal = self.normal_param.value.as_f32();

                    return event::Status::Captured;
                }
            }
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed { modifiers, .. } => {
                    state.pressed_modifiers = modifiers;

                    return event::Status::Captured;
                }
                keyboard::Event::KeyReleased { modifiers, .. } => {
                    state.pressed_modifiers = modifiers;

                    return event::Status::Captured;
                }
                keyboard::Event::ModifiersChanged(modifiers) => {
                    state.pressed_modifiers = modifiers;

                    return event::Status::Captured;
                }
                _ => {}
            },
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        _theme: &Renderer::Theme,
        _style: &iced_native::renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let state = state.state.downcast_ref::<State>();
        renderer.draw(
            layout.bounds(),
            cursor_position,
            state.is_dragging,
            &self.style,
        )
    }
}

/// The renderer of an [`ModRangeInput`].
///
/// Your renderer will need to implement this trait before being
/// able to use an [`ModRangeInput`] in your user interface.
///
/// [`ModRangeInput`]: struct.ModRangeInput.html
pub trait Renderer: iced_native::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws an [`ModRangeInput`].
    ///
    /// It receives:
    ///   * the bounds of the [`ModRangeInput`]
    ///   * the current cursor position
    ///   * whether the ModRangeInput is currently being dragged
    ///   * the style of the [`ModRangeInput`]
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        is_dragging: bool,
        style: &Self::Style,
    );
}

impl<'a, Message, Renderer> From<ModRangeInput<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + self::Renderer,
{
    fn from(
        mod_range_input: ModRangeInput<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(mod_range_input)
    }
}
