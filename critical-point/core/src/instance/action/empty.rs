use std::marker::PhantomData;

use crate::instance::action::base::{InstActionAny, InstActionBase, InstAnimation};
use crate::template::TmplType;
use crate::utils::{extend, TmplID, VirtualKey};

#[repr(C)]
#[derive(Debug)]
pub struct InstActionEmpty {
    pub _base: InstActionBase,
}

extend!(InstActionEmpty, InstActionBase);

unsafe impl InstActionAny for InstActionEmpty {
    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::ActionEmpty
    }

    fn animations<'a>(&'a self, animations: &mut Vec<&'a InstAnimation>) {
        self.animations().for_each(|anime| animations.push(anime));
    }

    fn derives(&self, _derives: &mut Vec<(VirtualKey, TmplID)>) {}
}

impl InstActionEmpty {
    pub fn new() -> InstActionEmpty {
        InstActionEmpty {
            _base: InstActionBase::default(),
        }
    }

    #[inline]
    pub fn animations(&self) -> InstActionEmptyIter<'_> {
        InstActionEmptyIter(PhantomData::default())
    }
}

pub struct InstActionEmptyIter<'t>(PhantomData<&'t ()>);

impl<'t> Iterator for InstActionEmptyIter<'t> {
    type Item = &'t InstAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
