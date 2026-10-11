use crate::prelude::*;
use lofty::id3::v2::Id3v2Tag;
use std::process::Stdio;
use tokio::join;

/// Job to transcode a single FLAC file to a target format.
pub(crate) struct TranscodeJob {
    /// Job identifier for progress tracking.
    pub id: String,
    /// Transcode operation to perform.
    pub variant: Variant,
    /// ID3 tags to write to MP3 output.
    pub tags: Option<Id3v2Tag>,
    /// Vorbis comment tag names to exclude from output.
    pub exclude_vorbis_comments: Vec<String>,
}

impl TranscodeJob {
    /// Execute the transcode, resample, or include operation and write tags.
    ///
    /// Tags named in [`exclude_vorbis_comments`](TranscodeJob::exclude_vorbis_comments) are
    /// stripped from the output via two paths:
    /// - **MP3 transcode**: excluded from the in-memory [`Id3v2Tag`] before writing
    /// - **FLAC resample**: excluded from the on-disk Vorbis comments after `SoX` writes the file
    pub(crate) async fn execute(self) -> Result<(), Failure<TranscodeAction>> {
        let output_path = match &self.variant {
            Variant::Transcode(_, encode) => encode.output.clone(),
            Variant::Resample(resample) => resample.output.clone(),
            Variant::Include(include) => include.output.clone(),
        };
        let output_dir = output_path
            .parent()
            .expect("output path should have a parent");
        create_dir_all(output_dir).map_err(Failure::wrap_with_path(
            TranscodeAction::CreateOutputDirectory,
            output_dir,
        ))?;
        match self.variant {
            Variant::Transcode(decode, encode) => execute_transcode(decode, encode).await?,
            Variant::Resample(resample) => {
                execute_resample(resample).await?;
                exclude_vorbis_comments_from_flac(&output_path, &self.exclude_vorbis_comments)
                    .map_err(Failure::wrap(TranscodeAction::ExcludeVorbisComments))?;
            }
            Variant::Include(include) => execute_include(include).await?,
        }
        if let Some(mut tags) = self.tags {
            exclude_tags(&mut tags, &vorbis_keys(&self.exclude_vorbis_comments));
            save_id3v2_deterministic(tags, &output_path).map_err(Failure::wrap_with_path(
                TranscodeAction::WriteTags,
                &output_path,
            ))?;
        }
        Ok(())
    }
}

/// Pipe decode output directly to encode input.
///
/// - Reports the failing program with its stderr
/// - Reports decode with the encode program and stderr attached when both fail, since a failure
///   of either one breaks the pipe for the other
/// - Attaches encode as context NOT a related diagnostic, since miette only renders related
///   diagnostics of the outermost failure
async fn execute_transcode(decode: Decode, encode: Encode) -> Result<(), Failure<TranscodeAction>> {
    let decode_input = decode.input.clone();
    let encode_output_path = encode.output.clone();
    let decode_info = decode.to_info();
    let encode_info = encode.to_info();
    trace!("Executing transcode: {decode_info} | {encode_info}");
    let decode_program = decode_info.program.clone();
    let mut decode_command = decode_info
        .to_command()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Failure::wrap_with(TranscodeAction::SpawnDecode, |f| {
            f.with("program", &decode_program)
        }))?;
    let pipe: Stdio = decode_command
        .stdout
        .take()
        .expect("should be able to take stdout")
        .try_into()
        .expect("should be able to convert stdout to pipe");
    let encode_program = encode_info.program.clone();
    let encode_command = encode_info
        .to_command()
        .stdin(pipe)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Failure::wrap_with(TranscodeAction::SpawnEncode, |f| {
            f.with("program", &encode_program)
        }))?;
    let (decode_output, encode_output) = join!(
        decode_command.wait_with_output(),
        encode_command.wait_with_output()
    );
    let decode_output = decode_output.map_err(Failure::wrap(TranscodeAction::WaitDecode))?;
    let encode_output = encode_output.map_err(Failure::wrap(TranscodeAction::WaitEncode))?;
    let decode = require_success(decode_output, &decode_program).map_err(Failure::wrap_with_path(
        TranscodeAction::Decode,
        &decode_input,
    ));
    let encode = require_success(encode_output, &encode_program).map_err(Failure::wrap_with_path(
        TranscodeAction::Transcode,
        &encode_output_path,
    ));
    match (decode, encode) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(decode), Ok(_)) => Err(decode),
        (Ok(_), Err(encode)) => Err(encode),
        (Err(decode), Err(encode)) => {
            let encode_error = encode
                .source()
                .and_then(Error::source)
                .map(ToString::to_string)
                .unwrap_or_default();
            Err(decode
                .with("encode", &encode_program)
                .with("encode error", encode_error))
        }
    }
}

async fn execute_resample(resample: Resample) -> Result<(), Failure<TranscodeAction>> {
    let output = resample.output.clone();
    let info = resample.to_info();
    trace!("Executing resample: {info}");
    info.to_command()
        .run()
        .await
        .map_err(Failure::wrap_with_path(TranscodeAction::Resample, &output))?;
    Ok(())
}

async fn execute_include(include: Include) -> Result<(), Failure<TranscodeAction>> {
    let verb = if include.hard_link {
        tokio_hard_link(&include.input, &include.output)
            .await
            .map_err(Failure::wrap_with_path(
                TranscodeAction::HardLinkFlac,
                &include.output,
            ))?;
        "Hard Linked"
    } else {
        tokio_copy(&include.input, &include.output)
            .await
            .map_err(Failure::wrap_with_path(
                TranscodeAction::CopyFlac,
                &include.output,
            ))?;
        "Copied"
    };
    trace!(
        "{} {} to {}",
        verb.bold(),
        include.input.display(),
        include.output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;
    use miette::{GraphicalReportHandler, GraphicalTheme};

    /// `lame` can't open a directory as its output, so `flac` also fails on the closed pipe.
    #[tokio::test]
    async fn transcode_job_execute_output_path_is_directory() {
        // Arrange
        init_logger();
        let test_dir = TestDirectory::new();
        let source = get_sample_flac().await;
        let output_path = test_dir.output().join("track.mp3");
        create_dir_all(&output_path).expect("should create directory at output path");
        let job = create_job(&test_dir, source, output_path).await;

        // Act
        let failure = job
            .execute()
            .await
            .expect_err("should fail when lame can't write the output");

        // Assert
        let report = render_unwrapped(failure);
        assert!(
            report.contains("Can't init outfile"),
            "lame stderr should be reported, got: {report}"
        );
    }

    /// `flac` can't decode a corrupt source, so `lame` also fails on the empty input.
    #[tokio::test]
    async fn transcode_job_execute_corrupt_source() {
        // Arrange
        init_logger();
        let test_dir = TestDirectory::new();
        create_dir_all(test_dir.output()).expect("should create output directory");
        let source = test_dir.output().join("corrupt.flac");
        write(&source, "not a flac file").expect("should write corrupt source");
        let output_path = test_dir.output().join("track.mp3");
        let job = create_job(&test_dir, source, output_path).await;

        // Act
        let failure = job
            .execute()
            .await
            .expect_err("should fail when flac can't decode the source");

        // Assert
        assert_eq!(failure.action(), &TranscodeAction::Decode);
        let report = render_unwrapped(failure);
        assert!(
            report.contains("not a FLAC file"),
            "flac stderr should be reported, got: {report}"
        );
    }

    /// Render the report as users see it, without line wrapping.
    ///
    /// - Nests the failure under [`TranscodeAction::ExecuteRunner`] as [`TranscodeCommand`] does,
    ///   since miette only renders related diagnostics of the outermost failure
    fn render_unwrapped(failure: Failure<TranscodeAction>) -> String {
        let outer = Failure::new(TranscodeAction::ExecuteRunner, failure);
        let mut report = String::new();
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
            .with_width(usize::MAX)
            .render_report(&mut report, &outer)
            .expect("report should render");
        report
    }

    async fn get_sample_flac() -> PathBuf {
        let album = AlbumProvider::get(SampleFormat::default()).await;
        Collector::get_flacs(&SAMPLE_SOURCES_DIR.join(album.dir_name()))
            .into_iter()
            .next()
            .expect("should find a flac")
            .path
    }

    async fn create_job(test_dir: &TestDirectory, input: PathBuf, output: PathBuf) -> TranscodeJob {
        let host = HostBuilder::mock()
            .with_test_options(test_dir)
            .await
            .expect_build();
        TranscodeJob {
            id: "Transcode".to_owned(),
            variant: Variant::Transcode(
                Decode {
                    input,
                    resample_rate: None,
                    repeatable: true,
                    sox: host.services.get_required::<SoxFactory>(),
                },
                Encode {
                    output,
                    format: TargetFormat::_320,
                },
            ),
            tags: None,
            exclude_vorbis_comments: Vec::new(),
        }
    }
}
