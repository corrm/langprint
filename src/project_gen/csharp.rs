//! C# project generator — emits an SDK-style `.csproj`.
//!
//! Like [`CargoGenerator`](super::CargoGenerator), the .NET SDK discovers
//! sources by convention (all `*.cs` under the project directory), so this
//! generator does not list [`sources`](ProjectSpec::sources),
//! [`headers`](ProjectSpec::headers), or
//! [`include_dirs`](ProjectSpec::include_dirs). The spec's
//! [`Arch`](super::Arch) is a build-time choice (`dotnet build -r`), and
//! [`exception_handling`](ProjectSpec::exception_handling) /
//! [`precompiled_header`](ProjectSpec::precompiled_header) have no C# analogue,
//! so they are intentionally not emitted. The manifest is driven by the
//! project name, the C# language version, and the [`OutputKind`].

use std::{fmt::Write as _, path::Path};

use super::{OutputKind, ProjectGenError, ProjectGenerator, ProjectSpec, xml_escape};

/// The target framework moniker emitted by [`CSharpProjectGenerator`].
const TARGET_FRAMEWORK: &str = "net8.0";

/// Generates an SDK-style `.csproj` for an SDK built as a C# project.
#[derive(Debug, Clone, Default)]
pub struct CSharpProjectGenerator;

impl CSharpProjectGenerator {
    /// Create a new [`CSharpProjectGenerator`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Render the `.csproj` contents for `spec`, rejecting any spec whose
    /// language standard is not a C# version.
    fn render(spec: &ProjectSpec) -> Result<String, ProjectGenError> {
        let Some(lang_version) = spec.language_standard.csharp_lang_version() else {
            return Err(ProjectGenError::UnsupportedLanguage {
                generator: "CSharpProjectGenerator",
                standard: spec.language_standard,
            });
        };

        let output_type = match spec.output_kind {
            OutputKind::Executable => "Exe",
            OutputKind::SharedLib | OutputKind::StaticLib => "Library",
        };

        let mut out = String::new();
        out.push_str("<Project Sdk=\"Microsoft.NET.Sdk\">\n");
        out.push_str("  <PropertyGroup>\n");
        let _ = writeln!(out, "    <OutputType>{output_type}</OutputType>");
        let _ = writeln!(out, "    <TargetFramework>{TARGET_FRAMEWORK}</TargetFramework>");
        let _ = writeln!(out, "    <AssemblyName>{}</AssemblyName>", xml_escape(&spec.name));
        let _ = writeln!(out, "    <LangVersion>{lang_version}</LangVersion>");
        out.push_str("    <Nullable>enable</Nullable>\n");
        out.push_str("    <ImplicitUsings>enable</ImplicitUsings>\n");

        if !spec.defines.is_empty() {
            let symbols = spec
                .defines
                .iter()
                .map(|(name, _)| xml_escape(name))
                .collect::<Vec<_>>()
                .join(";");
            let _ = writeln!(out, "    <DefineConstants>{symbols}</DefineConstants>");
        }

        out.push_str("  </PropertyGroup>\n");
        out.push_str("</Project>\n");

        Ok(out)
    }
}

impl ProjectGenerator for CSharpProjectGenerator {
    fn generate(&self, spec: &ProjectSpec, output_dir: &Path) -> Result<(), ProjectGenError> {
        super::validate_spec(spec)?;
        let contents = Self::render(spec)?;
        super::write_file(output_dir, &format!("{}.csproj", spec.name), &contents)
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
            exception_handling: None,
            precompiled_header: None,
        }
    }

    #[test]
    fn renders_library_with_lang_version() {
        let contents =
            CSharpProjectGenerator::render(&spec_with(LanguageStandard::CSharp12, OutputKind::SharedLib)).unwrap();
        let expected = "\
<Project Sdk=\"Microsoft.NET.Sdk\">
  <PropertyGroup>
    <OutputType>Library</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <AssemblyName>VampireSurvivors</AssemblyName>
    <LangVersion>12</LangVersion>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>
</Project>
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn renders_exe_output_type() {
        let contents =
            CSharpProjectGenerator::render(&spec_with(LanguageStandard::CSharp11, OutputKind::Executable)).unwrap();
        assert!(contents.contains("<OutputType>Exe</OutputType>"));
        assert!(contents.contains("<LangVersion>11</LangVersion>"));
    }

    #[test]
    fn emits_define_constants() {
        let mut spec = spec_with(LanguageStandard::CSharp12, OutputKind::SharedLib);
        spec.defines.push(("TRACE".to_string(), None));
        spec.defines.push(("VERSION_105".to_string(), None));
        let contents = CSharpProjectGenerator::render(&spec).unwrap();
        assert!(contents.contains("<DefineConstants>TRACE;VERSION_105</DefineConstants>"));
    }

    #[test]
    fn rejects_non_csharp_standard() {
        let err =
            CSharpProjectGenerator::render(&spec_with(LanguageStandard::Cpp20, OutputKind::SharedLib)).unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::UnsupportedLanguage {
                generator: "CSharpProjectGenerator",
                ..
            }
        ));
    }
}
