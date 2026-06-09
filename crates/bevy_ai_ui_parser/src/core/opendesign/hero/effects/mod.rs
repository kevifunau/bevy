mod atmosphere;
mod bands;
mod buttons;
mod controls;
mod focus;
mod meters;
mod panel;
mod stats;

use crate::core::model::BuiNode;

use atmosphere::{soften_crest, soften_image_layer_after, soften_image_layer_before};
use buttons::{soften_equip_slots, soften_skill_buttons};
use controls::{soften_backbutton, soften_title_text};
use focus::{soften_hero_cutout, soften_hero_glow};
use meters::{soften_meters, soften_xp_energy_fills};
use panel::soften_info_panel;
use stats::soften_stat_rows;

pub(super) fn soften_hero_game_ui_effect_fallbacks(root: &mut BuiNode) {
    soften_crest(root);
    soften_image_layer_before(root);
    soften_image_layer_after(root);
    soften_hero_glow(root);
    soften_hero_cutout(root);
    soften_backbutton(root);
    soften_title_text(root);
    soften_info_panel(root);
    soften_stat_rows(root);
    soften_meters(root);
    soften_xp_energy_fills(root);
    soften_skill_buttons(root);
    soften_equip_slots(root);
}
