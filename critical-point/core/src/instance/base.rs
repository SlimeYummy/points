use crate::template::TmplDatabase;

pub struct ContextAssemble<'t> {
    pub tmpl_db: &'t TmplDatabase,
}

impl<'t> ContextAssemble<'t> {
    pub fn new(tmpl_db: &'t TmplDatabase) -> ContextAssemble<'t> {
        ContextAssemble { tmpl_db }
    }
}
