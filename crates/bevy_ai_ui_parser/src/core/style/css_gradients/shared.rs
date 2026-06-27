#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SimpleGradientOverlayDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

pub(crate) struct SimpleGradientOverlaySpec {
    pub(crate) color: String,
    pub(crate) kind: SimpleGradientOverlayKind,
}

pub(crate) struct SimpleGradientOverlayBand {
    pub(crate) color: String,
    pub(crate) start_ratio: f32,
    pub(crate) end_ratio: f32,
}

pub(crate) enum SimpleGradientOverlayKind {
    Linear {
        direction: SimpleGradientOverlayDirection,
        diagonal_angle: Option<f32>,
        start_ratio: f32,
        end_ratio: f32,
    },
    Radial {
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        preserve_circle: bool,
    },
    RadialRing {
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        border_width: f32,
        preserve_circle: bool,
    },
    ConicArc {
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        rotation_degrees: f32,
    },
}

pub(crate) struct SimpleRadialGradientRingOverlay {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) border_width: f32,
    pub(crate) preserve_circle: bool,
    pub(crate) color: String,
}

pub(crate) struct CssGradientStop {
    pub(crate) color: String,
    pub(crate) start_ratio: f32,
    pub(crate) end_ratio: f32,
    pub(crate) is_multi_position: bool,
}

pub(crate) struct SimpleRadialGradientOverlay {
    pub(crate) center_x: f32,
    pub(crate) center_y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) preserve_circle: bool,
    pub(crate) color: String,
}

pub(crate) struct SimpleConicGradientOverlay {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) rotation_degrees: f32,
    pub(crate) color: String,
}
