///
/// Delete this file (they are just some simple tests).
///
/// You can also regard it as an example custom module, write any of your own code like this.
///
use critical_point_wasm_types::{Symbol, TmplID};

///
/// Call this function in turning-point like:
/// ```
/// let a = 1;
/// let b = 2;
/// your_code::your_func(a, b);
/// ```
#[allow(dead_code)]
pub fn your_func(a: i32, b: i32) -> i32 {
    a + b
}

#[unsafe(no_mangle)]
pub extern "C" fn test_tmpl_id_api() {
    let tid = TmplID::new("Character.Aaa^01").unwrap();
    let s = tid.to_string();
    assert_eq!("Character.Aaa^01", s);
}

#[unsafe(no_mangle)]
pub extern "C" fn test_symbol_api() {
    let sym = Symbol::new("test_symbol").unwrap();
    let s = sym.to_string();
    assert_eq!("test_symbol", s);
    assert_eq!(11, sym.len());
}
