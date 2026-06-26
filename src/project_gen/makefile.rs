//! GNU Make project generator — emits a Unix `Makefile`.
//!
//! Header files are not listed in the Makefile — the compiler finds them via
//! the `-I` include directories. The spec's
//! [`exception_handling`](super::ProjectSpec::exception_handling) and
//! [`precompiled_header`](super::ProjectSpec::precompiled_header) are
//! intentionally NOT emitted: the GNU/Clang toolchain has no MSVC `/EH*` or
//! MSBuild precompiled-header analogue.

use std::{collections::BTreeSet, fmt::Write as _, path::Path};

use super::{
    Arch, LanguageFamily, OutputKind, Platform, ProjectGenError, ProjectGenerator, ProjectSpec,
    format_define, path_to_forward_slashes,
};

/// The compiler toolchain conventions for a single language family.
struct MakeToolchain {
    /// The compiler variable name (e.g. `CXX`).
    compiler_var: &'static str,
    /// The flags variable name (e.g. `CXXFLAGS`).
    flags_var: &'static str,
    /// The default compiler binary (e.g. `c++`).
    compiler_default: &'static str,
}

/// Generates a Linux-oriented GNU `Makefile`. Requires no `cmake` dependency.
/// Accepts [`Platform::Linux`] and [`Platform::Any`]; rejects
/// [`Platform::Windows`].
#[derive(Debug, Clone, Default)]
pub struct MakefileGenerator;

impl MakefileGenerator {
    /// Create a new [`MakefileGenerator`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Resolve the [`MakeToolchain`] for the spec's language family.
    fn toolchain(spec: &ProjectSpec) -> Result<MakeToolchain, ProjectGenError> {
        match spec.language_standard.family() {
            LanguageFamily::Cpp => Ok(MakeToolchain {
                compiler_var: "CXX",
                flags_var: "CXXFLAGS",
                compiler_default: "c++",
            }),
            LanguageFamily::C => Ok(MakeToolchain {
                compiler_var: "CC",
                flags_var: "CFLAGS",
                compiler_default: "cc",
            }),
            LanguageFamily::CSharp | LanguageFamily::Rust => {
                Err(ProjectGenError::UnsupportedLanguage {
                    generator: "MakefileGenerator",
                    standard: spec.language_standard,
                })
            }
        }
    }

    /// Build the value of the compiler flags variable for `spec`.
    fn flags(spec: &ProjectSpec) -> String {
        let mut flags = Vec::new();
        if let Some(std) = spec.language_standard.compiler_std_flag() {
            flags.push(format!("-std={std}"));
        }
        let arch_flag = match spec.arch {
            Arch::X64 => "-m64",
            Arch::X86 => "-m32",
        };
        flags.push(arch_flag.to_string());
        if spec.output_kind == OutputKind::SharedLib {
            flags.push("-fPIC".to_string());
        }
        for dir in &spec.include_dirs {
            flags.push(format!("-I{}", path_to_forward_slashes(dir)));
        }
        for (name, value) in &spec.defines {
            flags.push(format!("-D{}", format_define(name, value.as_deref())));
        }
        flags.join(" ")
    }

    /// Render the full `Makefile` contents for `spec`.
    fn render(&self, spec: &ProjectSpec) -> Result<String, ProjectGenError> {
        let tc = Self::toolchain(spec)?;
        let name = &spec.name;
        let compiler = tc.compiler_var;
        let flags_var = tc.flags_var;

        let target = match spec.output_kind {
            OutputKind::SharedLib => format!("lib{name}.so"),
            OutputKind::StaticLib => format!("lib{name}.a"),
            OutputKind::Executable => name.clone(),
        };

        let sources: Vec<String> = spec
            .sources
            .iter()
            .map(|s| path_to_forward_slashes(s))
            .collect();
        // Object names are computed here (not via a single `$(SRCS:.ext=.o)`
        // substitution) so mixed source extensions all land in `OBJS`.
        let objects: Vec<String> = spec
            .sources
            .iter()
            .map(|s| path_to_forward_slashes(&s.with_extension("o")))
            .collect();
        let extensions: BTreeSet<String> = spec
            .sources
            .iter()
            .filter_map(|s| s.extension())
            .map(|e| e.to_string_lossy().into_owned())
            .collect();

        let mut out = String::new();
        let _ = writeln!(out, "{compiler} ?= {}", tc.compiler_default);
        if spec.output_kind == OutputKind::StaticLib {
            out.push_str("AR ?= ar\n");
        }
        let _ = writeln!(out, "{flags_var} += {}", Self::flags(spec));
        out.push('\n');
        let _ = writeln!(out, "SRCS := {}", sources.join(" "));
        let _ = writeln!(out, "OBJS := {}", objects.join(" "));
        let _ = writeln!(out, "TARGET := {target}");
        out.push('\n');
        out.push_str("all: $(TARGET)\n\n");

        let _ = writeln!(out, "$(TARGET): $(OBJS)");
        match spec.output_kind {
            OutputKind::SharedLib => {
                let _ = writeln!(out, "\t$({compiler}) -shared -o $@ $(OBJS)");
            }
            OutputKind::StaticLib => {
                out.push_str("\t$(AR) rcs $@ $(OBJS)\n");
            }
            OutputKind::Executable => {
                let _ = writeln!(out, "\t$({compiler}) -o $@ $(OBJS)");
            }
        }
        out.push('\n');

        // One pattern rule per distinct source extension, sorted, so `.cc` /
        // `.cxx` sources are compiled with the flags instead of being passed
        // raw to the link line.
        for ext in &extensions {
            let _ = writeln!(out, "%.o: %.{ext}");
            let _ = writeln!(out, "\t$({compiler}) $({flags_var}) -c $< -o $@");
            out.push('\n');
        }

        out.push_str("clean:\n");
        out.push_str("\trm -f $(OBJS) $(TARGET)\n\n");
        out.push_str(".PHONY: all clean\n");

        Ok(out)
    }
}

impl ProjectGenerator for MakefileGenerator {
    fn generate(&self, spec: &ProjectSpec, output_dir: &Path) -> Result<(), ProjectGenError> {
        super::validate_spec(spec)?;
        if spec.platform == Platform::Windows {
            return Err(ProjectGenError::IncompatiblePlatform {
                generator: "MakefileGenerator",
                platform: spec.platform,
            });
        }
        if spec.sources.is_empty() {
            return Err(ProjectGenError::NoSources(spec.name.clone()));
        }
        let contents = self.render(spec)?;
        super::write_file(output_dir, "Makefile", &contents)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::project_gen::LanguageStandard;

    fn sample_spec() -> ProjectSpec {
        ProjectSpec {
            name: "VampireSurvivors".to_string(),
            language_standard: LanguageStandard::Cpp17,
            sources: vec![
                PathBuf::from("Assembly-CSharp.cpp"),
                PathBuf::from("UnityEngine.cpp"),
            ],
            headers: Vec::new(),
            include_dirs: vec![PathBuf::from("Headers")],
            defines: vec![
                ("IL2CPP".to_string(), None),
                ("VERSION".to_string(), Some("105".to_string())),
            ],
            platform: Platform::Any,
            arch: Arch::X64,
            output_kind: OutputKind::SharedLib,
            exception_handling: None,
            precompiled_header: None,
        }
    }

    #[test]
    fn renders_shared_library_makefile() {
        let contents = MakefileGenerator::new().render(&sample_spec()).unwrap();
        let expected = "\
CXX ?= c++
CXXFLAGS += -std=c++17 -m64 -fPIC -IHeaders -DIL2CPP -DVERSION=105

SRCS := Assembly-CSharp.cpp UnityEngine.cpp
OBJS := Assembly-CSharp.o UnityEngine.o
TARGET := libVampireSurvivors.so

all: $(TARGET)

$(TARGET): $(OBJS)
\t$(CXX) -shared -o $@ $(OBJS)

%.o: %.cpp
\t$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
\trm -f $(OBJS) $(TARGET)

.PHONY: all clean
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn renders_mixed_extension_sources() {
        let spec = ProjectSpec {
            sources: vec![
                PathBuf::from("Assembly-CSharp.cpp"),
                PathBuf::from("UnityEngine.cc"),
                PathBuf::from("legacy.cxx"),
            ],
            ..sample_spec()
        };
        let contents = MakefileGenerator::new().render(&spec).unwrap();
        let expected = "\
CXX ?= c++
CXXFLAGS += -std=c++17 -m64 -fPIC -IHeaders -DIL2CPP -DVERSION=105

SRCS := Assembly-CSharp.cpp UnityEngine.cc legacy.cxx
OBJS := Assembly-CSharp.o UnityEngine.o legacy.o
TARGET := libVampireSurvivors.so

all: $(TARGET)

$(TARGET): $(OBJS)
\t$(CXX) -shared -o $@ $(OBJS)

%.o: %.cc
\t$(CXX) $(CXXFLAGS) -c $< -o $@

%.o: %.cpp
\t$(CXX) $(CXXFLAGS) -c $< -o $@

%.o: %.cxx
\t$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
\trm -f $(OBJS) $(TARGET)

.PHONY: all clean
";
        assert_eq!(contents, expected);
    }

    #[test]
    fn renders_static_library_with_archiver() {
        let spec = ProjectSpec {
            output_kind: OutputKind::StaticLib,
            ..sample_spec()
        };
        let contents = MakefileGenerator::new().render(&spec).unwrap();
        assert!(contents.contains("AR ?= ar\n"));
        assert!(contents.contains("TARGET := libVampireSurvivors.a\n"));
        assert!(contents.contains("\t$(AR) rcs $@ $(OBJS)\n"));
        // Static archives do not need position-independent code.
        assert!(!contents.contains("-fPIC"));
    }

    #[test]
    fn renders_executable_target() {
        let spec = ProjectSpec {
            output_kind: OutputKind::Executable,
            ..sample_spec()
        };
        let contents = MakefileGenerator::new().render(&spec).unwrap();
        assert!(contents.contains("TARGET := VampireSurvivors\n"));
        assert!(contents.contains("\t$(CXX) -o $@ $(OBJS)\n"));
    }

    #[test]
    fn uses_c_toolchain_for_c_standard() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::C11,
            sources: vec![PathBuf::from("main.c")],
            output_kind: OutputKind::Executable,
            ..sample_spec()
        };
        let contents = MakefileGenerator::new().render(&spec).unwrap();
        assert!(contents.starts_with("CC ?= cc\nCFLAGS += -std=c11 -m64 -IHeaders"));
        assert!(contents.contains("OBJS := main.o\n"));
        assert!(contents.contains("%.o: %.c\n"));
    }

    #[test]
    fn x86_arch_uses_m32_flag() {
        let spec = ProjectSpec {
            arch: Arch::X86,
            ..sample_spec()
        };
        let contents = MakefileGenerator::new().render(&spec).unwrap();
        assert!(contents.contains("CXXFLAGS += -std=c++17 -m32 -fPIC"));
    }

    #[test]
    fn rejects_csharp_standard() {
        let spec = ProjectSpec {
            language_standard: LanguageStandard::CSharp12,
            ..sample_spec()
        };
        let err = MakefileGenerator::new().render(&spec).unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::UnsupportedLanguage {
                generator: "MakefileGenerator",
                ..
            }
        ));
    }

    #[test]
    fn rejects_windows_platform() {
        let spec = ProjectSpec {
            platform: Platform::Windows,
            ..sample_spec()
        };
        let dir = tempfile::tempdir().unwrap();
        let err = MakefileGenerator::new()
            .generate(&spec, dir.path())
            .unwrap_err();
        assert!(matches!(
            err,
            ProjectGenError::IncompatiblePlatform {
                generator: "MakefileGenerator",
                platform: Platform::Windows,
            }
        ));
    }
}
