use anyhow::{Context, Result};
use ariadne::{Color, Label, Report, ReportKind, Source};
use mml::MmlCompiler;

pub async fn compile(mml: String) -> Result<()> {
    match MmlCompiler::new().compile(&mml).await {
        Err(mml::Error::CompileMmlBodyError(
            mml::message::body::compiler::Error::ParseMmlError(errs, mml),
        )) => {
            errs.into_iter().for_each(|e| {
                Report::build(ReportKind::Error, (), e.span().start)
                    .with_message("cannot parse MML message")
                    .with_label(
                        Label::new(e.span().into_range())
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .print(Source::from(&mml))
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
