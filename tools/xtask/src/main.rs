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
            "nastalli-boot",
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
        "nastalli-boot",
        "--target",
        "x86_64-unknown-none",
        "-Zbuild-std=core,compiler_builtins",
    ])
}

fn image() -> io::Result<()> {
    build()?;
    let root = workspace_root();
    let kernel = root.join("target/x86_64-unknown-none/debug/nastalli-boot");
    let out = root.join("target/nastalli-uefi.img");
    fs::create_dir_all(root.join("target"))?;
    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&out)
        .map_err(io::Error::other)?;
    println!("created {}", out.display());
    Ok(())
}

fn run_qemu() -> io::Result<()> {
    let root = workspace_root();
    let image = root.join("target/nastalli-uefi.img");
    if !program_in_path("qemu-system-x86_64") {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "qemu-system-x86_64 was not found; install QEMU",
        ));
    }

    let ovmf = find_ovmf().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "OVMF was not found; set NASTALLI_OVMF_CODE or install OVMF/edk2-ovmf",
        )
    })?;
    let display = env::var("NASTALLI_QEMU_DISPLAY").unwrap_or_else(|_| "gtk".to_owned());

    let mut args = Vec::new();
    if let Some(vars_template) = ovmf.vars {
        let vars = root.join("target/nastalli-ovmf-vars.fd");
        fs::copy(vars_template, &vars)?;
        args.extend([
            "-drive".to_owned(),
            format!(
                "if=pflash,format=raw,unit=0,readonly=on,file={}",
                ovmf.code.display()
            ),
            "-drive".to_owned(),
            format!("if=pflash,format=raw,unit=1,file={}", vars.display()),
        ]);
    } else {
        args.extend(["-bios".to_owned(), ovmf.code.display().to_string()]);
    }

    args.extend([
        "-drive".to_owned(),
        format!("format=raw,file={}", image.display()),
        "-serial".to_owned(),
        "stdio".to_owned(),
        "-display".to_owned(),
        display,
    ]);

    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    command("qemu-system-x86_64", &refs)
}

fn program_in_path(program: &str) -> bool {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|path| path.join(program))
        .any(|path| path.is_file())
}

struct OvmfFirmware {
    code: PathBuf,
    vars: Option<PathBuf>,
}

fn find_ovmf() -> Option<OvmfFirmware> {
    if let Ok(code) = env::var("NASTALLI_OVMF_CODE") {
        let code = PathBuf::from(code);
        let vars = env::var_os("NASTALLI_OVMF_VARS").map(PathBuf::from);
        return code.is_file().then_some(OvmfFirmware { code, vars });
    }

    [
        (
            "/usr/share/edk2/x64/OVMF_CODE.fd",
            Some("/usr/share/edk2/x64/OVMF_VARS.fd"),
        ),
        (
            "/usr/share/edk2/x64/OVMF_CODE.4m.fd",
            Some("/usr/share/edk2/x64/OVMF_VARS.4m.fd"),
        ),
        (
            "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd",
            Some("/usr/share/edk2-ovmf/x64/OVMF_VARS.fd"),
        ),
        (
            "/usr/share/OVMF/OVMF_CODE.fd",
            Some("/usr/share/OVMF/OVMF_VARS.fd"),
        ),
        (
            "/usr/share/OVMF/OVMF_CODE_4M.fd",
            Some("/usr/share/OVMF/OVMF_VARS_4M.fd"),
        ),
        ("/usr/share/edk2/x64/OVMF.4m.fd", None),
    ]
    .into_iter()
    .find_map(|(code, vars)| {
        let code = PathBuf::from(code);
        if !code.is_file() {
            return None;
        }

        let vars = vars.map(PathBuf::from).filter(|path| path.is_file());
        Some(OvmfFirmware { code, vars })
    })
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
