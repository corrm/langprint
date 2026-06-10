//! CMake project generator — emits a `CMakeLists.txt`.
//!
//! The spec's [`Arch`](super::Arch) is not baked in: for CMake the
//! architecture is a configure-time choice (`-A`, a toolchain file, or the
//! compiler default). Header files are not listed either — CMake finds them
//! via the include directories.

use std::{fmt::Write as _, path::Path};

use super::{
    LanguageFamily, OutputKind, ProjectGenError, ProjectGenerator, ProjectSpec, format_define, path_to_forward_slashes,
};

/// Default minimum CMake version. `3.20` is the first release where
/// `target_compile_features(... cxx_std_23)` and friends are reliable.
const DEFAULT_MINIMUM_VERSION: &str = "3.20";

/// Generates a cross-platform `CMakeLists.txt` (Linux + Windows); any
/// [`Platform`](super::Platform) is accepted.
#[derive(Debug, Clone)]
pub struct CmakeGenerator {
    /// The `cmake_minimum_required` version.
    minimum_version: String,
}

impl Default for CmakeGenerator {
    fn default() -> Self {
        Self {
            minimum_version: DEFAULT_MINIMUM_VERSION.to_string(),
        }
    }
}

impl CmakeGenerator {
    /// Create a generator using the default minimum CMake version (`3.20`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a generator with a specific `cmake_minimum_required` version.
    ///
    /// # Arguments
    ///
    /// * `version` - The minimum CMake version string (e.g. `"3.25"`).
    #[must_use]
    pub fn with_minimum_version(version: impl Into<String>) -> Self {
        Self {
            minimum_version: version.into(),
        }
    }

    /// The CMake `LANGUAGES` keyword for the spec's language family.
    fn cmake_language(spec: &ProjectSpec) -> Result<&'static str, ProjectGenError> {
        match spec.language_standard.family() {
            LanguageFamily::C => Ok("C"),
            LanguageFamily::Cpp => Ok("CXX"),
            LanguageFamily::CSharp | LanguageFamily::Rust => Err(ProjectGenError::UnsupportedLanguage {
                generator: "CmakeGenerator",
                standard: spec.language_standard,
            }),
        }
    }

    /// Render the full `CMakeLists.txt` contents for `spec`.
    fn render(&self, spec: &ProjectSpec) -> Result<String, ProjectGenError> {
        let language = Self::cmake_language(spec)?;
        let name = &spec.name;

        let mut out = String::new();
        let _ = writeln!(out, "cmake_minimum_required(VERSION {})", self.minimum_version);
        let _ = writeln!(out, "project({name} LANGUAGES {language})");
        out.push('\n');

        let target_command = match spec.output_kind {
            OutputKind::SharedLib => format!("add_library({name} SHARED"),
            OutputKind::StaticLib => format!("add_library({name} STATIC"),
            OutputKind::Executable => format!("add_executable({name}"),
        };
        let _ = writeln!(out, "{target_command}");
        for source in &spec.sources {
            let _ = writeln!(out, "    \"{}\"", path_to_forward_slashes(source));
        }
        out.push_str(")\n");

        if let Some(feature) = spec.language_standard.cmake_compile_feature() {
            let _ = writeln!(out, "target_compile_features({name} PUBLIC {feature})");
        }

        if !spec.include_dirs.is_empty() {
            let _ = writeln!(out, "target_include_directories({name} PUBLIC");
            for dir in &spec.include_dirs {
                let _ = writeln!(
                    out,
                    "    \"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\"",
                    path_to_forward_slashes(dir)
                );
            }
            out.push_str(")\n");
        }

        if !spec.defines.is_empty() {
            let _ = writeln!(out, "target_compile_definitions({name} PUBLIC");
            for (define_name, value) in &spec.defines {
                let _ = writeln!(out, "    {}", format_define(define_name, value.as_deref()));
            }
            out.push_str(")\n");
        }

        Ok(out)
    }
}

impl ProjectGenerator for CmakeGenerator {
    fn generate(&self, spec: &ProjectSpec, output_dir: &Path) -> Result<(), ProjectGenError> {
        super::validate_spec(spec)?;
        if spec.sources.is_empty() {
            return Err(ProjectGenError::NoSources(spec.name.clone()));
        }
        let contents = self.render(spec)?;
        super::write_file(output_dir, "CMakeLists.txt", &contents)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project_gen::{Arch, LanguageStandard, Platform};

    fn sample_spec() -> ProjectSpec {
        ProjectSpec {
            name: "VampireSurvivors".to_string(),
            language_standard: LanguageStandard::Cpp17,
            sources: vec![PathBuf::from("Assembly-CSharp.cpp"), PathBuf::from("UnityEngine.cpp")],
            headers: Vec::new(),
            include_dirs: vec![PathBuf::from("Headers")],
            defines: vec![
                ("IL2CPP".to_string(), None),
                ("VERSION".to_string(), Some("105".to_string())),
            ],
            platform: Platform::Any,
            arch: Arch::X64,
            output_kind: OutputKind::SharedLib,
        }
    }

    #[test]
    fn renders_shared_library_project() {
        let contents = CmakeGenerator::new().render(&sample_spec()).unwrap();
        let expected = "\
cmake_minimum_required(VERSION 3.20)
project(VampireSurvivors LANGUAGES CXX)

add_library(VampireSurvivors SHARED
    \"Assembly-CSharp.cpp\"
    \"UnityEngine.cpp\"
)
target_compile_features(VampireSurvivors PUBLIC cxx_std_17)
target_include_directories(VampireSurvivors PUBLIC
    \"${CMAKE_CURRENT_SOURCE_DIR}/Headers\"
)
target_compile_definitions(VampireSurvivors PUBLIC
    IL2CPP
    VERSION=105
)
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn renders_executable_without_optional_sections() {
        let spec = ProjectSpec {
            include_dirs: Vec::new(),
            defines: Vec::new(),
            output_kind: OutputKind::Executable,
            ..sample_spec()
        };
        let contents = CmakeGenerator::new().render(&spec).unwrap();
        let expected = "\
cmake_minimum_required(VERSION 3.20)
project(VampireSurvivors LANGUAGES CXX)

add_executable(VampireSurvivors
    \"Assembly-CSharp.cpp\"
    \"UnityEngine.cpp\"
)
target_compile_features(VampireSurvivors PUBLIC cxx_std_17)
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn honors_custom_minimum_version() {
        let contents = CmakeGenerator::with_minimum_version("3.25")
            .render(&sample_spec())
            .unwrap();
        assert!(contents.starts_with("cmake_minimum_required(VERSION 3.25)\n"));
    }

    #[test]
    fn rejects_rust_standard() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::Rust2021,
            ..sample_spec()
        };
        let err = CmakeGenerator::new().render(&spec).unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::UnsupportedLanguage {
                generator: "CmakeGenerator",
                ..
            }
        ));
    }

    #[test]
    fn empty_sources_is_an_error() {
        let spec = ProjectSpec {
            sources: Vec::new(),
            ..sample_spec()
        };
        let dir = std::env::temp_dir();
        let err = CmakeGenerator::new().generate(&spec, &dir).unwrap_err();
        assert!(matches!(err, ProjectGenError::NoSources(name) if name == "VampireSurvivors"));
    }

    #[test]
    fn generate_rejects_invalid_spec() {
        let spec = ProjectSpec {
            name: "Bad Name".to_string(),
            ..sample_spec()
        };
        let dir = std::env::temp_dir();
        let err = CmakeGenerator::new().generate(&spec, &dir).unwrap_err();
        assert!(matches!(err, ProjectGenError::InvalidName { name, .. } if name == "Bad Name"));
    }
}
