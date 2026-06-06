use std::cell::Cell;

const OPENDESIGN_DEFAULT_VIEWPORT_WIDTH: f32 = 1280.0;
const OPENDESIGN_DEFAULT_VIEWPORT_HEIGHT: f32 = 720.0;
const HERO_GAME_UI_COMPILE_VIEWPORT_WIDTH: f32 = 1680.0;
const HERO_GAME_UI_COMPILE_VIEWPORT_HEIGHT: f32 = 786.0;

#[derive(Clone, Copy)]
pub(crate) struct OpenDesignViewport {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl OpenDesignViewport {
    pub(crate) const DEFAULT: Self = Self {
        width: OPENDESIGN_DEFAULT_VIEWPORT_WIDTH,
        height: OPENDESIGN_DEFAULT_VIEWPORT_HEIGHT,
    };

    pub(crate) const fn hero_game_ui_compile() -> Self {
        Self {
            width: HERO_GAME_UI_COMPILE_VIEWPORT_WIDTH,
            height: HERO_GAME_UI_COMPILE_VIEWPORT_HEIGHT,
        }
    }
}

thread_local! {
    static OPENDESIGN_VIEWPORT: Cell<OpenDesignViewport> = const { Cell::new(OpenDesignViewport::DEFAULT) };
}

pub(crate) fn current_opendesign_viewport() -> OpenDesignViewport {
    OPENDESIGN_VIEWPORT.with(Cell::get)
}

pub(crate) fn with_opendesign_viewport<T>(
    viewport: OpenDesignViewport,
    f: impl FnOnce() -> T,
) -> T {
    OPENDESIGN_VIEWPORT.with(|cell| {
        let previous = cell.replace(viewport);
        let result = f();
        cell.set(previous);
        result
    })
}
