//! Visual Studio solution generators.
//!
//! [`VslnGenerator`] emits the legacy `<name>.sln`, while [`SlnxGenerator`]
//! emits the modern XML `<name>.slnx` (VS 2022 17.10+). Both additionally emit
//! the identical `<name>.vcxproj` and `<name>.vcxproj.filters`, rendered once
//! in [`super::vs_common`] (MSVC / xwin compatible).

use std::{fmt::Write as _, path::Path};

use super::{Platform, ProjectGenError, ProjectGenerator, ProjectSpec, vs_common, xml_escape};

/// The well-known Visual C++ project-type GUID used in `.sln` entries.
const VCXPROJ_TYPE_GUID: &str = "{8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942}";

/// Generates a legacy Visual Studio solution + MSBuild project for an SDK.
/// Accepts [`Platform::Windows`] and [`Platform::Any`]; rejects
/// [`Platform::Linux`].
#[derive(Debug, Clone, Default)]
pub struct VslnGenerator;

impl VslnGenerator {
    /// Create a new [`VslnGenerator`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Render the `<name>.sln` solution file.
    fn render_sln(spec: &ProjectSpec, guid: &str) -> String {
        let name = xml_escape(&spec.name);
        let plat = vs_common::msbuild_platform(spec.arch);
        let mut out = String::new();
        out.push_str("Microsoft Visual Studio Solution File, Format Version 12.00\n");
        out.push_str("# Visual Studio Version 17\n");
        out.push_str("VisualStudioVersion = 17.0.31903.59\n");
        out.push_str("MinimumVisualStudioVersion = 10.0.40219.1\n");
        let _ = writeln!(
            out,
            "Project(\"{VCXPROJ_TYPE_GUID}\") = \"{name}\", \"{name}.vcxproj\", \"{guid}\""
        );
        out.push_str("EndProject\n");
        out.push_str("Global\n");
        out.push_str("\tGlobalSection(SolutionConfigurationPlatforms) = preSolution\n");
        let _ = writeln!(out, "\t\tDebug|{plat} = Debug|{plat}");
        let _ = writeln!(out, "\t\tRelease|{plat} = Release|{plat}");
        out.push_str("\tEndGlobalSection\n");
        out.push_str("\tGlobalSection(ProjectConfigurationPlatforms) = postSolution\n");
        let _ = writeln!(out, "\t\t{guid}.Debug|{plat}.ActiveCfg = Debug|{plat}");
        let _ = writeln!(out, "\t\t{guid}.Debug|{plat}.Build.0 = Debug|{plat}");
        let _ = writeln!(out, "\t\t{guid}.Release|{plat}.ActiveCfg = Release|{plat}");
        let _ = writeln!(out, "\t\t{guid}.Release|{plat}.Build.0 = Release|{plat}");
        out.push_str("\tEndGlobalSection\n");
        out.push_str("\tGlobalSection(SolutionProperties) = preSolution\n");
        out.push_str("\t\tHideSolutionNode = FALSE\n");
        out.push_str("\tEndGlobalSection\n");
        out.push_str("EndGlobal\n");
        out
    }
}

impl ProjectGenerator for VslnGenerator {
    fn generate(&self, spec: &ProjectSpec, output_dir: &Path) -> Result<(), ProjectGenError> {
        super::validate_spec(spec)?;
        if spec.platform == Platform::Linux {
            return Err(ProjectGenError::IncompatiblePlatform {
                generator: "VslnGenerator",
                platform: spec.platform,
            });
        }
        if spec.sources.is_empty() {
            return Err(ProjectGenError::NoSources(spec.name.clone()));
        }
        let family = vs_common::ensure_supported(spec, "VslnGenerator")?;
        let guid = vs_common::deterministic_guid(&spec.name);
        let source_filter_guid = vs_common::deterministic_guid(&format!("{}:Source Files", spec.name));
        let header_filter_guid = vs_common::deterministic_guid(&format!("{}:Header Files", spec.name));

        super::write_file(
            output_dir,
            &format!("{}.sln", spec.name),
            &Self::render_sln(spec, &guid),
        )?;
        super::write_file(
            output_dir,
            &format!("{}.vcxproj", spec.name),
            &vs_common::render_vcxproj(spec, family, &guid),
        )?;
        super::write_file(
            output_dir,
            &format!("{}.vcxproj.filters", spec.name),
            &vs_common::render_filters(spec, &source_filter_guid, &header_filter_guid),
        )
    }
}

/// Generates a modern XML Visual Studio solution (`<name>.slnx`, VS 2022
/// 17.10+) plus the same MSBuild project an [`VslnGenerator`] emits. Accepts
/// [`Platform::Windows`] and [`Platform::Any`]; rejects [`Platform::Linux`].
#[derive(Debug, Clone, Default)]
pub struct SlnxGenerator;

impl SlnxGenerator {
    /// Create a new [`SlnxGenerator`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Render the modern `<name>.slnx` XML solution file.
    ///
    /// The format needs no GUIDs — it references the project purely by its
    /// `<name>.vcxproj` path.
    fn render_slnx(spec: &ProjectSpec) -> String {
        let project_path = xml_escape(&format!("{}.vcxproj", spec.name));
        let mut out = String::new();
        out.push_str("<Solution>\n");
        let _ = writeln!(out, "  <Project Path=\"{project_path}\" />");
        out.push_str("</Solution>\n");
        out
    }
}

impl ProjectGenerator for SlnxGenerator {
    fn generate(&self, spec: &ProjectSpec, output_dir: &Path) -> Result<(), ProjectGenError> {
        super::validate_spec(spec)?;
        if spec.platform == Platform::Linux {
            return Err(ProjectGenError::IncompatiblePlatform {
                generator: "SlnxGenerator",
                platform: spec.platform,
            });
        }
        if spec.sources.is_empty() {
            return Err(ProjectGenError::NoSources(spec.name.clone()));
        }
        let family = vs_common::ensure_supported(spec, "SlnxGenerator")?;
        let guid = vs_common::deterministic_guid(&spec.name);
        let source_filter_guid = vs_common::deterministic_guid(&format!("{}:Source Files", spec.name));
        let header_filter_guid = vs_common::deterministic_guid(&format!("{}:Header Files", spec.name));

        super::write_file(output_dir, &format!("{}.slnx", spec.name), &Self::render_slnx(spec))?;
        super::write_file(
            output_dir,
            &format!("{}.vcxproj", spec.name),
            &vs_common::render_vcxproj(spec, family, &guid),
        )?;
        super::write_file(
            output_dir,
            &format!("{}.vcxproj.filters", spec.name),
            &vs_common::render_filters(spec, &source_filter_guid, &header_filter_guid),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project_gen::{Arch, LanguageFamily, LanguageStandard, OutputKind, vs_common::deterministic_guid};

    fn sample_spec() -> ProjectSpec {
        ProjectSpec {
            name: "VampireSurvivors".to_string(),
            language_standard: LanguageStandard::Cpp17,
            sources: vec![PathBuf::from("Assembly-CSharp.cpp"), PathBuf::from("UnityEngine.cpp")],
            headers: vec![PathBuf::from("Headers/SDK.hpp")],
            include_dirs: vec![PathBuf::from("Headers")],
            defines: vec![
                ("IL2CPP".to_string(), None),
                ("VERSION".to_string(), Some("105".to_string())),
            ],
            platform: Platform::Windows,
            arch: Arch::X64,
            output_kind: OutputKind::SharedLib,
        }
    }

    #[test]
    fn guid_is_deterministic_and_well_formed() {
        let a = deterministic_guid("VampireSurvivors");
        let b = deterministic_guid("VampireSurvivors");
        assert_eq!(a, b);
        assert_ne!(a, deterministic_guid("OtherGame"));
        // {8-4-4-4-12} hex groups, braces included → 38 chars.
        assert_eq!(a.len(), 38);
        assert!(a.starts_with('{') && a.ends_with('}'));
        let inner = &a[1..a.len() - 1];
        let groups: Vec<&str> = inner.split('-').collect();
        assert_eq!(groups.iter().map(|g| g.len()).collect::<Vec<_>>(), vec![8, 4, 4, 4, 12]);
        assert!(inner.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
        // RFC 4122: version nibble 5, variant bits `10xx` (hex 8/9/A/B).
        assert!(groups[2].starts_with('5'));
        assert!(matches!(groups[3].chars().next(), Some('8' | '9' | 'A' | 'B')));
    }

    #[test]
    fn renders_full_solution() {
        let guid = deterministic_guid("VampireSurvivors");
        let contents = VslnGenerator::render_sln(&sample_spec(), &guid);
        let expected = format!(
            "\
Microsoft Visual Studio Solution File, Format Version 12.00
# Visual Studio Version 17
VisualStudioVersion = 17.0.31903.59
MinimumVisualStudioVersion = 10.0.40219.1
Project(\"{VCXPROJ_TYPE_GUID}\") = \"VampireSurvivors\", \"VampireSurvivors.vcxproj\", \"{guid}\"
EndProject
Global
\tGlobalSection(SolutionConfigurationPlatforms) = preSolution
\t\tDebug|x64 = Debug|x64
\t\tRelease|x64 = Release|x64
\tEndGlobalSection
\tGlobalSection(ProjectConfigurationPlatforms) = postSolution
\t\t{guid}.Debug|x64.ActiveCfg = Debug|x64
\t\t{guid}.Debug|x64.Build.0 = Debug|x64
\t\t{guid}.Release|x64.ActiveCfg = Release|x64
\t\t{guid}.Release|x64.Build.0 = Release|x64
\tEndGlobalSection
\tGlobalSection(SolutionProperties) = preSolution
\t\tHideSolutionNode = FALSE
\tEndGlobalSection
EndGlobal
"
        );
        assert_eq!(contents, expected);
    }

    #[test]
    fn renders_full_vcxproj() {
        let guid = deterministic_guid("VampireSurvivors");
        let contents = vs_common::render_vcxproj(&sample_spec(), LanguageFamily::Cpp, &guid);
        let expected = format!(
            "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>
<Project DefaultTargets=\"Build\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">
  <ItemGroup Label=\"ProjectConfigurations\">
    <ProjectConfiguration Include=\"Debug|x64\">
      <Configuration>Debug</Configuration>
      <Platform>x64</Platform>
    </ProjectConfiguration>
    <ProjectConfiguration Include=\"Release|x64\">
      <Configuration>Release</Configuration>
      <Platform>x64</Platform>
    </ProjectConfiguration>
  </ItemGroup>
  <PropertyGroup Label=\"Globals\">
    <VCProjectVersion>17.0</VCProjectVersion>
    <ProjectGuid>{guid}</ProjectGuid>
    <RootNamespace>VampireSurvivors</RootNamespace>
    <WindowsTargetPlatformVersion>10.0</WindowsTargetPlatformVersion>
  </PropertyGroup>
  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.Default.props\" />
  <PropertyGroup Condition=\"'$(Configuration)|$(Platform)'=='Debug|x64'\" Label=\"Configuration\">
    <ConfigurationType>DynamicLibrary</ConfigurationType>
    <UseDebugLibraries>true</UseDebugLibraries>
    <PlatformToolset>v143</PlatformToolset>
    <CharacterSet>Unicode</CharacterSet>
  </PropertyGroup>
  <PropertyGroup Condition=\"'$(Configuration)|$(Platform)'=='Release|x64'\" Label=\"Configuration\">
    <ConfigurationType>DynamicLibrary</ConfigurationType>
    <UseDebugLibraries>false</UseDebugLibraries>
    <WholeProgramOptimization>true</WholeProgramOptimization>
    <PlatformToolset>v143</PlatformToolset>
    <CharacterSet>Unicode</CharacterSet>
  </PropertyGroup>
  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.props\" />
  <ItemDefinitionGroup>
    <ClCompile>
      <LanguageStandard>stdcpp17</LanguageStandard>
      <AdditionalIncludeDirectories>Headers;%(AdditionalIncludeDirectories)</AdditionalIncludeDirectories>
      <PreprocessorDefinitions>IL2CPP;VERSION=105;%(PreprocessorDefinitions)</PreprocessorDefinitions>
    </ClCompile>
  </ItemDefinitionGroup>
  <ItemGroup>
    <ClCompile Include=\"Assembly-CSharp.cpp\" />
    <ClCompile Include=\"UnityEngine.cpp\" />
  </ItemGroup>
  <ItemGroup>
    <ClInclude Include=\"Headers\\SDK.hpp\" />
  </ItemGroup>
  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.targets\" />
</Project>
"
        );
        assert_eq!(contents, expected);
    }

    #[test]
    fn renders_full_filters() {
        let source_filter_guid = deterministic_guid("VampireSurvivors:Source Files");
        let header_filter_guid = deterministic_guid("VampireSurvivors:Header Files");
        let contents = vs_common::render_filters(&sample_spec(), &source_filter_guid, &header_filter_guid);
        let expected = format!(
            "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>
<Project ToolsVersion=\"4.0\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">
  <ItemGroup>
    <Filter Include=\"Source Files\">
      <UniqueIdentifier>{source_filter_guid}</UniqueIdentifier>
      <Extensions>cpp;c;cc;cxx</Extensions>
    </Filter>
    <Filter Include=\"Header Files\">
      <UniqueIdentifier>{header_filter_guid}</UniqueIdentifier>
      <Extensions>h;hh;hpp;hxx</Extensions>
    </Filter>
  </ItemGroup>
  <ItemGroup>
    <ClCompile Include=\"Assembly-CSharp.cpp\">
      <Filter>Source Files</Filter>
    </ClCompile>
    <ClCompile Include=\"UnityEngine.cpp\">
      <Filter>Source Files</Filter>
    </ClCompile>
  </ItemGroup>
  <ItemGroup>
    <ClInclude Include=\"Headers\\SDK.hpp\">
      <Filter>Header Files</Filter>
    </ClInclude>
  </ItemGroup>
</Project>
"
        );
        assert_eq!(contents, expected);
    }

    #[test]
    fn filters_without_headers_omit_clinclude_group() {
        let spec = ProjectSpec {
            headers: Vec::new(),
            ..sample_spec()
        };
        let source_filter_guid = deterministic_guid("VampireSurvivors:Source Files");
        let header_filter_guid = deterministic_guid("VampireSurvivors:Header Files");
        let contents = vs_common::render_filters(&spec, &source_filter_guid, &header_filter_guid);
        assert!(!contents.contains("<ClInclude"));
        let vcxproj = vs_common::render_vcxproj(&spec, LanguageFamily::Cpp, "{GUID}");
        assert!(!vcxproj.contains("<ClInclude"));
    }

    #[test]
    fn escapes_xml_special_characters() {
        let spec = ProjectSpec {
            name: "Tom&Jerry".to_string(),
            sources: vec![PathBuf::from("A&B.cpp")],
            headers: vec![PathBuf::from("A&B.hpp")],
            include_dirs: vec![PathBuf::from("Inc<1>")],
            defines: vec![("MSG".to_string(), Some("\"hi\"".to_string()))],
            ..sample_spec()
        };
        let contents = vs_common::render_vcxproj(&spec, LanguageFamily::Cpp, "{GUID}");
        let expected = "\
<?xml version=\"1.0\" encoding=\"utf-8\"?>
<Project DefaultTargets=\"Build\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">
  <ItemGroup Label=\"ProjectConfigurations\">
    <ProjectConfiguration Include=\"Debug|x64\">
      <Configuration>Debug</Configuration>
      <Platform>x64</Platform>
    </ProjectConfiguration>
    <ProjectConfiguration Include=\"Release|x64\">
      <Configuration>Release</Configuration>
      <Platform>x64</Platform>
    </ProjectConfiguration>
  </ItemGroup>
  <PropertyGroup Label=\"Globals\">
    <VCProjectVersion>17.0</VCProjectVersion>
    <ProjectGuid>{GUID}</ProjectGuid>
    <RootNamespace>Tom&amp;Jerry</RootNamespace>
    <WindowsTargetPlatformVersion>10.0</WindowsTargetPlatformVersion>
  </PropertyGroup>
  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.Default.props\" />
  <PropertyGroup Condition=\"'$(Configuration)|$(Platform)'=='Debug|x64'\" Label=\"Configuration\">
    <ConfigurationType>DynamicLibrary</ConfigurationType>
    <UseDebugLibraries>true</UseDebugLibraries>
    <PlatformToolset>v143</PlatformToolset>
    <CharacterSet>Unicode</CharacterSet>
  </PropertyGroup>
  <PropertyGroup Condition=\"'$(Configuration)|$(Platform)'=='Release|x64'\" Label=\"Configuration\">
    <ConfigurationType>DynamicLibrary</ConfigurationType>
    <UseDebugLibraries>false</UseDebugLibraries>
    <WholeProgramOptimization>true</WholeProgramOptimization>
    <PlatformToolset>v143</PlatformToolset>
    <CharacterSet>Unicode</CharacterSet>
  </PropertyGroup>
  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.props\" />
  <ItemDefinitionGroup>
    <ClCompile>
      <LanguageStandard>stdcpp17</LanguageStandard>
      <AdditionalIncludeDirectories>Inc&lt;1&gt;;%(AdditionalIncludeDirectories)</AdditionalIncludeDirectories>
      <PreprocessorDefinitions>MSG=&quot;hi&quot;;%(PreprocessorDefinitions)</PreprocessorDefinitions>
    </ClCompile>
  </ItemDefinitionGroup>
  <ItemGroup>
    <ClCompile Include=\"A&amp;B.cpp\" />
  </ItemGroup>
  <ItemGroup>
    <ClInclude Include=\"A&amp;B.hpp\" />
  </ItemGroup>
  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.targets\" />
</Project>
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn x86_arch_uses_win32_platform_name() {
        let spec = ProjectSpec {
            arch: Arch::X86,
            ..sample_spec()
        };
        let sln = VslnGenerator::render_sln(&spec, "{GUID}");
        assert!(sln.contains("\t\tDebug|Win32 = Debug|Win32\n"));
        assert!(sln.contains("\t\t{GUID}.Release|Win32.Build.0 = Release|Win32\n"));
        let vcxproj = vs_common::render_vcxproj(&spec, LanguageFamily::Cpp, "{GUID}");
        assert!(vcxproj.contains("<ProjectConfiguration Include=\"Debug|Win32\">"));
        assert!(vcxproj.contains("      <Platform>Win32</Platform>\n"));
        assert!(vcxproj.contains("'$(Configuration)|$(Platform)'=='Release|Win32'"));
        assert!(!vcxproj.contains("x64"));
    }

    #[test]
    fn c_family_emits_language_standard_c() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::C11,
            ..sample_spec()
        };
        let contents = vs_common::render_vcxproj(&spec, LanguageFamily::C, "{GUID}");
        assert!(contents.contains("      <LanguageStandard_C>stdc11</LanguageStandard_C>\n"));
        assert!(!contents.contains("<LanguageStandard>"));
    }

    #[test]
    fn c99_emits_no_language_standard_element() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::C99,
            ..sample_spec()
        };
        let contents = vs_common::render_vcxproj(&spec, LanguageFamily::C, "{GUID}");
        assert!(!contents.contains("<LanguageStandard_C>"));
        assert!(!contents.contains("<LanguageStandard>"));
    }

    #[test]
    fn static_library_configuration_type() {
        let spec = ProjectSpec {
            output_kind: OutputKind::StaticLib,
            ..sample_spec()
        };
        let guid = deterministic_guid(&spec.name);
        let contents = vs_common::render_vcxproj(&spec, LanguageFamily::Cpp, &guid);
        assert!(contents.contains("<ConfigurationType>StaticLibrary</ConfigurationType>"));
    }

    #[test]
    fn rejects_rust_standard() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::Rust2024,
            ..sample_spec()
        };
        let err = vs_common::ensure_supported(&spec, "VslnGenerator").unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::UnsupportedLanguage {
                generator: "VslnGenerator",
                ..
            }
        ));
    }

    #[test]
    fn rejects_linux_platform() {
        let spec = ProjectSpec {
            platform: Platform::Linux,
            ..sample_spec()
        };
        let dir = std::env::temp_dir();
        let err = VslnGenerator::new().generate(&spec, &dir).unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::IncompatiblePlatform {
                generator: "VslnGenerator",
                platform: Platform::Linux,
            }
        ));
    }

    #[test]
    fn slnx_renders_modern_solution() {
        let contents = SlnxGenerator::render_slnx(&sample_spec());
        let expected = "\
<Solution>
  <Project Path=\"VampireSurvivors.vcxproj\" />
</Solution>
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn slnx_vcxproj_and_filters_byte_identical_to_vsln() {
        let spec = sample_spec();
        let vsln_dir = std::env::temp_dir().join("langprint_vsln_eq_vsln");
        let slnx_dir = std::env::temp_dir().join("langprint_vsln_eq_slnx");
        std::fs::create_dir_all(&vsln_dir).unwrap();
        std::fs::create_dir_all(&slnx_dir).unwrap();
        VslnGenerator::new().generate(&spec, &vsln_dir).unwrap();
        SlnxGenerator::new().generate(&spec, &slnx_dir).unwrap();

        for file in ["VampireSurvivors.vcxproj", "VampireSurvivors.vcxproj.filters"] {
            let from_vsln = std::fs::read_to_string(vsln_dir.join(file)).unwrap();
            let from_slnx = std::fs::read_to_string(slnx_dir.join(file)).unwrap();
            assert_eq!(from_vsln, from_slnx, "{file} differs between the two VS generators");
        }
    }

    #[test]
    fn slnx_rejects_rust_standard() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::Rust2024,
            ..sample_spec()
        };
        let err = SlnxGenerator::new().generate(&spec, &std::env::temp_dir()).unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::UnsupportedLanguage {
                generator: "SlnxGenerator",
                ..
            }
        ));
    }

    #[test]
    fn slnx_rejects_linux_platform() {
        let spec = ProjectSpec {
            platform: Platform::Linux,
            ..sample_spec()
        };
        let err = SlnxGenerator::new().generate(&spec, &std::env::temp_dir()).unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::IncompatiblePlatform {
                generator: "SlnxGenerator",
                platform: Platform::Linux,
            }
        ));
    }
}
