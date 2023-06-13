use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/kernel.cu");

    let out_dir = env::var("OUT_DIR").unwrap();

    let filename = if cfg!(target_os = "windows") {
        "kernel.lib"
    } else if cfg!(target_os = "linux") {
        "libkernel.a"
    } else {
        panic!("Unsupported distribution.");
    };

    let status = Command::new("nvcc")
        .args([
            "-O3",
            "-lib",
            "-gencode=arch=compute_61,code=sm_61",
            "src/kernel.cu",
            "-o",
        ])
        .arg(&format!("{}/{}", &out_dir, filename))
        .status()
        .unwrap();
    if !status.success() {
        panic!("Failed to compile kernel.");
    }

    println!("cargo:rustc-link-search=native={}", &out_dir);
}
