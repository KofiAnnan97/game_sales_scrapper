use iced::{Window};
use iced::widget::image::Handle;
use std::path::PathBuf;
use std::sync::Arc;

use crate::Error;

pub fn open_file(
    window: &dyn Window,
) -> impl Future<Output = Result<(PathBuf, Arc<String>), Error>> + use<> {
    let dialog = rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .set_parent(&window);

    async move {
        let picked_file = dialog.pick_file().await.ok_or(Error::DialogClosed)?;

        load_file(picked_file).await
    }
}

async fn load_file(path: impl Into<PathBuf>) -> Result<(PathBuf, Arc<String>), Error> {
    let path = path.into();

    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok((path, contents))
}

pub async fn load_image_from_url(url: &str) -> Result<Handle, reqwest::Error> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    Ok(Handle::from_bytes(bytes.to_vec()))
}