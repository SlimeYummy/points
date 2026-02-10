use regex::Regex;
use rustc_hash::FxHasher;
use std::alloc::Layout;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{alloc, fmt, fs, mem, ptr, slice, str, u64};

use crate::utils::{rkyv_self, xerr, xfromf, xres, xresf, XError, XResult};

const MAX_SYMBOL_LEN: usize = 64;
const MAX_SYMBOL_COUNT: usize = (u16::MAX - 1) as usize;
const SYMBOL_EMPTY: u16 = u16::MAX;
const SUFFIX_EMPTY: u16 = 0x3FF;

//
// Template symbol cache
//

static mut SYMBOL_CACHE: TmplSymbolCache = TmplSymbolCache {
    buf: ptr::null_mut(),
    buf_size: 0,
    list: Vec::new(),
    map: Vec::new(),
    regex: None,
};

#[inline(always)]
fn symbol_cache() -> &'static TmplSymbolCache {
    unsafe { &*(&raw const SYMBOL_CACHE) }
}

#[allow(dead_code)]
pub(crate) unsafe fn init_id_static<P: AsRef<Path>>(path: P, force_reinit: bool) -> XResult<()> {
    #[allow(static_mut_refs)]
    if SYMBOL_CACHE.is_empty() || force_reinit {
        SYMBOL_CACHE = TmplSymbolCache::from_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
#[ctor::ctor]
fn test_init_id_static() {
    use crate::consts::TEST_TMPL_PATH;

    unsafe { init_id_static(TEST_TMPL_PATH, false).unwrap() };
}

struct TmplSymbol {
    hash: u32,
    next: u32,
    idx: u16,
    len: u8,
    chars: [u8; 0],
}

const NODE_SIZE: usize = mem::size_of::<TmplSymbol>();
const NODE_ALIGN: usize = mem::align_of::<TmplSymbol>();
const NODE_MASK: usize = NODE_ALIGN - 1;

impl TmplSymbol {
    #[inline]
    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(self.chars.as_ptr(), self.len as usize)) }
    }
}

pub struct TmplSymbolCache {
    buf: *mut u8,
    buf_size: usize,
    list: Vec<u32>,
    map: Vec<u32>,
    regex: Option<Regex>,
}

unsafe impl Sync for TmplSymbolCache {}

impl Drop for TmplSymbolCache {
    fn drop(&mut self) {
        if !self.buf.is_null() {
            unsafe { alloc::dealloc(self.buf, Layout::from_size_align_unchecked(NODE_SIZE, NODE_ALIGN)) };
        }
        self.list = Vec::new();
        self.map = Vec::new();

        #[cfg(feature = "debug-print")]
        log::debug!("TmplSymbolCache::drop()");
    }
}

impl TmplSymbolCache {
    pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> XResult<TmplSymbolCache> {
        use rkyv::rancor::Failure;
        use rkyv::Archived;

        let path = PathBuf::from(path.as_ref());
        let rkyv_path = path.join("symbol.rkyv");
        let json_path = path.join("symbol.json");

        if fs::exists(&rkyv_path).unwrap_or(false) {
            let buf = fs::read(&rkyv_path).map_err(xfromf!("rkyv_path={:?}", rkyv_path))?;
            let strings = rkyv::access::<Archived<Vec<String>>, Failure>(&buf).map_err(|_| xerr!(Rkyv))?;
            return TmplSymbolCache::from_strings(strings);
        }

        if fs::exists(&json_path).unwrap_or(false) {
            let buf = fs::read(&json_path).map_err(xfromf!("json_path={:?}", json_path))?;
            let strings: Vec<&str> =
                serde_json::from_slice::<Vec<&str>>(&buf).map_err(xfromf!("json_path={:?}", json_path))?;
            return TmplSymbolCache::from_strings(&strings);
        }

        xresf!(NotFound; "path={:?}", &path)
    }

    pub(crate) fn from_strings<S: AsRef<str>>(strings: &[S]) -> XResult<TmplSymbolCache> {
        if strings.len() > MAX_SYMBOL_COUNT {
            return xresf!(Overflow; "strings.len={}", strings.len());
        }

        let mut buf_size: usize = 0;
        for (idx, string) in strings.iter().enumerate() {
            if string.as_ref().is_empty() {
                continue;
            }
            if string.as_ref().len() > MAX_SYMBOL_LEN {
                return xresf!(Overflow; "idx={}, string.len={}", idx, strings.len());
            }

            let legal = string
                .as_ref()
                .as_bytes()
                .iter()
                .all(|c| c.is_ascii_digit() || c.is_ascii_alphabetic() || *c == b'_' || *c == b'-');
            if !legal {
                return xresf!(BadAsset; "idx={}", idx);
            }

            buf_size += ((string.as_ref().len() + NODE_MASK) & !NODE_MASK) + NODE_SIZE;
        }

        let buf = unsafe { alloc::alloc_zeroed(Layout::from_size_align_unchecked(buf_size, NODE_ALIGN)) };
        let mut list = vec![u32::MAX; strings.len()];
        let map_len = Self::find_map_len((strings.len() as f64 * 1.5) as usize);
        let mut map = vec![u32::MAX; map_len];

        let mut offset = 0;
        for (idx, string) in strings.iter().enumerate() {
            let string = string.as_ref();
            let node = unsafe { &mut *(buf.add(offset) as *mut TmplSymbol) };
            node.next = u32::MAX;
            node.hash = Self::hash(string);
            node.idx = idx as u16;
            node.len = string.len() as u8;
            unsafe {
                node.chars.as_mut_ptr().copy_from(string.as_ptr(), string.len());
            };

            list[idx] = offset as u32;

            let map_idx = node.hash as usize % map.len();
            let next_offset = map[map_idx];
            map[map_idx] = offset as u32;
            node.next = next_offset;

            offset += ((string.len() + NODE_MASK) & !NODE_MASK) + NODE_SIZE;
        }
        debug_assert_eq!(offset, buf_size);

        let regex = Regex::new(
            r"^(\#|\w+)\.([\w\-\_]+)(?:\.([\w\-\_]+))?(?:\.([\w\-\_]+))?(?:\^([0-9]?[0-9A-Z]|[A-Z][0-9]))?$",
        )
        .unwrap();

        Ok(TmplSymbolCache {
            buf,
            buf_size,
            list,
            map,
            regex: Some(regex),
        })
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_null()
    }

    pub fn allocated_size(&self) -> usize {
        self.buf_size
    }

    pub fn find_id(&self, string: &str) -> XResult<u16> {
        if string.is_empty() {
            return Ok(SYMBOL_EMPTY);
        }
        if string.len() > MAX_SYMBOL_LEN {
            return xres!(NotFound);
        }

        let hash = Self::hash(string);
        let map_idx = hash as usize % self.map.len();

        let mut offset = self.map[map_idx];
        while offset != u32::MAX {
            let node = unsafe { &*(self.buf.add(offset as usize) as *const TmplSymbol) };
            if node.as_str() == string {
                return Ok(node.idx);
            }
            offset = node.next;
        }
        return xres!(NotFound);
    }

    pub fn find_str(&self, idx: u16) -> XResult<&str> {
        let offset = match self.list.get(idx as usize) {
            Some(offset) => *offset as usize,
            None => return xres!(NotFound),
        };
        let node = unsafe { &*(self.buf.add(offset) as *const TmplSymbol) };
        Ok(node.as_str())
    }

    pub fn find_str_or<'t>(&'t self, idx: u16, def: &'t str) -> &'t str {
        let offset = match self.list.get(idx as usize) {
            Some(offset) => *offset as usize,
            None => return def,
        };
        let node = unsafe { &*(self.buf.add(offset) as *const TmplSymbol) };
        node.as_str()
    }

    fn find_map_len(mut num: usize) -> usize {
        'out: loop {
            for p in [2, 3, 5, 7, 11, 13, 17, 19, 23, 29] {
                if num % p == 0 {
                    num += 1;
                    continue 'out;
                }
            }
            return num;
        }
    }

    fn hash(string: &str) -> u32 {
        let mut hasher = FxHasher::default();
        hasher.write(string.as_bytes());
        let hash = hasher.finish();
        ((hash & 0xFFFFFFFF) ^ (hash >> 32)) as u32
    }
}

//
// Template prefix
//

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TmplPrefix {
    Var,
    Character,
    NpcCharacter,
    Style,
    Equipment,
    Entry,
    Perk,
    AccessoryPool,
    Accessory,
    Jewel,
    Action,
    NpcAction,
    Zone,
    Invalid = 0x3F,
}

rkyv_self!(TmplPrefix);

impl FromStr for TmplPrefix {
    type Err = XError;

    fn from_str(s: &str) -> XResult<TmplPrefix> {
        let prefix = match s {
            "#" => TmplPrefix::Var,
            "Character" => TmplPrefix::Character,
            "NpcCharacter" => TmplPrefix::NpcCharacter,
            "Style" => TmplPrefix::Style,
            "Equipment" => TmplPrefix::Equipment,
            "Entry" => TmplPrefix::Entry,
            "Perk" => TmplPrefix::Perk,
            "AccessoryPool" => TmplPrefix::AccessoryPool,
            "Accessory" => TmplPrefix::Accessory,
            "Jewel" => TmplPrefix::Jewel,
            "Action" => TmplPrefix::Action,
            "NpcAction" => TmplPrefix::NpcAction,
            "Zone" => TmplPrefix::Zone,
            _ => return xres!(NotFound; "TmplPrefix string"),
        };
        Ok(prefix)
    }
}

impl TryFrom<u8> for TmplPrefix {
    type Error = XError;

    fn try_from(value: u8) -> XResult<TmplPrefix> {
        let prefix = match value {
            0 => TmplPrefix::Var,
            1 => TmplPrefix::Character,
            2 => TmplPrefix::NpcCharacter,
            3 => TmplPrefix::Style,
            4 => TmplPrefix::Equipment,
            5 => TmplPrefix::Entry,
            6 => TmplPrefix::Perk,
            7 => TmplPrefix::AccessoryPool,
            8 => TmplPrefix::Accessory,
            9 => TmplPrefix::Jewel,
            10 => TmplPrefix::Action,
            11 => TmplPrefix::NpcAction,
            12 => TmplPrefix::Zone,
            _ => return xres!(NotFound; "TmplPrefix u8"),
        };
        Ok(prefix)
    }
}

impl AsRef<str> for TmplPrefix {
    fn as_ref(&self) -> &str {
        match self {
            TmplPrefix::Var => "#",
            TmplPrefix::Character => "Character",
            TmplPrefix::NpcCharacter => "NpcCharacter",
            TmplPrefix::Style => "Style",
            TmplPrefix::Equipment => "Equipment",
            TmplPrefix::Entry => "Entry",
            TmplPrefix::Perk => "Perk",
            TmplPrefix::AccessoryPool => "AccessoryPool",
            TmplPrefix::Accessory => "Accessory",
            TmplPrefix::Jewel => "Jewel",
            TmplPrefix::Action => "Action",
            TmplPrefix::NpcAction => "NpcAction",
            TmplPrefix::Zone => "Zone",
            TmplPrefix::Invalid => "?",
        }
    }
}

impl fmt::Display for TmplPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

//
// Template ID
//

#[repr(align(8))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TmplID(u64);

rkyv_self!(TmplID);

impl Default for TmplID {
    #[inline]
    fn default() -> Self {
        TmplID::INVALID
    }
}

impl TmplID {
    pub const INVALID: TmplID = TmplID(u64::MAX);

    #[inline(always)]
    pub fn is_invalid(&self) -> bool {
        *self == Self::INVALID
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self != Self::INVALID
    }

    #[inline(always)]
    pub fn prefix(&self) -> TmplPrefix {
        let prefix = (self.0 >> 58) as u8;
        unsafe { mem::transmute::<u8, TmplPrefix>(prefix) }
    }

    #[inline(always)]
    pub fn suffix(&self) -> u16 {
        ((self.0 >> 48) & 0x03FF) as u16
    }

    #[inline(always)]
    pub fn key1(&self) -> u16 {
        ((self.0 >> 32) & 0xFFFF) as u16
    }

    #[inline(always)]
    pub fn key2(&self) -> u16 {
        ((self.0 >> 16) & 0xFFFF) as u16
    }

    #[inline(always)]
    pub fn key3(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }

    // Parse string like: Class.Key1.Key2/1A
    #[inline]
    pub fn new(s: &str) -> XResult<TmplID> {
        Self::new_with(s, symbol_cache())
    }

    #[inline]
    pub fn to_string(&self) -> String {
        self.to_string_with(symbol_cache())
    }

    #[inline]
    pub fn to_u64(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn try_from_u64(id: u64, cache: &TmplSymbolCache) -> XResult<TmplID> {
        TmplPrefix::try_from((id >> 58) as u8)?; // prefix
        let key1 = ((id >> 32) & 0xFFFF) as u16;
        if key1 != SYMBOL_EMPTY {
            cache.find_str(key1)?;
        }
        let key2 = ((id >> 16) & 0xFFFF) as u16;
        if key2 != SYMBOL_EMPTY {
            cache.find_str(key2)?;
        }
        let key3 = (id & 0xFFFF) as u16;
        if key3 != SYMBOL_EMPTY {
            cache.find_str(key3)?;
        }
        Ok(TmplID(id))
    }

    fn new_with(s: &str, cache: &TmplSymbolCache) -> XResult<TmplID> {
        let segs = match Self::split_str(s, cache) {
            Some(segs) => segs,
            None if s == "" => return Ok(TmplID::default()),
            None => return xres!(BadArgument; "invalid string"),
        };
        let prefix = TmplPrefix::from_str(segs[0])?;
        let key1 = cache.find_id(segs[1])?;
        let key2 = cache.find_id(segs[2])?;
        let key3 = cache.find_id(segs[3])?;
        let suffix = Self::parse_suffix(segs[4]).ok_or_else(|| xerr!(BadArgument; "invalid string"))?;
        let id = ((prefix as u64) << 58)
            | ((suffix as u64) << 48)
            | ((key1 as u64) << 32)
            | ((key2 as u64) << 16)
            | ((key3 as u64) << 0);
        Ok(TmplID(id))
    }

    fn to_string_with(&self, cache: &TmplSymbolCache) -> String {
        let prefix = self.prefix();
        let key1 = cache.find_str_or(self.key1(), "?");
        let key2 = cache.find_str_or(self.key2(), "?");
        let key3 = cache.find_str_or(self.key3(), "?");
        let mut suffix = [0u8; 2];
        let suffix = Self::encode_suffix(&mut suffix, self.suffix(), "?");

        match (
            self.key2() != SYMBOL_EMPTY,
            self.key3() != SYMBOL_EMPTY,
            self.suffix() != SUFFIX_EMPTY,
        ) {
            (_, true, false) => format!("{}.{}.{}.{}", prefix, key1, key2, key3),
            (_, true, true) => format!("{}.{}.{}.{}^{}", prefix, key1, key2, key3, suffix),
            (true, false, false) => format!("{}.{}.{}", prefix, key1, key2),
            (true, false, true) => format!("{}.{}.{}^{}", prefix, key1, key2, suffix),
            (false, false, false) => format!("{}.{}", prefix, key1),
            (false, false, true) => format!("{}.{}^{}", prefix, key1, suffix),
        }
    }

    fn split_str<'t>(s: &'t str, cache: &TmplSymbolCache) -> Option<[&'t str; 5]> {
        let regex = cache.regex.as_ref()?;
        let caps = regex.captures(s)?;
        Some([
            caps.get(1)?.as_str(),
            caps.get(2)?.as_str(),
            caps.get(3).map(|m| m.as_str()).unwrap_or(""),
            caps.get(4).map(|m| m.as_str()).unwrap_or(""),
            caps.get(5).map(|m| m.as_str()).unwrap_or(""),
        ])
    }

    fn parse_suffix(s: &str) -> Option<u16> {
        fn parse_char(c: u8) -> Option<u16> {
            if c.is_ascii_digit() {
                Some((c - b'0') as u16)
            }
            else if c.is_ascii_uppercase() {
                Some((c - b'A') as u16 + 10)
            }
            else {
                None
            }
        }

        fn parse_digit(c: u8) -> Option<u16> {
            match c.is_ascii_digit() {
                true => Some((c - b'0') as u16),
                false => None,
            }
        }

        fn parse_uppercase(c: u8) -> Option<u16> {
            match c.is_ascii_uppercase() {
                true => Some((c - b'A') as u16),
                false => None,
            }
        }

        let bytes = s.as_bytes();
        let num = if bytes.len() == 0 {
            SUFFIX_EMPTY
        }
        else if bytes.len() == 1 {
            parse_char(bytes[0])?
        }
        else if bytes.len() == 2 {
            if bytes[0].is_ascii_digit() {
                36 + parse_digit(bytes[0])? * 36 + parse_char(bytes[1])?
            }
            else {
                36 + 360 + parse_uppercase(bytes[0])? * 10 + parse_digit(bytes[1])?
            }
        }
        else {
            return None;
        };

        debug_assert!(num <= SUFFIX_EMPTY);
        Some(num)
    }

    fn encode_suffix<'t>(buf: &'t mut [u8; 2], n: u16, def: &'t str) -> &'t str {
        fn encode_char(c: u16) -> u8 {
            if c < 10 {
                b'0' + c as u8
            }
            else {
                b'A' + (c - 10) as u8
            }
        }

        fn encode_digit(c: u16) -> u8 {
            b'0' + c as u8
        }

        fn encode_uppercase(c: u16) -> u8 {
            b'A' + c as u8
        }

        let len;
        if n < 36 {
            buf[0] = encode_char(n);
            len = 1;
        }
        else if n < 36 + 360 {
            buf[0] = encode_char((n - 36) / 36);
            buf[1] = encode_char((n - 36) % 36);
            len = 2;
        }
        else if n < 36 + 360 + 260 {
            buf[0] = encode_uppercase((n - 36 - 360) / 10);
            buf[1] = encode_digit((n - 36 - 360) % 10);
            len = 2;
        }
        else if n == SUFFIX_EMPTY {
            len = 0;
        }
        else {
            return def;
        }
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(buf.as_ptr(), len)) }
    }
}

impl From<TmplID> for u64 {
    #[inline]
    fn from(id: TmplID) -> u64 {
        id.0
    }
}

impl TryFrom<u64> for TmplID {
    type Error = XError;

    #[inline]
    fn try_from(id: u64) -> XResult<TmplID> {
        TmplID::try_from_u64(id, symbol_cache())
    }
}

impl FromStr for TmplID {
    type Err = XError;

    #[inline]
    fn from_str(s: &str) -> XResult<TmplID> {
        TmplID::new(s)
    }
}

impl TryFrom<&str> for TmplID {
    type Error = XError;

    #[inline]
    fn try_from(s: &str) -> XResult<TmplID> {
        TmplID::new(s)
    }
}

impl TryFrom<&String> for TmplID {
    type Error = XError;

    #[inline]
    fn try_from(s: &String) -> XResult<TmplID> {
        TmplID::new(s)
    }
}

impl TryFrom<String> for TmplID {
    type Error = XError;

    #[inline]
    fn try_from(s: String) -> XResult<TmplID> {
        TmplID::new(&s)
    }
}

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

impl fmt::Debug for TmplID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

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
    fn test_tmpl_symbol_cache_json() {
        let test_dir = prepare_tmp_dir("tmpl-symbol-cache-json");
        assert!(TmplSymbolCache::from_file(&test_dir).is_err());

        // Test invalid symbol length
        let json_dir = test_dir.join("symbol.json");
        write_json(&json_dir, &[&"X".repeat(MAX_SYMBOL_LEN + 1)]);
        assert!(TmplSymbolCache::from_file(&test_dir).is_err());

        // Test common symbols
        write_json(&json_dir, &["A", "BB", "CCC", "DDDDDDDDDD"]);
        let cache = TmplSymbolCache::from_file(&test_dir).unwrap();
        assert_eq!(cache.find_id("A").unwrap(), 0);
        assert_eq!(cache.find_id("BB").unwrap(), 1);
        assert_eq!(cache.find_id("CCC").unwrap(), 2);
        assert_eq!(cache.find_id("DDDDDDDDDD").unwrap(), 3);
        assert_eq!(cache.find_id("").unwrap(), u16::MAX);
        assert!(cache.find_id("Z").is_err());
        assert_eq!(cache.find_str(0).unwrap(), "A");
        assert_eq!(cache.find_str(1).unwrap(), "BB");
        assert_eq!(cache.find_str(2).unwrap(), "CCC");
        assert_eq!(cache.find_str(3).unwrap(), "DDDDDDDDDD");
        assert!(cache.find_str(4).is_err());

        // Test hash conflict
        write_json(&json_dir, &(0..20).map(|n| n.to_string()).collect::<Vec<_>>());
        let cache = TmplSymbolCache::from_file(&test_dir).unwrap();
        for n in 0..20 {
            assert_eq!(cache.find_id(&n.to_string()).unwrap(), n);
            assert_eq!(cache.find_str(n).unwrap(), n.to_string());
        }
    }

    #[test]
    fn test_tmpl_symbol_cache_rkyv() {
        let test_dir = prepare_tmp_dir("tmpl-symbol-cache-rkyv");
        assert!(TmplSymbolCache::from_file(&test_dir).is_err());

        // Test invalid symbol length
        let rkyv_dir = test_dir.join("symbol.rkyv");
        write_rkyv(&rkyv_dir, &vec!["X".repeat(MAX_SYMBOL_LEN + 1)]);
        assert!(TmplSymbolCache::from_file(&test_dir).is_err());

        // Test common symbols
        write_rkyv(&rkyv_dir, &vec![
            "A".to_string(),
            "BB".to_string(),
            "CCC".to_string(),
            "DDDDDDDDDD".to_string(),
        ]);
        let cache = TmplSymbolCache::from_file(&test_dir).unwrap();
        assert_eq!(cache.find_id("A").unwrap(), 0);
        assert_eq!(cache.find_id("BB").unwrap(), 1);
        assert_eq!(cache.find_id("CCC").unwrap(), 2);
        assert_eq!(cache.find_id("DDDDDDDDDD").unwrap(), 3);
        assert_eq!(cache.find_id("").unwrap(), u16::MAX);
        assert!(cache.find_id("Z").is_err());
        assert_eq!(cache.find_str(0).unwrap(), "A");
        assert_eq!(cache.find_str(1).unwrap(), "BB");
        assert_eq!(cache.find_str(2).unwrap(), "CCC");
        assert_eq!(cache.find_str(3).unwrap(), "DDDDDDDDDD");
        assert!(cache.find_str(4).is_err());

        // Test hash conflict
        write_rkyv(&rkyv_dir, &(0..20).map(|n| n.to_string()).collect::<Vec<String>>());
        let cache = TmplSymbolCache::from_file(&test_dir).unwrap();
        for n in 0..20 {
            assert_eq!(cache.find_id(&n.to_string()).unwrap(), n);
            assert_eq!(cache.find_str(n).unwrap(), n.to_string());
        }
    }

    fn make_id(prefix: TmplPrefix, key1: u16, key2: u16, key3: u16, suffix: u16) -> TmplID {
        let id = ((prefix as u64) << 58)
            | ((suffix as u64) << 48)
            | ((key1 as u64) << 32)
            | ((key2 as u64) << 16)
            | ((key3 as u64) << 0);
        TmplID(id)
    }

    #[test]
    fn test_tmpl_id_common() {
        let test_dir = prepare_tmp_dir("tmpl-id-common");
        let json_dir = test_dir.join("symbol.json");

        let strings = (0..26)
            .map(|n| format!("{0}{1}{1}", (b'A' + n) as char, (b'a' + n) as char))
            .collect::<Vec<String>>();
        write_json(&json_dir, &strings);
        let cache = TmplSymbolCache::from_file(&test_dir).unwrap();

        let id1 = TmplID::new_with("Character.Zzz", &cache).unwrap();
        assert_eq!(id1, make_id(Character, 25, SYMBOL_EMPTY, SYMBOL_EMPTY, SUFFIX_EMPTY));
        assert_eq!(id1.to_string_with(&cache), "Character.Zzz");

        let id2 = TmplID::new_with("Equipment.Aaa^Z", &cache).unwrap();
        assert_eq!(id2, make_id(Equipment, 0, SYMBOL_EMPTY, SYMBOL_EMPTY, 35));
        assert_eq!(id2.to_string_with(&cache), "Equipment.Aaa^Z");

        let id3 = TmplID::new_with("Equipment.Aaa^00", &cache).unwrap();
        assert_eq!(id3, make_id(Equipment, 0, SYMBOL_EMPTY, SYMBOL_EMPTY, 36 + 0));
        assert_eq!(id3.to_string_with(&cache), "Equipment.Aaa^00");

        let id4 = TmplID::new_with("Zone.Hhh.Iii", &cache).unwrap();
        assert_eq!(id4, make_id(Zone, 7, 8, SYMBOL_EMPTY, SUFFIX_EMPTY));
        assert_eq!(id4.to_string_with(&cache), "Zone.Hhh.Iii");

        let id5 = TmplID::new_with("Zone.Hhh.Iii^9Z", &cache).unwrap();
        assert_eq!(id5, make_id(Zone, 7, 8, SYMBOL_EMPTY, 36 + 359));
        assert_eq!(id5.to_string_with(&cache), "Zone.Hhh.Iii^9Z");

        let id6 = TmplID::new_with("Character.Xxx.Yyy.Ooo", &cache).unwrap();
        assert_eq!(id6, make_id(Character, 23, 24, 14, SUFFIX_EMPTY));
        assert_eq!(id6.to_string_with(&cache), "Character.Xxx.Yyy.Ooo");

        let id7 = TmplID::new_with("Character.Xxx.Yyy.Ooo^A0", &cache).unwrap();
        assert_eq!(id7, make_id(Character, 23, 24, 14, 36 + 360));
        assert_eq!(id7.to_string_with(&cache), "Character.Xxx.Yyy.Ooo^A0");

        let id8 = TmplID::new_with("Character.Xxx.Yyy.Ooo^Z9", &cache).unwrap();
        assert_eq!(id8, make_id(Character, 23, 24, 14, 36 + 360 + 259));
        assert_eq!(id8.to_string_with(&cache), "Character.Xxx.Yyy.Ooo^Z9");

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
        assert!(TmplID::new_with("Zone.Aaa.Bbb.Ccc^CC", &cache).is_err());
        assert!(TmplID::new_with("Zone.128.Bbb.Ccc", &cache).is_err());
    }

    #[test]
    fn test_tmpl_id_json_rkyv() {
        use rkyv::rancor::Error;

        let id1 = TmplID::new("Zone.Aaa^0").unwrap();
        let buf = serde_json::to_vec(&id1).unwrap();
        let id2 = serde_json::from_slice(&buf).unwrap();
        assert_eq!(id1, id2);

        let id3 = TmplID::new("Entry.Xxx.Yyy.Zzz^1F").unwrap();
        let buf = rkyv::to_bytes::<Error>(&id3).unwrap();
        let id4 = unsafe { rkyv::access_unchecked::<TmplID>(&buf) };
        let id5 = rkyv::deserialize::<_, Error>(id4).unwrap();
        assert_eq!(id3, (*id4).into());
        assert_eq!(id3, id5);
    }
}
