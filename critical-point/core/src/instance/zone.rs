use crate::instance::base::ContextAssemble;
use crate::parameter::ParamZone;
use crate::template::TmplZone;
use crate::utils::{TmplID, XResult};

#[inline]
pub fn assemble_zone(ctx: &mut ContextAssemble, param: &ParamZone) -> XResult<InstZone> {
    InstZone::new(ctx, param)
}

#[derive(Debug, Default)]
pub struct InstZone {
    pub tmpl_zone: TmplID,
}

impl InstZone {
    pub fn new(ctx: &mut ContextAssemble<'_>, param: &ParamZone) -> XResult<InstZone> {
        let _ = ctx.tmpl_db.find_as::<TmplZone>(param.zone)?;
        Ok(InstZone {
            tmpl_zone: param.zone.clone(),
        })
    }
}
