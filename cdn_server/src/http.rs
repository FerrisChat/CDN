use reqwest::Client;
use std::lazy::SyncOnceCell as OnceCell;

use reqwest::multipart::{Form, Part};

use bytes::Bytes;
use cdn_common::{CdnError, UploadResponse};

pub static CLIENT: OnceCell<Client> = OnceCell::new();

pub fn load_http() {
    let client = Client::builder()
        .user_agent("FerrisChat CDN Server")
        .build()
        .expect("Failed to build http client");

    CLIENT.set(client).expect("Failed to set http client");
}

pub async fn get_file(ip: String, file: String) -> Result<Bytes, CdnError> {
    debug!("Attempting to get file from storage node: {}", ip);

    let resp = CLIENT
        .get()
        .unwrap_or_else(|| panic!("Failed to get HTTP Client: did you call load_http()?"))
        .get(format!("http://{}:8085/uploads/{}", ip, file).as_str())
        .send()
        .await
        .map_err(CdnError::ReqwestFailed)?;

    let status = resp.status();

    if !status.is_success() {
        debug!("Failed to get file: {}", status);
        return Err(CdnError::RequestFailed(
            resp.text().await.unwrap_or_else(|_| "".to_string()),
            status.as_u16(),
        ));
    }

    Ok(resp.bytes().await.map_err(CdnError::ReqwestFailed)?)
}

pub async fn delete_file(ip: String, file: String) -> Result<(), CdnError> {
    debug!("Attempting to delete file from storage node: {}", ip);

    let resp = CLIENT
        .get()
        .unwrap_or_else(|| panic!("Failed to get HTTP Client: did you call load_http()?"))
        .delete(format!("http://{}:8085/uploads/{}", ip, file).as_str())
        .send()
        .await
        .map_err(CdnError::ReqwestFailed)?;

    let status = resp.status();

    if !status.is_success() {
        debug!("Failed to get file: {}", status);
        return Err(CdnError::RequestFailed(
            resp.text().await.unwrap_or_else(|_| "".to_string()),
            status.as_u16(),
        ));
    }

    Ok(())
}

pub async fn upload_file(
    ip: String,
    file_name: String,
    bytes: Vec<u8>,
) -> Result<UploadResponse, CdnError> {
    debug!("Attempting to upload file from storage node: {}", ip);

    let resp = CLIENT
        .get()
        .unwrap_or_else(|| panic!("Failed to get HTTP Client: did you call load_http()?"))
        .post(format!("http://{}:8085/upload", ip).as_str())
        .multipart(Form::new().part("file", Part::bytes(bytes).file_name(file_name)))
        .send()
        .await
        .map_err(CdnError::ReqwestFailed)?;

    let status = resp.status();

    if !status.is_success() {
        return Err(CdnError::RequestFailed(
            resp.text().await.unwrap_or_else(|_| "".to_string()),
            status.as_u16(),
        ));
    }

    Ok(resp
        .json::<UploadResponse>()
        .await
        .map_err(CdnError::ReqwestFailed)?)
}
