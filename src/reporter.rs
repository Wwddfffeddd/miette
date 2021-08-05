/*!
Basic reporter for Diagnostics. Probably good enough for most use-cases,
but largely meant to be an example.
*/
use std::fmt;

use indenter::indented;

use crate::chain::Chain;
use crate::protocol::{Diagnostic, DiagnosticReporter, DiagnosticSnippet, Severity};

/**
Reference implementation of the [DiagnosticReporter] trait. This is generally
good enough for simple use-cases, but you might want to implement your own if
you want custom reporting for your tool or app.
*/
pub struct MietteReporter;

impl DiagnosticReporter for MietteReporter {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }
        self.render_diagnostic(diagnostic, f)?;
        Ok(())
    }
}

impl MietteReporter {
    fn render_diagnostic(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.render_header(diagnostic, f)?;
        self.render_causes(diagnostic, f)?;

        if let Some(snippets) = diagnostic.snippets() {
            writeln!(f)?;
            for snippet in snippets {
                self.render_snippet(f, snippet)?;
            }
        }

        if let Some(help) = diagnostic.help() {
            writeln!(f)?;
            for msg in help {
                writeln!(f, "﹦{}", msg)?;
            }
        }

        Ok(())
    }

    fn render_header(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let sev = match diagnostic.severity() {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Advice => "Advice",
        };
        write!(f, "{}[{}]: {}", sev, diagnostic.code(), diagnostic)?;
        Ok(())
    }

    fn render_causes(
        &self,
        diagnostic: &(dyn Diagnostic),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        use fmt::Write as _;
        if let Some(cause) = diagnostic.source() {
            write!(f, "\n\nCaused by:")?;
            let multiple = cause.source().is_some();

            for (n, error) in Chain::new(cause).enumerate() {
                writeln!(f)?;
                if multiple {
                    write!(indented(f).ind(n), "{}", error)?;
                } else {
                    write!(indented(f), "{}", error)?;
                }
            }
        }

        Ok(())
    }

    fn render_snippet(
        &self,
        f: &mut fmt::Formatter<'_>,
        snippet: &DiagnosticSnippet,
    ) -> fmt::Result {
        use fmt::Write as _;
        write!(f, "\n[{}]", snippet.source_name)?;
        if let Some(msg) = &snippet.message {
            write!(f, " {}:", msg)?;
        }
        writeln!(f)?;
        writeln!(f)?;
        let context_data = snippet
            .source
            .read_span(&snippet.context)
            .map_err(|_| fmt::Error)?;
        let context = std::str::from_utf8(context_data.data()).expect("Bad utf8 detected");
        let mut line = context_data.line();
        let mut column = context_data.column();
        let mut offset = snippet.context.start.offset();
        let mut line_offset = offset;
        let mut iter = context.chars().peekable();
        let mut line_str = String::new();
        let highlights = snippet.highlights.as_ref();
        while let Some(char) = iter.next() {
            offset += char.len_utf8();
            match char {
                '\r' => {
                    if iter.next_if_eq(&'\n').is_some() {
                        offset += 1;
                        line += 1;
                        column = 0;
                    } else {
                        line_str.push(char);
                        column += 1;
                    }
                }
                '\n' => {
                    line += 1;
                    column = 0;
                }
                _ => {
                    line_str.push(char);
                    column += 1;
                }
            }
            if iter.peek().is_none() {
                line += 1;
            }

            if column == 0 || iter.peek().is_none() {
                writeln!(indented(f), "{: <2} | {}", line, line_str)?;
                line_str.clear();
                if let Some(highlights) = highlights {
                    for (label, span) in highlights {
                        if span.start.offset() >= line_offset && span.end.offset() < offset {
                            // Highlight only covers one line.
                            write!(indented(f), "{: <2} | ", "⫶")?;
                            write!(
                                f,
                                "{}{} ",
                                " ".repeat(span.start.offset() - line_offset),
                                "^".repeat(span.len())
                            )?;
                            writeln!(f, "{}", label)?;
                        } else if span.start.offset() < offset
                            && span.start.offset() >= line_offset
                            && span.end.offset() >= offset
                        {
                            // Multiline highlight.
                            todo!("Multiline highlights.");
                        }
                    }
                }
                line_offset = offset;
            }
        }
        Ok(())
    }
}

/// Literally what it says on the tin.
pub struct JokeReporter;

impl DiagnosticReporter for JokeReporter {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        let sev = match diagnostic.severity() {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "advice",
        };
        writeln!(
            f,
            "me, with {} {}: {}",
            sev,
            diagnostic,
            diagnostic
                .help()
                .unwrap_or_else(|| Box::new(vec!["have you tried not failing?"].into_iter()))
                .collect::<Vec<&str>>()
                .join(" ")
        )?;
        writeln!(
            f,
            "miette, her eyes enormous: you {} miette? you {}? oh! oh! jail for mother! jail for mother for One Thousand Years!!!!",
            diagnostic.code(),
            diagnostic.snippets().map(|snippets| {
                snippets.iter().map(|snippet| snippet.message.clone()).collect::<Option<Vec<String>>>()
            }).flatten().map(|x| x.join(", ")).unwrap_or_else(||"try and cause miette to panic".into())
        )?;

        Ok(())
    }
}
