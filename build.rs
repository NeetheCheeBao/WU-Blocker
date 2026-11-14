fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = std::path::Path::new(&dir);
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico = root.join("assets").join("icon.ico");
    let rc = out.join("icon.rc");
    let res = out.join("icon.res");
    let ico_arg = ico.display().to_string().replace('\\', "/");
    std::fs::write(&rc, format!("1 ICON \"{}\"\n", ico_arg)).unwrap();
    let mut done = false;
    for tool in [
        "windres",
        "C:/MinGW64/bin/windres.exe",
        "C:/MinGW/bin/windres.exe",
    ] {
        if let Ok(status) = std::process::Command::new(tool)
            .arg("-i")
            .arg(&rc)
            .arg("-o")
            .arg(&res)
            .arg("-O")
            .arg("coff")
            .arg("--target=pe-x86-64")
            .status()
        {
            if status.success() {
                done = true;
                break;
            }
        }
    }
    assert!(done, "windres is required to embed the icon");
    println!("cargo:rerun-if-changed={}", ico.display());
    println!("cargo:rerun-if-changed={}", root.join("Cargo.lock").display());
    println!("cargo:rustc-link-arg={}", res.display());
    let lock = root.join("Cargo.lock");
    if let Ok(text) = std::fs::read_to_string(&lock) {
        let cleaned: String = text
            .lines()
            .filter(|l| !l.starts_with("# "))
            .map(|l| format!("{}\n", l))
            .collect();
        if cleaned != text {
            let _ = std::fs::write(&lock, cleaned);
        }
    }
}
