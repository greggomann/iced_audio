//! Display an interactive horizontal slider that controls a `Param`

use std::fmt::Debug;

use iced_native::{
    input::{mouse, ButtonState, keyboard},
    layout, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use std::hash::Hash;

use crate::core::{Normal, Param};

static DEFAULT_MODIFIER_SCALAR: f32 = 0.02;

/// A horizontal slider GUI widget that controls a [`Param`]
///
/// An [`HSlider`] will try to fill the horizontal space of its container.
///
/// [`Param`]: trait.Param.html
/// [`HSlider`]: struct.HSlider.html
#[allow(missing_debug_implementations)]
pub struct HSlider<'a, Message, Renderer: self::Renderer, ID>
where
    ID: Debug + Copy + Clone
{
    state: &'a mut State<ID>,
    normal: Normal,
    default_normal: Normal,
    on_change: Box<dyn Fn((ID, Normal)) -> Message>,
    modifier_scalar: f32,
    modifier_keys: keyboard::ModifiersState,
    width: Length,
    style: Renderer::Style,
}

impl<'a, Message, Renderer: self::Renderer, ID>
    HSlider<'a, Message, Renderer, ID>
where
    ID: Debug + Copy + Clone
{
    /// Creates a new [`HSlider`].
    ///
    /// It expects:
    ///   * the local [`State`] of the [`HSlider`]
    ///   * a [`Param`] with the current and default values
    ///   * a function that will be called when the [`HSlider`] is dragged.
    ///   It receives the parameter's ID and the new [`Normal`] of the
    /// [`HSlider`].
    ///
    /// [`State`]: struct.Normal.State.html
    /// [`Param`]: trait.Param.html
    /// [`Normal`]: struct.Normal.html
    /// [`HSlider`]: struct.HSlider.html
    pub fn new<F>(
        state: &'a mut State<ID>,
        param: &impl Param<ID=ID>,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn((ID, Normal)) -> Message,
    {  
        HSlider {
            state,
            normal: param.normal(),
            default_normal: param.default_normal(),
            on_change: Box::new(on_change),
            modifier_scalar: DEFAULT_MODIFIER_SCALAR,
            modifier_keys: keyboard::ModifiersState {
                control: true,
                ..Default::default()
            },
            width: Length::Fill,
            style: Renderer::Style::default(),
        }
    }

    /// Sets the width of the [`HSlider`].
    ///
    /// [`HSlider`]: struct.HSlider.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the style of the [`HSlider`].
    ///
    /// [`HSlider`]: struct.HSlider.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the modifier keys of the [`HSlider`].
    ///
    /// The default modifier key is `Ctrl`.
    ///
    /// [`HSlider`]: struct.HSlider.html
    pub fn modifier_keys(
        mut self,
        modifier_keys: keyboard::ModifiersState,
    ) -> Self {
        self.modifier_keys = modifier_keys;
        self
    }

    /// Sets the scalar to use when the user drags the slider while holding down
    /// the modifier key.
    ///
    /// For example, a scalar of `0.5` will cause the slider to move half a
    /// pixel for every pixel the mouse moves.
    ///
    /// The default scalar is `0.02`, and the default modifier key is `Ctrl`.
    ///
    /// [`HSlider`]: struct.HSlider.html
    pub fn modifier_scalar(mut self, scalar: f32) -> Self {
        self.modifier_scalar = scalar;
        self
    }
}

/// The local state of an [`HSlider`].
///
/// It stores a unique identifier of user supplied type `ID`. This can be an
/// `enum`, `u32`, `i32`, etc. Each parameter must have a unique `ID` value!
///
/// [`HSlider`]: struct.HSlider.html
#[derive(Debug, Copy, Clone)]
pub struct State<ID: Debug + Copy + Clone> {
    id: ID,
    is_dragging: bool,
    prev_drag_x: f32,
    continuous_normal: f32,
    pressed_modifiers: keyboard::ModifiersState,
    last_click: Option<mouse::Click>,
}

impl<ID: Debug + Copy + Clone> State<ID> {
    /// Creates a new [`HSlider`] state.
    ///
    /// It expects:
    /// * a [`Param`] with the ID of the parameter to control
    ///
    /// [`Param`]: trait.Param.html
    /// [`HSlider`]: struct.HSlider.html
    pub fn new(param: &impl Param<ID=ID>) -> Self {
        Self {
            id: param.id(),
            is_dragging: false,
            prev_drag_x: 0.0,
            continuous_normal: param.normal().value(),
            pressed_modifiers: Default::default(),
            last_click: None,
        }
    }
}

impl<'a, Message, Renderer, ID> Widget<Message, Renderer>
    for HSlider<'a, Message, Renderer, ID>
where
    Renderer: self::Renderer,
    ID: Debug + Copy + Clone,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
            let limits = limits
            .width(self.width)
            .height(Length::from(Length::Units(
                renderer.height(&self.style)
                )));
        
            let size = limits.resolve(Size::ZERO);

            layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state,
            }) => match state {
                ButtonState::Pressed => {
                    if layout.bounds().contains(cursor_position) {
                        let click = mouse::Click::new(
                            cursor_position,
                            self.state.last_click,
                        );

                        match click.kind() {
                            mouse::click::Kind::Single => {
                                self.state.is_dragging = true;
                                self.state.prev_drag_x = cursor_position.x;
                            }
                            _ => {
                                self.state.is_dragging = false;

                                messages.push((self.on_change)(
                                    (self.state.id, self.default_normal)
                                ));
                            }
                        }

                        self.state.last_click = Some(click);
                    }
                }
                ButtonState::Released => {
                    self.state.is_dragging = false;
                    self.state.continuous_normal = self.normal.value();
                }
            },
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.state.is_dragging {
                    let bounds_width = layout.bounds().width;
                    if bounds_width != 0.0 {
                        let mut movement_x =
                            (cursor_position.x - self.state.prev_drag_x)
                                / bounds_width;

                        if self.state.pressed_modifiers.matches(
                            self.modifier_keys) {
                            movement_x *= self.modifier_scalar;
                        }

                        let normal =
                            self.state.continuous_normal + movement_x;

                        self.state.continuous_normal = normal;
                        self.state.prev_drag_x = cursor_position.x;

                        messages.push((self.on_change)(
                            (self.state.id, normal.into())
                        ));
                    }
                }
            },
            Event::Keyboard(keyboard::Event::Input {
                modifiers,
                ..
            }) => {
                self.state.pressed_modifiers = modifiers;
            },
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(
            layout.bounds(),
            cursor_position,
            self.normal,
            self.state.is_dragging,
            &self.style,
        )
    }

/// test
    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
    }
}

/// The renderer of an [`HSlider`].
///
/// Your renderer will need to implement this trait before being
/// able to use an [`HSlider`] in your user interface.
///
/// [`HSlider`]: struct.HSlider.html
pub trait Renderer: iced_native::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// returns the height of the HSlider
    fn height(&self, style_sheet: &Self::Style) -> u16;

    /// Draws an [`HSlider`].
    ///
    /// It receives:
    ///   * the bounds of the [`HSlider`]
    ///   * the current cursor position
    ///   * the current normal of the [`HSlider`]
    ///   * the local state of the [`HSlider`]
    ///   * the style of the ['HSlider']
    ///
    /// [`HSlider`]: struct.HSlider.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        normal: Normal,
        is_dragging: bool,
        style: &Self::Style,
    ) -> Self::Output;
}

impl<'a, Message, Renderer, ID>
    From<HSlider<'a, Message, Renderer, ID>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
    ID: Debug + Copy + Clone,
{
    fn from(
        h_slider: HSlider<'a, Message, Renderer, ID>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(h_slider)
    }
}