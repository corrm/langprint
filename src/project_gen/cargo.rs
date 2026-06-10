//! Cargo project generator — emits a `Cargo.toml`.
//!
//! Cargo discovers sources by convention (`src/`), so unlike the other
//! generators this one does not list [`sources`](ProjectSpec::sources),
//! [`headers`](ProjectSpec::headers), or
//! [`include_dirs`](ProjectSpec::include_dirs) — those have no Cargo
//! equivalent. The spec's [`Arch`](super::Arch) is not baked in either: for
//! Cargo the architecture is a build-time choice (`cargo build --target`).
//! The manifest is driven by the project name, the Rust edition, and the
//! [`OutputKind`].

use std::{fmt::Write as _, path::Path};

use super::{OutputKind, ProjectGenError, ProjectGenerator, ProjectSpec};

/// Generates a `Cargo.toml` manifest for an SDK built as a Rust crate.
#[derive(Debug, Clone, Default)]
pub struct CargoGenerator;

impl CargoGenerator {
    /// Create a new [`CargoGenerator`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Render the `Cargo.toml` contents for `spec`, rejecting any spec whose
    /// language standard is not a Rust edition.
    fn render(spec: &ProjectSpec) -> Result<String, ProjectGenError> {
        let Some(edition) = spec.language_standard.rust_edition() else {
            return Err(ProjectGenError::UnsupportedLanguage {
                generator: "CargoGenerator",
                standard: spec.language_standard,
            });
        };
        let name = &spec.name;

        let mut out = String::new();
        out.push_str("[package]\n");
        let _ = writeln!(out, "name = \"{name}\"");
        out.push_str("version = \"0.1.0\"\n");
        let _ = writeln!(out, "edition = \"{edition}\"");
        out.push('\n');

        match spec.output_kind {
            OutputKind::SharedLib => {
                out.push_str("[lib]\n");
                out.push_str("crate-type = [\"cdylib\"]\n");
            }
            OutputKind::StaticLib => {
                out.push_str("[lib]\n");
                out.push_str("crate-type = [\"staticlib\"]\n");
            }
            OutputKind::Executable => {
                out.push_str("[[bin]]\n");
                let _ = writeln!(out, "name = \"{name}\"");
                out.push_str("path = \"src/main.rs\"\n");
            }
        }

        Ok(out)
    }
}

impl ProjectGenerator for CargoGenerator {
    fn generate(&self, spec: &ProjectSpec, output_dir: &Path) -> Result<(), ProjectGenError> {
        super::validate_spec(spec)?;
        let contents = Self::render(spec)?;
        super::write_file(output_dir, "Cargo.toml", &contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_gen::{Arch, LanguageStandard, Platform};

    fn spec_with(language_standard: LanguageStandard, output_kind: OutputKind) -> ProjectSpec {
        ProjectSpec {
            name: "VampireSurvivors".to_string(),
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

    #[test]
    fn renders_cdylib_with_rust_edition() {
        let contents = CargoGenerator::render(&spec_with(LanguageStandard::Rust2024, OutputKind::SharedLib)).unwrap();
        let expected = "\
[package]
name = \"VampireSurvivors\"
version = \"0.1.0\"
edition = \"2024\"

[lib]
crate-type = [\"cdylib\"]
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn renders_staticlib() {
        let contents = CargoGenerator::render(&spec_with(LanguageStandard::Rust2021, OutputKind::StaticLib)).unwrap();
        assert!(contents.contains("edition = \"2021\"\n"));
        assert!(contents.contains("[lib]\ncrate-type = [\"staticlib\"]\n"));
    }

    #[test]
    fn renders_binary_target() {
        let contents = CargoGenerator::render(&spec_with(LanguageStandard::Rust2021, OutputKind::Executable)).unwrap();
        let expected = "\
[package]
name = \"VampireSurvivors\"
version = \"0.1.0\"
edition = \"2021\"

[[bin]]
name = \"VampireSurvivors\"
path = \"src/main.rs\"
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn rejects_non_rust_standard() {
        let err = CargoGenerator::render(&spec_with(LanguageStandard::Cpp20, OutputKind::SharedLib)).unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::UnsupportedLanguage {
                generator: "CargoGenerator",
                ..
            }
        ));
    }
}
