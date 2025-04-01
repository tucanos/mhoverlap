fn main() {
    // Ensure transition of dependencies RPATH
    println!("cargo:rerun-if-env-changed=DEP_CGNS_RPATH");
    for rpath_var in ["DEP_CGNS_RPATH"] {
        if let Ok(rpath) = std::env::var(rpath_var) {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{rpath}");
            println!("cargo:rpath={rpath}");
        }
    }
}
