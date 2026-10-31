//! The Anyone name in everything the program prints.

use std::borrow::Cow;
use std::io::{self, Write};

/// A word to replace, its replacement, and bytes that must not follow it.
type Swap = (&'static str, &'static str, &'static [u8]);

/// The words replaced in output, in order of precedence.
const SWAPS: [Swap; 5] = [
    ("a Tor", "an Anyone", b""),
    ("Tor", "Anyone", b""),
    ("tor", "anyone", b"/"),
    ("Arti", "anyone-rust", b""),
    ("arti", "anyone-rust", b".:/"),
];

/// Return `text` with the Anyone name in place of Tor and Arti.
pub(crate) fn rebrand(text: &str) -> Cow<'_, str> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let (mut i, mut copied) = (0, 0);
    while i < bytes.len() {
        match SWAPS.iter().find(|swap| swaps_at(bytes, i, swap)) {
            Some((from, to, _)) => {
                out.extend_from_slice(&bytes[copied..i]);
                out.extend_from_slice(to.as_bytes());
                i += from.len();
                copied = i;
            }
            None => i += 1,
        }
    }
    if copied == 0 {
        return Cow::Borrowed(text);
    }
    out.extend_from_slice(&bytes[copied..]);
    Cow::Owned(String::from_utf8_lossy(&out).into_owned())
}

/// Return true if `swap` replaces the word starting at byte `i` of `bytes`.
fn swaps_at(bytes: &[u8], i: usize, (from, _, not_after): &Swap) -> bool {
    let in_word = |b: &u8| b.is_ascii_alphanumeric() || b"_-".contains(b);
    let (before, rest) = bytes.split_at(i);
    rest.starts_with(from.as_bytes())
        && (ends_with_color_code(before)
            || !before
                .last()
                .is_some_and(|b| in_word(b) || b"./".contains(b)))
        && !before.ends_with(b"C ")
        && !rest
            .get(from.len())
            .is_some_and(|b| in_word(b) || not_after.contains(b))
}

/// Return true if `bytes` ends with a terminal color code, such as `ESC [ 1 m`.
fn ends_with_color_code(bytes: &[u8]) -> bool {
    let Some(esc) = bytes.iter().rposition(|b| *b == 0x1b) else {
        return false;
    };
    matches!(bytes.get(esc + 1..), Some([b'[', params @ .., b'm'])
        if params.iter().all(|b| b.is_ascii_digit() || *b == b';'))
}

/// A writer that rebrands each write before passing it on.
pub(crate) struct Rebranded<W>(pub(crate) W);

impl<W: Write> Write for Rebranded<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(text) => self.0.write_all(rebrand(text).as_bytes())?,
            Err(_) => self.0.write_all(buf)?,
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

/// Print a command-line error, help, or version text without color, and exit, as
/// [`clap::Error::exit`] does.
pub(crate) fn clap_exit(err: &clap::Error) -> ! {
    let text = err.render().to_string();
    let text = rebrand(&text);
    let _ = if err.use_stderr() {
        io::stderr().write_all(text.as_bytes())
    } else {
        let mut stdout = io::stdout();
        stdout
            .write_all(text.as_bytes())
            .and_then(|()| stdout.flush())
    };
    std::process::exit(err.exit_code())
}

/// Report a fatal error and exit, as [`tor_error::report_and_exit`] does.
pub(crate) fn report_and_exit<R>(e: anyhow::Error) -> R {
    let progname = std::env::args()
        .next()
        .map(|name| format!("{name}: "))
        .unwrap_or_default();
    let report = tor_error::Report(e).to_string();
    let _ = writeln!(io::stderr(), "{progname}{}", rebrand(&report));
    std::process::exit(127)
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

    #[test]
    fn names_are_replaced() {
        for (from, to) in [
            (
                "Can't reach the Tor network",
                "Can't reach the Anyone network",
            ),
            (
                "handshaking with Tor relays",
                "handshaking with Anyone relays",
            ),
            (
                "can't be represented in a Tor cell",
                "can't be represented in an Anyone cell",
            ),
            (
                "Starting Arti 2.7.0 in proxy mode",
                "Starting anyone-rust 2.7.0 in proxy mode",
            ),
            (
                "Usage: arti <SUBCOMMAND> [OPTIONS]",
                "Usage: anyone-rust <SUBCOMMAND> [OPTIONS]",
            ),
            ("Tor's directory. Arti.", "Anyone's directory. anyone-rust."),
            ("Tor – réseau", "Anyone – réseau"),
            (
                "error: tor: problem with filesystem permissions",
                "error: anyone: problem with filesystem permissions",
            ),
            ("tor operation timed out", "anyone operation timed out"),
            (
                "\x1b[1mUsage:\x1b[0m \x1b[1marti\x1b[0m proxy",
                "\x1b[1mUsage:\x1b[0m \x1b[1manyone-rust\x1b[0m proxy",
            ),
            (
                "\x1b[3merror\x1b[0m\x1b[2m=\x1b[0mtor: timed out",
                "\x1b[3merror\x1b[0m\x1b[2m=\x1b[0manyone: timed out",
            ),
        ] {
            assert_eq!(rebrand(from), to);
        }
    }

    #[test]
    fn other_words_are_kept() {
        for text in [
            "Tor-Family-Preference",
            "X-Tor-Stream-Isolation",
            "TorClient",
            "Torrc",
            "Artist",
            "C Tor keystore",
            "arti-client",
            "arti_client",
            "/usr/local/bin/arti",
            "~/.config/arti/arti.toml",
            "arti:get_config",
            "https://gitlab.torproject.org/tpo/core/arti",
            "tor_dirmgr=debug",
            "tor-spec",
            "/var/lib/tor/",
            "tor/",
            "torrc",
            "\x1b[1mTorrc",
            "no names here",
        ] {
            assert!(matches!(rebrand(text), Cow::Borrowed(_)), "{text}");
        }
    }

    #[test]
    fn writer_rebrands_each_write() {
        let mut writer = Rebranded(Vec::new());
        let line = b"WARN unable to handshake with Tor relays\n";
        assert_eq!(writer.write(line).unwrap(), line.len());
        assert_eq!(writer.0, b"WARN unable to handshake with Anyone relays\n");
    }
}
