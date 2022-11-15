use heck::{ToSnakeCase, ToUpperCamelCase};
use std::fs;
use std::path::Path;
use std::process::Command;

macro_rules! codegen_test {
    ($name:ident $test:tt) => {
        #[test]
        fn $name() {
            test_helpers::run_world_codegen_test(
                "guest-teavm-java",
                $test.as_ref(),
                |name, interfaces, files| {
                    wit_bindgen_gen_guest_teavm_java::Opts {
                        generate_stub: true,
                    }
                    .build()
                    .generate(name, interfaces, files)
                },
                verify,
            )
        }
    };
}
test_helpers::codegen_tests!("*.wit");

fn verify(dir: &Path, name: &str) {
    let java_dir = &dir.join("src/main/java");
    let snake = name.to_snake_case();
    let package_dir = &java_dir.join(format!("wit_{snake}"));

    fs::create_dir_all(package_dir).unwrap();

    let upper = name.to_upper_camel_case();
    for file_name in [
        format!("{upper}World.java"),
        format!("{upper}.java"),
        format!("{upper}Impl.java"),
    ] {
        let src = &dir.join(&file_name);
        let dst = &package_dir.join(&file_name);
        if src.exists() {
            fs::rename(src, dst).unwrap();
        }
    }

    fs::write(
        dir.join("pom.xml"),
        pom_xml(&[&format!("wit_{snake}.{upper}",)]),
    )
    .unwrap();
    fs::write(java_dir.join("Main.java"), include_bytes!("Main.java")).unwrap();

    let mut cmd = mvn();
    cmd.arg("prepare-package").current_dir(dir);

    test_helpers::run_command(&mut cmd);
}

#[cfg(unix)]
fn mvn() -> Command {
    Command::new("mvn")
}

#[cfg(windows)]
fn mvn() -> Command {
    let mut cmd = Command::new("cmd");
    cmd.args(&["/c", "mvn"]);
    cmd
}

fn pom_xml(classes_to_preserve: &[&str]) -> Vec<u8> {
    let xml = include_str!("pom.xml");
    let position = xml.find("<mainClass>").unwrap();
    let (before, after) = xml.split_at(position);
    let classes_to_preserve = classes_to_preserve
        .iter()
        .map(|&class| format!("<param>{class}</param>"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{before}
         <classesToPreserve>
            {classes_to_preserve}
         </classesToPreserve>
         {after}"
    )
    .into_bytes()
}
