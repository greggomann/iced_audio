//! Various styles for a [`TickMarkGroup`] in a bar meter widget
///
/// [`TickMarkGroup`]: ../../core/tick_marks/struct.TickMarkGroup.html
use iced_native::Color;

use crate::core::Offset;
use crate::style::default_colors;

/// The placement of tick marks relative to the widget
#[derive(Debug, Clone, PartialEq)]
pub enum Placement {
    /// Tick marks on both sides of the widget.
    BothSides {
        /// The offset from the edge of the widget.
        offset: Offset,
        /// Whether to place the tick marks inside the widget (true) or
        /// outside the widget (false).
        inside: bool,
    },
    /// Tick marks only on the outside left/top side of the widget.
    LeftOrTop {
        /// The offset from the edge of the widget.
        offset: Offset,
        /// Whether to place the tick marks inside the widget (true) or
        /// outside the widget (false).
        inside: bool,
    },
    /// Tick marks only on the right/bottom side of the widget.
    RightOrBottom {
        /// The offset from the edge of the widget.
        offset: Offset,
        /// Whether to place the tick marks inside the widget (true) or
        /// outside the widget (false).
        inside: bool,
    },
    /// Tick marks in the center of the widget.
    Center {
        /// The offset from the center of the widget.
        offset: Offset,
        /// Whether to fill the length of the widget (true), or not (false).
        /// If this is true, then the length of each tick mark will act as the
        /// padding from the edge of the widget to the tick mark.
        fill_length: bool,
    },
    /// Split tick marks in the center of the widget.
    CenterSplit {
        /// The offset from the center of the widget.
        offset: Offset,
        /// Whether to fill the length of the widget (true), or not (false).
        /// If this is true, then the length of each tick mark will extend from
        /// the edges of the widget.
        fill_length: bool,
        /// The gap between the split tick marks. This has no effect if `fill_length`
        /// is true.
        gap: u16,
    },
}

impl std::default::Default for Placement {
    fn default() -> Self {
        Placement::BothSides {
            offset: Default::default(),
            inside: false,
        }
    }
}

/// The style of a tick mark
#[derive(Debug, Clone)]
pub struct Style {
    /// The style of a tier 1 tick mark.
    pub tier_1: Shape,
    /// The style of a tier 2 tick mark.
    pub tier_2: Shape,
    /// The style of a tier 3 tick mark.
    pub tier_3: Shape,
}

/// The shape of a tick mark
#[derive(Debug, Clone)]
pub enum Shape {
    /// No shape
    None,
    /// Line shape
    Line {
        /// The length of the tick mark.
        length: u16,

        /// The width (thickness) of the tick mark.
        width: u16,

        /// The color of the tick mark.
        color: Color,
    },
    /// Circle shape
    Circle {
        /// The diameter of the tick mark.
        diameter: u16,

        /// The color of the tick mark.
        color: Color,
    },
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            tier_1: Shape::Line {
                length: 4,
                width: 2,
                color: default_colors::TICK_TIER_1,
            },
            tier_2: Shape::Line {
                length: 3,
                width: 2,
                color: default_colors::TICK_TIER_2,
            },
            tier_3: Shape::Line {
                length: 2,
                width: 1,
                color: default_colors::TICK_TIER_3,
            },
        }
    }
}
