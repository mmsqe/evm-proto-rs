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
pub mod cosmos {
    pub mod authz {
        pub mod v1beta1 {
            include_proto!("cosmos.authz.v1beta1.rs");
        }
    }
    pub mod bank {
        pub mod v1beta1 {
            include_proto!("cosmos.bank.v1beta1.rs");
        }
    }
    pub mod distribution {
        pub mod v1beta1 {
            include_proto!("cosmos.distribution.v1beta1.rs");
        }
    }
    pub mod gov {
        pub mod v1 {
            include_proto!("cosmos.gov.v1.rs");
        }
        pub mod v1beta1 {
            include_proto!("cosmos.gov.v1beta1.rs");
        }
    }
    pub mod slashing {
        pub mod v1beta1 {
            include_proto!("cosmos.slashing.v1beta1.rs");
        }
    }
    pub mod base {
        pub mod tendermint {
            pub mod v1beta1 {
                include_proto!("cosmos.base.tendermint.v1beta1.rs");
            }
        }
    }
    pub mod tx {
        pub mod v1beta1 {
            include_proto!("cosmos.tx.v1beta1.rs");
        }
        pub mod signing {
            pub mod v1beta1 {
                include_proto!("cosmos.tx.signing.v1beta1.rs");
            }
        }
    }
    pub mod crypto {
        pub mod secp256k1 {
            include_proto!("cosmos.crypto.secp256k1.rs");
        }
        pub mod ed25519 {
            include_proto!("cosmos.crypto.ed25519.rs");
        }
        pub mod multisig {
            include_proto!("cosmos.crypto.multisig.rs");
            pub mod v1beta1 {
                include_proto!("cosmos.crypto.multisig.v1beta1.rs");
            }
        }
    }
}

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

