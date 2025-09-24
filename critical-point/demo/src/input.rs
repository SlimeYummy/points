use approx::abs_diff_eq;
use cirtical_point_core::ifelse;
use cirtical_point_core::utils::{RawEvent, RawKey};
use core::f32;
use glam::Vec2;
use jolt_physics_rs::debug::{DebugKeyboard, DebugMouse};
use jolt_physics_rs::keys::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterType {
    Melee,
    Magic,
    Shot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum JoltMean {
    Dodge,
    Guard,
    Interact,
    Lock,
    CombExtra,
    Attack1,
    Attack2,
    Attack5,
    Spell,
    Shot1,
    Aim,
    Switch,
    CombSkill1,
    CombSkill2,
    CombSkill3,
    CombSkill4,
    Skill1,
    Skill2,
    Item1,
    Item2,
    Item3,
    Item4,
    Item5,
    Item6,
    Item7,
    Item8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JoltKey {
    M(i32),
    K(i32),
}

const KEYS: &[(JoltMean, JoltKey)] = &[
    (JoltMean::CombExtra, JoltKey::K(DIK_SPACE)),
    (JoltMean::Dodge, JoltKey::K(DIK_LSHIFT)),
    (JoltMean::Guard, JoltKey::K(DIK_LALT)),
    (JoltMean::Interact, JoltKey::K(DIK_F)),
    (JoltMean::Lock, JoltKey::K(DIK_TAB)),
    (JoltMean::Attack1, JoltKey::M(1)),
    (JoltMean::Attack2, JoltKey::M(2)),
    (JoltMean::Attack5, JoltKey::M(3)),
    (JoltMean::Spell, JoltKey::M(3)),
    (JoltMean::Shot1, JoltKey::M(1)),
    (JoltMean::Aim, JoltKey::M(2)),
    (JoltMean::Switch, JoltKey::M(3)),
    (JoltMean::CombSkill1, JoltKey::K(DIK_Q)),
    (JoltMean::CombSkill2, JoltKey::K(DIK_E)),
    (JoltMean::CombSkill3, JoltKey::K(DIK_R)),
    (JoltMean::CombSkill4, JoltKey::K(DIK_C)),
    (JoltMean::Skill1, JoltKey::M(1)),
    (JoltMean::Skill2, JoltKey::M(2)),
    (JoltMean::Item1, JoltKey::K(DIK_1)),
    (JoltMean::Item2, JoltKey::K(DIK_2)),
    (JoltMean::Item3, JoltKey::K(DIK_3)),
    (JoltMean::Item4, JoltKey::K(DIK_4)),
    (JoltMean::Item5, JoltKey::K(DIK_5)),
    (JoltMean::Item6, JoltKey::K(DIK_6)),
    (JoltMean::Item7, JoltKey::K(DIK_7)),
    (JoltMean::Item8, JoltKey::K(DIK_8)),
];

const COMB_EXTRA: u32 = 1 << 3;
const COMB_SKILL1: u32 = 1;
const COMB_SKILL2: u32 = 2;
const COMB_SKILL3: u32 = 3;
const COMB_SKILL4: u32 = 4;
const COMB_SKILL_ANY: u32 = 0x7;

pub struct InputHandler {
    character: CharacterType,
    xkeys_state: Vec<bool>,
    mkeys_state: Vec<bool>,
    move_dir: Vec2,        // last move direction
    view_rads: Vec2,       // last view radius (yaw, pitch)
    combination_keys: u32, // any combination key down
    skilling: bool,        // is skilling
    aiming: bool,          // is aiming
    holding: HashMap<JoltMean, RawKey>,
    derive_holding: HashMap<JoltMean, RawKey>,
    events: Vec<RawEvent>,
}

impl InputHandler {
    pub fn new(character: CharacterType) -> InputHandler {
        InputHandler {
            character,
            xkeys_state: vec![false; KEYS.len()],
            mkeys_state: vec![false; 4],
            move_dir: Vec2::ZERO,
            view_rads: Vec2::new(f32::NAN, f32::NAN),
            combination_keys: 0,
            skilling: false,
            aiming: false,
            holding: HashMap::new(),
            derive_holding: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn take_events(&mut self) -> Vec<RawEvent> {
        return self.events.drain(..).collect();
    }

    pub fn handle(&mut self, mouse: &mut DebugMouse, keyboard: &mut DebugKeyboard, view_rads: Vec2) {
        self.handle_view(view_rads);
        self.handle_move(keyboard);

        for idx in 0..KEYS.len() {
            let (mean, pressed) = match self.handle_key(mouse, keyboard, idx) {
                Some(x) => x,
                None => continue,
            };

            if pressed {
                match mean {
                    JoltMean::CombExtra => self.start_extra(),
                    JoltMean::Dodge => self.start_dodge(mean),
                    JoltMean::Guard => self.start_common(mean, RawKey::Guard),
                    JoltMean::Interact => self.start_common(mean, RawKey::Interact),
                    JoltMean::Lock => self.start_common(mean, RawKey::Lock),
                    JoltMean::Attack1 => self.start_attack(mean, RawKey::Attack1),
                    JoltMean::Attack2 => self.start_attack(mean, RawKey::Attack2),
                    JoltMean::Attack5 => self.start_attack(mean, RawKey::Attack5),
                    JoltMean::Spell => self.start_spell(mean),
                    JoltMean::Shot1 => self.start_shot(mean),
                    JoltMean::Aim => self.start_aim(mean),
                    JoltMean::Switch => self.start_switch(mean),
                    JoltMean::CombSkill1 => self.start_comb_skill(COMB_SKILL1),
                    JoltMean::CombSkill2 => self.start_comb_skill(COMB_SKILL2),
                    JoltMean::CombSkill3 => self.start_comb_skill(COMB_SKILL3),
                    JoltMean::CombSkill4 => self.start_comb_skill(COMB_SKILL4),
                    JoltMean::Skill1 => self.start_skill(mean, true),
                    JoltMean::Skill2 => self.start_skill(mean, false),
                    JoltMean::Item1 => self.start_common(mean, RawKey::Item1),
                    JoltMean::Item2 => self.start_common(mean, RawKey::Item2),
                    JoltMean::Item3 => self.start_common(mean, RawKey::Item3),
                    JoltMean::Item4 => self.start_common(mean, RawKey::Item4),
                    JoltMean::Item5 => self.start_common(mean, RawKey::Item5),
                    JoltMean::Item6 => self.start_common(mean, RawKey::Item6),
                    JoltMean::Item7 => self.start_common(mean, RawKey::Item7),
                    JoltMean::Item8 => self.start_common(mean, RawKey::Item8),
                };
            }
            else {
                match mean {
                    JoltMean::CombExtra => self.cancel_extra(),
                    JoltMean::CombSkill1 => self.cancel_comb_skill(COMB_SKILL1),
                    JoltMean::CombSkill2 => self.cancel_comb_skill(COMB_SKILL2),
                    JoltMean::CombSkill3 => self.cancel_comb_skill(COMB_SKILL3),
                    JoltMean::CombSkill4 => self.cancel_comb_skill(COMB_SKILL4),
                    JoltMean::Skill1 => self.cancel_skill(mean),
                    JoltMean::Skill2 => self.cancel_skill(mean),
                    JoltMean::Aim => self.cancel_aim(mean),
                    _ => self.cancel_common(mean),
                }
            }
        }
    }

    fn handle_view(&mut self, view_rads: Vec2) {
        if abs_diff_eq!(view_rads, self.view_rads) {
            return;
        }
        self.view_rads = view_rads;

        if let Some(last) = self.events.last_mut() {
            if last.key == RawKey::View {
                last.motion = view_rads;
                return;
            }
        }
        self.events.push(RawEvent::new_view(view_rads));
    }

    fn handle_move(&mut self, keyboard: &mut DebugKeyboard) {
        let mut key_events = [None; 4];
        for (idx, key) in [DIK_A, DIK_D, DIK_S, DIK_W].iter().enumerate() {
            let pressed = keyboard.is_key_pressed(*key);
            let prev_state = self.mkeys_state[idx];
            self.mkeys_state[idx] = pressed;
            match (prev_state, pressed) {
                (false, true) => key_events[idx] = Some(true),
                (true, false) => key_events[idx] = Some(false),
                _ => (),
            }
        }

        let prev_move_dir = self.move_dir;
        match key_events[0] {
            Some(true) => self.move_dir.x = -1.0,
            Some(false) if self.move_dir.x == -1.0 => self.move_dir.x = 0.0,
            _ => (),
        }
        match key_events[1] {
            Some(true) => self.move_dir.x = 1.0,
            Some(false) if self.move_dir.x == 1.0 => self.move_dir.x = 0.0,
            _ => (),
        }
        match key_events[2] {
            Some(true) => self.move_dir.y = -1.0,
            Some(false) if self.move_dir.y == -1.0 => self.move_dir.y = 0.0,
            _ => (),
        }
        match key_events[3] {
            Some(true) => self.move_dir.y = 1.0,
            Some(false) if self.move_dir.y == 1.0 => self.move_dir.y = 0.0,
            _ => (),
        }

        if self.move_dir == prev_move_dir {
            return;
        }

        let mut move_dir = self.move_dir;
        if move_dir != Vec2::ZERO {
            move_dir = move_dir.normalize();
        }
        if let Some(last) = self.events.last_mut() {
            if last.key == RawKey::Move {
                last.motion = move_dir;
                return;
            }
        }
        self.events.push(RawEvent::new_move(move_dir));
    }

    fn handle_key(
        &mut self,
        mouse: &mut DebugMouse,
        keyboard: &mut DebugKeyboard,
        idx: usize,
    ) -> Option<(JoltMean, bool)> {
        let (mean, rkey) = KEYS[idx];
        let pressed = match rkey {
            JoltKey::M(1) => mouse.is_left_pressed(),
            JoltKey::M(2) => mouse.is_right_pressed(),
            JoltKey::M(3) => mouse.is_middle_pressed(),
            JoltKey::K(key) => keyboard.is_key_pressed(key),
            _ => return None,
        };
        let prev_state = self.xkeys_state[idx];
        self.xkeys_state[idx] = pressed;
        match (prev_state, pressed) {
            (false, true) => Some((mean, true)),
            (true, false) => Some((mean, false)),
            _ => None,
        }
    }

    fn start_common(&mut self, mean: JoltMean, code: RawKey) {
        self.holding.insert(mean, code);
        self.events.push(RawEvent::new_button(code, true));
    }

    fn cancel_common(&mut self, mean: JoltMean) {
        if let Some(code) = self.holding.remove(&mean) {
            self.events.push(RawEvent::new_button(code, false));
        }
        if let Some(code) = self.derive_holding.remove(&mean) {
            self.events.push(RawEvent::new_button(code, false));
        }
    }

    fn start_extra(&mut self) {
        self.combination_keys |= COMB_EXTRA;
    }

    fn cancel_extra(&mut self) {
        self.combination_keys &= !COMB_EXTRA;
    }

    fn start_dodge(&mut self, mean: JoltMean) {
        let code = ifelse!((self.combination_keys & COMB_EXTRA) != 0, RawKey::Jump, RawKey::Dodge);
        self.holding.insert(mean, code);
        self.events.push(RawEvent::new_button(code, true));
    }

    fn start_attack(&mut self, mean: JoltMean, mut code: RawKey) {
        if self.character != CharacterType::Melee && self.character != CharacterType::Magic {
            return;
        }
        if self.skilling && (self.combination_keys & COMB_SKILL_ANY) != 0 {
            return;
        }

        let mut derive = None;
        if self.character == CharacterType::Melee {
            if code == RawKey::Attack1 {
                code = ifelse!(
                    (self.combination_keys & COMB_EXTRA) != 0,
                    RawKey::Attack3,
                    RawKey::Attack1
                );
                derive = ifelse!((self.combination_keys & COMB_EXTRA) != 0, None, Some(RawKey::Derive1));
            }
            else if code == RawKey::Attack2 {
                code = ifelse!(
                    (self.combination_keys & COMB_EXTRA) != 0,
                    RawKey::Attack4,
                    RawKey::Attack2
                );
                derive = ifelse!((self.combination_keys & COMB_EXTRA) != 0, None, Some(RawKey::Derive2));
            }
            else if code == RawKey::Attack5 {
                code = ifelse!(
                    (self.combination_keys & COMB_EXTRA) != 0,
                    RawKey::Attack6,
                    RawKey::Attack5
                );
                derive = ifelse!((self.combination_keys & COMB_EXTRA) != 0, None, Some(RawKey::Derive3));
            }
            else {
                return;
            }
        }
        else if self.character == CharacterType::Magic {
            if code == RawKey::Attack1 {
                code = ifelse!(
                    (self.combination_keys & COMB_EXTRA) != 0,
                    RawKey::Attack3,
                    RawKey::Attack1
                );
                derive = ifelse!((self.combination_keys & COMB_EXTRA) != 0, None, Some(RawKey::Derive1));
            }
            else if code == RawKey::Attack2 {
                code = ifelse!(
                    (self.combination_keys & COMB_EXTRA) != 0,
                    RawKey::Attack4,
                    RawKey::Attack2
                );
                derive = ifelse!((self.combination_keys & COMB_EXTRA) != 0, None, Some(RawKey::Derive2));
            }
            else {
                return;
            }
        }

        self.holding.insert(mean, code);
        self.events.push(RawEvent::new_button(code, true));

        if let Some(derive) = derive {
            self.derive_holding.insert(mean, derive);
            self.events.push(RawEvent::new_button(derive, true));
        }
    }

    fn start_spell(&mut self, mean: JoltMean) {
        if self.character != CharacterType::Magic {
            return;
        }
        if !self.skilling && (self.combination_keys & COMB_SKILL_ANY) != 0 {
            return;
        }

        self.holding.insert(mean, RawKey::Spell);
        self.events.push(RawEvent::new_button(RawKey::Spell, true));

        self.derive_holding.insert(mean, RawKey::Derive3);
        self.events.push(RawEvent::new_button(RawKey::Derive3, true));
    }

    fn start_shot(&mut self, mean: JoltMean) {
        if self.character != CharacterType::Shot {
            return;
        }
        if !self.skilling && (self.combination_keys & COMB_SKILL_ANY) != 0 {
            return;
        }

        let code = ifelse!((self.combination_keys & COMB_EXTRA) != 0, RawKey::Shot2, RawKey::Shot1);
        self.holding.insert(mean, code);
        self.events.push(RawEvent::new_button(code, true));

        if (self.combination_keys & COMB_EXTRA) != 0 {
            self.derive_holding.insert(mean, RawKey::Derive2);
            self.events.push(RawEvent::new_button(RawKey::Derive2, true));
        }
    }

    fn start_aim(&mut self, mean: JoltMean) {
        if self.character != CharacterType::Shot {
            return;
        }
        if !self.skilling && (self.combination_keys & COMB_SKILL_ANY) != 0 {
            return;
        }

        self.holding.insert(mean, RawKey::Aim);
        self.events.push(RawEvent::new_button(RawKey::Aim, true));
        self.aiming = true;
    }

    fn cancel_aim(&mut self, mean: JoltMean) {
        self.cancel_common(mean);
        self.aiming = false;
    }

    fn start_switch(&mut self, mean: JoltMean) {
        if self.character != CharacterType::Shot {
            return;
        }
        if !self.skilling && (self.combination_keys & COMB_SKILL_ANY) != 0 {
            return;
        }

        self.holding.insert(mean, RawKey::Switch);
        self.events.push(RawEvent::new_button(RawKey::Switch, true));

        self.derive_holding.insert(mean, RawKey::Derive3);
        self.events.push(RawEvent::new_button(RawKey::Derive3, true));
    }

    fn start_comb_skill(&mut self, comb_skill: u32) {
        self.combination_keys &= !COMB_SKILL_ANY;
        self.combination_keys |= comb_skill;

        if self.character == CharacterType::Shot && self.aiming {
            self.start_skill(JoltMean::Attack2, false);
        }
    }

    fn cancel_comb_skill(&mut self, comb_skill: u32) {
        if (self.combination_keys & COMB_SKILL_ANY) == comb_skill {
            self.combination_keys &= !COMB_SKILL_ANY;
        }
    }

    fn start_skill(&mut self, mean: JoltMean, one: bool) {
        if self.skilling {
            return;
        }

        let code = match self.combination_keys & COMB_SKILL_ANY {
            1 => ifelse!(one, RawKey::Skill1, RawKey::Skill5),
            2 => ifelse!(one, RawKey::Skill2, RawKey::Skill6),
            3 => ifelse!(one, RawKey::Skill3, RawKey::Skill7),
            4 => ifelse!(one, RawKey::Skill4, RawKey::Skill8),
            _ => return,
        };
        self.holding.insert(mean, code);
        self.events.push(RawEvent::new_button(code, true));

        self.skilling = true;
    }

    fn cancel_skill(&mut self, mean: JoltMean) {
        if let Some(code) = self.holding.remove(&mean) {
            self.events.push(RawEvent::new_button(code, false));
            self.skilling = false;
        }
    }
}
