use std::{
    fmt::{Debug, Display},
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    str::FromStr,
};

use async_trait::async_trait;

use crate::config::Config;

use super::{traits::UpdatesStorage, IpFetchAttemptInfo};

pub struct FileSystemUpdatesStorage(PathBuf);

impl From<Config> for FileSystemUpdatesStorage {
    fn from(config: Config) -> Self {
        Self(config.check_file_path.into())
    }
}

#[async_trait]
impl<IpAddress> UpdatesStorage<IpAddress> for FileSystemUpdatesStorage
where
    for<'async_trait> IpAddress: Send + Sync + 'async_trait + FromStr + Debug + Display,
{
    async fn get_last_ip_attempt(&self) -> Option<IpFetchAttemptInfo<IpAddress>> {
        let file = fs::File::open(&self.0);
        if let Err(err) = file {
            log::warn!(
                "Error while opening check file: {:?}. Considering as it is does not exist.",
                err
            );
            // TODO: handle other possible errors
            return None;
        }

        let reader_iter = BufReader::new(file.unwrap()).lines();
        Some(IpFetchAttemptInfo::parse(reader_iter))
    }

    async fn save_new_attempt(&self, new_attempt: IpFetchAttemptInfo<IpAddress>) {
        log::info!("Saving new attempt to file: {:?}...", new_attempt);
        let datetime_line = new_attempt.datetime.to_rfc2822();
        let ip_line = new_attempt.ip_address.to_string();

        // Create parent directories if not existing.
        if let Some(parent_path) = self.0.parent() {
            log::debug!(
                "Parent path '{:?}' does not exist. Creating one...",
                parent_path
            );
            fs::create_dir_all(parent_path).expect("Failed to initialize parent directories.");
        }

        // Open file for overwrite
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.0)
            .expect("Error when opening the check file to update the information.");

        writeln!(file, "{}", datetime_line).expect("Failed to write datetime info to check file.");
        writeln!(file, "{}", ip_line).expect("Failed to write IP info to check file.");
        if new_attempt.last_delivery_success {
            writeln!(file, "1").expect("Failed to write alert information to check file.");
        }
        log::info!("New attempt saved!");
    }
}
