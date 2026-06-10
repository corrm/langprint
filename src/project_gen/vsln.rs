//! Visual Studio solution generator — emits `<name>.sln`,
//! `<name>.vcxproj`, and `<name>.vcxproj.filters` (MSVC / xwin compatible).

use std::{fmt::Write as _, path::Path};

use super::{
    Arch, LanguageFamily, OutputKind, Platform, ProjectGenError, ProjectGenerator, ProjectSpec, format_define,
    path_to_back_slashes, xml_escape,
};

/// The well-known Visual C++ project-type GUID used in `.sln` entries.
const VCXPROJ_TYPE_GUID: &str = "{8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942}";

/// FNV-1a 64-bit offset basis.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64-bit prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Salt hashed after the name when deriving the low GUID half, so the two
/// 64-bit halves are not correlated.
const GUID_LOW_SALT: &[u8] = b"langprint.vsln.guid.low";

/// Generates a Visual Studio solution + MSBuild project for an SDK.
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

    /// The MSBuild `ConfigurationType` for an [`OutputKind`].
    fn configuration_type(kind: OutputKind) -> &'static str {
        match kind {
            OutputKind::SharedLib => "DynamicLibrary",
            OutputKind::StaticLib => "StaticLibrary",
            OutputKind::Executable => "Application",
        }
    }

    /// The MSBuild platform name for an [`Arch`].
    fn msbuild_platform(arch: Arch) -> &'static str {
        match arch {
            Arch::X64 => "x64",
            Arch::X86 => "Win32",
        }
    }

    /// Reject language families MSBuild's C/C++ toolchain cannot build.
    fn ensure_supported(spec: &ProjectSpec) -> Result<LanguageFamily, ProjectGenError> {
        match spec.language_standard.family() {
            family @ (LanguageFamily::C | LanguageFamily::Cpp) => Ok(family),
            LanguageFamily::CSharp | LanguageFamily::Rust => Err(ProjectGenError::UnsupportedLanguage {
                generator: "VslnGenerator",
                standard: spec.language_standard,
            }),
        }
    }

    /// Render the `<name>.sln` solution file.
    fn render_sln(spec: &ProjectSpec, guid: &str) -> String {
        let name = xml_escape(&spec.name);
        let plat = Self::msbuild_platform(spec.arch);
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

    /// Render the `<name>.vcxproj` MSBuild project file.
    fn render_vcxproj(spec: &ProjectSpec, family: LanguageFamily, guid: &str) -> String {
        let name = xml_escape(&spec.name);
        let plat = Self::msbuild_platform(spec.arch);
        let config_type = Self::configuration_type(spec.output_kind);

        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        out.push_str(
            "<Project DefaultTargets=\"Build\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">\n",
        );

        out.push_str("  <ItemGroup Label=\"ProjectConfigurations\">\n");
        for cfg in ["Debug", "Release"] {
            let _ = writeln!(out, "    <ProjectConfiguration Include=\"{cfg}|{plat}\">");
            let _ = writeln!(out, "      <Configuration>{cfg}</Configuration>");
            let _ = writeln!(out, "      <Platform>{plat}</Platform>");
            out.push_str("    </ProjectConfiguration>\n");
        }
        out.push_str("  </ItemGroup>\n");

        out.push_str("  <PropertyGroup Label=\"Globals\">\n");
        out.push_str("    <VCProjectVersion>17.0</VCProjectVersion>\n");
        let _ = writeln!(out, "    <ProjectGuid>{guid}</ProjectGuid>");
        let _ = writeln!(out, "    <RootNamespace>{name}</RootNamespace>");
        out.push_str("    <WindowsTargetPlatformVersion>10.0</WindowsTargetPlatformVersion>\n");
        out.push_str("  </PropertyGroup>\n");
        out.push_str("  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.Default.props\" />\n");

        for (cfg, debug_libs, whole_program) in [("Debug", "true", false), ("Release", "false", true)] {
            let _ = writeln!(
                out,
                "  <PropertyGroup Condition=\"'$(Configuration)|$(Platform)'=='{cfg}|{plat}'\" Label=\"Configuration\">"
            );
            let _ = writeln!(out, "    <ConfigurationType>{config_type}</ConfigurationType>");
            let _ = writeln!(out, "    <UseDebugLibraries>{debug_libs}</UseDebugLibraries>");
            if whole_program {
                out.push_str("    <WholeProgramOptimization>true</WholeProgramOptimization>\n");
            }
            out.push_str("    <PlatformToolset>v143</PlatformToolset>\n");
            out.push_str("    <CharacterSet>Unicode</CharacterSet>\n");
            out.push_str("  </PropertyGroup>\n");
        }
        out.push_str("  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.props\" />\n");

        out.push_str("  <ItemDefinitionGroup>\n");
        out.push_str("    <ClCompile>\n");
        match family {
            LanguageFamily::Cpp => {
                if let Some(standard) = spec.language_standard.msvc_language_standard() {
                    let _ = writeln!(out, "      <LanguageStandard>{standard}</LanguageStandard>");
                }
            }
            LanguageFamily::C => {
                if let Some(standard) = spec.language_standard.msvc_language_standard_c() {
                    let _ = writeln!(out, "      <LanguageStandard_C>{standard}</LanguageStandard_C>");
                }
            }
            LanguageFamily::CSharp | LanguageFamily::Rust => {}
        }
        if !spec.include_dirs.is_empty() {
            let dirs = spec
                .include_dirs
                .iter()
                .map(|d| xml_escape(&path_to_back_slashes(d)))
                .collect::<Vec<_>>()
                .join(";");
            let _ = writeln!(
                out,
                "      <AdditionalIncludeDirectories>{dirs};%(AdditionalIncludeDirectories)</AdditionalIncludeDirectories>"
            );
        }
        if !spec.defines.is_empty() {
            let defs = spec
                .defines
                .iter()
                .map(|(n, v)| xml_escape(&format_define(n, v.as_deref())))
                .collect::<Vec<_>>()
                .join(";");
            let _ = writeln!(
                out,
                "      <PreprocessorDefinitions>{defs};%(PreprocessorDefinitions)</PreprocessorDefinitions>"
            );
        }
        out.push_str("    </ClCompile>\n");
        out.push_str("  </ItemDefinitionGroup>\n");

        out.push_str("  <ItemGroup>\n");
        for source in &spec.sources {
            let _ = writeln!(
                out,
                "    <ClCompile Include=\"{}\" />",
                xml_escape(&path_to_back_slashes(source))
            );
        }
        out.push_str("  </ItemGroup>\n");
        if !spec.headers.is_empty() {
            out.push_str("  <ItemGroup>\n");
            for header in &spec.headers {
                let _ = writeln!(
                    out,
                    "    <ClInclude Include=\"{}\" />",
                    xml_escape(&path_to_back_slashes(header))
                );
            }
            out.push_str("  </ItemGroup>\n");
        }
        out.push_str("  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.targets\" />\n");
        out.push_str("</Project>\n");
        out
    }

    /// Render the `<name>.vcxproj.filters` file grouping sources under a
    /// `Source Files` filter and headers under a `Header Files` filter.
    fn render_filters(spec: &ProjectSpec, source_filter_guid: &str, header_filter_guid: &str) -> String {
        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        out.push_str("<Project ToolsVersion=\"4.0\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">\n");
        out.push_str("  <ItemGroup>\n");
        out.push_str("    <Filter Include=\"Source Files\">\n");
        let _ = writeln!(out, "      <UniqueIdentifier>{source_filter_guid}</UniqueIdentifier>");
        out.push_str("      <Extensions>cpp;c;cc;cxx</Extensions>\n");
        out.push_str("    </Filter>\n");
        out.push_str("    <Filter Include=\"Header Files\">\n");
        let _ = writeln!(out, "      <UniqueIdentifier>{header_filter_guid}</UniqueIdentifier>");
        out.push_str("      <Extensions>h;hh;hpp;hxx</Extensions>\n");
        out.push_str("    </Filter>\n");
        out.push_str("  </ItemGroup>\n");
        out.push_str("  <ItemGroup>\n");
        for source in &spec.sources {
            let _ = writeln!(
                out,
                "    <ClCompile Include=\"{}\">",
                xml_escape(&path_to_back_slashes(source))
            );
            out.push_str("      <Filter>Source Files</Filter>\n");
            out.push_str("    </ClCompile>\n");
        }
        out.push_str("  </ItemGroup>\n");
        if !spec.headers.is_empty() {
            out.push_str("  <ItemGroup>\n");
            for header in &spec.headers {
                let _ = writeln!(
                    out,
                    "    <ClInclude Include=\"{}\">",
                    xml_escape(&path_to_back_slashes(header))
                );
                out.push_str("      <Filter>Header Files</Filter>\n");
                out.push_str("    </ClInclude>\n");
            }
            out.push_str("  </ItemGroup>\n");
        }
        out.push_str("</Project>\n");
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
        let family = Self::ensure_supported(spec)?;
        let guid = deterministic_guid(&spec.name);
        let source_filter_guid = deterministic_guid(&format!("{}:Source Files", spec.name));
        let header_filter_guid = deterministic_guid(&format!("{}:Header Files", spec.name));

        super::write_file(
            output_dir,
            &format!("{}.sln", spec.name),
            &Self::render_sln(spec, &guid),
        )?;
        super::write_file(
            output_dir,
            &format!("{}.vcxproj", spec.name),
            &Self::render_vcxproj(spec, family, &guid),
        )?;
        super::write_file(
            output_dir,
            &format!("{}.vcxproj.filters", spec.name),
            &Self::render_filters(spec, &source_filter_guid, &header_filter_guid),
        )
    }
}

/// FNV-1a 64-bit hash of `bytes` starting from `seed`.
fn fnv1a(seed: u64, bytes: &[u8]) -> u64 {
    let mut hash = seed;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Derive a stable, registry-format, name-based (UUID-v5-style) GUID from
/// `name`.
///
/// The high half hashes `name`; the low half continues the hash over a fixed
/// salt — equivalent to hashing `name ‖ salt` — so the two halves are not
/// correlated. The RFC 4122 version nibble is set to `5` and the variant
/// bits to `10xx`. The same `name` always yields the same GUID (no
/// randomness, no clock), so regenerating a project does not churn the
/// solution and tests stay deterministic.
fn deterministic_guid(name: &str) -> String {
    let hi = fnv1a(FNV_OFFSET, name.as_bytes());
    let lo = fnv1a(hi, GUID_LOW_SALT);
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&hi.to_be_bytes());
    bytes[8..].copy_from_slice(&lo.to_be_bytes());
    bytes[6] = (bytes[6] & 0x0F) | 0x50;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    format!(
        "{{{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project_gen::{LanguageStandard, Platform};

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
        let contents = VslnGenerator::render_vcxproj(&sample_spec(), LanguageFamily::Cpp, &guid);
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
        let contents = VslnGenerator::render_filters(&sample_spec(), &source_filter_guid, &header_filter_guid);
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
        let contents = VslnGenerator::render_filters(&spec, &source_filter_guid, &header_filter_guid);
        assert!(!contents.contains("<ClInclude"));
        let vcxproj = VslnGenerator::render_vcxproj(&spec, LanguageFamily::Cpp, "{GUID}");
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
        let contents = VslnGenerator::render_vcxproj(&spec, LanguageFamily::Cpp, "{GUID}");
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
        let vcxproj = VslnGenerator::render_vcxproj(&spec, LanguageFamily::Cpp, "{GUID}");
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
        let contents = VslnGenerator::render_vcxproj(&spec, LanguageFamily::C, "{GUID}");
        assert!(contents.contains("      <LanguageStandard_C>stdc11</LanguageStandard_C>\n"));
        assert!(!contents.contains("<LanguageStandard>"));
    }

    #[test]
    fn c99_emits_no_language_standard_element() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::C99,
            ..sample_spec()
        };
        let contents = VslnGenerator::render_vcxproj(&spec, LanguageFamily::C, "{GUID}");
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
        let contents = VslnGenerator::render_vcxproj(&spec, LanguageFamily::Cpp, &guid);
        assert!(contents.contains("<ConfigurationType>StaticLibrary</ConfigurationType>"));
    }

    #[test]
    fn rejects_rust_standard() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::Rust2024,
            ..sample_spec()
        };
        let err = VslnGenerator::ensure_supported(&spec).unwrap_err();
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
}
