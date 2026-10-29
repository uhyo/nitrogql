//! Framing of messages with `Content-Length` headers, as in the Language Server Protocol.

use log::error;

const HEADER_DELIMITER: &[u8] = b"\r\n\r\n";

/// Returns the header to send before given message.
pub fn header(message: &str) -> String {
    format!("Content-Length: {}\r\n\r\n", message.len())
}

/// Splits received data into messages.
#[derive(Default)]
pub struct MessageReader {
    /// Received data that does not form a complete message yet.
    buffer: Vec<u8>,
    /// Length of the content of the next message, once its header has been read.
    content_length: Option<usize>,
}

impl MessageReader {
    /// Adds received data.
    pub fn push(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Takes the next complete message, if any.
    pub fn next_message(&mut self) -> Option<String> {
        loop {
            let Some(content_length) = self.content_length else {
                let header_end = self
                    .buffer
                    .windows(HEADER_DELIMITER.len())
                    .position(|window| window == HEADER_DELIMITER)?;
                self.content_length = parse_content_length(&self.buffer[..header_end]);
                if self.content_length.is_none() {
                    error!("Missing Content-Length header");
                }
                self.buffer.drain(..header_end + HEADER_DELIMITER.len());
                continue;
            };
            if self.buffer.len() < content_length {
                return None;
            }
            let message = String::from_utf8_lossy(&self.buffer[..content_length]).into_owned();
            self.buffer.drain(..content_length);
            self.content_length = None;
            return Some(message);
        }
    }
}

/// Reads the value of `Content-Length` field from given header.
fn parse_content_length(header: &[u8]) -> Option<usize> {
    String::from_utf8_lossy(header)
        .split("\r\n")
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if !name.trim().eq_ignore_ascii_case("content-length") {
                return None;
            }
            value.trim().parse().ok()
        })
}

#[cfg(test)]
mod tests {
    use super::{MessageReader, header};

    fn frame(message: &str) -> Vec<u8> {
        [header(message).as_bytes(), message.as_bytes()].concat()
    }

    fn read_all(reader: &mut MessageReader) -> Vec<String> {
        std::iter::from_fn(|| reader.next_message()).collect()
    }

    #[test]
    fn single_message() {
        let mut reader = MessageReader::default();
        reader.push(&frame(r#"{"id":1}"#));
        assert_eq!(read_all(&mut reader), [r#"{"id":1}"#]);
    }

    #[test]
    fn message_split_across_chunks() {
        let data = frame(r#"{"id":1}"#);
        let mut reader = MessageReader::default();
        // Splits in the header, at the delimiter, and in the content.
        for chunk in [&data[..5], &data[5..18], &data[18..21], &data[21..]] {
            assert_eq!(read_all(&mut reader), Vec::<String>::new());
            reader.push(chunk);
        }
        assert_eq!(read_all(&mut reader), [r#"{"id":1}"#]);
    }

    #[test]
    fn several_messages_in_one_chunk() {
        let mut reader = MessageReader::default();
        reader.push(&[frame("a"), frame("bb"), frame("ccc")].concat());
        assert_eq!(read_all(&mut reader), ["a", "bb", "ccc"]);
    }

    #[test]
    fn chunk_ending_in_next_message() {
        let mut reader = MessageReader::default();
        let next = frame("second");
        reader.push(&[frame("first"), next[..10].to_vec()].concat());
        assert_eq!(read_all(&mut reader), ["first"]);
        reader.push(&next[10..]);
        assert_eq!(read_all(&mut reader), ["second"]);
    }

    #[test]
    fn header_with_extra_fields() {
        let mut reader = MessageReader::default();
        reader.push(
            b"Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: 2\r\n\r\n{}",
        );
        assert_eq!(read_all(&mut reader), ["{}"]);
    }

    #[test]
    fn header_field_name_in_any_case() {
        let mut reader = MessageReader::default();
        reader.push(b"content-length:2\r\n\r\n{}");
        assert_eq!(read_all(&mut reader), ["{}"]);
    }

    #[test]
    fn skips_header_without_content_length() {
        let mut reader = MessageReader::default();
        reader.push(&[b"Content-Type: text/plain\r\n\r\n".to_vec(), frame("{}")].concat());
        assert_eq!(read_all(&mut reader), ["{}"]);
    }

    #[test]
    fn byte_length_for_non_ascii_content() {
        let message = r#"{"text":"日本語 🎉"}"#;
        assert_eq!(header(message), "Content-Length: 25\r\n\r\n");
        let data = frame(message);
        let mut reader = MessageReader::default();
        // A chunk may end in the middle of a character.
        let split = data.len() - 3;
        reader.push(&data[..split]);
        assert_eq!(read_all(&mut reader), Vec::<String>::new());
        reader.push(&data[split..]);
        assert_eq!(read_all(&mut reader), [message]);
    }
}
