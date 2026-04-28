use futures_util::{Stream, StreamExt, stream, stream::BoxStream};
use serde::{Serialize, Serializer};

use crate::error::OpenRouterError;

#[macro_export]
macro_rules! strip_option_vec_setter {
    ($field:ident, $item_ty:ty) => {
        pub fn $field<T, S>(&mut self, items: T) -> &mut Self
        where
            T: IntoIterator<Item = S>,
            S: Into<$item_ty>,
        {
            self.$field = Some(Some(items.into_iter().map(Into::into).collect()));
            self
        }
    };
}

#[macro_export]
macro_rules! strip_option_map_setter {
    ($field:ident, $key_ty:ty, $val_ty:ty) => {
        pub fn $field<K, V, T>(&mut self, items: T) -> &mut Self
        where
            T: IntoIterator<Item = (K, V)>,
            K: Into<$key_ty>,
            V: Into<$val_ty>,
        {
            let map: std::collections::HashMap<$key_ty, $val_ty> = items
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect();

            self.$field = Some(Some(map));
            self
        }
    };
}

pub(crate) fn serialize_optional_empty_vec_as_null<T, S>(
    value: &Option<Vec<T>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    match value {
        Some(items) if items.is_empty() => serializer.serialize_none(),
        Some(items) => items.serialize(serializer),
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SseFrame {
    pub event: Option<String>,
    pub data: String,
}

pub(crate) fn parse_sse_frames<S>(lines: S) -> BoxStream<'static, Result<SseFrame, OpenRouterError>>
where
    S: Stream<Item = std::io::Result<String>> + Send + Unpin + 'static,
{
    stream::unfold(
        (lines, None::<String>, String::new()),
        |(mut lines, mut event_name, mut data)| async move {
            loop {
                match lines.next().await {
                    Some(Ok(line)) => {
                        if line.is_empty() {
                            if data.is_empty() {
                                event_name = None;
                                continue;
                            }

                            return Some((
                                Ok(SseFrame {
                                    event: event_name.take(),
                                    data: std::mem::take(&mut data),
                                }),
                                (lines, None, String::new()),
                            ));
                        }

                        if line.starts_with(':') {
                            continue;
                        }

                        if let Some(event) = line.strip_prefix("event:") {
                            event_name = Some(event.strip_prefix(' ').unwrap_or(event).to_string());
                            continue;
                        }

                        if let Some(chunk) = line.strip_prefix("data:") {
                            if !data.is_empty() {
                                data.push('\n');
                            }
                            data.push_str(chunk.strip_prefix(' ').unwrap_or(chunk));
                        }
                    }
                    Some(Err(error)) => {
                        return Some((
                            Err(OpenRouterError::Io(error)),
                            (lines, None, String::new()),
                        ));
                    }
                    None => {
                        if data.is_empty() {
                            return None;
                        }

                        return Some((
                            Ok(SseFrame {
                                event: event_name.take(),
                                data: std::mem::take(&mut data),
                            }),
                            (lines, None, String::new()),
                        ));
                    }
                }
            }
        },
    )
    .boxed()
}
