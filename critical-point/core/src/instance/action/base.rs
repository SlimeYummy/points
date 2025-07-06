use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;

use crate::template::{
    ArchivedTmplActionAttributes, ArchivedTmplAnimation, ArchivedTmplDeriveRule, ArchivedTmplTimeline, ArchivedTmplVar,
    TmplHashMap, TmplType,
};
use crate::utils::{
    interface, ratio_saturating, ratio_warpping, sb, Symbol, TimeRange, TimeRangeWith, TmplID, VirtualDir, VirtualKey,
    VirtualKeyDir,
};

pub unsafe trait InstActionAny: Debug + Any {
    fn typ(&self) -> TmplType;
    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>);
    fn derives(&self, derives: &mut Vec<(VirtualKey, TmplID)>);
}

#[derive(Default, Debug)]
pub struct InstActionBase {
    pub tmpl_id: TmplID,
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
    pub var_indexes: &'t TmplHashMap<u32>,
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
    pub duration: f32,
    pub fade_in: f32,
    pub root_motion: bool,
    pub root_max_distance: f32,
}

impl InstAnimation {
    #[inline]
    pub fn from_rkyv(archived: &ArchivedTmplAnimation) -> InstAnimation {
        InstAnimation {
            files: sb!(&archived.files),
            duration: archived.duration.into(),
            fade_in: archived.fade_in.into(),
            root_motion: archived.root_motion,
            root_max_distance: archived.root_max_distance.into(),
        }
    }

    #[inline]
    pub fn fade_in_weight(&self, time: f32) -> f32 {
        ratio_saturating!(time, self.fade_in)
    }

    #[inline]
    pub fn ratio_saturating(&self, time: f32) -> f32 {
        ratio_saturating!(time, self.duration)
    }

    #[inline]
    pub fn ratio_warpping(&self, time: f32) -> f32 {
        ratio_warpping!(time, self.duration)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstTimeline<T>(Vec<TimeRangeWith<T>>);

impl<T> Deref for InstTimeline<T> {
    type Target = Vec<TimeRangeWith<T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> InstTimeline<T> {
    pub fn from_rkyv<V, F>(archived: &ArchivedTmplTimeline<V>, handle_value: F) -> InstTimeline<T>
    where
        V: rkyv::Archive,
        F: Fn(&rkyv::Archived<V>) -> T,
    {
        let mut timeline = InstTimeline(Vec::with_capacity(archived.fragments.len()));
        for fragment in archived.fragments.iter() {
            let range = TimeRange::new(fragment.begin.into(), fragment.end.into());
            let value = handle_value(&archived.values[fragment.index as usize]);
            timeline.0.push(TimeRangeWith::new(range, value));
        }
        timeline
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

    pub fn end_value(&self) -> Option<&T> {
        self.0.last().map(|item| &item.value)
    }

    #[inline]
    pub fn value_by_time(&self, time: f32) -> Option<&T> {
        self.element_by_time(time).map(|item| &item.value)
    }

    #[inline]
    pub fn range_by_time(&self, time: f32) -> Option<&TimeRange> {
        self.element_by_time(time).map(|item| &item.range)
    }

    #[inline]
    pub fn element_by_time(&self, time: f32) -> Option<&TimeRangeWith<T>> {
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
pub struct InstDeriveRule {
    pub key: VirtualKey,
    pub dir: Option<VirtualDir>,
    pub action: TmplID,
}

impl InstDeriveRule {
    #[inline]
    pub(crate) fn from_rkyv(ctx: &ContextActionAssemble<'_>, archived: &ArchivedTmplDeriveRule) -> InstDeriveRule {
        InstDeriveRule {
            key: archived.key,
            dir: match archived.dir {
                rkyv::option::ArchivedOption::Some(dir) => Some(dir),
                rkyv::option::ArchivedOption::None => None,
            },
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

pub(crate) fn calc_motion_distance_ratio(raw: [rkyv::Archived<f32>; 2], anim: &ArchivedTmplAnimation) -> [f32; 2] {
    let mut ratios = [1.0; 2];
    ratios[0] = raw[0].into();
    ratios[1] = raw[1].into();
    let root_max_distance: f32 = anim.root_max_distance.into();
    if root_max_distance > 0.0 {
        ratios[0] /= root_max_distance;
        ratios[1] /= root_max_distance;
    }
    ratios
}

macro_rules! continue_if_none {
    ($expr:expr) => {
        match $expr {
            Some(val) => Some(val),
            None => continue,
        }
    };
}
pub(crate) use continue_if_none;
