use anyhow::{Context, Result};
use ariadne::{Color, Label, Report, ReportKind, Source};
use mml::{message::body::compiler::Error as CompileMmlBodyError, Error, MmlCompiler};

pub async fn compile(mml: String) -> Result<()> {
    let compiler = MmlCompiler::new();

    // TODO: add PGP args?
    // compiler = compiler.with_pgp(…)

    match compiler.compile(&mml).await {
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
        Ok(mime) => {
            let mime = mime.write_to_string()?;
            print!("{mime}");
            Ok(())
        }
    }
}
