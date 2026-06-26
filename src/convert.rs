//! Shared cross-language conversion configuration and the leaf helpers every backend's `from_ir`
//! uses to re-spell types and apply idiomatic identifier renaming.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use crate::conversion::{ConversionResult, ConversionWarning};
use crate::ir::{Annotation, AnnotationKind, LanguageEnum, LanguageFunction, LanguageStruct};
use crate::naming::{to_camel_case, to_pascal_case, to_snake_case};
use crate::type_map::{TargetLanguage, TypeMap};

impl TargetLanguage {
    /// The human-readable language name used in conversion warnings.
    pub fn name(self) -> &'static str {
        match self {
            TargetLanguage::Cpp => "C++",
            TargetLanguage::Rust => "Rust",
            TargetLanguage::CSharp => "C#",
            TargetLanguage::Python => "Python",
            TargetLanguage::Lua => "Lua",
            TargetLanguage::Js => "JS",
        }
    }
}

/// A custom type resolver consulted before the [`TypeMap`]: given a source spelling and the target
/// language, it returns `Some(spelling)` to override the mapping or `None` to defer to the map.
pub type TypeOverride = Arc<dyn Fn(&str, TargetLanguage) -> Option<String> + Send + Sync>;

/// Opt-in lifecycle hooks invoked on the cross-language IR path. Every method is a no-op by
/// default, so existing behavior is unchanged unless a hook is set. Hooks mutate the IR item in
/// place: `after_to_ir_*` fires once the IR value is built, `before_from_ir_*` fires before the IR
/// is lowered into a target backend type.
pub trait ConversionHooks: Send + Sync {
    /// Invoked after a struct/class has been raised into the IR.
    fn after_to_ir_struct(&self, _s: &mut LanguageStruct) {}
    /// Invoked after a function/method has been raised into the IR.
    fn after_to_ir_function(&self, _f: &mut LanguageFunction) {}
    /// Invoked after an enum has been raised into the IR.
    fn after_to_ir_enum(&self, _e: &mut LanguageEnum) {}
    /// Invoked before a struct/class is lowered out of the IR.
    fn before_from_ir_struct(&self, _s: &mut LanguageStruct) {}
    /// Invoked before a function/method is lowered out of the IR.
    fn before_from_ir_function(&self, _f: &mut LanguageFunction) {}
    /// Invoked before an enum is lowered out of the IR.
    fn before_from_ir_enum(&self, _e: &mut LanguageEnum) {}
}

/// Configuration shared by every `from_ir` conversion: how primitive types are re-spelled and
/// whether identifiers are renamed to the target language's convention.
#[derive(Clone)]
pub struct ConversionConfig {
    /// The primitive type mapping applied at conversion boundaries.
    pub type_map: Arc<TypeMap>,
    /// Whether identifiers are renamed to the target language's naming convention.
    pub rename: bool,
    /// A custom type resolver consulted before the [`TypeMap`].
    pub type_override: Option<TypeOverride>,
    /// The per-`(language, kind)` case convention applied when renaming.
    pub naming_map: NamingMap,
    /// The per-language reserved words an identifier is escaped against.
    pub keyword_map: KeywordMap,
    /// The per-`(language, kind)` native spelling Tier-1 annotations lower to.
    pub annotation_map: AnnotationMap,
    /// Opt-in lifecycle hooks invoked on the cross-language IR path.
    pub hooks: Option<Arc<dyn ConversionHooks>>,
}

impl fmt::Debug for ConversionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConversionConfig")
            .field("type_map", &self.type_map)
            .field("rename", &self.rename)
            .field(
                "type_override",
                &self.type_override.as_ref().map(|_| "<fn>"),
            )
            .field("naming_map", &self.naming_map)
            .field("keyword_map", &self.keyword_map)
            .field("annotation_map", &self.annotation_map)
            .field("hooks", &self.hooks.as_ref().map(|_| "<hooks>"))
            .finish()
    }
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            type_map: Arc::new(TypeMap::default()),
            rename: true,
            type_override: None,
            naming_map: NamingMap::default(),
            keyword_map: KeywordMap::default(),
            annotation_map: AnnotationMap::default(),
            hooks: None,
        }
    }
}

impl ConversionConfig {
    /// Create a configuration from a type map and a rename flag.
    pub fn new(type_map: TypeMap, rename: bool) -> Self {
        Self {
            type_map: Arc::new(type_map),
            rename,
            type_override: None,
            naming_map: NamingMap::default(),
            keyword_map: KeywordMap::default(),
            annotation_map: AnnotationMap::default(),
            hooks: None,
        }
    }
}

/// The kind of identifier being renamed; selects the target convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierKind {
    /// A type name (struct/class/enum).
    Type,
    /// A function or method name.
    Function,
    /// A field name.
    Field,
    /// An enum member name.
    EnumMember,
    /// A namespace / module name.
    Namespace,
}

/// The case style an identifier is rewritten into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStyle {
    /// `snake_case`.
    Snake,
    /// `PascalCase`.
    Pascal,
    /// `camelCase`.
    Camel,
}

/// The per-`(language, kind)` case convention applied when renaming identifiers.
///
/// A cross-language mapping table mirroring [`TypeMap`]: the [`default`](NamingMap::default) table
/// encodes each language's idiomatic case per [`IdentifierKind`], and callers can override, extend,
/// or clear it. A pair with no entry leaves the identifier verbatim.
#[derive(Clone, Debug)]
pub struct NamingMap {
    styles: HashMap<(TargetLanguage, IdentifierKind), CaseStyle>,
}

impl Default for NamingMap {
    fn default() -> Self {
        use CaseStyle::{Camel, Pascal, Snake};
        use IdentifierKind::{Field, Function, Namespace, Type};
        use TargetLanguage::{CSharp, Js, Lua, Python, Rust};

        let mut map = Self {
            styles: HashMap::new(),
        };

        for kind in [Function, Field, Namespace] {
            map.insert(Rust, kind, Snake);
            map.insert(Python, kind, Snake);
            map.insert(Lua, kind, Snake);
        }
        map.insert(Python, Type, Pascal);

        for kind in [Type, Function, Field, Namespace, IdentifierKind::EnumMember] {
            map.insert(CSharp, kind, Pascal);
        }

        map.insert(Js, Function, Camel);
        map.insert(Js, Field, Camel);
        map.insert(Js, Type, Pascal);

        map
    }
}

impl NamingMap {
    /// Set the case style for a `(language, kind)` pair (extends or overrides).
    pub fn insert(&mut self, language: TargetLanguage, kind: IdentifierKind, style: CaseStyle) {
        self.styles.insert((language, kind), style);
    }

    /// Merge another map into this one; entries from `other` take precedence.
    pub fn extend(&mut self, other: NamingMap) {
        self.styles.extend(other.styles);
    }

    /// Remove every entry.
    pub fn clear(&mut self) {
        self.styles.clear();
    }

    /// Resolve the case style for a `(language, kind)` pair, or `None` if none is configured.
    pub fn resolve(&self, language: TargetLanguage, kind: IdentifierKind) -> Option<CaseStyle> {
        self.styles.get(&(language, kind)).copied()
    }
}

/// The per-language reserved words an identifier is escaped against when it would otherwise collide.
///
/// A cross-language mapping table mirroring [`TypeMap`]: the [`default`](KeywordMap::default) set of
/// reserved words per language drives [`escape`](KeywordMap::escape), and callers can extend or clear
/// it. Escaping is per-language idiom (Rust `r#ident`, C# `@ident`, others `ident_`).
///
/// Unlike the other maps, this one resolves via [`escape`](KeywordMap::escape): it *transforms* an
/// identifier rather than performing a `resolve()` lookup.
#[derive(Clone, Debug)]
pub struct KeywordMap {
    reserved: HashMap<TargetLanguage, HashSet<String>>,
}

impl Default for KeywordMap {
    fn default() -> Self {
        let mut map = Self {
            reserved: HashMap::new(),
        };

        let python = [
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield", "match", "case", "type",
        ];
        let rust = [
            "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false",
            "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
            "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
            "unsafe", "use", "where", "while", "async", "await", "abstract", "become", "box", "do",
            "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
            "union",
        ];
        let csharp = [
            "abstract",
            "as",
            "base",
            "bool",
            "break",
            "byte",
            "case",
            "catch",
            "char",
            "checked",
            "class",
            "const",
            "continue",
            "decimal",
            "default",
            "delegate",
            "do",
            "double",
            "else",
            "enum",
            "event",
            "explicit",
            "extern",
            "false",
            "finally",
            "fixed",
            "float",
            "for",
            "foreach",
            "goto",
            "if",
            "implicit",
            "in",
            "int",
            "interface",
            "internal",
            "is",
            "lock",
            "long",
            "namespace",
            "new",
            "null",
            "object",
            "operator",
            "out",
            "override",
            "params",
            "private",
            "protected",
            "public",
            "readonly",
            "ref",
            "return",
            "sbyte",
            "sealed",
            "short",
            "sizeof",
            "stackalloc",
            "static",
            "string",
            "struct",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "uint",
            "ulong",
            "unchecked",
            "unsafe",
            "ushort",
            "using",
            "virtual",
            "void",
            "volatile",
            "while",
        ];
        let cpp = [
            "alignas",
            "alignof",
            "and",
            "asm",
            "auto",
            "bool",
            "break",
            "case",
            "catch",
            "char",
            "class",
            "const",
            "constexpr",
            "continue",
            "decltype",
            "default",
            "delete",
            "do",
            "double",
            "else",
            "enum",
            "explicit",
            "export",
            "extern",
            "false",
            "float",
            "for",
            "friend",
            "goto",
            "if",
            "inline",
            "int",
            "long",
            "mutable",
            "namespace",
            "new",
            "noexcept",
            "nullptr",
            "operator",
            "or",
            "private",
            "protected",
            "public",
            "register",
            "return",
            "short",
            "signed",
            "sizeof",
            "static",
            "struct",
            "switch",
            "template",
            "this",
            "throw",
            "true",
            "try",
            "typedef",
            "typeid",
            "typename",
            "union",
            "unsigned",
            "using",
            "virtual",
            "void",
            "volatile",
            "while",
            "xor",
        ];
        let js = [
            "await",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "enum",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "new",
            "null",
            "return",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
            "yield",
            "let",
            "static",
        ];
        let lua = [
            "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto",
            "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "true", "until",
            "while",
        ];

        for word in python {
            map.insert(TargetLanguage::Python, word);
        }
        for word in rust {
            map.insert(TargetLanguage::Rust, word);
        }
        for word in csharp {
            map.insert(TargetLanguage::CSharp, word);
        }
        for word in cpp {
            map.insert(TargetLanguage::Cpp, word);
        }
        for word in js {
            map.insert(TargetLanguage::Js, word);
        }
        for word in lua {
            map.insert(TargetLanguage::Lua, word);
        }

        map
    }
}

impl KeywordMap {
    /// Create an empty map that reserves nothing.
    pub fn empty() -> Self {
        Self {
            reserved: HashMap::new(),
        }
    }

    /// Reserve a word in a language (extends or overrides).
    pub fn insert(&mut self, language: TargetLanguage, word: impl Into<String>) {
        self.reserved
            .entry(language)
            .or_default()
            .insert(word.into());
    }

    /// Whether a word is reserved in a language.
    pub fn contains(&self, language: TargetLanguage, word: &str) -> bool {
        self.reserved
            .get(&language)
            .is_some_and(|set| set.contains(word))
    }

    /// Merge another map into this one.
    pub fn extend(&mut self, other: KeywordMap) {
        for (language, words) in other.reserved {
            self.reserved.entry(language).or_default().extend(words);
        }
    }

    /// Remove every reserved word.
    pub fn clear(&mut self) {
        self.reserved.clear();
    }

    /// Escape an identifier that collides with a reserved word, per the target language's idiom.
    ///
    /// Rust uses raw identifiers (`r#ident`) except for the non-rawable keywords `crate`, `self`,
    /// `Self`, and `super`, which fall back to `ident_`. C# prefixes `@`. Every other language
    /// suffixes `_`. A non-reserved identifier is returned unchanged.
    pub fn escape(&self, language: TargetLanguage, ident: &str) -> String {
        if !self.contains(language, ident) {
            return ident.to_string();
        }

        match language {
            TargetLanguage::Rust => match ident {
                "crate" | "self" | "Self" | "super" => format!("{ident}_"),
                _ => format!("r#{ident}"),
            },
            TargetLanguage::CSharp => format!("@{ident}"),
            _ => format!("{ident}_"),
        }
    }
}

/// The native attribute spelling each Tier-1 [`Annotation`] lowers to, per target language.
///
/// A cross-language mapping table mirroring [`TypeMap`]: the [`default`](AnnotationMap::default) table
/// holds the idiomatic spelling per `(language, kind)`, and callers can override, extend, or clear
/// it. The stored string is a template — for [`Annotation::Aligned`] the literal `{n}` is replaced
/// with the alignment value. A pair with no entry emits nothing for that annotation.
///
/// Template contract: an [`Annotation::Aligned`] template **must** contain the `{n}` placeholder —
/// it is substituted with the alignment value, so an override that omits `{n}` silently drops the
/// alignment. [`ReprC`](AnnotationKind::ReprC) and [`Packed`](AnnotationKind::Packed) templates are
/// emitted verbatim, and any `{n}` in them is left literal.
///
/// This governs only the languages that render annotations as text (Rust, C#). C++ lowers alignment
/// through dedicated `CppStruct` fields (`alignas(N)`, `#pragma pack`) rather than this map, so it
/// has no entries here.
#[derive(Clone, Debug)]
pub struct AnnotationMap {
    spellings: HashMap<(TargetLanguage, AnnotationKind), String>,
}
impl Default for AnnotationMap {
    fn default() -> Self {
        use AnnotationKind::{Aligned, Packed, ReprC};
        use TargetLanguage::{CSharp, Rust};

        let mut map = Self {
            spellings: HashMap::new(),
        };

        map.insert(Rust, ReprC, "repr(C)");
        map.insert(Rust, Packed, "repr(packed)");
        map.insert(Rust, Aligned, "repr(align({n}))");

        map.insert(CSharp, ReprC, "StructLayout(LayoutKind.Sequential)");
        map.insert(
            CSharp,
            Packed,
            "StructLayout(LayoutKind.Sequential, Pack = 1)",
        );

        map
    }
}

impl AnnotationMap {
    /// Create an empty map; every annotation emits nothing.
    pub fn empty() -> Self {
        Self {
            spellings: HashMap::new(),
        }
    }

    /// Set the native spelling template for a `(language, kind)` pair (extends or overrides).
    ///
    /// An [`Aligned`](AnnotationKind::Aligned) template must include the `{n}` placeholder, which
    /// [`resolve`](AnnotationMap::resolve) substitutes with the alignment value; omitting it drops
    /// the alignment. Other kinds are emitted verbatim.
    pub fn insert(
        &mut self,
        language: TargetLanguage,
        kind: AnnotationKind,
        template: impl Into<String>,
    ) {
        self.spellings.insert((language, kind), template.into());
    }

    /// Merge another map into this one; entries from `other` take precedence.
    pub fn extend(&mut self, other: AnnotationMap) {
        self.spellings.extend(other.spellings);
    }

    /// Remove every entry.
    pub fn clear(&mut self) {
        self.spellings.clear();
    }

    /// Render an annotation as its native attribute spelling in a target language.
    ///
    /// For [`Annotation::Aligned`] the `{n}` placeholder in the template is replaced with the
    /// alignment value. Returns `None` when the `(language, kind)` pair has no entry.
    pub fn resolve(&self, language: TargetLanguage, annotation: &Annotation) -> Option<String> {
        let template = self.spellings.get(&(language, annotation.kind()))?;
        match annotation {
            Annotation::Aligned(n) => Some(template.replace("{n}", &n.to_string())),
            _ => Some(template.clone()),
        }
    }
}

/// Re-spell a type into the target language using the configured [`TypeMap`].
///
/// # Arguments
///
/// * `config` - The active conversion configuration.
/// * `spelling` - The source type spelling.
/// * `language` - The target language.
///
/// # Returns
///
/// The mapped spelling, or the original spelling plus an `UnsupportedFeature` warning when the
/// type is not a recognized primitive.
pub fn map_type(
    config: &ConversionConfig,
    spelling: &str,
    language: TargetLanguage,
) -> ConversionResult<String> {
    if let Some(resolver) = &config.type_override
        && let Some(mapped) = resolver(spelling, language)
    {
        return ConversionResult::new(mapped);
    }

    match config.type_map.map(spelling, language) {
        Some(mapped) => ConversionResult::new(mapped),
        None => ConversionResult::with_warning(
            spelling.to_string(),
            ConversionWarning::UnsupportedFeature {
                feature: format!("unmapped type `{spelling}`"),
                resolution: format!("no TypeMap entry for {}; emitted verbatim", language.name()),
            },
        ),
    }
}

/// Rename an identifier to the target language's convention when renaming is enabled.
///
/// # Arguments
///
/// * `config` - The active conversion configuration.
/// * `name` - The source identifier.
/// * `language` - The target language.
/// * `kind` - The kind of identifier, which selects the convention.
///
/// # Returns
///
/// The renamed identifier, or the original; a `NamingConventionChanged` warning is attached when the
/// identifier changes — either from case conversion or from escaping a collision with a target
/// reserved word. Keyword escaping applies even when renaming is off, since a collision is a
/// correctness issue regardless of case style.
pub fn rename_identifier(
    config: &ConversionConfig,
    name: &str,
    language: TargetLanguage,
    kind: IdentifierKind,
) -> ConversionResult<String> {
    let candidate = if config.rename {
        match config.naming_map.resolve(language, kind) {
            Some(CaseStyle::Snake) => to_snake_case(name),
            Some(CaseStyle::Pascal) => to_pascal_case(name),
            Some(CaseStyle::Camel) => to_camel_case(name),
            None => name.to_string(),
        }
    } else {
        name.to_string()
    };

    let escaped = config.keyword_map.escape(language, &candidate);
    let final_name = if escaped != candidate {
        escaped
    } else {
        candidate
    };

    if final_name == name {
        return ConversionResult::new(final_name);
    }

    ConversionResult::with_warning(
        final_name.clone(),
        ConversionWarning::NamingConventionChanged {
            original: name.to_string(),
            converted: final_name,
        },
    )
}
