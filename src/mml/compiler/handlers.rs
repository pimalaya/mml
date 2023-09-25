use anyhow::{Context, Result};
use ariadne::{Color, Label, Report, ReportKind, Source};
use mml::{message::body::compiler::Error as CompileMmlBodyError, Error, MmlCompilerBuilder};

pub async fn compile(mml: String) -> Result<()> {
    let compiler = MmlCompilerBuilder::new()
        // TODO:
        // .with_pgp(…)
        .build(&mml)
        .context("cannot build MML compiler")?;

    match compiler.compile().await {
        Err(Error::CompileMmlBodyError(CompileMmlBodyError::ParseMmlError(errs, body))) => {
            errs.into_iter().for_each(|err| {
                Report::build(ReportKind::Error, (), err.span().start)
                    .with_message("cannot parse MML message")
                    .with_label(
                        Label::new(err.span().into_range())
                            .with_message(err.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .print(Source::from(&body))
                    .unwrap()
            });
            Ok(())
        }
        Err(err) => Err(err).context("cannot compile MML message"),
        Ok(res) => {
            let mime = res.into_string()?;
            print!("{mime}");
            Ok(())
        }
    }
}
