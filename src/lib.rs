/// The version (commit hash) of Evm used when generating this library.
pub const EVM_COMMIT: &str = include_str!("EVM_COMMIT");

#[macro_export]
macro_rules! include_proto {
    ($path:literal) => {
        include!(concat!("prost/", $path));
    };
}

// Proto modules will be included here after running the compiler
// Run: cargo run -- compile -e <evm_proto_path> -o src/prost
// to generate the proto files

#[cfg(feature = "proto-generated")]
pub mod evm {
    pub mod crypto {
        pub mod v1 {
            pub mod ethsecp256k1 {
                include_proto!("cosmos.evm.crypto.v1.ethsecp256k1.rs");
            }
        }
    }
    pub mod vm {
        pub mod v1 {
            include_proto!("cosmos.evm.vm.v1.rs");
        }
    }
    pub mod erc20 {
        pub mod v1 {
            include_proto!("cosmos.evm.erc20.v1.rs");
        }
    }
    pub mod feemarket {
        pub mod v1 {
            include_proto!("cosmos.evm.feemarket.v1.rs");
        }
    }
    pub mod types {
        pub mod v1 {
            include_proto!("cosmos.evm.types.v1.rs");
        }
    }
}

