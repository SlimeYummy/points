use lasso::{Capacity, MiniSpur, Rodeo, RodeoReader};
use proc_macro::TokenStream;
use regex::Regex;
use rustc_hash::FxBuildHasher;
use serde_json;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::{env, fs, mem};

struct TmplKeyCache {
    prefixes: HashMap<&'static str, &'static str>,
    keys: RodeoReader<MiniSpur, FxBuildHasher>,
    regex: Regex,
}

impl TmplKeyCache {
    fn new() -> TmplKeyCache {
        let prefixes = HashMap::from([
            ("#", "Var"),
            ("Character", "Character"),
            ("CharacterNpc", "CharacterNpc"),
            ("Style", "Style"),
            ("Equipment", "Equipment"),
            ("Entry", "Entry"),
            ("Perk", "Perk"),
            ("AccessoryPool", "AccessoryPool"),
            ("Accessory", "Accessory"),
            ("Jewel", "Jewel"),
            ("Action", "Action"),
            ("NpcAction", "NpcAction"),
            ("AiBrain", "AiBrain"),
            ("AiTask", "AiTask"),
            ("Zone", "Zone"),
        ]);

        let path = match env::var("TMPL_KEYS_PATH") {
            Ok(path) => path,
            Err(_) => {
                println!("TMPL_KEYS_PATH environment variable not set, using test path !!!");
                "../test-tmp/tmpl-id-common/key.json".to_string()
            }
        };
        let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("Read keys failed {:?}: {}", path, e));
        let keys: Vec<&str> =
            serde_json::from_str(&content).unwrap_or_else(|e| panic!("Parse keys failed {:?}: {}", path, e));
        let mut rodeo = Rodeo::with_capacity_and_hasher(Capacity::for_strings(keys.len()), FxBuildHasher::default());
        for key in keys {
            rodeo.get_or_intern(key);
        }

        let regex =
            Regex::new(r#"^"(\#|\w+)\.([\w\-\_]+)(?:\.([\w\-\_]+))?(?:\.([\w\-\_]+))?(?:\^([0-9A-Z]{1,3}))?"$"#)
                .unwrap();

        TmplKeyCache {
            prefixes,
            keys: rodeo.into_reader(),
            regex,
        }
    }

    fn parse_tmpl_id(&self, s: &str) -> (&'static str, [u16; 3], u16) {
        let caps = self.regex.captures(s).expect("Invalid TmplID");
        let prefix = caps.get(1).expect("Invalid prefix").as_str();
        let key1 = caps.get(2).expect("Invalid key1").as_str();
        let key2 = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let key3 = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        let suffix = caps.get(5).map(|m| m.as_str()).unwrap_or("");

        let prefix = self.parse_prefix(prefix);
        let key1 = self.parse_key(key1, false);
        let key2 = self.parse_key(key2, true);
        let key3 = self.parse_key(key3, true);
        let suffix = self.parse_suffix(suffix);

        (prefix, [key1, key2, key3], suffix)
    }

    fn parse_prefix(&self, s: &str) -> &'static str {
        self.prefixes.get(s).expect("Invalid prefix")
    }

    fn parse_key(&self, s: &str, allow_empty: bool) -> u16 {
        if s.is_empty() {
            if allow_empty {
                return 0;
            }
            else {
                panic!("Invalid key");
            }
        }
        match self.keys.get(s) {
            Some(spur) => unsafe { mem::transmute::<MiniSpur, u16>(spur) },
            None => panic!("Invalid key"),
        }
    }

    fn parse_suffix(&self, s: &str) -> u16 {
        // bytes[0] * 37 * 37
        // bytes[1] * 37
        // bytes[2] * 0

        let bytes = s.as_bytes();
        if bytes.len() > 3 {
            panic!("Invalid suffix");
        }

        let mut num = 0;
        for b in bytes {
            num = num * 37
                + match b {
                    b'0'..=b'9' => (b - b'0') as u16 + 1,
                    b'A'..=b'Z' => (b - b'A') as u16 + 11,
                    _ => panic!("Invalid suffix"),
                };
        }
        num
    }
}

static KEY_CACHE: OnceLock<TmplKeyCache> = OnceLock::new();

#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    let cache = KEY_CACHE.get_or_init(|| TmplKeyCache::new());

    let id = input.to_string();
    if id.trim() == "\"\"" {
        return "TmplID::default()".parse().unwrap();
    }

    let (prefix, key, suffix) = cache.parse_tmpl_id(&id);
    format!(
        "TmplID::new3(TmplPrefix::{}, [{}, {}, {}], {})",
        prefix, key[0], key[1], key[2], suffix
    )
    .parse()
    .unwrap()
}
