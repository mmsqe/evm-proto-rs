use std::path::{Path, PathBuf};
use std::{fs, process};

use regex::Regex;
use similar::TextDiff;
use walkdir::WalkDir;
use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// EVM Proto Compiler
struct App {
    #[argh(subcommand)]
    cmd: Command,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum Command {
    Compile(CompileCmd),
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "compile")]
/// Compile EVM proto files to Rust
pub struct CompileCmd {
    #[argh(switch, short = 't')]
    /// generate transport client/server code
    transport: bool,

    #[argh(option, short = 'e')]
    /// path to the Evm proto files
    evm: PathBuf,

    #[argh(option, short = 'c')]
    /// path to the Cosmos SDK proto files (optional)
    cosmos_sdk: Option<PathBuf>,

    #[argh(option, short = 'o')]
    /// path to output the generated Rust sources into
    out: PathBuf,
}

impl CompileCmd {
    pub fn run(&self) {
        Self::compile_protos(
            self.transport,
            self.evm.as_ref(),
            self.cosmos_sdk.as_deref(),
            self.out.as_ref(),
        )
        .unwrap_or_else(|e| {
            eprintln!("[error] failed to compile protos: {}", e);
            process::exit(1);
        });

        Self::patch_generated_files(self.out.as_ref()).unwrap_or_else(|e| {
            eprintln!("[error] failed to patch generated files: {}", e);
            process::exit(1);
        });

        Self::build_pbjson_impls(self.out.as_ref()).unwrap_or_else(|e| {
            eprintln!("[error] failed to build pbjson impls: {}", e);
            process::exit(1);
        });

        println!("[info ] Done!");
    }

    fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    fn compile_protos(
        transport: bool,
        evm_dir: &Path,
        cosmos_sdk_dir: Option<&Path>,
        out_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[info ] Compiling EVM .proto files to Rust into '{}'...",
            out_dir.display()
        );

        let (_temp_dir, proto_paths, proto_includes_paths) = if let Some(cosmos_sdk_dir) = cosmos_sdk_dir {
            println!("[info ] Including Cosmos SDK proto files from '{}'", cosmos_sdk_dir.display());
            
            // Create a temporary directory to merge proto files without conflicts
            let temp_dir = tempfile::tempdir()?;
            let merged_proto_dir = temp_dir.path();
            
            // Copy cosmos SDK protos first
            Self::copy_dir_recursive(cosmos_sdk_dir, merged_proto_dir)?;
            
            // Copy EVM protos, overwriting any conflicts (EVM takes priority for EVM-specific files)
            let evm_cosmos_dir = evm_dir.join("cosmos");
            if evm_cosmos_dir.exists() {
                let target_cosmos_dir = merged_proto_dir.join("cosmos");
                Self::copy_dir_recursive(&evm_cosmos_dir, &target_cosmos_dir)?;
            }
            
            // Include the entire cosmos directory to find all proto files
            let proto_paths = vec![merged_proto_dir.join("cosmos")];
            
            let proto_includes_paths = vec![merged_proto_dir.to_path_buf()];
            
            (Some(temp_dir), proto_paths, proto_includes_paths)
        } else {
            let proto_paths = vec![
                evm_dir.join("cosmos"),
            ];
            let proto_includes_paths = vec![evm_dir.to_path_buf()];
            (None, proto_paths, proto_includes_paths)
        };

        // List available proto files
        let mut protos: Vec<PathBuf> = vec![];
        for proto_path in &proto_paths {
            println!("Looking for proto files in '{}'", proto_path.display());

            protos.append(
                &mut WalkDir::new(proto_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_type().is_file()
                            && e.path().extension().is_some()
                            && e.path().extension().unwrap() == "proto"
                    })
                    .map(|e| e.into_path())
                    .collect(),
            );
        }

        println!("Found the following protos:");

        // Show which protos will be compiled
        for proto in &protos {
            println!("\t-> {}", proto.display());
        }

        println!("[info ] Compiling..");

        // Automatically derive a `prost::Name` implementation.
        let mut config = prost_build::Config::new();
        config.enable_type_names();

        tonic_build::configure()
            .build_transport(transport)
            .build_client(true)
            .compile_well_known_types(true)
            .client_mod_attribute(".", r#"#[cfg(feature = "client")]"#)
            .build_server(true)
            .server_mod_attribute(".", r#"#[cfg(feature = "server")]"#)
            .out_dir(out_dir)
            .file_descriptor_set_path(out_dir.join("proto_descriptor.bin"))
            .extern_path(".google.protobuf.Any", "::cosmos_sdk_proto::Any")
            .extern_path(".google.protobuf.Timestamp", "::cosmos_sdk_proto::Timestamp")
            .extern_path(".google.protobuf.Duration", "::tendermint_proto::google::protobuf::Duration")
            .extern_path(".cosmos.base", "::cosmos_sdk_proto::cosmos::base")
            .extern_path(".cosmos.bank", "::cosmos_sdk_proto::cosmos::bank")
            .extern_path(".tendermint", "::tendermint_proto")
            .compile_protos_with_config(config, &protos, &proto_includes_paths)?;

        println!("[info ] Protos compiled successfully");

        Ok(())
    }

    fn build_pbjson_impls(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("[info ] Building pbjson Serialize, Deserialize impls...");

        let descriptor_set_path = out_dir.join("proto_descriptor.bin");
        let descriptor_set = std::fs::read(descriptor_set_path)?;

        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .out_dir(out_dir)
            .emit_fields()
            .build(&[".cosmos.evm"])?;

        Ok(())
    }

    fn patch_generated_files(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[info ] Patching generated files in '{}'...",
            out_dir.display()
        );

        const PATCHES: &[(&str, &[(&str, &str)])] = &[
            // No EVM-specific patches needed currently
        ];

        for (file, patches) in PATCHES {
            println!("[info ] Patching {file}...");

            let path = out_dir.join(file);
            let original = std::fs::read_to_string(&path)?;
            let mut patched = original.clone();

            for (before, after) in patches.iter() {
                patched = patched.replace(before, after);
            }

            let diff = TextDiff::from_lines(&original, &patched);
            println!("{}", diff.unified_diff().context_radius(3));

            std::fs::write(&path, patched)?;
        }

        // patches applied to all generated files
        println!("applying global patches");
        const GLOBAL_REPLACEMENTS: &[(&str, &str)] = &[
            // Feature-gate gRPC impls which use `tonic::transport`
            (
                "impl(.+)tonic::transport(.+)",
                "#[cfg(feature = \"transport\")]\n    \
                impl${1}tonic::transport${2}",
            ),
        ];

        let files_iter = WalkDir::new(out_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension() == Some("rs".as_ref()));

        for file in files_iter {
            println!("patching file: {:?}", file.path());
            let mut contents = fs::read_to_string(file.path())?;

            for &(regex, replacement) in GLOBAL_REPLACEMENTS {
                contents = Regex::new(regex)
                    .unwrap_or_else(|_| panic!("invalid regex: {}", regex))
                    .replace_all(&contents, replacement)
                    .to_string();
            }

            fs::write(file.path(), contents)?;
        }
        println!("finished global patches");

        Ok(())
    }
}

fn main() {
    let app: App = argh::from_env();

    match app.cmd {
        Command::Compile(compile) => compile.run(),
    }
}