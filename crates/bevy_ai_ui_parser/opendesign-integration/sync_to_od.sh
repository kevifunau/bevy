#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    shift
fi

OD_REPO="${1:-}"
if [[ -z "$OD_REPO" ]]; then
    echo "Usage: sync_to_od.sh [--dry-run] <path-to-open-design-repo>"
    echo "Example: sync_to_od.sh /path/to/open-design"
    echo "Example: sync_to_od.sh --dry-run /path/to/open-design"
    exit 1
fi

if [[ ! -d "$OD_REPO" ]]; then
    echo "Error: OD repo directory does not exist: $OD_REPO"
    exit 1
fi

SRC_DIR="$(cd "$(dirname "$0")" && pwd)"

RSYNC_FLAGS="-av --delete"
if $DRY_RUN; then
    RSYNC_FLAGS="-avn --delete"
fi

echo "Syncing from: $SRC_DIR"
echo "Syncing to:   $OD_REPO"
if $DRY_RUN; then
    echo "Mode: DRY RUN (no files will be changed)"
fi
echo ""

# --- Plugin top-level files ---
echo "=== Plugin manifest & top-level docs ==="
rsync $RSYNC_FLAGS \
    --include='open-design.json' \
    --include='SKILL.md' \
    --include='AUTHORING.md' \
    --include='API.md' \
    --include='INTEGRATION.md' \
    --exclude='*' \
    "$SRC_DIR/" "$OD_REPO/plugins/_official/examples/bevy-ui-screen/"

# --- Plugin references ---
echo "=== Plugin references ==="
rsync $RSYNC_FLAGS \
    "$SRC_DIR/references/" "$OD_REPO/plugins/_official/examples/bevy-ui-screen/references/"

# --- Plugin assets ---
echo "=== Plugin assets ==="
rsync $RSYNC_FLAGS \
    "$SRC_DIR/assets/" "$OD_REPO/plugins/_official/examples/bevy-ui-screen/assets/"

# --- 4 sub-skills (artist, programmer, ta, ux-planner) → plugin/skills/ ---
echo "=== Sub-skills → plugin/skills ==="
for SKILL in bevy-ui-artist bevy-ui-programmer bevy-ui-ta bevy-ux-planner; do
    rsync $RSYNC_FLAGS \
        "$SRC_DIR/skills/$SKILL/" "$OD_REPO/plugins/_official/examples/bevy-ui-screen/skills/$SKILL/"
done

# --- Generator skill → OD top-level skills/ ---
echo "=== Generator skill → skills/bevy-ui-generator ==="
rsync $RSYNC_FLAGS \
    "$SRC_DIR/skills/bevy-ui-generator/SKILL.md" "$OD_REPO/skills/bevy-ui-generator/SKILL.md"
rsync $RSYNC_FLAGS \
    "$SRC_DIR/skills/bevy-ui-generator/references/" "$OD_REPO/skills/bevy-ui-generator/references/"

# --- Design system → OD design-systems/ ---
echo "=== Design system ==="
rsync $RSYNC_FLAGS \
    --exclude='comfyui' \
    --exclude='v2-validation' \
    --exclude='API_CONTRACT_UPDATE_NOTES.md' \
    --exclude='V2_ENHANCEMENT_SUMMARY.md' \
    "$SRC_DIR/design-systems/bevy-ui/" "$OD_REPO/design-systems/bevy-ui/"

# --- Design templates → OD design-templates/ ---
echo "=== Design templates ==="
rsync $RSYNC_FLAGS \
    "$SRC_DIR/design-templates/bevy-ui/" "$OD_REPO/design-templates/bevy-ui/"

echo ""
if $DRY_RUN; then
    echo "DRY RUN complete. No files were changed."
    echo "Run without --dry-run to apply changes."
else
    echo "Sync complete."
    echo ""
    echo "Next step: validate in OD repo:"
    echo "  cd $OD_REPO && pnpm guard && pnpm typecheck"
fi
