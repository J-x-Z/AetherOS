fn main() {
    println!("cargo:rustc-link-lib=framework=Hypervisor");
    println!("cargo:rustc-link-search=framework=/System/Library/Frameworks");
}
