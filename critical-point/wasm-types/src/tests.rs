use crate::auto_gen::*;
use critical_point_wasm_macros::id;

#[test]
fn test_id_macro() {
    let id1 = id!("Character.Zzz");
    assert!(id1 == TmplID::new3(TmplPrefix::Character, [26, 0, 0], 0));

    let id2 = id!("Equipment.Aaa^Z");
    assert!(id2 == TmplID::new3(TmplPrefix::Equipment, [1, 0, 0], 36));

    let id3 = id!("Equipment.Aaa^00");
    let suffix = 37 + 1;
    assert!(id3 == TmplID::new3(TmplPrefix::Equipment, [1, 0, 0], suffix));

    let id4 = id!("Zone.Hhh.Iii");
    assert!(id4 == TmplID::new3(TmplPrefix::Zone, [8, 9, 0], 0));

    let id5 = id!("Zone.Hhh.Iii^9Z");
    let suffix = (10 * 37) + 36;
    assert!(id5 == TmplID::new3(TmplPrefix::Zone, [8, 9, 0], suffix));

    let id6 = id!("Character.Xxx.Yyy.Ooo");
    assert!(id6 == TmplID::new3(TmplPrefix::Character, [24, 25, 15], 0));

    let id7 = id!("Character.Xxx.Yyy.Ooo^A00");
    let suffix = (11 * 37 * 37) + (1 * 37) + 1;
    assert!(id7 == TmplID::new3(TmplPrefix::Character, [24, 25, 15], suffix));

    let id8 = id!("Character.Xxx.Yyy.Ooo^Z90");
    let suffix = (36 * 37 * 37) + (10 * 37) + 1;
    assert!(id8 == TmplID::new3(TmplPrefix::Character, [24, 25, 15], suffix));

    let id9 = id!("");
    assert!(id9.is_invalid());
}
