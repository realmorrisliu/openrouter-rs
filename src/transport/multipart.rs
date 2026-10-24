//! `multipart/form-data` bodies. The reqwest backend hands the parts to `reqwest::multipart`;
//! other backends write them with [`Form::encode`], which follows reqwest's layout.

use crate::error::HttpRequestError;

pub(crate) struct Form {
    parts: Vec<(String, Part)>,
}

impl Form {
    pub(crate) fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub(crate) fn part(mut self, name: impl Into<String>, part: Part) -> Self {
        self.parts.push((name.into(), part));
        self
    }

    #[cfg(feature = "reqwest")]
    pub(crate) fn into_parts(self) -> Vec<(String, Part)> {
        self.parts
    }

    /// The `Content-Type` header value and the body, delimited by `boundary`.
    #[cfg_attr(not(all(feature = "wasip3", target_os = "wasi")), allow(dead_code))]
    pub(crate) fn encode(&self, boundary: &str) -> (String, Vec<u8>) {
        let mut body = Vec::new();
        for (name, part) in &self.parts {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            body.extend_from_slice(b"Content-Disposition: form-data; name=\"");
            body.extend_from_slice(escape_quoted(name).as_bytes());
            body.extend_from_slice(b"\"");
            if let Some(file_name) = &part.file_name {
                body.extend_from_slice(b"; filename=\"");
                body.extend_from_slice(escape_quoted(file_name).as_bytes());
                body.extend_from_slice(b"\"");
            }
            if let Some(mime) = &part.mime {
                body.extend_from_slice(b"\r\nContent-Type: ");
                body.extend_from_slice(mime.as_ref().as_bytes());
            }
            body.extend_from_slice(b"\r\n\r\n");
            body.extend_from_slice(&part.bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        (format!("multipart/form-data; boundary={boundary}"), body)
    }
}

pub(crate) struct Part {
    pub(crate) bytes: Vec<u8>,
    pub(crate) file_name: Option<String>,
    pub(crate) mime: Option<mime::Mime>,
}

impl Part {
    pub(crate) fn bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
            file_name: None,
            mime: None,
        }
    }

    pub(crate) fn file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    pub(crate) fn mime_str(mut self, mime: &str) -> Result<Self, HttpRequestError> {
        self.mime = Some(
            mime.parse()
                .map_err(|error| HttpRequestError::new(format!("builder error: {error}")))?,
        );
        Ok(self)
    }
}

/// Escapes a quoted `Content-Disposition` parameter the way reqwest does.
fn escape_quoted(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\\r")
        .replace('\n', "\\\n")
}

#[cfg(test)]
mod tests {
    use super::{Form, Part};

    #[test]
    fn test_encode_writes_file_part_with_mime() {
        let form = Form::new().part(
            "file",
            Part::bytes(b"hello".to_vec())
                .file_name("a\"b.txt")
                .mime_str("text/plain")
                .expect("mime should parse"),
        );

        let (content_type, body) = form.encode("XYZ");

        assert_eq!(content_type, "multipart/form-data; boundary=XYZ");
        assert_eq!(
            String::from_utf8(body).expect("body should be utf-8"),
            "--XYZ\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"a\\\"b.txt\"\r\n\
             Content-Type: text/plain\r\n\r\n\
             hello\r\n\
             --XYZ--\r\n"
        );
    }

    #[test]
    fn test_mime_str_rejects_invalid_mime() {
        assert!(Part::bytes(Vec::new()).mime_str("not a mime").is_err());
    }
}
