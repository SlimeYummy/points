use critical_point_macros::{wasm_enum, wasm_impl, wasm_struct};
use lasso::{Capacity, MiniSpur, Rodeo, RodeoReader};
use regex::Regex;
use rustc_hash::FxBuildHasher;
use std::hint::{likely, unlikely};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt, fs, mem, slice, str, u64};

use crate::utils::{XError, XResult, rkyv_self, xerr, xfromf, xpos, xres, xresf};

const MAX_KEY_LEN: usize = 48;
const MAX_KEY_COUNT: usize = (u16::MAX - 1) as usize;

static mut KEY_CACHE: Option<TmplKeyCache> = None;

#[inline(always)]
fn key_cache() -> Option<&'static TmplKeyCache> {
    unsafe { &*(&raw const KEY_CACHE) }.as_ref()
}

#[allow(dead_code)]
pub(crate) unsafe fn init_ids_static<P: AsRef<Path>>(path: P, force_reinit: bool) -> XResult<()> {
    unsafe {
        #[allow(static_mut_refs)]
        if KEY_CACHE.is_none() || force_reinit {
            KEY_CACHE = Some(TmplKeyCache::from_file(path)?);
        }
        Ok(())
    }
}

#[cfg(test)]
#[ctor::ctor]
fn test_init_ids_static() {
    use crate::consts::TEST_TMPL_PATH;

    unsafe { init_ids_static(TEST_TMPL_PATH, false).unwrap() };
}

#[cfg(feature = "for-turning-point")]
pub fn init_ids<P: AsRef<Path>>(path: P) {
    unsafe { init_ids_static(path, false).unwrap() };
}

pub struct TmplKeyCache {
    cache: RodeoReader<MiniSpur, FxBuildHasher>,
    memory_usage: usize,
    regex: Regex,
}

impl TmplKeyCache {
    pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> XResult<TmplKeyCache> {
        use rkyv::Archived;
        use rkyv::rancor::Failure;

        let path = PathBuf::from(path.as_ref());
        let rkyv_path = path.join("key.rkyv");
        let json_path = path.join("key.json");

        if fs::exists(&rkyv_path).unwrap_or(false) {
            let buf = fs::read(&rkyv_path).map_err(xfromf!("rkyv_path={:?}", rkyv_path))?;
            let strings = rkyv::access::<Archived<Vec<String>>, Failure>(&buf).map_err(|_| xerr!(Rkyv))?;
            return TmplKeyCache::from_strings(strings);
        }

        if fs::exists(&json_path).unwrap_or(false) {
            let buf = fs::read(&json_path).map_err(xfromf!("json_path={:?}", json_path))?;
            let strings: Vec<&str> =
                serde_json::from_slice::<Vec<&str>>(&buf).map_err(xfromf!("json_path={:?}", json_path))?;
            return TmplKeyCache::from_strings(&strings);
        }

        xresf!(AssetNotFound; "path={:?}", &path)
    }

    pub(crate) fn from_strings<S: AsRef<str>>(strings: &[S]) -> XResult<TmplKeyCache> {
        if strings.len() > MAX_KEY_COUNT {
            return xresf!(UninitedTmplID; "strings.len={}", strings.len());
        }

        let builder = FxBuildHasher::default();
        let mut cache = Rodeo::with_capacity_and_hasher(Capacity::for_strings(strings.len()), builder);
        let memory_usage = cache.current_memory_usage();

        for (idx, string) in strings.iter().enumerate() {
            let string = string.as_ref();
            if string.is_empty() {
                continue;
            }
            if string.len() > MAX_KEY_LEN {
                return xresf!(UninitedTmplID; "idx={}, string.len={}", idx, string.len());
            }
            cache.get_or_intern(string);
        }

        let regex =
            Regex::new(r"^(\#|\w+)\.([\w\-\_]+)(?:\.([\w\-\_]+))?(?:\.([\w\-\_]+))?(?:\^([0-9A-Z]{1,3}))?$").unwrap();

        Ok(TmplKeyCache {
            cache: cache.into_reader(),
            memory_usage,
            regex,
        })
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }

    pub fn find_id(&self, string: &str) -> XResult<u16> {
        if string.is_empty() {
            return Ok(0);
        }
        if string.len() > MAX_KEY_LEN {
            return xres!(InvalidTmplID; "too long");
        }

        match self.cache.get(string) {
            Some(spur) => Ok(unsafe { mem::transmute::<MiniSpur, u16>(spur) }),
            None => xres!(InvalidTmplID; "no key"),
        }
    }

    pub fn find_str(&self, idx: u16) -> XResult<&str> {
        if unlikely(idx == 0) {
            return xres!(InvalidTmplID; "zero");
        }
        let spur = unsafe { mem::transmute::<u16, MiniSpur>(idx) };
        match self.cache.try_resolve(&spur) {
            Some(string) => Ok(string),
            None => xres!(InvalidTmplID; "no key"),
        }
    }

    pub fn find_str_or<'t>(&'t self, idx: u16, def: &'t str) -> &'t str {
        if idx == 0 {
            return "";
        }
        let spur = unsafe { mem::transmute::<u16, MiniSpur>(idx) };
        match self.cache.try_resolve(&spur) {
            Some(string) => string,
            None => def,
        }
    }
}

//
// Template prefix
//

#[wasm_enum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TmplPrefix {
    Invalid = 0,
    Var,
    Character,
    CharacterNpc,
    Style,
    Equipment,
    Entry,
    Perk,
    AccessoryPool,
    Accessory,
    Jewel,
    Action,
    NpcAction,
    AiBrain,
    AiTask,
    Zone,
}

rkyv_self!(TmplPrefix);

#[wasm_impl]
impl FromStr for TmplPrefix {
    type Err = XError;

    fn from_str(s: &str) -> XResult<TmplPrefix> {
        let prefix = match s {
            "#" => TmplPrefix::Var,
            "Character" => TmplPrefix::Character,
            "CharacterNpc" => TmplPrefix::CharacterNpc,
            "Style" => TmplPrefix::Style,
            "Equipment" => TmplPrefix::Equipment,
            "Entry" => TmplPrefix::Entry,
            "Perk" => TmplPrefix::Perk,
            "AccessoryPool" => TmplPrefix::AccessoryPool,
            "Accessory" => TmplPrefix::Accessory,
            "Jewel" => TmplPrefix::Jewel,
            "Action" => TmplPrefix::Action,
            "NpcAction" => TmplPrefix::NpcAction,
            "AiBrain" => TmplPrefix::AiBrain,
            "AiTask" => TmplPrefix::AiTask,
            "Zone" => TmplPrefix::Zone,
            _ => return xres!(InvalidTmplID; "prefix"),
        };
        Ok(prefix)
    }
}

// #[wasm_impl]
// impl TryFrom<u8> for TmplPrefix {
//     type Error = XError;

//     fn try_from(value: u8) -> XResult<TmplPrefix> {
//         let prefix = match value {
//             1 => TmplPrefix::Var,
//             2 => TmplPrefix::Character,
//             3 => TmplPrefix::CharacterNpc,
//             4 => TmplPrefix::Style,
//             5 => TmplPrefix::Equipment,
//             6 => TmplPrefix::Entry,
//             7 => TmplPrefix::Perk,
//             8 => TmplPrefix::AccessoryPool,
//             9 => TmplPrefix::Accessory,
//             10 => TmplPrefix::Jewel,
//             11 => TmplPrefix::Action,
//             12 => TmplPrefix::NpcAction,
//             13 => TmplPrefix::AiBrain,
//             14 => TmplPrefix::AiTask,
//             15 => TmplPrefix::Zone,
//             _ => return xres!(InvalidTmplID, "prefix u8"),
//         };
//         Ok(prefix)
//     }
// }

#[wasm_impl]
impl AsRef<str> for TmplPrefix {
    fn as_ref(&self) -> &str {
        match self {
            TmplPrefix::Var => "#",
            TmplPrefix::Character => "Character",
            TmplPrefix::CharacterNpc => "CharacterNpc",
            TmplPrefix::Style => "Style",
            TmplPrefix::Equipment => "Equipment",
            TmplPrefix::Entry => "Entry",
            TmplPrefix::Perk => "Perk",
            TmplPrefix::AccessoryPool => "AccessoryPool",
            TmplPrefix::Accessory => "Accessory",
            TmplPrefix::Jewel => "Jewel",
            TmplPrefix::Action => "Action",
            TmplPrefix::NpcAction => "NpcAction",
            TmplPrefix::AiBrain => "AiBrain",
            TmplPrefix::AiTask => "AiTask",
            TmplPrefix::Zone => "Zone",
            TmplPrefix::Invalid => "?",
        }
    }
}

#[wasm_impl]
impl fmt::Display for TmplPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

//
// Template ID
//

#[wasm_struct(12, 4)]
#[repr(align(4))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TmplID {
    pub prefix: TmplPrefix,
    pub keys: [u16; 3],
    pub suffix: u16,
    pub package: u16,
}

// #[wasm_struct]
// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
// pub struct TmplID(u64);

rkyv_self!(TmplID);

#[wasm_impl]
impl Default for TmplID {
    #[inline]
    fn default() -> Self {
        TmplID::INVALID
    }
}

#[wasm_impl]
impl TmplID {
    pub const INVALID: TmplID = TmplID {
        prefix: TmplPrefix::Invalid,
        keys: [0; 3],
        suffix: 0,
        package: 0,
    };

    #[inline(always)]
    pub fn is_invalid(&self) -> bool {
        *self == Self::INVALID
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self != Self::INVALID
    }

    #[inline]
    pub fn new3(prefix: TmplPrefix, keys: [u16; 3], suffix: u16) -> Self {
        Self {
            prefix,
            keys,
            suffix,
            package: 0,
        }
    }

    #[inline]
    pub fn new4(prefix: TmplPrefix, keys: [u16; 3], suffix: u16, package: u16) -> Self {
        Self {
            prefix,
            keys,
            suffix,
            package,
        }
    }
}

impl TmplID {
    // Parse string like: Class.Key1.Key2^1A
    #[inline]
    pub fn new(s: &str) -> XResult<TmplID> {
        if likely(key_cache().is_some()) {
            let cache = unsafe { key_cache().unwrap_unchecked() };
            Self::new_with(s, cache)
        }
        else {
            xres!(UninitedTmplID; "uninitialized")
        }
    }

    fn new_with(s: &str, cache: &TmplKeyCache) -> XResult<TmplID> {
        let segs = match Self::split_str(s, cache) {
            Some(segs) => segs,
            None if s == "" => return Ok(TmplID::default()),
            None => return xres!(InvalidTmplID; "invalid string"),
        };
        let prefix = TmplPrefix::from_str(segs[0])?;
        let key1 = cache.find_id(segs[1])?;
        let key2 = cache.find_id(segs[2])?;
        let key3 = cache.find_id(segs[3])?;
        let suffix = Self::parse_suffix(segs[4]).ok_or_else(|| xerr!(InvalidTmplID; "invalid suffix"))?;
        Ok(TmplID {
            prefix,
            keys: [key1, key2, key3],
            suffix,
            package: 0,
        })
    }

    fn split_str<'t>(s: &'t str, cache: &TmplKeyCache) -> Option<[&'t str; 5]> {
        let caps = cache.regex.captures(s)?;
        Some([
            caps.get(1)?.as_str(),
            caps.get(2)?.as_str(),
            caps.get(3).map(|m| m.as_str()).unwrap_or(""),
            caps.get(4).map(|m| m.as_str()).unwrap_or(""),
            caps.get(5).map(|m| m.as_str()).unwrap_or(""),
        ])
    }

    fn parse_suffix(s: &str) -> Option<u16> {
        // bytes[0] * 37 * 37
        // bytes[1] * 37
        // bytes[2] * 0

        let bytes = s.as_bytes();
        if bytes.len() > 3 {
            return None;
        }

        let mut num = 0;
        for b in bytes {
            num = num * 37
                + match b {
                    b'0'..=b'9' => (b - b'0') as u16 + 1,
                    b'A'..=b'Z' => (b - b'A') as u16 + 11,
                    _ => return None,
                };
        }
        Some(num)
    }

    #[inline]
    pub fn to_string(&self) -> String {
        if likely(key_cache().is_some()) {
            let cache = unsafe { key_cache().unwrap_unchecked() };
            self.to_string_with(cache)
        }
        else {
            "Uninitialized.?".to_string()
        }
    }

    fn to_string_with(&self, cache: &TmplKeyCache) -> String {
        if self.is_invalid() {
            return "Invalid.?".to_string();
        }

        let prefix = self.prefix;
        let key0 = cache.find_str_or(self.keys[0], "?");
        let key1 = cache.find_str_or(self.keys[1], "?");
        let key2 = cache.find_str_or(self.keys[2], "?");
        let mut buf = [0u8; 3];
        let suffix = Self::encode_suffix(&mut buf, self.suffix, "?");

        debug_assert!(self.keys[0] != 0);
        if self.keys[1] == 0 {
            debug_assert!(self.keys[2] == 0);
            match self.suffix == 0 {
                true => format!("{}.{}", prefix, key0),
                false => format!("{}.{}^{}", prefix, key0, suffix),
            }
        }
        else if self.keys[2] == 0 {
            match self.suffix == 0 {
                true => format!("{}.{}.{}", prefix, key0, key1),
                false => format!("{}.{}.{}^{}", prefix, key0, key1, suffix),
            }
        }
        else {
            match self.suffix == 0 {
                true => format!("{}.{}.{}.{}", prefix, key0, key1, key2),
                false => format!("{}.{}.{}.{}^{}", prefix, key0, key1, key2, suffix),
            }
        }
    }

    #[inline]
    pub fn make_func_name(&self, func: &str) -> XResult<String> {
        if likely(key_cache().is_some()) {
            let cache = unsafe { key_cache().unwrap_unchecked() };
            self.make_func_name_with(cache, func)
        }
        else {
            xres!(UninitedTmplID; "uninitialized")
        }
    }

    fn make_func_name_with(&self, cache: &TmplKeyCache, func: &str) -> XResult<String> {
        if self.is_invalid() {
            return xres!(InvalidTmplID; "invalid id");
        }

        let prefix = self.prefix;
        let mut buf = [0u8; 3];
        let mut suffix = "";
        if self.suffix != 0 {
            suffix = Self::encode_suffix(&mut buf, self.suffix, "");
            if suffix == "" {
                return xres!(InvalidTmplID; "invalid suffix");
            }
        }

        let key0 = cache.find_str(self.keys[0]).map_err(|e| e.set_pos(xpos!("key0")))?;
        let name = if self.keys[1] == 0 {
            debug_assert!(self.keys[2] == 0);
            match suffix {
                "" => format!("{}_{}__{}", prefix, key0, func),
                _ => format!("{}_{}_{}__{}", prefix, key0, suffix, func),
            }
        }
        else {
            let key1 = cache.find_str(self.keys[1]).map_err(|e| e.set_pos(xpos!("key1")))?;
            if self.keys[2] == 0 {
                match suffix {
                    "" => format!("{}_{}_{}__{}", prefix, key0, key1, func),
                    _ => format!("{}_{}_{}_{}__{}", prefix, key0, key1, suffix, func),
                }
            }
            else {
                let key2 = cache.find_str(self.keys[2]).map_err(|e| e.set_pos(xpos!("key2")))?;
                match suffix {
                    "" => format!("{}_{}_{}_{}__{}", prefix, key0, key1, key2, func),
                    _ => format!("{}_{}_{}_{}_{}__{}", prefix, key0, key1, key2, suffix, func),
                }
            }
        };
        Ok(name)
    }

    fn encode_suffix<'t>(buf: &mut [u8; 3], n: u16, def: &'t str) -> &'t str {
        if n == 0 {
            return "";
        }
        if n > 37 * 37 * 37 {
            return def; // Invalid suffix
        }

        let mut num = n;
        let mut len = 0;

        while num > 0 {
            let v = num % 37;
            if unlikely(v == 0) {
                return def; // Invalid suffix
            }
            else if v <= 10 {
                buf[len] = b'0' + (v - 1) as u8
            }
            else if v <= 36 {
                buf[len] = b'A' + (v - 11) as u8
            }
            num /= 37;
            len += 1;
        }
        buf[0..len].reverse();
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(buf.as_ptr(), len)) }
    }
}

#[wasm_impl]
impl FromStr for TmplID {
    type Err = XError;

    #[inline]
    fn from_str(s: &str) -> XResult<TmplID> {
        TmplID::new(s)
    }
}

#[wasm_impl]
impl TryFrom<&str> for TmplID {
    type Error = XError;

    #[inline]
    fn try_from(s: &str) -> XResult<TmplID> {
        TmplID::new(s)
    }
}

#[wasm_impl]
impl TryFrom<&String> for TmplID {
    type Error = XError;

    #[inline]
    fn try_from(s: &String) -> XResult<TmplID> {
        TmplID::new(s)
    }
}

#[wasm_impl]
impl TryFrom<String> for TmplID {
    type Error = XError;

    #[inline]
    fn try_from(s: String) -> XResult<TmplID> {
        TmplID::new(&s)
    }
}

#[wasm_impl]
impl From<TmplID> for String {
    #[inline]
    fn from(id: TmplID) -> String {
        id.to_string()
    }
}

impl TryFrom<&rkyv::string::ArchivedString> for TmplID {
    type Error = XError;

    #[inline]
    fn try_from(s: &rkyv::string::ArchivedString) -> XResult<TmplID> {
        TmplID::new(s.as_str())
    }
}

#[wasm_impl]
impl fmt::Debug for TmplID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[wasm_impl]
impl fmt::Display for TmplID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

const _: () = {
    use serde::de::{Deserialize, Deserializer, Error};
    use serde::ser::{Serialize, Serializer};

    impl Serialize for TmplID {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_string())
        }
    }

    impl<'de> Deserialize<'de> for TmplID {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<TmplID, D::Error> {
            let s: Option<&str> = Deserialize::deserialize(deserializer)?;
            match s {
                None => Ok(TmplID::default()),
                Some(s) => match TmplID::new(s) {
                    Ok(id) => Ok(id),
                    Err(err) => Err(D::Error::custom(err.to_string())),
                },
            }
        }
    }
};

#[macro_export]
macro_rules! id {
    ($string:expr) => {
        $crate::utils::TmplID::try_from($string).unwrap()
    };
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        $crate::utils::TmplID::new(&res).unwrap()
    }}
}
pub use id;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::*;
    use TmplPrefix::*;

    #[test]
    fn test_tmpl_key_cache_json() {
        let test_dir = prepare_tmp_dir("tmpl-key-cache-json");
        assert!(TmplKeyCache::from_file(&test_dir).is_err());

        // Test invalid key length
        let json_dir = test_dir.join("key.json");
        write_json(&json_dir, &[&"X".repeat(MAX_KEY_LEN + 1)]);
        assert!(TmplKeyCache::from_file(&test_dir).is_err());

        // Test common keys
        write_json(&json_dir, &["A", "BB", "CCC", "DDDDDDDDDD"]);
        let cache = TmplKeyCache::from_file(&test_dir).unwrap();
        assert_eq!(cache.find_id("A").unwrap(), 1);
        assert_eq!(cache.find_id("BB").unwrap(), 2);
        assert_eq!(cache.find_id("CCC").unwrap(), 3);
        assert_eq!(cache.find_id("DDDDDDDDDD").unwrap(), 4);
        assert_eq!(cache.find_id("").unwrap(), 0);
        assert!(cache.find_id("Z").is_err());
        assert_eq!(cache.find_str(1).unwrap(), "A");
        assert_eq!(cache.find_str(2).unwrap(), "BB");
        assert_eq!(cache.find_str(3).unwrap(), "CCC");
        assert_eq!(cache.find_str(4).unwrap(), "DDDDDDDDDD");
        assert!(cache.find_str(0).is_err());
        assert!(cache.find_str(5).is_err());

        // Test hash conflict
        write_json(&json_dir, &(0..20).map(|n| n.to_string()).collect::<Vec<_>>());
        let cache = TmplKeyCache::from_file(&test_dir).unwrap();
        for n in 0..20 {
            assert_eq!(cache.find_id(&n.to_string()).unwrap(), n + 1);
            assert_eq!(cache.find_str(n + 1).unwrap(), n.to_string());
        }
    }

    #[test]
    fn test_tmpl_key_cache_rkyv() {
        let test_dir = prepare_tmp_dir("tmpl-key-cache-rkyv");
        assert!(TmplKeyCache::from_file(&test_dir).is_err());

        // Test invalid key length
        let rkyv_dir = test_dir.join("key.rkyv");
        write_rkyv(&rkyv_dir, &vec!["X".repeat(MAX_KEY_LEN + 1)]);
        assert!(TmplKeyCache::from_file(&test_dir).is_err());

        // Test common keys
        write_rkyv(&rkyv_dir, &vec![
            "A".to_string(),
            "BB".to_string(),
            "CCC".to_string(),
            "DDDDDDDDDD".to_string(),
        ]);
        let cache = TmplKeyCache::from_file(&test_dir).unwrap();
        assert_eq!(cache.find_id("A").unwrap(), 1);
        assert_eq!(cache.find_id("BB").unwrap(), 2);
        assert_eq!(cache.find_id("CCC").unwrap(), 3);
        assert_eq!(cache.find_id("DDDDDDDDDD").unwrap(), 4);
        assert_eq!(cache.find_id("").unwrap(), 0);
        assert!(cache.find_id("Z").is_err());
        assert_eq!(cache.find_str(1).unwrap(), "A");
        assert_eq!(cache.find_str(2).unwrap(), "BB");
        assert_eq!(cache.find_str(3).unwrap(), "CCC");
        assert_eq!(cache.find_str(4).unwrap(), "DDDDDDDDDD");
        assert!(cache.find_str(0).is_err());
        assert!(cache.find_str(5).is_err());

        // Test hash conflict
        write_rkyv(&rkyv_dir, &(0..20).map(|n| n.to_string()).collect::<Vec<String>>());
        let cache = TmplKeyCache::from_file(&test_dir).unwrap();
        for n in 0..20 {
            assert_eq!(cache.find_id(&n.to_string()).unwrap(), n + 1);
            assert_eq!(cache.find_str(n + 1).unwrap(), n.to_string());
        }
    }

    #[test]
    fn test_tmpl_id_common() {
        let test_dir = prepare_tmp_dir("tmpl-id-common");
        let json_dir = test_dir.join("key.json");

        let strings = (0..26)
            .map(|n| format!("{0}{1}{1}", (b'A' + n) as char, (b'a' + n) as char))
            .collect::<Vec<String>>();
        write_json(&json_dir, &strings);
        let cache = TmplKeyCache::from_file(&test_dir).unwrap();

        let id1 = TmplID::new_with("Character.Zzz", &cache).unwrap();
        assert_eq!(id1, TmplID::new3(Character, [26, 0, 0], 0));
        assert_eq!(id1.to_string_with(&cache), "Character.Zzz");

        let id2 = TmplID::new_with("Equipment.Aaa^Z", &cache).unwrap();
        assert_eq!(id2, TmplID::new3(Equipment, [1, 0, 0], 36));
        assert_eq!(id2.to_string_with(&cache), "Equipment.Aaa^Z");

        let id3 = TmplID::new_with("Equipment.Aaa^00", &cache).unwrap();
        let suffix = 37 + 1;
        assert_eq!(id3, TmplID::new3(Equipment, [1, 0, 0], suffix));
        assert_eq!(id3.to_string_with(&cache), "Equipment.Aaa^00");

        let id4 = TmplID::new_with("Zone.Hhh.Iii", &cache).unwrap();
        assert_eq!(id4, TmplID::new3(Zone, [8, 9, 0], 0));
        assert_eq!(id4.to_string_with(&cache), "Zone.Hhh.Iii");

        let id5 = TmplID::new_with("Zone.Hhh.Iii^9Z", &cache).unwrap();
        let suffix = (10 * 37) + 36;
        assert_eq!(id5, TmplID::new3(Zone, [8, 9, 0], suffix));
        assert_eq!(id5.to_string_with(&cache), "Zone.Hhh.Iii^9Z");

        let id6 = TmplID::new_with("Character.Xxx.Yyy.Ooo", &cache).unwrap();
        assert_eq!(id6, TmplID::new3(Character, [24, 25, 15], 0));
        assert_eq!(id6.to_string_with(&cache), "Character.Xxx.Yyy.Ooo");

        let id7 = TmplID::new_with("Character.Xxx.Yyy.Ooo^A00", &cache).unwrap();
        let suffix = (11 * 37 * 37) + (1 * 37) + 1;
        assert_eq!(id7, TmplID::new3(Character, [24, 25, 15], suffix));
        assert_eq!(id7.to_string_with(&cache), "Character.Xxx.Yyy.Ooo^A00");

        let id8 = TmplID::new_with("Character.Xxx.Yyy.Ooo^Z90", &cache).unwrap();
        let suffix = (36 * 37 * 37) + (10 * 37) + 1;
        assert_eq!(id8, TmplID::new3(Character, [24, 25, 15], suffix));
        assert_eq!(id8.to_string_with(&cache), "Character.Xxx.Yyy.Ooo^Z90");

        let id9 = TmplID::new_with("", &cache).unwrap();
        assert!(id9.is_invalid());

        assert!(TmplID::new_with("Zzz", &cache).is_err());
        assert!(TmplID::new_with("Character", &cache).is_err());
        assert!(TmplID::new_with("Character.S+", &cache).is_err());
        assert!(TmplID::new_with("Character.Ab", &cache).is_err());
        assert!(TmplID::new_with("Zone.Aaa.S+", &cache).is_err());
        assert!(TmplID::new_with("Zone.Aaa.Ab", &cache).is_err());
        assert!(TmplID::new_with("Zone.Aaa^", &cache).is_err());
        assert!(TmplID::new_with("Zone.Aaa^a", &cache).is_err());
        assert!(TmplID::new_with("Zone.Aaa.Bbb.11", &cache).is_err());
        assert!(TmplID::new_with("Zone.Aaa.Bbb.Ccc.Ddd", &cache).is_err());
        assert!(TmplID::new_with("Zone.Aaa.Bbb.Ccc^CCCC", &cache).is_err());
        assert!(TmplID::new_with("Zone.128.Bbb.Ccc", &cache).is_err());
    }

    #[test]
    fn test_tmpl_id_json_rkyv() {
        use rkyv::rancor::Error;

        let id1 = TmplID::new("Zone.Aaa^0").unwrap();
        let buf = serde_json::to_string(&id1).unwrap();
        let id2 = serde_json::from_str(&buf).unwrap();
        assert_eq!(id1, id2);

        let id3 = TmplID::new("Entry.Xxx.Yyy.Zzz^1F").unwrap();
        let buf = rkyv::to_bytes::<Error>(&id3).unwrap();
        let id4 = unsafe { rkyv::access_unchecked::<TmplID>(&buf) };
        let id5 = rkyv::deserialize::<_, Error>(id4).unwrap();
        assert_eq!(id3, (*id4).into());
        assert_eq!(id3, id5);
    }

    #[test]
    fn test_tmpl_id_make_func_name() {
        let test_dir = prepare_tmp_dir("tmpl-id-make-func-name");
        let json_dir = test_dir.join("key.json");

        let strings = (0..26)
            .map(|n| format!("{0}{1}{1}", (b'A' + n) as char, (b'a' + n) as char))
            .collect::<Vec<String>>();
        write_json(&json_dir, &strings);
        let cache = TmplKeyCache::from_file(&test_dir).unwrap();

        let id1 = TmplID::new_with("Character.Zzz", &cache).unwrap();
        assert_eq!(
            id1.make_func_name_with(&cache, "ai_main").unwrap(),
            "Character_Zzz_ai__main"
        );

        let id2 = TmplID::new_with("Equipment.Aaa^Z", &cache).unwrap();
        assert_eq!(
            id2.make_func_name_with(&cache, "on_equip").unwrap(),
            "Equipment_Aaa_Z__on_equip"
        );

        let id3 = TmplID::new_with("Zone.Hhh.Iii", &cache).unwrap();
        assert_eq!(
            id3.make_func_name_with(&cache, "on_enter").unwrap(),
            "Zone_Hhh_Iii__on_enter"
        );

        let id4 = TmplID::new_with("Zone.Hhh.Iii^9Z", &cache).unwrap();
        assert_eq!(
            id4.make_func_name_with(&cache, "on_exit").unwrap(),
            "Zone_Hhh_Iii_9Z__on_exit"
        );

        let id5 = TmplID::new_with("Character.Xxx.Yyy.Ooo", &cache).unwrap();
        assert_eq!(
            id5.make_func_name_with(&cache, "on_tick").unwrap(),
            "Character_Xxx_Yyy_Ooo__on_tick"
        );

        let id6 = TmplID::new_with("Character.Xxx.Yyy.Ooo^A00", &cache).unwrap();
        assert_eq!(
            id6.make_func_name_with(&cache, "on_hit").unwrap(),
            "Character_Xxx_Yyy_Ooo_A00__on_hit"
        );

        assert!(TmplID::INVALID.make_func_name_with(&cache, "func").is_err());
    }
}
