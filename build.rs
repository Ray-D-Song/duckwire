fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=Rstrtmgr");
        println!("cargo:rustc-link-lib=Bcrypt");
    }
}
