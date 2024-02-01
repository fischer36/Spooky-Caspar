fn main() {
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=kernel32");
}

// TODO What the fuck?
