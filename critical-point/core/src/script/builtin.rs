use std::collections::HashMap;

use crate::script::command::{CmdOpt, CmdType};
use crate::utils::Num;

pub fn consts() -> HashMap<String, Num> {
    HashMap::from([
        ("math.PI".into(), core::f64::consts::PI),
        ("math.E".into(), core::f64::consts::E),
        ("math.TAU".into(), core::f64::consts::TAU),
        ("math.MAX".into(), core::f64::MAX),
        ("math.MIN".into(), core::f64::MIN),
        ("math.POS_INF".into(), core::f64::INFINITY),
        ("math.NEG_INF".into(), core::f64::NEG_INFINITY),
    ])
}

pub fn functions() -> HashMap<String, (CmdOpt, Vec<CmdType>)> {
    use CmdOpt::*;
    use CmdType::*;
    HashMap::from([
        ("G.init".into(), (XInit, vec![Str, Num])),
        ("G.get".into(), (XGet, vec![Str])),
        ("G.set".into(), (XSet, vec![Str, Num])),
        ("G.has".into(), (XHas, vec![Str])),
        ("G.del".into(), (XDel, vec![Str])),
        ("math.is_nan".into(), (Add, vec![Num])),
        ("math.is_inf".into(), (Add, vec![Num])),
        ("math.abs".into(), (Abs, vec![Num])),
        ("math.min".into(), (Min, vec![Num, Num])),
        ("math.max".into(), (Max, vec![Num, Num])),
        ("math.floor".into(), (Floor, vec![Num])),
        ("math.ceil".into(), (Ceil, vec![Num])),
        ("math.round".into(), (Round, vec![Num])),
        ("math.clamp".into(), (Clamp, vec![Num, Num, Num])),
        ("math.saturate".into(), (Saturate, vec![Num])),
        ("math.lerp".into(), (Lerp, vec![Num, Num, Num])),
        ("math.sqrt".into(), (Sqrt, vec![Num])),
        ("math.degrees".into(), (Degrees, vec![Num])),
        ("math.radians".into(), (Radians, vec![Num])),
        ("math.sin".into(), (Sin, vec![Num])),
        ("math.cos".into(), (Cos, vec![Num])),
        ("math.tan".into(), (Tan, vec![Num])),
    ])
}
