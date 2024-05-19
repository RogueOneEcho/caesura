use crate::config::config::apply_config;
use crate::fs::util::get_all_files_with_extension;
use crate::redacted::api::client::RedactedApi;
use crate::redacted::api::constants::{FORBIDDEN_CHARACTERS, TRACKER_URL};
use crate::redacted::api::model::{Group, Torrent};
use crate::redacted::api::path::is_path_exceeding_redacted_path_limit;
use crate::redacted::models::ReleaseType::{Flac, Flac24, Mp3320, Mp3V0};
use crate::redacted::models::{Category, Media, ReleaseType};
use crate::redacted::upload::TorrentUploadData;
use crate::redacted::util::{
    create_description, get_bitrate, get_existing_release_types, get_format, get_group_id_from_url,
    get_permalink, get_release_type, get_torrent_id_from_url,
};
use crate::tags::util::valid_tags;
use crate::transcode::transcode::transcode_release;
use crate::transcode::util::copy_other_allowed_files;
use crate::{imdl, spectrogram, transcode, TranscodeCommand, ERROR, PAUSE, SUCCESS, WARNING};
use console::Term;
use dialoguer::{Confirm, Input};
use html_escape::decode_html_entities;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::collections::HashSet;
use std::env::temp_dir;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::create_dir_all;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

pub async fn transcode(cmd: &TranscodeCommand, term: &Term) -> anyhow::Result<()> {
    let cmd = apply_config(&cmd, term).await?;

    let mut api = RedactedApi::new(cmd.api_key.as_ref().unwrap())?;
    let index_response = api.index().await?.response;

    term.write_line(&format!(
        "{} Logged in as {} on the Redacted API",
        SUCCESS, index_response.username
    ))?;

    let urls = &cmd.urls;
    for url in urls {
        let result = handle_url(
            url.as_str(),
            term,
            &mut api,
            &cmd,
            index_response.passkey.clone(),
        )
        .await;

        if let Err(e) = result {
            term.write_line(&format!(
                "{} Skipping due to encountered error: {}",
                ERROR, e
            ))?;
        }
    }

    Ok(())
}

async fn handle_url(
    url: &str,
    term: &Term,
    api: &mut RedactedApi,
    cmd: &TranscodeCommand,
    passkey: String,
) -> anyhow::Result<()> {
    let group_id = get_group_id_from_url(url);
    let torrent_id = get_torrent_id_from_url(url);
    if group_id.is_none() || torrent_id.is_none() {
        term.write_line(&format!(
            "{} Could not parse permalink {}, please make sure you are using a valid permalink including group id and torrent id",
            ERROR, url
        ))?;
        return Ok(());
    }
    let group_id = group_id.unwrap();
    let torrent_id = torrent_id.unwrap();
    term.write_line(&format!(
        "{} Got torrent {} from group {}",
        SUCCESS, torrent_id, group_id
    ))?;

    let group_info = api.get_torrent_group(group_id).await?;

    let group_torrents = group_info.response.torrents;
    let group = group_info.response.group;

    let torrent_opt = group_torrents
        .iter()
        .find(|torrent| torrent.id == torrent_id);

    let torrent = match torrent_opt {
        None => {
            term.write_line(&format!(
                "{} Could not find torrent {} in group {}, this shouldn't happen...",
                ERROR, torrent_id, group_id
            ))?;
            return Ok(());
        }
        Some(t) => t,
    };

    if torrent.scene {
        term.write_line(&format!(
            "{} Torrent {} in group {} is a scene release which is unsupported, skipping",
            WARNING, torrent_id, group_id
        ))?;
        return Ok(());
    }

    if torrent.lossy_web_approved || torrent.lossy_master_approved {
        term.write_line(&format!(
            "{} Torrent {} in group {} is a lossy web/master release, once you upload a transcode you should report it and get staff approval",
            WARNING, torrent_id, group_id
        ))?;
    }

    let existing_formats = get_existing_release_types(torrent, &group_torrents);
    if existing_formats.contains(&None) {
        term.write_line(&format!(
            "{} Unknown encoding for torrent {} in group {}, this shouldn't happen...",
            ERROR, torrent.id, group_id
        ))
        .unwrap();
    }
    let existing_formats: HashSet<ReleaseType> =
        existing_formats.into_iter().filter_map(|x| x).collect();

    let source_format = get_release_type(torrent).unwrap();
    if source_format != Flac24 || source_format != Flac {
        term.write_line(&format!(
            "{} Torrent {} in group {} is {} not FLAC... skipping",
            WARNING, torrent_id, group_id, source_format
        ))?;
        return Ok(());
    }

    let transcode_formats = if cmd.skip_existing_formats_check {
        get_transcode_formats(
            &cmd.allowed_transcode_formats,
            HashSet::from([source_format]),
        )
    } else {
        get_transcode_formats(&cmd.allowed_transcode_formats, existing_formats)
    };

    if transcode_formats.is_empty() {
        term.write_line(&format!(
            "{} Torrent {} in group {} has all possible/wanted formats already... skipping",
            WARNING, torrent_id, group_id
        ))?;
        return Ok(());
    }

    term.write_line(&format!(
        "{} Found missing format(s) {} for torrent {} in group {}",
        SUCCESS,
        transcode_formats
            .iter()
            .map(|f| f.to_string())
            .fold(String::new(), |acc, s| acc + &s + ","),
        torrent_id,
        group_id
    ))?;

    let content_directory = cmd.content_directory.as_ref().unwrap();

    let flac_path = content_directory.join(decode_html_entities(&torrent.file_path).to_string());

    let media = Media::from(&*torrent.media);

    let (valid, invalid_track_number_vinyl) = valid_tags(&flac_path, &media).await?;

    let mut manual_upload_required = false;
    if !valid && invalid_track_number_vinyl {
        term.write_line(&format!(
            "{} Release is Vinyl and has either no set track number or in a non standard format (e.g. A1, A2 etc), you will be prompted once transcode is done to manually check & adjust the transcode tags as needed!", WARNING
        ))?;
        manual_upload_required = true;
    } else if !valid {
        term.write_line(&format!(
            "{} Torrent {} in group {} has FLAC files with invalid tags, skipping...\n You might be able to trump it.",
            WARNING, torrent_id, group_id
        ))?;
        return Ok(());
    }

    if !cmd.skip_hash_check {
        let downloaded_torrent = api.download_torrent(torrent.id).await?;

        let mut tmp = temp_dir();
        tmp.push(format!("red_oxide-torrent-{}", torrent_id));

        tokio::fs::write(&tmp, downloaded_torrent).await?;

        let result = imdl::hash::verify_torrent_hash(
            flac_path.as_path().to_str().unwrap(),
            tmp.to_str().unwrap(),
        )
        .await?;

        if result {
            term.write_line(&format!(
                "{} Local file torrent hash check succeeded for torrent {} in group {}",
                SUCCESS, torrent_id, group_id
            ))?;
        } else {
            term.write_line(&format!(
                "{} Local file torrent hash check failed for torrent {} in group {}",
                ERROR, torrent_id, group_id
            ))?;
            return Ok(());
        }

        tokio::fs::remove_file(&tmp).await?;
    }

    let flacs = get_all_files_with_extension(&flac_path, ".flac").await?;
    let flacs_count = flacs.len() as u64;

    if !cmd.skip_spectrogram {
        let spectrogram_directory = cmd.spectrogram_directory.as_ref().unwrap();
        let concurrency = cmd.concurrency.unwrap();
        let result =
            create_spectrograms(&flacs, &flac_path, spectrogram_directory, concurrency).await;
        if result.is_err() {
            return Ok(());
        }

        term.write_line(&*format!(
            "{} Created Spectrograms at {}, please manual check if FLAC is lossless before continuing!",
            PAUSE, spectrogram_directory.to_str().unwrap()
        ))?;

        let prompt = Confirm::new()
            .with_prompt("Do those spectrograms look good?")
            .default(true);

        let response = prompt.interact()?;

        if !response {
            term.write_line(&format!(
                "{} Spectrogram check failed for torrent {} in group {}, skipping",
                ERROR, torrent_id, group_id
            ))?;
            return Ok(());
        };
    }

    if transcode::util::is_multichannel(&flac_path).await? {
        term.write_line(&format!(
            "{} Torrent {} in group {} is a multichannel release which is unsupported, skipping",
            WARNING, torrent_id, group_id
        ))?;
        return Ok(());
    }

    let multi_progress = MultiProgress::with_draw_target(ProgressDrawTarget::stdout());
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");

    let pb_main = multi_progress.add(ProgressBar::new(
        flacs_count * transcode_formats.len() as u64,
    ));
    pb_main.set_style(sty.clone());
    pb_main.set_message("Total");

    pb_main.tick();

    let semaphore = Arc::new(Semaphore::new(cmd.concurrency.unwrap()));
    let mut join_set = JoinSet::new();

    multi_progress.println("[➡️] Transcoding...").unwrap();

    let transcode_directory = cmd.transcode_directory.as_ref().unwrap();

    let base_name = get_base_name(group.clone(), torrent.clone());

    for format in &transcode_formats {
        let pb_format = multi_progress.insert_before(&pb_main, ProgressBar::new(flacs_count));
        pb_format.set_style(sty.clone());

        let transcode_format_str = match format {
            Flac24 => "FLAC 24bit",
            Flac => "FLAC",
            Mp3320 => "MP3 - 320",
            Mp3V0 => "MP3 - V0",
        };

        let transcode_release_name = format!(
            "{} ({} - {})",
            base_name,
            torrent.media.to_uppercase(),
            transcode_format_str
        );

        let flac_path_clone = flac_path.clone();
        let torrent_id_clone = torrent_id.clone();
        let term = Arc::new(term.clone());
        let mut output_dir = transcode_directory.clone();
        let format = format.clone();
        let pb_main_clone = pb_main.clone();
        let semaphore_clone = semaphore.clone();
        join_set.spawn(tokio::spawn(async move {
            let (folder_path, command) = transcode_release(
                &flac_path_clone,
                &mut output_dir,
                transcode_release_name.clone(),
                format,
                term,
                torrent_id_clone,
                pb_format,
                pb_main_clone,
                semaphore_clone,
            )
            .await?;

            let transcode_folder_path = output_dir.join(&folder_path);

            copy_other_allowed_files(&flac_path_clone, &flac_path_clone, &transcode_folder_path)
                .await?;

            return Ok::<(PathBuf, ReleaseType, String), anyhow::Error>((
                folder_path,
                format,
                command,
            ));
        }));
    }

    let mut path_format_command_triple = Vec::new();

    while let Some(res) = join_set.join_next().await {
        let transcode_folder = res???;

        path_format_command_triple.push(transcode_folder);
    }

    multi_progress.println(format!("{} Transcoding Done!", SUCCESS))?;
    multi_progress.clear()?;

    if invalid_track_number_vinyl {
        let mut prompt = Confirm::new();

        prompt = prompt
            .with_prompt(format!("{} Please check tags of trancoded media and adjust as needed (release is vinyl and has either no track number or in an non standard format e.g. A1, A2 etc which the audiotags library used can't parse), continue?", WARNING))
            .default(true);

        prompt.interact()?;
    }

    let torrent_directory = cmd.torrent_directory.as_ref().unwrap();

    for (path, format, command) in &path_format_command_triple {
        let release_name = path.file_name().unwrap().to_str().unwrap();
        let mut exceeds_red_path_length = is_path_exceeding_redacted_path_limit(&path).await?;

        while exceeds_red_path_length {
            let editor = Input::new();

            let edited_text = editor
                .with_prompt(format!(
                    "{} Folder Name {} is too long for RED, please shorten the folder name\n",
                    ERROR, release_name
                ))
                .default(release_name.to_string())
                .interact_text()?;

            let new_path = path.parent().unwrap().join(edited_text);
            exceeds_red_path_length = is_path_exceeding_redacted_path_limit(&new_path).await?;
        }

        let torrent_path = torrent_directory.join(release_name.to_owned() + ".torrent");

        imdl::torrent::create_torrent(
            path,
            &torrent_path,
            format!("{}/{}/announce", TRACKER_URL, passkey),
        )
        .await?;

        term.write_line(&format!(
            "{} Created .torrent files for format {}",
            SUCCESS, format
        ))?;

        let torrent_file_data = tokio::fs::read(&torrent_path).await?;

        if cmd.move_transcode_to_content {
            tokio::fs::rename(&path, &content_directory.join(path.file_name().unwrap())).await?;

            term.write_line(&format!(
                "{} Moved transcode release to content directory",
                SUCCESS,
            ))?;
        }

        if !cmd.automatic_upload || manual_upload_required {
            term.write_line(&*format!(
                "{} Manual mode enabled, skipping automatic upload",
                PAUSE
            ))?;
            term.write_line(&format!("Link: {}", get_permalink(group_id, torrent_id)))?;
            term.write_line(&format!("Name: {}", group.name))?;
            term.write_line(&format!(
                "Artist(s): {}",
                group
                    .music_info
                    .artists
                    .iter()
                    .map(|a| a.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            ))?;
            term.write_line(&format!("Edition Year: {}", torrent.remaster_year))?;
            term.write_line(&format!("Edition Title: {}", torrent.remaster_title))?;
            term.write_line(&format!("Record Label: {}", torrent.remaster_record_label))?;
            term.write_line(&format!(
                "Catalogue Number: {}",
                torrent.remaster_catalogue_number
            ))?;
            term.write_line(&format!(
                "Scene: {}",
                if torrent.scene { "Yes" } else { "No" }
            ))?;
            term.write_line(&format!("Format: {}", get_format(format)))?;
            term.write_line(&format!("Bitrate: {}", get_bitrate(format)))?;
            term.write_line(&format!("Media: {}", torrent.media))?;
            term.write_line("Release Description:")?;
            term.write_line(
                create_description(get_permalink(group_id, torrent_id), command).as_str(),
            )?;

            let mut prompt = Confirm::new();

            prompt = prompt
                .with_prompt("Confirm once you are done uploading...")
                .default(true);

            prompt.interact()?;
        } else if !cmd.dry_run {
            let year = if torrent.remaster_year == 0 {
                group.year
            } else {
                torrent.remaster_year
            };

            let upload_data = TorrentUploadData {
                torrent: torrent_file_data,
                torrent_name: torrent_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                r#type: Category::from(&*group.category_name),
                remaster_year: year,
                remaster_title: torrent.remaster_title.clone(),
                remaster_record_label: torrent.remaster_record_label.clone(),
                remaster_catalogue_number: torrent.remaster_catalogue_number.clone(),
                format: get_format(format),
                bitrate: get_bitrate(format),
                media: torrent.media.clone(),
                release_desc: create_description(get_permalink(group_id, torrent_id), command),
                group_id: group.id as u64,
            };

            let res = api.upload_torrent(upload_data).await?;

            term.write_line(&format!("[🔼] Uploaded {} release to REDacted https://redacted.ch/torrents.php?id={}&torrentid={}", format, group_id, res.response.torrent_id))?;
        }
    }

    Ok(())
}

fn get_transcode_formats(
    allowed_transcode_formats: &Vec<ReleaseType>,
    existing_formats: HashSet<ReleaseType>,
) -> Vec<ReleaseType> {
    allowed_transcode_formats
        .iter()
        .filter(|&&release_type| release_type != Flac24)
        .filter(|&release_type| !existing_formats.contains(release_type))
        .cloned()
        .collect()
}
fn get_base_name(group: Group, torrent: Torrent) -> String {
    let artist = if group.music_info.artists.len() > 1 {
        "Various Artists".to_string()
    } else {
        group.music_info.artists[0].name.clone()
    };

    // Fixes edge case where remaster year is 0 (likely unintentional)
    let year = if torrent.remaster_year != 0 {
        torrent.remaster_year
    } else {
        group.year
    };

    let group_name = group.name.replace(":", "");

    let raw_base_name = if torrent.remaster_title.len() > 1 {
        format!(
            "{} - {} ({}) [{}]",
            artist, group_name, torrent.remaster_title, year
        )
    } else {
        format!("{} - {} [{}]", artist, group_name, year)
    };
    raw_base_name.replace(&FORBIDDEN_CHARACTERS[..], "_")
}

async fn create_spectrograms(
    flacs: &Vec<PathBuf>,
    flac_path: &PathBuf,
    spectrogram_directory: &PathBuf,
    concurrency: usize,
) -> anyhow::Result<()> {
    let pb = ProgressBar::new(flacs.len() as u64);

    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {msg} {pos:>7}/{len:7} File(s)",
        )?
        .progress_chars("#>-"),
    );

    pb.set_message("Creating Spectrograms... (This may take a while)");

    let parent = flac_path.file_name().unwrap().to_str().unwrap();
    let to_create = spectrogram_directory.join(parent);

    create_dir_all(&to_create).await?;

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut tasks = vec![];

    for flac in flacs {
        let semaphore = Arc::clone(&semaphore);
        let spectrogram_directory = spectrogram_directory.clone();
        let flac_path = flac_path.clone();
        let flac = flac.clone();
        let pb = pb.clone();
        tasks.push(tokio::spawn(async move {
            let mut join_set = JoinSet::new();

            let semaphore_clone = Arc::clone(&semaphore);
            let spectrogram_directory_clone = spectrogram_directory.clone();
            let flac_path_clone = flac_path.clone();
            let flac_clone = flac.clone();

            join_set.spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                spectrogram::spectrogram::make_spectrogram_zoom(
                    &flac_path_clone,
                    &flac_clone,
                    &spectrogram_directory_clone,
                )
                .await?;

                Ok::<(), anyhow::Error>(())
            });

            let semaphore_clone = Arc::clone(&semaphore);
            let spectrogram_directory_clone = spectrogram_directory.clone();
            let flac_path_clone = flac_path.clone();
            let flac_clone = flac.clone();
            join_set.spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                spectrogram::spectrogram::make_spectrogram_full(
                    &flac_path_clone,
                    &flac_clone,
                    &spectrogram_directory_clone,
                )
                .await?;

                Ok::<(), anyhow::Error>(())
            });

            while let Some(result) = join_set.join_next().await {
                result??;
            }

            pb.inc(1);

            Ok::<(), anyhow::Error>(())
        }));
    }

    for task in tasks {
        task.await??;
    }

    pb.finish_and_clear();

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::current_dir;
    use std::time::SystemTime;

    #[test]
    fn get_transcode_formats_from_flac24_skips_existing() {
        let allowed_transcode_formats = vec![Flac, Mp3320, Mp3V0];
        let existing_formats = HashSet::from([Flac24, Mp3320]);
        let result = get_transcode_formats(&allowed_transcode_formats, existing_formats);
        assert_eq!(result, [Flac, Mp3V0]);
    }

    #[test]
    fn get_transcode_formats_from_flac_skips_existing() {
        let allowed_transcode_formats = vec![Flac, Mp3320, Mp3V0];
        let existing_formats = HashSet::from([Flac, Mp3320]);
        let result = get_transcode_formats(&allowed_transcode_formats, existing_formats);
        assert_eq!(result, [Mp3V0]);
    }

    #[test]
    fn get_transcode_formats_from_flac24_without_skips_exiting() {
        let allowed_transcode_formats = vec![Flac, Mp3320, Mp3V0];
        let source_format = Flac;
        let existing_formats = HashSet::from([source_format]);
        let result = get_transcode_formats(&allowed_transcode_formats, existing_formats);
        // TODO: Why is Flac not included?
        assert_eq!(result, [Mp3320, Mp3V0]);
    }

    #[test]
    fn get_transcode_formats_from_flac_without_skips_existing() {
        let allowed_transcode_formats = vec![Flac, Mp3320, Mp3V0];
        let source_format = Flac;
        let existing_formats = HashSet::from([source_format]);
        let result = get_transcode_formats(&allowed_transcode_formats, existing_formats);
        assert_eq!(result, [Mp3320, Mp3V0]);
    }

    #[test]
    fn get_transcode_formats_from_flac_applies_allowed() {
        let allowed_transcode_formats = vec![Mp3320, Mp3V0];
        let existing_formats = HashSet::from([Flac, Mp3320]);
        let result = get_transcode_formats(&allowed_transcode_formats, existing_formats);
        assert_eq!(result, [Mp3V0]);
    }

    #[test]
    fn get_transcode_formats_from_flac_applies_allowed_none() {
        let allowed_transcode_formats = vec![Mp3320];
        let existing_formats = HashSet::from([Flac, Mp3320]);
        let result = get_transcode_formats(&allowed_transcode_formats, existing_formats);
        assert_eq!(result, []);
    }

    #[tokio::test]
    async fn create_spectrograms_test() {
        let flac_path = get_flacs_sample_dir();
        let flacs = get_flacs(&flac_path);
        let hello = current_dir().unwrap();
        println!("Current dir: {}", hello.to_str().unwrap());
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        let spectrogram_directory = temp_dir().join("spectrograms").join(timestamp);
        let concurrency = 2;
        let result =
            create_spectrograms(&flacs, &flac_path, &spectrogram_directory, concurrency).await;
        assert!(result.is_ok());
        let generated_files: Vec<PathBuf> = read_dir_recursive(&spectrogram_directory);
        assert_eq!(generated_files.len(), flacs.len() * 2);
    }

    fn get_flacs_sample_dir() -> PathBuf {
        std::fs::read_dir("samples/flacs")
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .nth(0)
            .unwrap()
    }

    fn get_flacs(flac_path: &PathBuf) -> Vec<PathBuf> {
        // Get the content
        let files: Vec<PathBuf> = std::fs::read_dir(flac_path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().unwrap_or_default() == "flac")
            .collect();
        if files.len() == 0 {
            panic!("No flac files found in the sample directory");
        }
        return files;
    }

    fn read_dir_recursive(directory_path: &PathBuf) -> Vec<PathBuf> {
        std::fs::read_dir(directory_path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .map(|entry| {
                if entry.is_dir() {
                    read_dir_recursive(&entry)
                } else {
                    vec![entry]
                }
            })
            .flatten()
            .collect()
    }
}
