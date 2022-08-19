use std::{
    collections::HashSet,
    env,
    fs::{canonicalize, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

use autotools::Config;

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

const REF: &str = if cfg!(feature = "vendored-wolfssl540") {
    "v5.4.0-stable"
} else if cfg!(feature = "vendored-wolfssl530") {
    "v5.3.0-stable"
} else if cfg!(feature = "vendored-wolfssl520") {
    "v5.2.0-stable"
} else if cfg!(feature = "vendored-wolfssl510") {
    "v5.1.0-stable"
} else if cfg!(feature = "vendored-wolfssl430") {
    "v4.3.0-stable"
} else {
    "master"
};

fn clone_wolfssl(dest: &str) -> std::io::Result<()> {
    std::fs::remove_dir_all(dest)?;
    Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(REF)
        .arg("https://github.com/wolfSSL/wolfssl.git")
        .arg(dest)
        .status()?;

    Ok(())
}

fn build_wolfssl(dest: &str) -> PathBuf {
    let cc = "clang".to_owned();

    let mut config = Config::new(dest);

    config
        .reconf("-ivf")
        .enable_static()
        .disable_shared()
        .enable("debug", None)
        .enable("opensslextra", None)
        .enable("context-extra-user-data", None)
        .enable("keygen", None) // Support for RSA certs
        .enable("certgen", None) // Support x509 decoding
        .enable("tls13", None)
        .enable("aesni", None)
        .enable("dtls", None)
        .enable("sp", None)
        .enable("sp-asm", None)
        .enable("dtls-mtu", None)
        .disable("sha3", None)
        .enable("intelasm", None)
        .enable("curve25519", None)
        .enable("secure-renegotiation", None)
        .enable("postauth", None)
        .cflag("-fPIC");

    config.env("CC", cc).build()
}

fn main() -> std::io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    clone_wolfssl(&out_dir)?;
    let dst = build_wolfssl(&out_dir);

    // Block some macros:https://github.com/rust-lang/rust-bindgen/issues/687
    let mut ignored_macros = HashSet::new();
    for i in &[
        "IPPORT_RESERVED",
        "EVP_PKEY_DH",
        "BIO_CLOSE",
        "BIO_NOCLOSE",
        "CRYPTO_LOCK",
        "ASN1_STRFLGS_ESC_MSB",
        "SSL_MODE_RELEASE_BUFFERS",
        // Woflss 4.3.0
        "GEN_IPADD",
        "EVP_PKEY_RSA",
    ] {
        ignored_macros.insert(i.to_string());
    }
    let ignored_macros = IgnoreMacros(ignored_macros);

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .header(format!("{}/wolfssl/internal.h", out_dir))
        .clang_arg(format!("-I{}/include/", out_dir))
        .parse_callbacks(Box::new(ignored_macros))
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(dst.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Linking Time!
    println!("cargo:rustc-link-lib=static=wolfssl");
    println!(
        "cargo:rustc-link-search=native={}",
        format!("{}/lib/", out_dir)
    );
    println!("cargo:include={}", out_dir);
    println!("cargo:rerun-if-changed=wrapper.h");

    Ok(())
}
