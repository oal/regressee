// SPDX-License-Identifier: MPL-2.0
//! Public executable reservation. Product commands arrive in subsequent phases.
use std::process::ExitCode;

fn run(args: &[String]) -> Result<String, String> {
    match args {
        [] => Ok(help()),
        [arg] if matches!(arg.as_str(), "--help" | "-h" | "help") => Ok(help()),
        [arg] if matches!(arg.as_str(), "--version" | "-V") => {
            Ok(format!("regressee {}", env!("CARGO_PKG_VERSION")))
        }
        [arg] if arg == "--build-info" => Ok(regressee_app::launch::build_info().to_string()),
        _ => Err("The public CLI is not implemented yet. Use regressee-desktop for the application; run regressee --help for details.".into()),
    }
}

fn help() -> String {
    "Regressee CLI (implementation skeleton)\n\nUsage: regressee [--help | --version | --build-info]\n\nProduct commands and the shared engine are not available yet.\nLaunch the native application with regressee-desktop.\nThis skeleton does not open storage, start recovery, or launch a browser.".into()
}

fn main() -> ExitCode {
    match run(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_is_explicit_about_unimplemented_workflows() {
        assert!(run(&[]).unwrap().contains("not available yet"));
        assert!(
            run(&["--help".into()])
                .unwrap()
                .contains("does not open storage")
        );
    }

    #[test]
    fn metadata_is_data_free_and_headless() {
        let value: serde_json::Value =
            serde_json::from_str(&run(&["--build-info".into()]).unwrap()).unwrap();
        assert_eq!(value["features"]["ui"], false);
        assert!(
            run(&["--version".into()])
                .unwrap()
                .starts_with("regressee ")
        );
    }

    #[test]
    fn unknown_and_future_commands_fail_instead_of_operating_on_data() {
        for command in ["engine", "capture", "projects", "--unknown"] {
            assert!(run(&[command.into()]).is_err());
        }
    }
}
