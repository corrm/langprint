//! Shared Visual Studio MSBuild rendering used by every VS-family generator.
//!
//! Both [`VslnGenerator`](super::vsln::VslnGenerator) (legacy `.sln`) and
//! [`SlnxGenerator`](super::vsln::SlnxGenerator) (modern `.slnx`) emit the
//! same `<name>.vcxproj` and `<name>.vcxproj.filters`. That rendering, the
//! deterministic name-based GUID derivation, and the MSBuild platform / output
//! mapping live here exactly once so neither generator duplicates it.

use std::fmt::Write as _;

use super::{
    Arch, LanguageFamily, OutputKind, ProjectGenError, ProjectSpec, format_define,
    path_to_back_slashes, xml_escape,
};

/// FNV-1a 64-bit offset basis.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64-bit prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Salt hashed after the name when deriving the low GUID half, so the two
/// 64-bit halves are not correlated.
const GUID_LOW_SALT: &[u8] = b"langprint.vsln.guid.low";

/// The MSBuild `ConfigurationType` for an [`OutputKind`].
pub(crate) fn configuration_type(kind: OutputKind) -> &'static str {
    match kind {
        OutputKind::SharedLib => "DynamicLibrary",
        OutputKind::StaticLib => "StaticLibrary",
        OutputKind::Executable => "Application",
    }
}

/// The MSBuild platform name for an [`Arch`].
pub(crate) fn msbuild_platform(arch: Arch) -> &'static str {
    match arch {
        Arch::X64 => "x64",
        Arch::X86 => "Win32",
    }
}

/// Reject language families MSBuild's C/C++ toolchain cannot build.
///
/// # Arguments
///
/// * `spec` - The spec whose language standard is checked.
/// * `generator` - The generator name, embedded in the error for diagnostics.
///
/// # Returns
///
/// The [`LanguageFamily`] (C or C++) on success, or
/// [`ProjectGenError::UnsupportedLanguage`] for C# and Rust.
pub(crate) fn ensure_supported(
    spec: &ProjectSpec,
    generator: &'static str,
) -> Result<LanguageFamily, ProjectGenError> {
    match spec.language_standard.family() {
        family @ (LanguageFamily::C | LanguageFamily::Cpp) => Ok(family),
        LanguageFamily::CSharp | LanguageFamily::Rust => {
            Err(ProjectGenError::UnsupportedLanguage {
                generator,
                standard: spec.language_standard,
            })
        }
    }
}

/// Render the `<name>.vcxproj` MSBuild project file.
pub(crate) fn render_vcxproj(spec: &ProjectSpec, family: LanguageFamily, guid: &str) -> String {
    let name = xml_escape(&spec.name);
    let plat = msbuild_platform(spec.arch);
    let config_type = configuration_type(spec.output_kind);

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<Project DefaultTargets=\"Build\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">\n");

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
        let _ = writeln!(
            out,
            "    <ConfigurationType>{config_type}</ConfigurationType>"
        );
        let _ = writeln!(
            out,
            "    <UseDebugLibraries>{debug_libs}</UseDebugLibraries>"
        );
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
                let _ = writeln!(
                    out,
                    "      <LanguageStandard_C>{standard}</LanguageStandard_C>"
                );
            }
        }
        LanguageFamily::CSharp | LanguageFamily::Rust => {}
    }
    if let Some(handling) = spec.exception_handling {
        let _ = writeln!(
            out,
            "      <ExceptionHandling>{}</ExceptionHandling>",
            handling.msbuild_value()
        );
    }
    if let Some(pch) = &spec.precompiled_header {
        out.push_str("      <PrecompiledHeader>Use</PrecompiledHeader>\n");
        let _ = writeln!(
            out,
            "      <PrecompiledHeaderFile>{}</PrecompiledHeaderFile>",
            xml_escape(&path_to_back_slashes(&pch.header))
        );
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
        let include = xml_escape(&path_to_back_slashes(source));
        let is_pch_creator = spec
            .precompiled_header
            .as_ref()
            .is_some_and(|pch| source == &pch.create_source);
        if is_pch_creator {
            let _ = writeln!(out, "    <ClCompile Include=\"{include}\">");
            out.push_str("      <PrecompiledHeader>Create</PrecompiledHeader>\n");
            out.push_str("    </ClCompile>\n");
        } else {
            let _ = writeln!(out, "    <ClCompile Include=\"{include}\" />");
        }
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
pub(crate) fn render_filters(
    spec: &ProjectSpec,
    source_filter_guid: &str,
    header_filter_guid: &str,
) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<Project ToolsVersion=\"4.0\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">\n");
    out.push_str("  <ItemGroup>\n");
    out.push_str("    <Filter Include=\"Source Files\">\n");
    let _ = writeln!(
        out,
        "      <UniqueIdentifier>{source_filter_guid}</UniqueIdentifier>"
    );
    out.push_str("      <Extensions>cpp;c;cc;cxx</Extensions>\n");
    out.push_str("    </Filter>\n");
    out.push_str("    <Filter Include=\"Header Files\">\n");
    let _ = writeln!(
        out,
        "      <UniqueIdentifier>{header_filter_guid}</UniqueIdentifier>"
    );
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
pub(crate) fn deterministic_guid(name: &str) -> String {
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
