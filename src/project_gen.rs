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
pub mod csharp;
pub mod makefile;
pub mod vs_common;
pub mod vsln;

pub use cargo::CargoGenerator;
pub use cmake::CmakeGenerator;
pub use csharp::CSharpProjectGenerator;
pub use makefile::MakefileGenerator;
pub use vsln::{SlnxGenerator, VslnGenerator};

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

    /// The MSBuild `<LangVersion>` value (e.g. `12`) for a C# standard, or
    /// `None` for non-C# families.
    #[must_use]
    pub fn csharp_lang_version(self) -> Option<&'static str> {
        match self {
            Self::CSharp10 => Some("10"),
            Self::CSharp11 => Some("11"),
            Self::CSharp12 => Some("12"),
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

/// The C++ exception-handling model requested from the compiler.
///
/// Maps to MSVC `/EH*`. Toolchains without an `/EH` analogue (GCC / Clang on
/// non-Windows) leave exceptions at their default and ignore this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionHandling {
    /// Synchronous C++ exceptions; `extern "C"` assumed non-throwing (MSVC `/EHsc`).
    Standard,
    /// Synchronous C++ exceptions plus asynchronous structured (SEH) unwinding (MSVC `/EHa`).
    Asynchronous,
}

impl ExceptionHandling {
    /// The MSBuild `<ExceptionHandling>` element value.
    #[must_use]
    pub fn msbuild_value(self) -> &'static str {
        match self {
            Self::Standard => "Sync",
            Self::Asynchronous => "Async",
        }
    }

    /// The MSVC / clang-cl command-line flag.
    #[must_use]
    pub fn msvc_flag(self) -> &'static str {
        match self {
            Self::Standard => "/EHsc",
            Self::Asynchronous => "/EHa",
        }
    }
}

/// A precompiled header for the generated project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecompiledHeader {
    /// The header that is precompiled and used by every translation unit (e.g. `pch.h`).
    pub header: PathBuf,
    /// The translation unit that *creates* the precompiled header (e.g. `pch.cpp`).
    pub create_source: PathBuf,
}

/// Fluent builder for [`ProjectSpec`].
///
/// Provides a chainable API for constructing project descriptions. All fields
/// have sensible defaults; only `name`, `language_standard`, and `output_kind`
/// are required before calling [`build`](Self::build).
///
/// ```
/// use langprint::project_gen::{ProjectBuilder, LanguageStandard, OutputKind};
///
/// let spec = ProjectBuilder::new("my_lib", LanguageStandard::Cpp17, OutputKind::StaticLib)
///     .source("src/main.cpp")
///     .header("include/types.h")
///     .define("DEBUG", Some("1"))
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ProjectBuilder {
    name: String,
    language_standard: LanguageStandard,
    output_kind: OutputKind,
    sources: Vec<PathBuf>,
    headers: Vec<PathBuf>,
    include_dirs: Vec<PathBuf>,
    defines: Vec<(String, Option<String>)>,
    platform: Platform,
    arch: Arch,
    exception_handling: Option<ExceptionHandling>,
    precompiled_header: Option<PrecompiledHeader>,
}

impl ProjectBuilder {
    /// Create a new builder with the required fields.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        language_standard: LanguageStandard,
        output_kind: OutputKind,
    ) -> Self {
        Self {
            name: name.into(),
            language_standard,
            output_kind,
            sources: Vec::new(),
            headers: Vec::new(),
            include_dirs: Vec::new(),
            defines: Vec::new(),
            platform: Platform::Any,
            arch: Arch::X64,
            exception_handling: None,
            precompiled_header: None,
        }
    }

    /// Add a source file (relative path).
    #[must_use]
    pub fn source(mut self, path: impl Into<PathBuf>) -> Self {
        self.sources.push(path.into());
        self
    }

    /// Add multiple source files.
    #[must_use]
    pub fn sources(mut self, paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        self.sources.extend(paths.into_iter().map(Into::into));
        self
    }

    /// Add a header file (relative path).
    #[must_use]
    pub fn header(mut self, path: impl Into<PathBuf>) -> Self {
        self.headers.push(path.into());
        self
    }

    /// Add multiple header files.
    #[must_use]
    pub fn headers(mut self, paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        self.headers.extend(paths.into_iter().map(Into::into));
        self
    }

    /// Add an include directory (relative path).
    #[must_use]
    pub fn include_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_dirs.push(path.into());
        self
    }

    /// Add multiple include directories.
    #[must_use]
    pub fn include_dirs(mut self, paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        self.include_dirs.extend(paths.into_iter().map(Into::into));
        self
    }

    /// Add a preprocessor define with an optional value.
    #[must_use]
    pub fn define(mut self, name: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        self.defines.push((
            name.into(),
            value.map(|v| v.into()),
        ));
        self
    }

    /// Add multiple preprocessor defines.
    #[must_use]
    pub fn defines(mut self, defs: impl IntoIterator<Item = (impl Into<String>, Option<impl Into<String>>)>) -> Self {
        self.defines.extend(
            defs.into_iter().map(|(n, v)| (n.into(), v.map(|x| x.into())))
        );
        self
    }

    /// Set the target platform.
    #[must_use]
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// Set the target CPU architecture.
    #[must_use]
    pub fn arch(mut self, arch: Arch) -> Self {
        self.arch = arch;
        self
    }

    /// Set the C++ exception-handling model.
    #[must_use]
    pub fn exception_handling(mut self, eh: ExceptionHandling) -> Self {
        self.exception_handling = Some(eh);
        self
    }

    /// Set the precompiled header configuration.
    #[must_use]
    pub fn precompiled_header(mut self, pch: PrecompiledHeader) -> Self {
        self.precompiled_header = Some(pch);
        self
    }

    /// Populate sources and headers from rendered file pairs, classifying by
    /// extension (`.h`/`.hpp`/`.hxx` → headers; everything else → sources)
    /// and inferring `include_dirs` from parent directories.
    #[must_use]
    pub fn populate_from_files(mut self, files: &[(PathBuf, String)]) -> Self {
        let mut include_dirs_set: Vec<PathBuf> = Vec::new();
        for (path, _) in files {
            match path.extension().and_then(|e| e.to_str()) {
                Some("h") | Some("hpp") | Some("hxx") => {
                    self.headers.push(path.clone());
                }
                _ => {
                    self.sources.push(path.clone());
                }
            }
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
                && !include_dirs_set.iter().any(|p| p.as_path() == parent)
            {
                include_dirs_set.push(parent.to_path_buf());
            }
        }
        self.include_dirs = include_dirs_set;
        self
    }

    /// Consume the builder and produce a validated [`ProjectSpec`].
    pub fn build(self) -> Result<ProjectSpec, ProjectGenError> {
        let spec = ProjectSpec {
            name: self.name,
            language_standard: self.language_standard,
            sources: self.sources,
            headers: self.headers,
            include_dirs: self.include_dirs,
            defines: self.defines,
            platform: self.platform,
            arch: self.arch,
            output_kind: self.output_kind,
            exception_handling: self.exception_handling,
            precompiled_header: self.precompiled_header,
        };
        validate_spec(&spec)?;
        Ok(spec)
    }
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
    /// Requested C++ exception-handling model. `None` leaves it at the
    /// compiler default. Honored by the Visual Studio and CMake generators;
    /// ignored by Makefile/Cargo.
    pub exception_handling: Option<ExceptionHandling>,
    /// Precompiled-header configuration. `None` disables PCH. Honored by the
    /// Visual Studio and CMake generators; ignored by Makefile/Cargo.
    pub precompiled_header: Option<PrecompiledHeader>,
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
            exception_handling: None,
            precompiled_header: None,
        }
    }
    /// Populate sources and headers from a list of rendered files.
    ///
    /// Files are classified by extension: `.h`/`.hpp`/`.hxx` → headers,
    /// everything else → sources. Include directories are inferred from
    /// the parent directories of all files.
    ///
    /// This is a convenience helper — callers may still populate fields
    /// manually for full control.
    ///
    /// # Arguments
    ///
    /// * `files` - Pairs of (relative path, file content). Content is
    ///   discarded; only paths are used to populate the spec.
    #[must_use]
    pub fn populate_from_files(mut self, files: &[(PathBuf, String)]) -> Self {
        let mut include_dirs_set: Vec<PathBuf> = Vec::new();
        for (path, _) in files {
            match path.extension().and_then(|e| e.to_str()) {
                Some("h") | Some("hpp") | Some("hxx") => {
                    self.headers.push(path.clone());
                }
                _ => {
                    self.sources.push(path.clone());
                }
            }
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
                && !include_dirs_set.iter().any(|p| p.as_path() == parent)
            {
                include_dirs_set.push(parent.to_path_buf());
            }
        }
        self.include_dirs = include_dirs_set;
        self
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

    /// The precompiled-header configuration is inconsistent with the spec's
    /// source/header lists (its creator source or header is missing).
    #[error("invalid precompiled header: {reason}")]
    InvalidPrecompiledHeader {
        /// Why the precompiled-header configuration was rejected.
        reason: &'static str,
    },
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

/// `true` if `c` is allowed in a [`ProjectSpec`] name (`[A-Za-z0-9_.+-]`).
fn is_allowed_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-')
}

/// Sanitize an arbitrary string into a valid [`ProjectSpec`] name.
///
/// Every character outside `[A-Za-z0-9_.+-]` is replaced with `_`. An input
/// that is empty (or becomes empty) yields `"Project"`, so the result always
/// satisfies [`validate_spec`]'s non-empty and charset invariants.
///
/// # Arguments
///
/// * `raw` - The raw, possibly-invalid name (e.g. a game title).
///
/// # Returns
///
/// A name containing only `[A-Za-z0-9_.+-]`, never empty.
#[must_use]
pub fn sanitize_project_name(raw: &str) -> String {
    let sanitized: String = raw
        .chars()
        .map(|c| if is_allowed_name_char(c) { c } else { '_' })
        .collect();
    if sanitized.is_empty() {
        "Project".to_string()
    } else {
        sanitized
    }
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
    if !spec.name.chars().all(is_allowed_name_char) {
        return Err(ProjectGenError::InvalidName {
            name: spec.name.clone(),
            reason: "name may only contain characters in [A-Za-z0-9_.+-]",
        });
    }
    let pch_paths = spec
        .precompiled_header
        .iter()
        .flat_map(|pch| [&pch.header, &pch.create_source]);
    for path in spec
        .sources
        .iter()
        .chain(spec.headers.iter())
        .chain(spec.include_dirs.iter())
        .chain(pch_paths)
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
    if let Some(pch) = &spec.precompiled_header {
        if !spec.sources.contains(&pch.create_source) {
            return Err(ProjectGenError::InvalidPrecompiledHeader {
                reason: "create_source must be one of the spec's sources",
            });
        }
        if !spec.headers.contains(&pch.header) {
            return Err(ProjectGenError::InvalidPrecompiledHeader {
                reason: "header must be one of the spec's headers",
            });
        }
    }
    if spec.sources.is_empty() {
        return Err(ProjectGenError::NoSources(spec.name.clone()));
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
/// Write rendered source files to disk, creating parent directories as needed.
///
/// A convenience helper for the common workflow: render declarations to strings,
/// write them to files, then generate build files. Callers retain full control
/// over rendering (backend choice, options) and spec construction.
///
/// # Arguments
///
/// * `files` - Pairs of (relative path, file content).
/// * `output_dir` - The base directory to write into.
///
/// # Returns
///
/// `Ok(())` if all files were written successfully.
///
/// # Errors
///
/// Returns [`ProjectGenError::Io`] if any file write fails.
pub fn write_files(
    files: &[(PathBuf, String)],
    output_dir: &Path,
) -> Result<(), ProjectGenError> {
    for (path, content) in files {
        let full_path = output_dir.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProjectGenError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        fs::write(&full_path, content).map_err(|e| ProjectGenError::Io {
            path: full_path.clone(),
            source: e,
        })?;
    }
    Ok(())
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

    #[test]
    fn sanitize_project_name_replaces_invalid_characters() {
        assert_eq!(sanitize_project_name("Tom & Jerry!"), "Tom___Jerry_");
    }

    #[test]
    fn sanitize_project_name_maps_empty_to_project() {
        assert_eq!(sanitize_project_name(""), "Project");
    }

    #[test]
    fn sanitize_project_name_leaves_valid_name_unchanged() {
        assert_eq!(sanitize_project_name("Already_Valid-1.0+x"), "Already_Valid-1.0+x");
    }

    #[test]
    fn validate_accepts_sanitized_special_char_name() {
        let spec = ProjectSpec {
            name: sanitize_project_name("Tom & Jerry!"),
            ..valid_spec()
        };
        assert!(validate_spec(&spec).is_ok());
    }

    fn spec_with_pch() -> ProjectSpec {
        let mut spec = valid_spec();
        spec.sources.push(PathBuf::from("pch.cpp"));
        spec.headers.push(PathBuf::from("pch.h"));
        spec.precompiled_header = Some(PrecompiledHeader {
            header: PathBuf::from("pch.h"),
            create_source: PathBuf::from("pch.cpp"),
        });
        spec
    }

    #[test]
    fn validate_accepts_consistent_precompiled_header() {
        assert!(validate_spec(&spec_with_pch()).is_ok());
    }

    #[test]
    fn validate_rejects_pch_create_source_not_in_sources() {
        let mut spec = spec_with_pch();
        spec.sources.retain(|s| s != Path::new("pch.cpp"));
        let err = validate_spec(&spec).unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidPrecompiledHeader { .. }));
    }

    #[test]
    fn validate_rejects_pch_header_not_in_headers() {
        let mut spec = spec_with_pch();
        spec.headers.retain(|h| h != Path::new("pch.h"));
        let err = validate_spec(&spec).unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidPrecompiledHeader { .. }));
    }

    #[test]
    fn validate_rejects_pch_path_with_whitespace() {
        let mut spec = valid_spec();
        spec.sources.push(PathBuf::from("p ch.cpp"));
        spec.headers.push(PathBuf::from("p ch.h"));
        spec.precompiled_header = Some(PrecompiledHeader {
            header: PathBuf::from("p ch.h"),
            create_source: PathBuf::from("p ch.cpp"),
        });
        let err = validate_spec(&spec).unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidPath { .. }));
    }
    #[test]
    fn populate_from_files_classifies_headers_and_sources() {
        let spec = ProjectSpec::new("test", LanguageStandard::Cpp17, OutputKind::StaticLib)
            .populate_from_files(&[
                (PathBuf::from("src/main.cpp"), "int main()".into()),
                (PathBuf::from("src/utils.h"), "int foo()".into()),
                (PathBuf::from("include/types.hpp"), "struct S".into()),
                (PathBuf::from("src/types.cpp"), "S s".into()),
            ]);
        assert_eq!(spec.sources.len(), 2);
        assert!(spec.sources.contains(&PathBuf::from("src/main.cpp")));
        assert!(spec.sources.contains(&PathBuf::from("src/types.cpp")));
        assert_eq!(spec.headers.len(), 2);
        assert!(spec.headers.contains(&PathBuf::from("src/utils.h")));
        assert!(spec.headers.contains(&PathBuf::from("include/types.hpp")));
        assert_eq!(spec.include_dirs.len(), 2);
        assert!(spec.include_dirs.contains(&PathBuf::from("src")));
        assert!(spec.include_dirs.contains(&PathBuf::from("include")));
    }

    #[test]
    fn populate_from_files_skips_root_parent() {
        let spec = ProjectSpec::new("test", LanguageStandard::Cpp17, OutputKind::StaticLib)
            .populate_from_files(&[(PathBuf::from("main.cpp"), "x".into())]);
        assert!(spec.include_dirs.is_empty());
    }

    #[test]
    fn write_files_creates_dirs_and_writes_content() {
        let dir = tempfile::tempdir().unwrap();
        let files = [(PathBuf::from("src/main.cpp"), "int main()".into())];
        assert!(write_files(&files, dir.path()).is_ok());
        assert!(dir.path().join("src/main.cpp").exists());
        assert_eq!(fs::read_to_string(dir.path().join("src/main.cpp")).unwrap(), "int main()");
    }

    #[test]
    fn write_files_empty_list_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        assert!(write_files(&[], dir.path()).is_ok());
    }
    #[test]
    fn populate_from_files_classifies_hxx_as_header() {
        let spec = ProjectSpec::new("test", LanguageStandard::Cpp17, OutputKind::StaticLib)
            .populate_from_files(&[(PathBuf::from("foo.hxx"), "x".into())]);
        assert_eq!(spec.headers.len(), 1);
        assert!(spec.sources.is_empty());
    }

    #[test]
    fn populate_from_files_deduplicates_include_dirs() {
        let spec = ProjectSpec::new("test", LanguageStandard::Cpp17, OutputKind::StaticLib)
            .populate_from_files(&[
                (PathBuf::from("src/a.cpp"), "x".into()),
                (PathBuf::from("src/b.cpp"), "x".into()),
            ]);
        assert_eq!(spec.include_dirs.len(), 1);
    }

    #[test]
    fn write_files_creates_deeply_nested_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let files = [(PathBuf::from("a/b/c/d.cpp"), "x".into())];
        assert!(write_files(&files, dir.path()).is_ok());
        assert!(dir.path().join("a/b/c/d.cpp").exists());
    }
    #[test]
    fn builder_basic_build() {
        let spec = ProjectBuilder::new("my_lib", LanguageStandard::Cpp17, OutputKind::StaticLib)
            .source("src/main.cpp")
            .header("include/types.h")
            .define("DEBUG", Some("1"))
            .build()
            .unwrap();
        assert_eq!(spec.name, "my_lib");
        assert_eq!(spec.sources.len(), 1);
        assert_eq!(spec.headers.len(), 1);
        assert_eq!(spec.defines.len(), 1);
        assert_eq!(spec.language_standard, LanguageStandard::Cpp17);
    }

    #[test]
    fn builder_with_all_fields() {
        let spec = ProjectBuilder::new("full", LanguageStandard::Cpp20, OutputKind::SharedLib)
            .sources(["src/a.cpp", "src/b.cpp"])
            .headers(["inc/a.h", "inc/b.hpp"])
            .include_dirs(["inc"])
            .defines([("A", Some("1")), ("B", None::<&str>)])
            .platform(Platform::Windows)
            .arch(Arch::X86)
            .exception_handling(ExceptionHandling::Standard)
            .build()
            .unwrap();
        assert_eq!(spec.sources.len(), 2);
        assert_eq!(spec.headers.len(), 2);
        assert_eq!(spec.include_dirs.len(), 1);
        assert_eq!(spec.defines.len(), 2);
        assert_eq!(spec.platform, Platform::Windows);
        assert_eq!(spec.arch, Arch::X86);
        assert_eq!(spec.exception_handling, Some(ExceptionHandling::Standard));
    }

    #[test]
    fn builder_populate_from_files() {
        let spec = ProjectBuilder::new("test", LanguageStandard::Cpp17, OutputKind::StaticLib)
            .populate_from_files(&[
                (PathBuf::from("src/main.cpp"), "x".into()),
                (PathBuf::from("src/types.h"), "x".into()),
            ])
            .build()
            .unwrap();
        assert_eq!(spec.sources.len(), 1);
        assert_eq!(spec.headers.len(), 1);
        assert_eq!(spec.include_dirs.len(), 1);
    }

    #[test]
    fn builder_rust_project() {
        let spec = ProjectBuilder::new("my_crate", LanguageStandard::Rust2021, OutputKind::SharedLib)
            .source("src/lib.rs")
            .build()
            .unwrap();
        assert_eq!(spec.name, "my_crate");
        assert_eq!(spec.language_standard, LanguageStandard::Rust2021);
    }

    #[test]
    fn builder_csharp_project() {
        let spec = ProjectBuilder::new("MyLib", LanguageStandard::CSharp12, OutputKind::SharedLib)
            .source("Program.cs")
            .build()
            .unwrap();
        assert_eq!(spec.language_standard, LanguageStandard::CSharp12);
    }

    #[test]
    fn builder_rejects_empty_name() {
        let err = ProjectBuilder::new("", LanguageStandard::Cpp17, OutputKind::StaticLib)
            .source("main.cpp")
            .build()
            .unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidName { .. }));
    }

    #[test]
    fn builder_rejects_no_sources() {
        let err = ProjectBuilder::new("test", LanguageStandard::Cpp17, OutputKind::StaticLib)
            .build()
            .unwrap_err();
        assert!(matches!(err, ProjectGenError::NoSources(_)));
    }
}
