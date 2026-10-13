use futures_util::{Stream, StreamExt, stream, stream::BoxStream};

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

pub(crate) fn deserialize_nullable_option<'de, D, T>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    <Option<T> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

// Keep public Option fields while retaining explicit-null clear flags on PATCH input.
macro_rules! deserialize_nullable_patch {
    ($request:ty, { $($field:ident: $field_type:ty),* $(,)? },
        { $($nullable:ident: $nullable_type:ty => $clear:ident),* $(,)? }) => {
        impl<'de> serde::Deserialize<'de> for $request {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                #[derive(serde::Deserialize)]
                struct Wire {
                    $($field: Option<$field_type>,)*
                    $(#[serde(default, deserialize_with = "crate::utils::deserialize_nullable_option")]
                    $nullable: Option<Option<$nullable_type>>,)*
                }
                let wire = <Wire as serde::Deserialize>::deserialize(deserializer)?;
                Ok(Self {
                    $($field: wire.$field,)*
                    $($clear: wire.$nullable.as_ref().is_some_and(Option::is_none),
                    $nullable: wire.$nullable.flatten(),)*
                })
            }
        }
    };
}
pub(crate) use deserialize_nullable_patch;

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
