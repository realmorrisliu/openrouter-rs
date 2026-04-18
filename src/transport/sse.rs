use std::io::Error;

use futures_util::{StreamExt, TryStreamExt, stream, stream::BoxStream};
use reqwest::Response;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;

pub(crate) fn response_lines(response: Response) -> BoxStream<'static, std::io::Result<String>> {
    let byte_stream = response.bytes_stream().map_err(Error::other);
    let lines = BufReader::new(StreamReader::new(byte_stream)).lines();

    stream::unfold(lines, |mut lines| async move {
        match lines.next_line().await {
            Ok(Some(line)) => Some((Ok(line), lines)),
            Ok(None) => None,
            Err(error) => Some((Err(error), lines)),
        }
    })
    .boxed()
}
