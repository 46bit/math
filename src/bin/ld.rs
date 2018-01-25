use std::process::Command;

fn main() {
    assert!(
        Command::new("ld")
            .args(&[
                "-o",
                "a.out",
                "a.out.o",
                "-macosx_version_min",
                "10.12",
                "-lc",
            ])
            .spawn()
            .expect("could not invoke ld for linking")
            .wait()
            .unwrap()
            .success()
    );
}
