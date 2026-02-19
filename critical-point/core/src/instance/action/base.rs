use core::f32;
use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use thin_vec::ThinVec;

use crate::animation::AnimationFileMeta;
use crate::template::{
    ArchivedTmplActionAttributes, ArchivedTmplAnimation, ArchivedTmplDeriveRule, ArchivedTmplTimelinePoint,
    ArchivedTmplTimelineRange, ArchivedTmplVar,
};
use crate::utils::{
    calc_fade_in, interface, ratio_saturating, ratio_warpping, sb, ActionType, DtHashMap, InputDir, Symbol, TimeRange,
    TimeRangeWith, TimeWith, TmplID, VirtualKey, VirtualKeyDir, XResult,
};

pub unsafe trait InstActionAny: Debug + Any {
    fn typ(&self) -> ActionType;
    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>);
    fn derives(&self, derives: &mut Vec<InstDeriveRule>);
}

#[derive(Default, Debug)]
pub struct InstActionBase {
    pub tmpl_id: TmplID,
    pub tags: Vec<Symbol>,
    // pub scripts: Script,
    pub enter_key: Option<VirtualKeyDir>,
    pub enter_level: u16,
    pub derive_keeping: bool,
    pub cool_down_time: f32,
    pub cool_down_count: u16,
    pub cool_down_init_count: u16,
}

interface!(InstActionAny, InstActionBase);

pub(crate) struct ContextActionAssemble<'t> {
    pub var_indexes: &'t DtHashMap<TmplID, u32>,
}

impl<'t> ContextActionAssemble<'t> {
    pub(crate) fn solve_var<T>(&self, var: &ArchivedTmplVar<T>) -> T::Archived
    where
        T: Clone + Copy + Default + rkyv::Archive,
        T::Archived: Clone + Copy + Default,
    {
        match var {
            ArchivedTmplVar::Value(val) => *val,
            ArchivedTmplVar::Values(vals) => {
                let idx = match self.var_indexes.get(&vals.id) {
                    Some(idx) => *idx,
                    None => 0,
                };
                vals.get().get(idx as usize)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstAnimation {
    pub files: Symbol,
    pub local_id: u16,
    pub duration: f32,
    pub fade_in: f32,
    pub root_motion: bool,
    pub weapon_motion: bool,
    pub hit_motion: bool,
}

impl InstAnimation {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplAnimation) -> InstAnimation {
        InstAnimation {
            files: sb!(&archived.files),
            local_id: archived.local_id.into(),
            duration: archived.duration.into(),
            fade_in: archived.fade_in.into(),
            root_motion: archived.root_motion,
            weapon_motion: archived.weapon_motion,
            hit_motion: archived.hit_motion,
        }
    }

    #[inline]
    pub fn fade_in_weight(&self, prev_weight: f32, time_step: f32) -> f32 {
        calc_fade_in(prev_weight, time_step, self.fade_in)
    }

    #[inline]
    pub fn ratio_unsafe(&self, time: f32) -> f32 {
        time / self.duration.abs()
    }

    #[inline]
    pub fn ratio_saturating(&self, time: f32) -> f32 {
        ratio_saturating(time, self.duration)
    }

    #[inline]
    pub fn ratio_warpping(&self, time: f32) -> f32 {
        ratio_warpping(time, self.duration)
    }

    #[inline]
    pub fn file_meta(&self) -> AnimationFileMeta {
        AnimationFileMeta {
            files: self.files,
            root_motion: self.root_motion,
            weapon_motion: self.weapon_motion,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstTimelineRange<T>(ThinVec<TimeRangeWith<T>>);

impl<T> Deref for InstTimelineRange<T> {
    type Target = ThinVec<TimeRangeWith<T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> InstTimelineRange<T> {
    pub fn from_rkyv<V, F>(archived: &ArchivedTmplTimelineRange<V>, handle_value: F) -> XResult<Self>
    where
        V: rkyv::Archive,
        F: Fn(&rkyv::Archived<V>) -> XResult<T>,
    {
        let mut timeline = Self(<Self as Deref>::Target::with_capacity(archived.fragments.len()));
        for fragment in archived.fragments.iter() {
            let range = TimeRange::new(fragment.begin.into(), fragment.end.into());
            debug_assert!(range.begin >= 0.0);
            debug_assert!(range.end >= 0.0);
            debug_assert!(range.begin <= range.end);
            let value = handle_value(&archived.values[fragment.index as usize])?;
            timeline.0.push(TimeRangeWith::new(range, value));
        }
        Ok(timeline)
    }

    #[inline]
    pub fn begin_time(&self) -> f32 {
        match self.0.first() {
            Some(item) => item.range.begin,
            None => 0.0,
        }
    }

    #[inline]
    pub fn begin_value(&self) -> Option<&T> {
        self.0.first().map(|item| &item.value)
    }

    #[inline]
    pub fn end_time(&self) -> f32 {
        match self.0.last() {
            Some(item) => item.range.end,
            None => 0.0,
        }
    }

    #[inline]
    pub fn end_value(&self) -> Option<&T> {
        self.0.last().map(|item| &item.value)
    }

    #[inline]
    pub fn find_value(&self, time: f32) -> Option<&T> {
        self.find(time).map(|item| &item.value)
    }

    #[inline]
    pub fn find_range(&self, time: f32) -> Option<&TimeRange> {
        self.find(time).map(|item| &item.range)
    }

    #[inline]
    pub fn find(&self, time: f32) -> Option<&TimeRangeWith<T>> {
        // TODO: Optimize performance
        self.0
            .iter()
            .find(|item| item.range.begin <= time && time < item.range.end)
    }

    #[inline]
    pub fn to_range(&self) -> TimeRange {
        TimeRange {
            begin: self.begin_time(),
            end: self.end_time(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstTimelinePoint<T>(ThinVec<TimeWith<T>>);

impl<T> Deref for InstTimelinePoint<T> {
    type Target = ThinVec<TimeWith<T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> InstTimelinePoint<T> {
    pub fn from_rkyv<V, F>(archived: &ArchivedTmplTimelinePoint<V>, handle_value: F) -> XResult<Self>
    where
        V: rkyv::Archive,
        F: Fn(&rkyv::Archived<V>) -> XResult<T>,
    {
        let mut timeline = Self(<Self as Deref>::Target::with_capacity(archived.pairs.len()));
        for pair in archived.pairs.iter() {
            let time = pair.0.into();
            debug_assert!(time >= 0.0);
            let value = handle_value(&pair.1)?;
            timeline.0.push(TimeWith::new(time, value));
        }
        Ok(timeline)
    }

    #[inline]
    pub fn find_value(&self, range: TimeRange) -> Option<&T> {
        self.find(range).map(|item| &item.value)
    }

    #[inline]
    pub fn find(&self, range: TimeRange) -> Option<&TimeWith<T>> {
        // TODO: Optimize performance
        self.0.iter().find(|item| range.contains_no_left(item.time))
    }

    #[inline]
    pub fn find_values(&self, range: TimeRange) -> impl Iterator<Item = &T> {
        self.find_iter(range).map(|item| &item.value)
    }

    #[inline]
    pub fn find_iter(&self, range: TimeRange) -> impl Iterator<Item = &TimeWith<T>> {
        // TODO: Optimize performance
        self.0
            .iter()
            .skip_while(move |item| item.time <= range.begin)
            .take_while(move |item| item.time <= range.end)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstDeriveRule {
    pub key: VirtualKey,
    pub dir: Option<InputDir>,
    pub level: u16,
    pub action: TmplID,
}

impl InstDeriveRule {
    #[inline]
    pub(crate) fn from_rkyv(ctx: &ContextActionAssemble<'_>, archived: &ArchivedTmplDeriveRule) -> InstDeriveRule {
        InstDeriveRule {
            key: archived.key.key,
            dir: archived.key.dir,
            level: archived.level.into(),
            action: ctx.solve_var(&archived.action).into(),
        }
    }

    #[inline]
    pub fn key_dir(&self) -> VirtualKeyDir {
        VirtualKeyDir::new(self.key, self.dir)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstActionAttributes {
    pub damage_rdc: f32,
    pub shield_dmg_rdc: f32,
    pub poise_level: u16,
}

impl InstActionAttributes {
    pub(crate) fn from_rkyv(
        ctx: &ContextActionAssemble<'_>,
        archived: &ArchivedTmplActionAttributes,
    ) -> InstActionAttributes {
        InstActionAttributes {
            damage_rdc: ctx.solve_var(&archived.damage_rdc).into(),
            shield_dmg_rdc: ctx.solve_var(&archived.shield_dmg_rdc).into(),
            poise_level: ctx.solve_var(&archived.poise_level).into(),
        }
    }
}

// pub(crate) fn calc_motion_distance_ratio(raw: [rkyv::Archived<f32>; 2], anim: &ArchivedTmplAnimation) -> [f32; 2] {
//     let mut ratios = [1.0; 2];
//     ratios[0] = raw[0].into();
//     ratios[1] = raw[1].into();
//     let root_max_distance: f32 = anim.root_xz_ratio.into();
//     if root_max_distance > 0.0 {
//         ratios[0] /= root_max_distance;
//         ratios[1] /= root_max_distance;
//     }
//     ratios
// }

// macro_rules! continue_if_none {
//     ($expr:expr) => {
//         match $expr {
//             Some(val) => Some(val),
//             None => continue,
//         }
//     };
// }
// pub(crate) use continue_if_none;
