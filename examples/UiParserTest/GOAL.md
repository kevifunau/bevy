/goal Build and continuously refine a strict external-facing Bevy UI JSON contract by reverse-engineering every official `examples/ui` case into BUI JSON one case at a time, validating each case through `bevy_ai_ui_parser`, and keeping parser code, schema, test cases, and contract documentation in exact sync until the `examples/ui` backlog is exhausted or a `Stop if` condition is reached.

Scope: Only work in `crates/bevy_ai_ui_parser/`, `examples/UiParserTest/`, `Cargo.toml` example registrations related to BUI parser cases, and `/mnt/d/CodeRepo/GusdDoc/GusdDoc/RUST学习/引擎UI能力设计/Bevy UI JSON 契约.md`. Use `examples/ui/` only as the upstream capability reference. Do not modify unrelated Bevy engine crates or unrelated examples unless a parser capability truly requires it and the stop conditions do not trigger.

Constraints:
- The external contract markdown must never describe a field, node type, rule, or behavior that the current parser runtime does not actually support.
- Keep the contract strict: unknown fields must remain errors, and unsupported Bevy UI capabilities must be excluded or explicitly documented as unsupported rather than loosely accepted.
- Every newly supported capability must be backed by an official `examples/ui` source case or a directly derived minimal validation case under `examples/UiParserTest/`.
- Keep the parser contract aligned across four layers: Rust parser runtime, Rust validator, JSON schema, and external markdown contract.
- Do not broaden the contract into a generic HTML/CSS-style DSL; only expose the subset that maps cleanly and stably to Bevy UI.
- Do not modify unrelated public APIs or unrelated engine behavior outside the BUI parser workstream.
- Do not replace an official example with a reduced or representative subset when the goal is an example reversal; `uiParse_*` cases should preserve the source example's meaningful structure, combinations, and coverage unless the unsupported gap is explicitly documented.
- Process the official `examples/ui` backlog case by case; do not stop after a single success if more source cases remain and no `Stop if` condition has fired.

Done when:
1. For each newly handled official UI capability, there is a matching `uiParse_*` example or clearly named validation case under `examples/UiParserTest/`.
2. Any new or changed parser behavior is implemented in `crates/bevy_ai_ui_parser/src/lib.rs`.
3. Any new or changed contract shape is reflected in `crates/bevy_ai_ui_parser/schema/bui.schema.json`.
4. Any new or changed external-facing rule is reflected in `/mnt/d/CodeRepo/GusdDoc/GusdDoc/RUST学习/引擎UI能力设计/Bevy UI JSON 契约.md`.
5. Related example registrations in `Cargo.toml` are updated when a new runnable example is added.
6. The new JSON case validates successfully with `cargo run --example validate_bui_json -- <path-to-json>` or an equivalent direct validator path.
7. The resulting parser-rendered UI case is runnable through the corresponding `cargo run --example uiParse_*` path when applicable.
8. The work produces at least one concrete contract rule, supported capability, or explicit unsupported boundary that can be traced back to an official `examples/ui` source case.
9. For reversed official examples, the produced `uiParse_*` case is not a shorthand demo: it must match the official source example's meaningful layout/content combinations closely enough to serve as a real equivalence check.
10. The overall goal is only complete when every relevant official file under `examples/ui/` has been processed into one of these outcomes:
    - supported with a matching `uiParse_*` or validation case
    - partially supported with the unsupported boundary explicitly documented
    - rejected under a recorded `Stop if` condition

Stop if:
- Completing the next capability requires adding a new dependency not already justified by the BUI parser work.
- Completing the next capability requires modifying unrelated Bevy subsystems outside `crates/bevy_ai_ui_parser/`, `examples/UiParserTest/`, relevant `Cargo.toml` example entries, or the contract markdown.
- The official `examples/ui` source behavior cannot be mapped into a stable JSON contract without introducing ambiguous, heuristic, or speculative parser behavior.
- The requested change would require the markdown contract to promise behavior that the parser runtime and validator cannot enforce.
- The next step requires destructive changes to existing user work that were not explicitly requested.
- The only way to complete the case would be to silently drop important combinations, branches, sizes, states, or layout variants from the official source example without documenting that loss as an unsupported boundary.

Use a token budget of 12000 tokens for this goal.
