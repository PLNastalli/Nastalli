use std::{env, fs, io, path::PathBuf, process::Command};

fn main() -> io::Result<()> {
    match env::args().nth(1).as_deref() {
        Some("build") => build(),
        Some("image") => image(),
        Some("run") => {
            image()?;
            run_qemu()
        }
        Some("test") => command("cargo", &[
            "test",
            "--workspace",
            "--exclude",
            "novaos-boot",
        ]),
        _ => {
            eprintln!("usage: cargo xtask <build|image|run|test>");
            Ok(())
        }
    }
}

fn build() -> io::Result<()> {
    command("cargo", &[
        "build",
        "-p",
        "novaos-boot",
        "--target",
        "x86_64-unknown-none",
        "-Zbuild-std=core,compiler_builtins",
    ])
}

fn image() -> io::Result<()> {
    build()?;
    let root = workspace_root();
    let kernel = root.join("target/x86_64-unknown-none/debug/novaos-boot");
    let out = root.join("target/novaos-uefi.img");
    fs::create_dir_all(root.join("target"))?;
    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&out)
        .map_err(io::Error::other)?;
    println!("created {}", out.display());
    Ok(())
}

fn run_qemu() -> io::Result<()> {
    let image = workspace_root().join("target/novaos-uefi.img");
    if !program_in_path("qemu-system-x86_64") {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "qemu-system-x86_64 não encontrado; instale QEMU",
        ));
    }
    let ovmf = find_ovmf().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "OVMF não encontrado; defina NOVAOS_OVMF_CODE ou instale edk2-ovmf",
        )
    })?;
    let display = env::var("NOVAOS_QEMU_DISPLAY").unwrap_or_else(|_| "gtk".to_owned());
    command("qemu-system-x86_64", &[
        "-bios",
        &ovmf,
        "-drive",
        &format!("format=raw,file={}", image.display()),
        "-serial",
        "stdio",
        "-display",
        &display,
    ])
}

fn program_in_path(program: &str) -> bool {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|path| path.join(program))
        .any(|path| path.is_file())
}

fn find_ovmf() -> Option<String> {
    if let Ok(path) = env::var("NOVAOS_OVMF_CODE") {
        return Some(path);
    }

    [
        "/usr/share/edk2/x64/OVMF.4m.fd",
        "/usr/share/edk2/x64/OVMF_CODE.fd",
        "/usr/share/edk2/x64/OVMF_CODE.4m.fd",
        "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd",
        "/usr/share/OVMF/OVMF_CODE.fd",
    ]
    .into_iter()
    .find(|path| std::path::Path::new(path).exists())
    .map(String::from)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn command(program: &str, args: &[&str]) -> io::Result<()> {
    let status = Command::new(program).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{program} exited with {status}")))
    }
}
