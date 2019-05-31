use deno::{PinnedBuf};
use deno::bindings::{BindingOpResult, BindingResult, new_binding_error};
use futures;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use cargo::ops::compile;
use cargo::ops::CompileOptions;
use cargo::util::homedir;
use cargo::util::config::Config;
use cargo::core::Workspace;
use cargo::core::SourceId;
use cargo::core::shell::Shell;
use cargo::core::manifest::Target;
use cargo::core::manifest::EitherManifest;
use cargo::core::compiler::CompileMode;
use cargo::core::compiler::Compilation;
use cargo::util::toml::read_manifest;

#[macro_use]
extern crate deno;

pub fn op_cargo_build(
    is_sync: bool,
    data: &[u8],
    _zero_copy: Option<PinnedBuf>,
) -> BindingOpResult {
    if !is_sync {
        return BindingOpResult::Async(Box::new(futures::future::err(new_binding_error(String::from("Async not supported!")))));
    }
    let data_str = std::str::from_utf8(&data[..]).unwrap();
    let build_config = BuildConfig::from_json(data_str);
    let build_result = match run_cargo_build(build_config) {
        Ok(br) => br,
        Err(err) => return BindingOpResult::Sync(Err(err)),
    };
    let result = serde_json::to_string(&build_result).unwrap();
    BindingOpResult::Sync(Ok(result.as_bytes().into()))
}

declare_binding_function!(cargo_build, op_cargo_build);

#[derive(Serialize, Deserialize)]
struct BuildConfigRaw {
    manifest_path: String,
    lib_only: bool,
    verbose: usize,
}

struct BuildConfig {
    pub manifest_path: PathBuf,
    pub lib_only: bool,
    pub verbose: usize,
}

impl BuildConfig {
    fn from_json(json: &str) -> Self {
        let json_value: BuildConfigRaw = serde_json::from_str(json).unwrap();
        Self {
            manifest_path: Path::new(&json_value.manifest_path).into(),
            lib_only: json_value.lib_only,
            verbose: json_value.verbose,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct BuildArtifact {
    pub output_name: String,
    pub is_lib: bool,
    pub is_dylib: bool,
    pub is_cdylib: bool,
}

impl BuildArtifact {
    fn from_target(target: &Target) -> Self {
        Self {
            output_name: target.crate_name(),
            is_lib: target.is_lib(),
            is_dylib: target.is_dylib(),
            is_cdylib: target.is_cdylib(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct BuildResult {
    pub output_root: String,
    pub artifacts: Vec<BuildArtifact>,
}

impl BuildResult {
    fn from_build_result(result: Compilation) -> Self {
        Self {
            output_root: result.root_output.to_str().unwrap().to_string(),
            artifacts: Vec::new(),
        }
    }
}

fn run_cargo_build(config: BuildConfig) -> BindingResult<BuildResult> {
    // TODO(afinch7) configure build with data from BuildConfig
    let manifest_path: PathBuf = config.manifest_path.into();
    let mut plugin_wd = manifest_path.clone();
    plugin_wd.pop();
    let shell = Shell::new();
    let home_dir = homedir(&plugin_wd).unwrap();

    let config = Config::new(shell, plugin_wd.clone(), home_dir);
    let manifest = read_manifest(&manifest_path, SourceId::for_directory(&plugin_wd).unwrap(), &config).unwrap();
    let manifest = match manifest.0 {
        EitherManifest::Real(man) => man,
        _ => unimplemented!(),
    };
    let ws = Workspace::new(&manifest_path, &config).unwrap();

    let mut compile_opts = CompileOptions::new(&ws.config(), CompileMode::Build).unwrap();
    compile_opts.build_config.release = true;

    let compile_result = compile(&ws, &compile_opts).unwrap();

    let mut build_result = BuildResult::from_build_result(compile_result);
    for target in manifest.targets() {
        let artifact = BuildArtifact::from_target(target);
        build_result.artifacts.push(artifact);
    }
    Ok(build_result)
}