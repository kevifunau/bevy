use crate::core::style::css_gradients::SimpleGradientOverlayDirection;

pub(crate) fn css_simple_linear_gradient_direction(
    args: &[&str],
) -> Option<(SimpleGradientOverlayDirection, Option<f32>, usize)> {
    let first = args.first()?.trim();
    if let Some((direction, diagonal_angle)) = css_linear_gradient_direction_from_token(first) {
        return Some((direction, diagonal_angle, 1));
    }

    Some((SimpleGradientOverlayDirection::TopToBottom, None, 0))
}

fn css_linear_gradient_direction_from_token(
    token: &str,
) -> Option<(SimpleGradientOverlayDirection, Option<f32>)> {
    let token = token.trim().to_ascii_lowercase();
    if let Some(result) = css_linear_gradient_direction_from_keyword(&token) {
        return Some(result);
    }

    let degrees = token.strip_suffix("deg")?.trim().parse::<f32>().ok()?;
    css_linear_gradient_direction_from_degrees(degrees)
}

fn css_linear_gradient_direction_from_keyword(
    token: &str,
) -> Option<(SimpleGradientOverlayDirection, Option<f32>)> {
    let token = token.trim();
    if !token.starts_with("to ") {
        return None;
    }

    let has_left = token.contains("left");
    let has_right = token.contains("right");
    let has_top = token.contains("top");
    let has_bottom = token.contains("bottom");

    let diagonal = has_left ^ has_right && has_top ^ has_bottom;

    if diagonal {
        let degrees: f32 = match (has_bottom, has_right) {
            (true, true) => 135.0,
            (true, false) => 225.0,
            (false, true) => 45.0,
            (false, false) => 315.0,
        };
        let direction = if has_right {
            SimpleGradientOverlayDirection::LeftToRight
        } else {
            SimpleGradientOverlayDirection::RightToLeft
        };
        return Some((direction, Some(degrees)));
    }

    if has_left ^ has_right {
        return Some((
            if has_right {
                SimpleGradientOverlayDirection::LeftToRight
            } else {
                SimpleGradientOverlayDirection::RightToLeft
            },
            None,
        ));
    }

    if has_top ^ has_bottom {
        return Some((
            if has_bottom {
                SimpleGradientOverlayDirection::TopToBottom
            } else {
                SimpleGradientOverlayDirection::BottomToTop
            },
            None,
        ));
    }

    None
}

pub(crate) fn css_linear_gradient_direction_from_degrees(
    degrees: f32,
) -> Option<(SimpleGradientOverlayDirection, Option<f32>)> {
    if !degrees.is_finite() {
        return None;
    }

    let normalized = degrees.rem_euclid(360.0);
    let radians = normalized.to_radians();
    let horizontal = radians.sin();
    let vertical = -radians.cos();

    let direction = if horizontal.abs() >= vertical.abs() {
        if horizontal >= 0.0 {
            SimpleGradientOverlayDirection::LeftToRight
        } else {
            SimpleGradientOverlayDirection::RightToLeft
        }
    } else if vertical >= 0.0 {
        SimpleGradientOverlayDirection::TopToBottom
    } else {
        SimpleGradientOverlayDirection::BottomToTop
    };

    let diagonal_angle = if horizontal.abs() > 0.01 && vertical.abs() > 0.01 {
        Some(normalized)
    } else {
        None
    };

    Some((direction, diagonal_angle))
}
