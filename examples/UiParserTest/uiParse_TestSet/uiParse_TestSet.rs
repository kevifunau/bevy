//! Batch runner for `examples/UiParserTest/uiParse_*` screenshot regression.
//!
//! Supported workflows:
//! - run every `uiParse_*` example and compare with checked-in Bevy references
//! - refresh Bevy reference screenshots from the latest parser output
//! - compare against optional official-example references when they exist
//!
//! Reference naming:
//! - Bevy parser reference: `assets/screenshots/<short-name>-bevy.png`
//! - Official upstream reference: `assets/screenshots/<short-name>-official.png`

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use image::{GenericImageView, ImageReader};

const CASES: &[UiParseCase] = &[
    UiParseCase::new("uiParse_anchor_layout", "examples/ui/layout/anchor_layout.rs"),
    UiParseCase::new("uiParse_button", "examples/ui/widgets/button.rs"),
    UiParseCase::new(
        "uiParse_display_and_visibility",
        "examples/ui/layout/display_and_visibility.rs",
    ),
    UiParseCase::new("uiParse_fixed_node", "examples/ui/layout/fixed_node.rs"),
    UiParseCase::new("uiParse_flex_layout", "examples/ui/layout/flex_layout.rs"),
    UiParseCase::new("uiParse_grid", "examples/ui/layout/grid.rs"),
    UiParseCase::new("uiParse_image_node", "examples/ui/images/image_node.rs"),
    UiParseCase::new(
        "uiParse_image_node_resizing",
        "examples/ui/images/image_node_resizing.rs",
    ),
    UiParseCase::new("uiParse_overflow", "examples/ui/scroll_and_overflow/overflow.rs"),
    UiParseCase::new(
        "uiParse_overflow_clip_margin",
        "examples/ui/scroll_and_overflow/overflow_clip_margin.rs",
    ),
    UiParseCase::new(
        "uiParse_relative_cursor_position",
        "examples/ui/relative_cursor_position.rs",
    ),
    UiParseCase::new(
        "uiParse_size_constraints",
        "examples/ui/layout/size_constraints.rs",
    ),
    UiParseCase::new("uiParse_tab_navigation", "examples/ui/widgets/tab_navigation.rs"),
    UiParseCase::new("uiParse_text_input", "examples/ui/text/text_input.rs"),
    UiParseCase::new(
        "uiParse_transparency_ui",
        "examples/ui/styling/transparency_ui.rs",
    ),
    UiParseCase::new("uiParse_ui_scaling", "examples/ui/ui_scaling.rs"),
    UiParseCase::new("uiParse_ui_target_camera", "examples/ui/ui_target_camera.rs"),
    UiParseCase::new(
        "uiParse_ui_texture_atlas",
        "examples/ui/images/ui_texture_atlas.rs",
    ),
    UiParseCase::new(
        "uiParse_ui_texture_atlas_slice",
        "examples/ui/images/ui_texture_atlas_slice.rs",
    ),
    UiParseCase::new(
        "uiParse_ui_texture_slice",
        "examples/ui/images/ui_texture_slice.rs",
    ),
    UiParseCase::new(
        "uiParse_ui_texture_slice_flip_and_tile",
        "examples/ui/images/ui_texture_slice_flip_and_tile.rs",
    ),
    UiParseCase::new("uiParse_ui_transform", "examples/ui/ui_transform.rs"),
    UiParseCase::new(
        "uiParse_window_fallthrough",
        "examples/ui/window_fallthrough.rs",
    ),
    UiParseCase::new("uiParse_z_index", "examples/ui/layout/z_index.rs"),
];

#[derive(Clone, Copy, Debug)]
struct UiParseCase {
    example_name: &'static str,
    official_example: &'static str,
}

impl UiParseCase {
    const fn new(example_name: &'static str, official_example: &'static str) -> Self {
        Self {
            example_name,
            official_example,
        }
    }

    fn case_dir(self, workspace_root: &Path) -> PathBuf {
        workspace_root
            .join("examples")
            .join("UiParserTest")
            .join("uiParse_TestSet")
            .join(self.example_name)
    }

    fn short_name(self) -> String {
        self.example_name.trim_start_matches("uiParse_").replace('_', "-")
    }

    fn results_dir(workspace_root: &Path) -> PathBuf {
        workspace_root
            .join("examples")
            .join("UiParserTest")
            .join("uiParse_TestSet")
            .join("results")
    }

    fn result_path(self, workspace_root: &Path) -> PathBuf {
        Self::results_dir(workspace_root).join(format!("{}.png", self.example_name))
    }

    fn bevy_reference_path(self, workspace_root: &Path) -> PathBuf {
        self.case_dir(workspace_root)
            .join("assets")
            .join("screenshots")
            .join(format!("{}-bevy.png", self.short_name()))
    }

    fn official_reference_path(self, workspace_root: &Path) -> PathBuf {
        self.case_dir(workspace_root)
            .join("assets")
            .join("screenshots")
            .join(format!("{}-official.png", self.short_name()))
    }

    fn legacy_reference_candidates(self, workspace_root: &Path) -> [PathBuf; 3] {
        let case_dir = self.case_dir(workspace_root);
        [
            case_dir
                .join("assets")
                .join("screenshots")
                .join(format!("{}.png", self.example_name)),
            case_dir.join("assets").join("screenshots").join("reference.png"),
            case_dir.join("reference.png"),
        ]
    }

    fn resolve_reference(self, workspace_root: &Path, kind: ReferenceKind) -> Option<PathBuf> {
        let primary = match kind {
            ReferenceKind::Bevy => self.bevy_reference_path(workspace_root),
            ReferenceKind::Official => self.official_reference_path(workspace_root),
        };

        if primary.exists() {
            return Some(primary);
        }

        match kind {
            ReferenceKind::Bevy => self
                .legacy_reference_candidates(workspace_root)
                .into_iter()
                .find(|path| path.exists()),
            ReferenceKind::Official => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReferenceKind {
    Bevy,
    Official,
}

impl ReferenceKind {
    fn label(self) -> &'static str {
        match self {
            ReferenceKind::Bevy => "bevy",
            ReferenceKind::Official => "official",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunMode {
    Check,
    RefreshBevyReference,
}

#[derive(Clone, Debug)]
struct RunnerConfig {
    mode: RunMode,
    reference_kind: ReferenceKind,
    selected_cases: Vec<UiParseCase>,
}

#[derive(Debug)]
enum CaseStatus {
    Passed,
    MissingReference,
    Failed,
    UpdatedReference,
}

#[derive(Debug)]
struct CaseResult {
    case: &'static str,
    official_example: &'static str,
    status: CaseStatus,
    screenshot: PathBuf,
    reference: Option<PathBuf>,
    changed_pixels: Option<u64>,
    dimensions: Option<(u32, u32)>,
    note: String,
}

fn main() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = parse_args();
    let target_dir = cargo_target_examples_dir(&workspace_root);
    fs::create_dir_all(UiParseCase::results_dir(&workspace_root))
        .expect("failed to create uiParse_TestSet results directory");

    ensure_example_binaries(&workspace_root, &target_dir, &config.selected_cases);

    let mut results = Vec::with_capacity(config.selected_cases.len());
    for case in &config.selected_cases {
        results.push(run_case(*case, &workspace_root, &target_dir, &config));
    }

    print_summary(&config, &results);

    if results
        .iter()
        .any(|result| matches!(result.status, CaseStatus::Failed))
    {
        std::process::exit(1);
    }
}

fn cargo_target_examples_dir(workspace_root: &Path) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"));
    base.join("debug").join("examples")
}

fn parse_args() -> RunnerConfig {
    let mut args = std::env::args().skip(1);
    let mut mode = RunMode::Check;
    let mut reference_kind = ReferenceKind::Bevy;
    let mut names = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--case" => {
                let Some(name) = args.next() else {
                    eprintln!("Missing value after --case");
                    std::process::exit(2);
                };
                names.push(name);
            }
            "--refresh-bevy-reference" => mode = RunMode::RefreshBevyReference,
            "--use-official-reference" => reference_kind = ReferenceKind::Official,
            "--use-bevy-reference" => reference_kind = ReferenceKind::Bevy,
            other => names.push(other.to_string()),
        }
    }

    let selected_cases = if names.is_empty() {
        CASES.to_vec()
    } else {
        let mut selected = Vec::new();
        for name in names {
            let Some(case) = CASES.iter().copied().find(|case| case.example_name == name) else {
                eprintln!("Unknown uiParse case: {name}");
                std::process::exit(2);
            };
            selected.push(case);
        }
        selected
    };

    RunnerConfig {
        mode,
        reference_kind,
        selected_cases,
    }
}

fn ensure_example_binaries(workspace_root: &Path, target_dir: &Path, selected: &[UiParseCase]) {
    let mut command = Command::new("cargo");
    command.arg("build");
    for case in selected {
        command.arg("--example").arg(case.example_name);
    }
    command.current_dir(workspace_root);

    let output = command
        .output()
        .unwrap_or_else(|error| panic!("failed to build uiParse examples: {error}"));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("failed to build uiParse examples:\n{stderr}");
    }

    for case in selected {
        let executable = example_binary_path(target_dir, case.example_name);
        assert!(
            executable.exists(),
            "compiled example binary is missing after build step: {}",
            executable.display()
        );
    }
}

fn run_case(
    case: UiParseCase,
    workspace_root: &Path,
    target_dir: &Path,
    config: &RunnerConfig,
) -> CaseResult {
    let screenshot = case.result_path(workspace_root);
    if screenshot.exists() {
        let _ = fs::remove_file(&screenshot);
    }

    let executable = example_binary_path(target_dir, case.example_name);
    if !executable.exists() {
        return CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::Failed,
            screenshot,
            reference: None,
            changed_pixels: None,
            dimensions: None,
            note: format!(
                "compiled example binary is missing after build step: {}",
                executable.display(),
            ),
        };
    }

    let mut command = Command::new(&executable);
    command
        .env("BUI_SCREENSHOT_PATH", &screenshot)
        .current_dir(workspace_root)
        .stdin(Stdio::null());

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            return CaseResult {
                case: case.example_name,
                official_example: case.official_example,
                status: CaseStatus::Failed,
                screenshot,
                reference: None,
                changed_pixels: None,
                dimensions: None,
                note: format!("failed to start example binary {}: {error}", executable.display()),
            };
        }
    };

    if !output.status.success() {
        let mut note = String::from("example run failed");
        if !output.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr.trim();
            if !stderr.is_empty() {
                let _ = write!(note, ": {stderr}");
            }
        }
        return CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::Failed,
            screenshot,
            reference: None,
            changed_pixels: None,
            dimensions: None,
            note,
        };
    }

    if !screenshot.exists() {
        return CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::Failed,
            screenshot,
            reference: None,
            changed_pixels: None,
            dimensions: None,
            note: "run succeeded but screenshot was not produced".to_string(),
        };
    }

    match config.mode {
        RunMode::RefreshBevyReference => refresh_bevy_reference(case, workspace_root, &screenshot),
        RunMode::Check => compare_against_reference(case, workspace_root, &screenshot, config.reference_kind),
    }
}

fn refresh_bevy_reference(
    case: UiParseCase,
    workspace_root: &Path,
    screenshot: &Path,
) -> CaseResult {
    let reference = case.bevy_reference_path(workspace_root);
    let parent = reference
        .parent()
        .expect("bevy reference path should have a parent directory");
    if let Err(error) = fs::create_dir_all(parent) {
        return CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::Failed,
            screenshot: screenshot.to_path_buf(),
            reference: Some(reference),
            changed_pixels: None,
            dimensions: None,
            note: format!("failed to create reference directory: {error}"),
        };
    }

    if let Err(error) = fs::copy(screenshot, &reference) {
        return CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::Failed,
            screenshot: screenshot.to_path_buf(),
            reference: Some(reference),
            changed_pixels: None,
            dimensions: None,
            note: format!("failed to refresh bevy reference: {error}"),
        };
    }

    match png_dimensions(screenshot) {
        Ok(dimensions) => CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::UpdatedReference,
            screenshot: screenshot.to_path_buf(),
            reference: Some(reference),
            changed_pixels: Some(0),
            dimensions: Some(dimensions),
            note: "refreshed bevy reference screenshot".to_string(),
        },
        Err(error) => CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::Failed,
            screenshot: screenshot.to_path_buf(),
            reference: Some(reference),
            changed_pixels: None,
            dimensions: None,
            note: error,
        },
    }
}

fn compare_against_reference(
    case: UiParseCase,
    workspace_root: &Path,
    screenshot: &Path,
    reference_kind: ReferenceKind,
) -> CaseResult {
    let reference = case.resolve_reference(workspace_root, reference_kind);

    let Some(reference) = reference else {
        return CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::MissingReference,
            screenshot: screenshot.to_path_buf(),
            reference: None,
            changed_pixels: None,
            dimensions: None,
            note: format!("no checked-in {} reference image yet", reference_kind.label()),
        };
    };

    match compare_pngs(&reference, screenshot) {
        Ok((changed_pixels, dimensions)) => CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: if changed_pixels == 0 {
                CaseStatus::Passed
            } else {
                CaseStatus::Failed
            },
            screenshot: screenshot.to_path_buf(),
            reference: Some(reference),
            changed_pixels: Some(changed_pixels),
            dimensions: Some(dimensions),
            note: if changed_pixels == 0 {
                format!("pixel-perfect match against {} reference", reference_kind.label())
            } else {
                format!(
                    "pixel diff detected against {} reference: {changed_pixels}",
                    reference_kind.label()
                )
            },
        },
        Err(error) => CaseResult {
            case: case.example_name,
            official_example: case.official_example,
            status: CaseStatus::Failed,
            screenshot: screenshot.to_path_buf(),
            reference: Some(reference),
            changed_pixels: None,
            dimensions: None,
            note: error,
        },
    }
}

fn example_binary_path(target_dir: &Path, example_name: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        target_dir.join(format!("{example_name}.exe"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        target_dir.join(example_name)
    }
}

fn png_dimensions(path: &Path) -> Result<(u32, u32), String> {
    let image = ImageReader::open(path)
        .map_err(|error| format!("failed to open image {}: {error}", path.display()))?
        .decode()
        .map_err(|error| format!("failed to decode image {}: {error}", path.display()))?;
    Ok(image.dimensions())
}

fn compare_pngs(reference: &Path, candidate: &Path) -> Result<(u64, (u32, u32)), String> {
    let reference_image = ImageReader::open(reference)
        .map_err(|error| format!("failed to open reference {}: {error}", reference.display()))?
        .decode()
        .map_err(|error| format!("failed to decode reference {}: {error}", reference.display()))?;
    let candidate_image = ImageReader::open(candidate)
        .map_err(|error| format!("failed to open candidate {}: {error}", candidate.display()))?
        .decode()
        .map_err(|error| format!("failed to decode candidate {}: {error}", candidate.display()))?;

    if reference_image.dimensions() != candidate_image.dimensions() {
        return Err(format!(
            "image size mismatch: reference {:?}, candidate {:?}",
            reference_image.dimensions(),
            candidate_image.dimensions()
        ));
    }

    let dimensions = reference_image.dimensions();
    let reference_rgba = reference_image.to_rgba8();
    let candidate_rgba = candidate_image.to_rgba8();
    let changed_pixels = reference_rgba
        .pixels()
        .zip(candidate_rgba.pixels())
        .filter(|(left, right)| left.0 != right.0)
        .count() as u64;

    Ok((changed_pixels, dimensions))
}

fn print_summary(config: &RunnerConfig, results: &[CaseResult]) {
    println!("uiParse_TestSet summary");
    println!("cases: {}", results.len());
    println!("mode: {:?}", config.mode);
    println!("reference_kind: {:?}", config.reference_kind);

    let mut passed = 0usize;
    let mut missing_reference = 0usize;
    let mut failed = 0usize;
    let mut updated_reference = 0usize;

    for result in results {
        match result.status {
            CaseStatus::Passed => passed += 1,
            CaseStatus::MissingReference => missing_reference += 1,
            CaseStatus::Failed => failed += 1,
            CaseStatus::UpdatedReference => updated_reference += 1,
        }

        println!();
        println!("case: {}", result.case);
        println!("official: {}", result.official_example);
        println!("screenshot: {}", result.screenshot.display());

        if let Some(reference) = &result.reference {
            println!("reference: {}", reference.display());
        } else {
            println!("reference: <missing>");
        }

        if let Some((width, height)) = result.dimensions {
            println!("dimensions: {}x{}", width, height);
        }

        if let Some(changed_pixels) = result.changed_pixels {
            println!("changed_pixels: {}", changed_pixels);
        }

        println!(
            "status: {}",
            match result.status {
                CaseStatus::Passed => "passed",
                CaseStatus::MissingReference => "missing_reference",
                CaseStatus::Failed => "failed",
                CaseStatus::UpdatedReference => "updated_reference",
            }
        );
        println!("note: {}", result.note);
    }

    println!();
    println!(
        "totals: passed={}, missing_reference={}, failed={}, updated_reference={}",
        passed, missing_reference, failed, updated_reference
    );
}
