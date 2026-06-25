//! Cross-language primitive type mapping.
//!
//! The neutral IR carries type spellings as opaque strings. When a declaration is converted from
//! one language to another, a primitive written in the source language (`f32`, `uint8_t`, `int`)
//! must be re-spelled in the target language (`float`, `byte`, `int`). [`TypeMap`] performs that
//! re-spelling. A [`builtin`](TypeMap::builtin) table covers the common primitives, and callers can
//! override, extend, or clear it before driving a conversion.

use std::collections::HashMap;

/// One row of the built-in primitive table: the neutral primitive, its recognized spellings, then
/// the output spelling per typed target (C++, Rust, C#, Python, JS). Lua is omitted — it is untyped.
type BuiltinRow = (PrimitiveType, &'static [&'static str], &'static str, &'static str, &'static str, &'static str, &'static str);

/// A neutral, language-independent primitive type identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    /// Boolean.
    Bool,
    /// Signed 8-bit integer.
    I8,
    /// Unsigned 8-bit integer.
    U8,
    /// Signed 16-bit integer.
    I16,
    /// Unsigned 16-bit integer.
    U16,
    /// Signed 32-bit integer.
    I32,
    /// Unsigned 32-bit integer.
    U32,
    /// Signed 64-bit integer.
    I64,
    /// Unsigned 64-bit integer.
    U64,
    /// Signed 128-bit integer.
    I128,
    /// Unsigned 128-bit integer.
    U128,
    /// Pointer-sized signed integer.
    ISize,
    /// Pointer-sized unsigned integer.
    USize,
    /// 32-bit floating point.
    F32,
    /// 64-bit floating point.
    F64,
    /// Character.
    Char,
    /// No value / no return.
    Void,
    /// String.
    Str,
}

/// A language a [`TypeMap`] can render a primitive into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetLanguage {
    /// C++.
    Cpp,
    /// Rust.
    Rust,
    /// C#.
    CSharp,
    /// Python.
    Python,
    /// Lua.
    Lua,
    /// JavaScript.
    Js,
}

/// Bidirectional mapping between primitive type spellings across languages.
///
/// `recognize` maps any known spelling (from any language) to its neutral [`PrimitiveType`];
/// `output` maps a `(PrimitiveType, TargetLanguage)` pair to that language's canonical spelling.
#[derive(Debug, Clone)]
pub struct TypeMap {
    recognize: HashMap<String, PrimitiveType>,
    output: HashMap<(PrimitiveType, TargetLanguage), String>,
}

impl Default for TypeMap {
    fn default() -> Self {
        let mut map = Self::empty();

        let table: &[BuiltinRow] = &[
            (PrimitiveType::Bool, &["bool"], "bool", "bool", "bool", "bool", "boolean"),
            (PrimitiveType::I8, &["int8_t", "i8", "sbyte", "signed char"], "int8_t", "i8", "sbyte", "int", "number"),
            (PrimitiveType::U8, &["uint8_t", "u8", "byte", "unsigned char"], "uint8_t", "u8", "byte", "int", "number"),
            (PrimitiveType::I16, &["int16_t", "i16", "short"], "int16_t", "i16", "short", "int", "number"),
            (PrimitiveType::U16, &["uint16_t", "u16", "ushort"], "uint16_t", "u16", "ushort", "int", "number"),
            (PrimitiveType::I32, &["int32_t", "i32", "int"], "int32_t", "i32", "int", "int", "number"),
            (PrimitiveType::U32, &["uint32_t", "u32", "uint", "unsigned int"], "uint32_t", "u32", "uint", "int", "number"),
            (PrimitiveType::I64, &["int64_t", "i64", "long"], "int64_t", "i64", "long", "int", "number"),
            (PrimitiveType::U64, &["uint64_t", "u64", "ulong"], "uint64_t", "u64", "ulong", "int", "number"),
            (PrimitiveType::I128, &["__int128", "i128", "Int128"], "__int128", "i128", "Int128", "int", "number"),
            (PrimitiveType::U128, &["unsigned __int128", "u128", "UInt128"], "unsigned __int128", "u128", "UInt128", "int", "number"),
            (PrimitiveType::ISize, &["intptr_t", "isize", "nint"], "intptr_t", "isize", "nint", "int", "number"),
            (PrimitiveType::USize, &["uintptr_t", "size_t", "usize", "nuint"], "uintptr_t", "usize", "nuint", "int", "number"),
            (PrimitiveType::F32, &["float", "f32", "single"], "float", "f32", "float", "float", "number"),
            (PrimitiveType::F64, &["double", "f64"], "double", "f64", "double", "float", "number"),
            (PrimitiveType::Char, &["char"], "char", "char", "char", "str", "string"),
            (PrimitiveType::Void, &["void", "()"], "void", "()", "void", "None", "void"),
            (PrimitiveType::Str, &["string", "std::string", "String"], "std::string", "String", "string", "str", "string"),
        ];

        for (primitive, spellings, cpp, rust, csharp, python, js) in table {
            for spelling in *spellings {
                map.insert_spelling(*spelling, *primitive);
            }
            map.set_output(*primitive, TargetLanguage::Cpp, *cpp);
            map.set_output(*primitive, TargetLanguage::Rust, *rust);
            map.set_output(*primitive, TargetLanguage::CSharp, *csharp);
            map.set_output(*primitive, TargetLanguage::Python, *python);
            map.set_output(*primitive, TargetLanguage::Js, *js);
        }

        map
    }
}

impl TypeMap {
    /// Create an empty map that recognizes and renders nothing.
    pub fn empty() -> Self {
        Self {
            recognize: HashMap::new(),
            output: HashMap::new(),
        }
    }
    /// Recognize a type spelling as a neutral primitive.
    ///
    /// # Arguments
    ///
    /// * `spelling` - The type spelling, in any supported language.
    ///
    /// # Returns
    ///
    /// The [`PrimitiveType`] the spelling denotes, or `None` if unrecognized.
    pub fn resolve(&self, spelling: &str) -> Option<PrimitiveType> {
        self.recognize.get(spelling.trim()).copied()
    }

    /// Render a primitive in a target language.
    ///
    /// # Arguments
    ///
    /// * `primitive` - The neutral primitive.
    /// * `language` - The language to render in.
    ///
    /// # Returns
    ///
    /// The target-language spelling, or `None` if the map has no output for the pair.
    pub fn render(&self, primitive: PrimitiveType, language: TargetLanguage) -> Option<String> {
        self.output.get(&(primitive, language)).cloned()
    }

    /// Translate a type spelling into a target language.
    ///
    /// # Arguments
    ///
    /// * `spelling` - The source type spelling.
    /// * `language` - The target language.
    ///
    /// # Returns
    ///
    /// The target-language spelling, or `None` if the spelling is not a recognized primitive.
    pub fn map(&self, spelling: &str, language: TargetLanguage) -> Option<String> {
        self.render(self.resolve(spelling)?, language)
    }

    /// Register a spelling as denoting a primitive (extends or overrides recognition).
    pub fn insert_spelling(&mut self, spelling: impl Into<String>, primitive: PrimitiveType) {
        self.recognize.insert(spelling.into(), primitive);
    }

    /// Set the spelling a primitive renders to in a language (overrides the default output).
    pub fn set_output(&mut self, primitive: PrimitiveType, language: TargetLanguage, spelling: impl Into<String>) {
        self.output.insert((primitive, language), spelling.into());
    }

    /// Merge another map into this one; entries from `other` take precedence.
    pub fn extend(&mut self, other: TypeMap) {
        self.recognize.extend(other.recognize);
        self.output.extend(other.output);
    }

    /// Remove every recognition and output entry.
    pub fn clear(&mut self) {
        self.recognize.clear();
        self.output.clear();
    }
}
