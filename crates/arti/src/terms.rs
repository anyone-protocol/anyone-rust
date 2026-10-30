//! Agreement to the Anyone terms and conditions.

use std::io::{BufRead, IsTerminal as _, Write};

use anyhow::{Context as _, Result, bail};
use arti_client::TorClientConfig;
use fs_mistrust::CheckedDir;
use tracing::info;

/// Where the terms and conditions are published.
const TERMS_URL: &str = "https://www.anyone.io/terms";

/// File in the state directory that records agreement.
const AGREEMENT_FILE: &str = "terms-agreement";

/// Content of the agreement file once the user has agreed.
const AGREED: &str = "agreed";

/// Return an error unless the user has agreed to the terms and conditions.
///
/// Agreement comes from `agree_flag`, from `agree_config`, from an agreement
/// recorded earlier in the state directory, or from a prompt when standard
/// input is a terminal.
pub(crate) fn ensure_agreed(
    agree_flag: bool,
    agree_config: bool,
    client_config: &TorClientConfig,
) -> Result<()> {
    let state_dir = client_config
        .expanded_state_dir()
        .context("Could not resolve the state directory")?;
    let dir = client_config
        .fs_mistrust()
        .verifier()
        .make_secure_dir(&state_dir)
        .context("Could not open the state directory")?;
    let prompt = std::io::stdin()
        .is_terminal()
        .then(|| (std::io::stdin().lock(), std::io::stdout()));
    check(&dir, agree_flag, agree_config, prompt)
}

/// Decide agreement, prompting on `prompt` when it is given.
fn check<R: BufRead, W: Write>(
    dir: &CheckedDir,
    agree_flag: bool,
    agree_config: bool,
    prompt: Option<(R, W)>,
) -> Result<()> {
    if agree_flag {
        record(dir)?;
    }
    if agree_config || recorded(dir) {
        return Ok(());
    }
    let Some((mut input, mut output)) = prompt else {
        bail!("User has not agreed to the terms and conditions. Exiting.");
    };
    loop {
        write!(
            output,
            "\nPlease read terms and conditions at: {TERMS_URL}\nDo you agree to the terms and conditions? (y/n): "
        )?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            bail!("Not agreed to terms");
        }
        match line.chars().next() {
            Some('y' | 'Y') => return record(dir),
            Some('n' | 'N') => bail!("Not agreed to terms"),
            _ => writeln!(output, "Invalid response. Please enter 'y' or 'n'.")?,
        }
    }
}

/// Return true if the state directory records agreement.
fn recorded(dir: &CheckedDir) -> bool {
    dir.read_to_string(AGREEMENT_FILE)
        .is_ok_and(|content| content == AGREED)
}

/// Record agreement in the state directory.
fn record(dir: &CheckedDir) -> Result<()> {
    dir.write_and_replace(AGREEMENT_FILE, AGREED)
        .context("Could not record agreement to the terms and conditions")?;
    info!("Marked agreement as set");
    Ok(())
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::mixed_attributes_style)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_time_subtraction)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::needless_pass_by_value)]
    #![allow(clippy::string_slice)] // See arti#2571
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->

    use super::*;

    fn state() -> (tempfile::TempDir, CheckedDir) {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = fs_mistrust::Mistrust::new_dangerously_trust_everyone()
            .verifier()
            .make_secure_dir(tmp.path().join("state"))
            .unwrap();
        (tmp, dir)
    }

    fn run(dir: &CheckedDir, flag: bool, config: bool, answers: Option<&str>) -> (bool, String) {
        let mut output = Vec::new();
        let result = check(dir, flag, config, answers.map(|a| (a.as_bytes(), &mut output)));
        (result.is_ok(), String::from_utf8(output).unwrap())
    }

    #[test]
    fn refuses_without_agreement_or_terminal() {
        let (_tmp, dir) = state();
        assert_eq!(run(&dir, false, false, None).0, false);
        assert_eq!(recorded(&dir), false);
    }

    #[test]
    fn flag_records_agreement() {
        let (_tmp, dir) = state();
        assert_eq!(run(&dir, true, false, None).0, true);
        assert_eq!(recorded(&dir), true);
        assert_eq!(run(&dir, false, false, None).0, true);
    }

    #[test]
    fn config_agrees_without_recording() {
        let (_tmp, dir) = state();
        assert_eq!(run(&dir, false, true, None).0, true);
        assert_eq!(recorded(&dir), false);
    }

    #[test]
    fn prompt_yes_records_agreement() {
        let (_tmp, dir) = state();
        let (ok, output) = run(&dir, false, false, Some("y\n"));
        assert_eq!(ok, true);
        assert!(output.contains(TERMS_URL));
        assert_eq!(recorded(&dir), true);
    }

    #[test]
    fn prompt_no_refuses() {
        let (_tmp, dir) = state();
        assert_eq!(run(&dir, false, false, Some("n\n")).0, false);
        assert_eq!(recorded(&dir), false);
    }

    #[test]
    fn prompt_repeats_on_invalid_response() {
        let (_tmp, dir) = state();
        let (ok, output) = run(&dir, false, false, Some("maybe\n\nY\n"));
        assert_eq!(ok, true);
        assert_eq!(output.matches("Invalid response").count(), 2);
        assert_eq!(output.matches(TERMS_URL).count(), 3);
    }

    #[test]
    fn prompt_end_of_input_refuses() {
        let (_tmp, dir) = state();
        assert_eq!(run(&dir, false, false, Some("")).0, false);
        assert_eq!(recorded(&dir), false);
    }
}
