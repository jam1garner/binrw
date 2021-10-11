use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=BINTRACE_LINK_SEARCH");
    println!("cargo:rerun-if-env-changed=BINTRACE_LINK_LIB");
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(bintrace) {
        if let Ok(link_search) = env::var("BINTRACE_LINK_SEARCH") {
            println!("cargo:rustc-link-search={}", link_search)
        }

        if let Ok(link_lib) = env::var("BINTRACE_LINK_LIB") {
            println!("cargo:rustc-link-lib={}", link_lib);
        } else {
            println!("cargo:warning=No bintrace backend provided");
        }
    }
}
