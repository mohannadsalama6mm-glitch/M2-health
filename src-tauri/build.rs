fn main() {
    println!("cargo:rerun-if-env-changed=M2_BUILD_ID");
    println!("cargo:rerun-if-env-changed=M2_LICENSE_PUBLIC_KEY");
    println!("cargo:rerun-if-changed=../config/license-public-key.txt");
    if std::env::var_os("M2_LICENSE_PUBLIC_KEY").is_none() {
        if let Ok(public_key) = std::fs::read_to_string("../config/license-public-key.txt") {
            println!(
                "cargo:rustc-env=M2_LICENSE_PUBLIC_KEY={}",
                public_key.trim()
            );
        }
    }
    tauri_build::build();
}
