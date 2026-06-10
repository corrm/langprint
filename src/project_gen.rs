//! Build-system project file generation.
//!
//! A [`ProjectGenerator`] turns an engine-neutral [`ProjectSpec`] into the
//! project file(s) of a concrete build system (CMake, Make, MSBuild, Cargo).
//! Generators only ever **write text files** — they never invoke `cmake`,
//! `make`, `msbuild`, `cargo`, or any other tool, and they have no knowledge
//! of where the [`ProjectSpec`] came from (Unity, Unreal, or anything else).

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;

pub mod cargo;
pub mod cmake;
pub mod makefile;
pub mod vsln;

pub use cargo::CargoGenerator;
pub use cmake::CmakeGenerator;
pub use makefile::MakefileGenerator;
pub use vsln::VslnGenerator;

/// The language family a [`LanguageStandard`] belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageFamily {
    /// The C language.
    C,
    /// The C++ language.
    Cpp,
    /// The C# language.
    CSharp,
    /// The Rust language.
    Rust,
}

/// A source-language standard / revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageStandard {
    /// ISO C99.
    C99,
    /// ISO C11.
    C11,
    /// ISO C17.
    C17,
    /// ISO C++14.
    Cpp14,
    /// ISO C++17.
    Cpp17,
    /// ISO C++20.
    Cpp20,
    /// ISO C++23.
    Cpp23,
    /// C# 10.
    CSharp10,
    /// C# 11.
    CSharp11,
    /// C# 12.
    CSharp12,
    /// Rust 2018 edition.
    Rust2018,
    /// Rust 2021 edition.
    Rust2021,
    /// Rust 2024 edition.
    Rust2024,
}

impl LanguageStandard {
    /// The [`LanguageFamily`] this standard belongs to.
    #[must_use]
    pub fn family(self) -> LanguageFamily {
        match self {
            Self::C99 | Self::C11 | Self::C17 => LanguageFamily::C,
            Self::Cpp14 | Self::Cpp17 | Self::Cpp20 | Self::Cpp23 => LanguageFamily::Cpp,
            Self::CSharp10 | Self::CSharp11 | Self::CSharp12 => LanguageFamily::CSharp,
            Self::Rust2018 | Self::Rust2021 | Self::Rust2024 => LanguageFamily::Rust,
        }
    }

    /// The CMake `target_compile_features` feature name (e.g. `cxx_std_17`),
    /// or `None` for families CMake does not model this way (C#, Rust).
    #[must_use]
    pub fn cmake_compile_feature(self) -> Option<&'static str> {
        match self {
            Self::C99 => Some("c_std_99"),
            Self::C11 => Some("c_std_11"),
            Self::C17 => Some("c_std_17"),
            Self::Cpp14 => Some("cxx_std_14"),
            Self::Cpp17 => Some("cxx_std_17"),
            Self::Cpp20 => Some("cxx_std_20"),
            Self::Cpp23 => Some("cxx_std_23"),
            Self::CSharp10 | Self::CSharp11 | Self::CSharp12 | Self::Rust2018 | Self::Rust2021 | Self::Rust2024 => None,
        }
    }

    /// The compiler `-std=` argument value (e.g. `c++17`, `c11`) for a
    /// Makefile, or `None` for families that do not use it (C#, Rust).
    #[must_use]
    pub fn compiler_std_flag(self) -> Option<&'static str> {
        match self {
            Self::C99 => Some("c99"),
            Self::C11 => Some("c11"),
            Self::C17 => Some("c17"),
            Self::Cpp14 => Some("c++14"),
            Self::Cpp17 => Some("c++17"),
            Self::Cpp20 => Some("c++20"),
            Self::Cpp23 => Some("c++23"),
            Self::CSharp10 | Self::CSharp11 | Self::CSharp12 | Self::Rust2018 | Self::Rust2021 | Self::Rust2024 => None,
        }
    }

    /// The MSVC `<LanguageStandard>` project value (e.g. `stdcpp17`) for a C++
    /// standard, or `None` for everything else (C standards use
    /// [`msvc_language_standard_c`](Self::msvc_language_standard_c); C# and
    /// Rust are not modeled by MSBuild's C/C++ toolchain).
    #[must_use]
    pub fn msvc_language_standard(self) -> Option<&'static str> {
        match self {
            Self::Cpp14 => Some("stdcpp14"),
            Self::Cpp17 => Some("stdcpp17"),
            Self::Cpp20 => Some("stdcpp20"),
            Self::Cpp23 => Some("stdcpplatest"),
            Self::C99
            | Self::C11
            | Self::C17
            | Self::CSharp10
            | Self::CSharp11
            | Self::CSharp12
            | Self::Rust2018
            | Self::Rust2021
            | Self::Rust2024 => None,
        }
    }

    /// The MSVC `<LanguageStandard_C>` project value (e.g. `stdc11`) for a C
    /// standard, or `None` when MSVC has no switch for it (C99) or the
    /// standard is not a C standard.
    #[must_use]
    pub fn msvc_language_standard_c(self) -> Option<&'static str> {
        match self {
            Self::C11 => Some("stdc11"),
            Self::C17 => Some("stdc17"),
            _ => None,
        }
    }

    /// The Cargo `edition` value (e.g. `2021`) for a Rust standard, or `None`
    /// for non-Rust families.
    #[must_use]
    pub fn rust_edition(self) -> Option<&'static str> {
        match self {
            Self::Rust2018 => Some("2018"),
            Self::Rust2021 => Some("2021"),
            Self::Rust2024 => Some("2024"),
            _ => None,
        }
    }
}

/// The platform a generated project targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Microsoft Windows.
    Windows,
    /// Linux.
    Linux,
    /// Platform-independent / any platform.
    Any,
}

/// The CPU architecture a generated project targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// 64-bit x86 (`x86_64` / MSBuild `x64`).
    X64,
    /// 32-bit x86 (`i686` / MSBuild `Win32`).
    X86,
}

/// The kind of artifact a generated project builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    /// A shared/dynamic library (`.so`, `.dll`, `cdylib`).
    SharedLib,
    /// A static library (`.a`, `.lib`, `staticlib`).
    StaticLib,
    /// An executable.
    Executable,
}

/// Engine-neutral description of a generated SDK as a buildable project.
///
/// All paths in [`sources`](Self::sources), [`headers`](Self::headers), and
/// [`include_dirs`](Self::include_dirs) are relative to the project root
/// (the `output_dir` passed to [`ProjectGenerator::generate`]).
#[derive(Debug, Clone)]
pub struct ProjectSpec {
    /// The project / target name.
    pub name: String,
    /// The source-language standard the project is built with.
    pub language_standard: LanguageStandard,
    /// Source files, relative to the project root.
    pub sources: Vec<PathBuf>,
    /// Header files, relative to the project root. Only MSBuild projects list
    /// headers explicitly; the other build systems find them via
    /// [`include_dirs`](Self::include_dirs).
    pub headers: Vec<PathBuf>,
    /// Include directories, relative to the project root.
    pub include_dirs: Vec<PathBuf>,
    /// Preprocessor defines as `(name, value)`; `None` value means a bare define.
    pub defines: Vec<(String, Option<String>)>,
    /// The target platform.
    pub platform: Platform,
    /// The target CPU architecture.
    pub arch: Arch,
    /// The kind of artifact the project builds.
    pub output_kind: OutputKind,
}

impl ProjectSpec {
    /// Create a [`ProjectSpec`] with no sources, headers, includes, or
    /// defines, a [`Platform::Any`] target, and an [`Arch::X64`] architecture.
    ///
    /// # Arguments
    ///
    /// * `name` - The project / target name.
    /// * `language_standard` - The source-language standard.
    /// * `output_kind` - The artifact kind to build.
    #[must_use]
    pub fn new(name: impl Into<String>, language_standard: LanguageStandard, output_kind: OutputKind) -> Self {
        Self {
            name: name.into(),
            language_standard,
            sources: Vec::new(),
            headers: Vec::new(),
            include_dirs: Vec::new(),
            defines: Vec::new(),
            platform: Platform::Any,
            arch: Arch::X64,
            output_kind,
        }
    }
}

/// An error produced while generating project files.
#[derive(Debug, Error)]
pub enum ProjectGenError {
    /// Writing a project file to disk failed.
    #[error("failed to write project file `{path}`: {source}")]
    Io {
        /// The path that could not be written.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// The spec's language standard is not supported by this generator.
    #[error("{generator} does not support language standard {standard:?}")]
    UnsupportedLanguage {
        /// The generator that rejected the standard.
        generator: &'static str,
        /// The rejected standard.
        standard: LanguageStandard,
    },

    /// The spec's target platform is not supported by this generator.
    #[error("{generator} does not support platform {platform:?}")]
    IncompatiblePlatform {
        /// The generator that rejected the platform.
        generator: &'static str,
        /// The rejected platform.
        platform: Platform,
    },

    /// The spec's project name is empty or contains forbidden characters.
    #[error("invalid project name `{name}`: {reason}")]
    InvalidName {
        /// The rejected name.
        name: String,
        /// Why the name was rejected.
        reason: &'static str,
    },

    /// A source or include path is absolute or contains whitespace.
    #[error("invalid project path `{}`: paths must be relative and contain no whitespace", path.display())]
    InvalidPath {
        /// The rejected path.
        path: PathBuf,
    },

    /// A define value contains `;`, the list separator of MSBuild and CMake.
    #[error("define `{name}` has a value containing `;`")]
    InvalidDefine {
        /// The name of the rejected define.
        name: String,
    },

    /// The spec has no source files, so no project could be generated.
    #[error("project spec `{0}` has no source files")]
    NoSources(String),
}

/// Emits build-system project file(s) for a [`ProjectSpec`].
pub trait ProjectGenerator {
    /// Write this generator's project file(s) under `output_dir`.
    ///
    /// # Arguments
    ///
    /// * `spec` - The engine-neutral project description.
    /// * `output_dir` - The directory to write project file(s) into; it must
    ///   already exist.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a [`ProjectGenError`] describing the failure.
    ///
    /// # Remarks
    ///
    /// Implementations must not invoke any external build tool — they only
    /// write text files.
    fn generate(&self, spec: &ProjectSpec, output_dir: &Path) -> Result<(), ProjectGenError>;
}

/// Validate a [`ProjectSpec`] before any generator renders it.
///
/// Enforces the invariants every generator relies on: a non-empty name made
/// only of `[A-Za-z0-9_.+-]`, relative whitespace-free source/include paths,
/// and define values free of the `;` list separator.
pub(crate) fn validate_spec(spec: &ProjectSpec) -> Result<(), ProjectGenError> {
    if spec.name.is_empty() {
        return Err(ProjectGenError::InvalidName {
            name: spec.name.clone(),
            reason: "name is empty",
        });
    }
    let is_allowed_name_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-');
    if !spec.name.chars().all(is_allowed_name_char) {
        return Err(ProjectGenError::InvalidName {
            name: spec.name.clone(),
            reason: "name may only contain characters in [A-Za-z0-9_.+-]",
        });
    }
    for path in spec
        .sources
        .iter()
        .chain(spec.headers.iter())
        .chain(spec.include_dirs.iter())
    {
        if !path.is_relative() || path.to_string_lossy().chars().any(char::is_whitespace) {
            return Err(ProjectGenError::InvalidPath { path: path.clone() });
        }
    }
    for (name, value) in &spec.defines {
        if value.as_deref().is_some_and(|v| v.contains(';')) {
            return Err(ProjectGenError::InvalidDefine { name: name.clone() });
        }
    }
    Ok(())
}

/// Escape `input` for interpolation into XML text or attribute values.
///
/// `&` is escaped first so already-escaped entities are not double-mangled
/// in reverse: a literal `&` always becomes `&amp;`.
pub(crate) fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Render a relative path as a `/`-separated string for build systems that
/// expect forward slashes (CMake, Make).
pub(crate) fn path_to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Render a relative path as a `\`-separated string for MSBuild project files.
pub(crate) fn path_to_back_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
}

/// Render a single define as a `NAME` or `NAME=value` token.
pub(crate) fn format_define(name: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => format!("{name}={v}"),
        None => name.to_string(),
    }
}

/// Write `contents` to `output_dir/file_name`, mapping any I/O failure to
/// [`ProjectGenError::Io`].
pub(crate) fn write_file(output_dir: &Path, file_name: &str, contents: &str) -> Result<(), ProjectGenError> {
    let path = output_dir.join(file_name);
    fs::write(&path, contents).map_err(|source| ProjectGenError::Io { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_spec() -> ProjectSpec {
        let mut spec = ProjectSpec::new("VampireSurvivors", LanguageStandard::Cpp17, OutputKind::SharedLib);
        spec.sources.push(PathBuf::from("SDK/Assembly-CSharp.cpp"));
        spec.include_dirs.push(PathBuf::from("Headers"));
        spec.defines.push(("VERSION".to_string(), Some("105".to_string())));
        spec
    }

    #[test]
    fn new_defaults_to_any_platform_x64_and_empty_lists() {
        let spec = ProjectSpec::new("Game", LanguageStandard::Cpp17, OutputKind::SharedLib);
        assert_eq!(spec.platform, Platform::Any);
        assert_eq!(spec.arch, Arch::X64);
        assert!(spec.sources.is_empty());
        assert!(spec.headers.is_empty());
        assert!(spec.include_dirs.is_empty());
        assert!(spec.defines.is_empty());
    }

    #[test]
    fn xml_escape_escapes_all_special_characters() {
        assert_eq!(xml_escape("a&b<c>d\"e'f"), "a&amp;b&lt;c&gt;d&quot;e&apos;f");
    }

    #[test]
    fn xml_escape_escapes_ampersand_first() {
        assert_eq!(xml_escape("&lt;"), "&amp;lt;");
    }

    #[test]
    fn validate_accepts_well_formed_spec() {
        assert!(validate_spec(&valid_spec()).is_ok());
    }

    #[test]
    fn validate_rejects_empty_name() {
        let spec = ProjectSpec {
            name: String::new(),
            ..valid_spec()
        };
        let err = validate_spec(&spec).unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidName { name, .. } if name.is_empty()));
    }

    #[test]
    fn validate_rejects_name_with_forbidden_characters() {
        for bad in ["Tom & Jerry", "Game<1>", "a\"b", "name with space"] {
            let spec = ProjectSpec {
                name: bad.to_string(),
                ..valid_spec()
            };
            let err = validate_spec(&spec).unwrap_err();
            assert!(matches!(err, ProjectGenError::InvalidName { name, .. } if name == bad));
        }
    }

    #[test]
    fn validate_rejects_absolute_source_path() {
        let mut spec = valid_spec();
        spec.sources.push(PathBuf::from("/abs/main.cpp"));
        let err = validate_spec(&spec).unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidPath { path } if path == Path::new("/abs/main.cpp")));
    }

    #[test]
    fn validate_rejects_include_dir_with_whitespace() {
        let mut spec = valid_spec();
        spec.include_dirs.push(PathBuf::from("My Headers"));
        let err = validate_spec(&spec).unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidPath { path } if path == Path::new("My Headers")));
    }

    #[test]
    fn validate_rejects_define_value_with_semicolon() {
        let mut spec = valid_spec();
        spec.defines.push(("LIST".to_string(), Some("a;b".to_string())));
        let err = validate_spec(&spec).unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidDefine { name } if name == "LIST"));
    }
}
