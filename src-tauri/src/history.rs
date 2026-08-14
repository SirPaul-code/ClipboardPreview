use std::{collections::VecDeque, io::Cursor};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use image::{DynamicImage, ImageOutputFormat, RgbaImage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::models::{
    ClipboardContentType, ClipboardItem, ClipboardMetadata, HISTORY_MEMORY_BUDGET_MIB,
};

const MAX_FRONTEND_CONTENT_CHARS: usize = 8_192;
const MAX_TEXT_BYTES: usize = 16 * 1024 * 1024;
const MAX_RAW_IMAGE_BYTES: usize = 128 * 1024 * 1024;
const MAX_ENCODED_IMAGE_BYTES: usize = 32 * 1024 * 1024;
const THUMBNAIL_WIDTH: u32 = 192;
const THUMBNAIL_HEIGHT: u32 = 120;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub item: ClipboardItem,
    pub image_png: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedHistoryEntry {
    pub item: ClipboardItem,
    #[serde(default)]
    pub image_png_base64: Option<String>,
}

impl HistoryEntry {
    pub fn from_legacy_item(mut item: ClipboardItem) -> Self {
        if item.metadata.byte_size == 0 {
            item.metadata.byte_size = item.content.as_ref().map_or(0, |text| text.len());
        }
        Self {
            item,
            image_png: None,
        }
    }

    pub fn from_persisted(value: PersistedHistoryEntry) -> Option<Self> {
        let image_png = value
            .image_png_base64
            .as_deref()
            .and_then(|encoded| STANDARD.decode(encoded).ok());

        if matches!(value.item.content_type, ClipboardContentType::Image) && image_png.is_none() {
            return None;
        }

        Some(Self {
            item: value.item,
            image_png,
        })
    }

    pub fn to_persisted(&self) -> PersistedHistoryEntry {
        PersistedHistoryEntry {
            item: self.item.clone(),
            image_png_base64: self.image_png.as_ref().map(|bytes| STANDARD.encode(bytes)),
        }
    }

    pub fn frontend_item(&self) -> ClipboardItem {
        let mut item = self.item.clone();
        if let Some(content) = item.content.as_mut() {
            if content.chars().count() > MAX_FRONTEND_CONTENT_CHARS {
                *content = truncate_chars(content, MAX_FRONTEND_CONTENT_CHARS);
            }
        }
        item
    }

    fn memory_bytes(&self) -> usize {
        self.item.content.as_ref().map_or(0, String::len)
            + self.item.preview.len()
            + self
                .item
                .thumbnail_data_url
                .as_ref()
                .map_or(0, String::len)
            + self.image_png.as_ref().map_or(0, Vec::len)
            + 256
    }
}

#[derive(Debug, Default)]
pub struct ClipboardHistory {
    items: VecDeque<HistoryEntry>,
}

impl ClipboardHistory {
    pub fn from_entries(items: Vec<HistoryEntry>, limit: usize) -> Self {
        let mut history = Self {
            items: items.into(),
        };
        history.truncate(limit);
        history
    }

    pub fn add_text(&mut self, text: String, max: usize) -> bool {
        if text.len() > MAX_TEXT_BYTES {
            log::warn!("Clipboard text exceeded the per-item memory limit and was not added to history");
            return false;
        }

        let hash = hash_text(&text);
        if self.items.front().is_some_and(|entry| entry.item.hash == hash) {
            return false;
        }

        let character_count = text.chars().count();
        let line_count = text.lines().count().max(1);
        let byte_size = text.len();
        let item = ClipboardItem {
            id: Uuid::new_v4().to_string(),
            content_type: classify(&text),
            preview: preview(&text),
            metadata: ClipboardMetadata {
                character_count,
                line_count,
                width: None,
                height: None,
                byte_size,
            },
            content: Some(text),
            created_at: Utc::now(),
            hash,
            thumbnail_data_url: None,
        };

        self.items.push_front(HistoryEntry {
            item,
            image_png: None,
        });
        self.truncate(max);
        true
    }

    pub fn add_image(
        &mut self,
        width: usize,
        height: usize,
        rgba: Vec<u8>,
        max: usize,
    ) -> Result<bool, String> {
        let expected = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or("Clipboard image dimensions overflowed")?;

        if rgba.len() != expected {
            return Err("Clipboard image pixel buffer had an unexpected size".into());
        }
        if expected > MAX_RAW_IMAGE_BYTES {
            return Err("Clipboard image is too large to keep in lightweight history".into());
        }

        let hash = hash_image(width, height, &rgba);
        if self.items.front().is_some_and(|entry| entry.item.hash == hash) {
            return Ok(false);
        }

        let width_u32 = u32::try_from(width).map_err(|_| "Clipboard image width is too large")?;
        let height_u32 = u32::try_from(height).map_err(|_| "Clipboard image height is too large")?;
        let rgba_image = RgbaImage::from_raw(width_u32, height_u32, rgba)
            .ok_or("Could not construct clipboard image")?;
        let image = DynamicImage::ImageRgba8(rgba_image);
        let image_png = encode_png(&image)?;

        if image_png.len() > MAX_ENCODED_IMAGE_BYTES {
            return Err("Clipboard image compressed size exceeds the lightweight history limit".into());
        }

        let thumbnail = image.thumbnail(THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);
        let thumbnail_png = encode_png(&thumbnail)?;
        let thumbnail_data_url = Some(format!(
            "data:image/png;base64,{}",
            STANDARD.encode(thumbnail_png)
        ));

        let item = ClipboardItem {
            id: Uuid::new_v4().to_string(),
            content_type: ClipboardContentType::Image,
            content: None,
            preview: format!("Image {width}×{height}"),
            created_at: Utc::now(),
            hash,
            metadata: ClipboardMetadata {
                character_count: 0,
                line_count: 0,
                width: Some(width_u32),
                height: Some(height_u32),
                byte_size: image_png.len(),
            },
            thumbnail_data_url,
        };

        self.items.push_front(HistoryEntry {
            item,
            image_png: Some(image_png),
        });
        self.truncate(max);
        Ok(true)
    }

    pub fn items(&self) -> Vec<ClipboardItem> {
        self.items.iter().map(HistoryEntry::frontend_item).collect()
    }

    pub fn entries(&self) -> Vec<HistoryEntry> {
        self.items.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: usize) -> Option<HistoryEntry> {
        self.items.get(index).cloned()
    }

    pub fn find(&self, id: &str) -> Option<HistoryEntry> {
        self.items.iter().find(|entry| entry.item.id == id).cloned()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn promote(&mut self, id: &str) -> bool {
        let Some(index) = self.items.iter().position(|entry| entry.item.id == id) else {
            return false;
        };
        if index == 0 {
            return false;
        }
        if let Some(entry) = self.items.remove(index) {
            self.items.push_front(entry);
            true
        } else {
            false
        }
    }

    pub fn truncate(&mut self, max: usize) {
        let max = max.max(1);
        while self.items.len() > max {
            self.items.pop_back();
        }

        let budget = HISTORY_MEMORY_BUDGET_MIB * 1024 * 1024;
        while self.memory_bytes() > budget && self.items.len() > 1 {
            self.items.pop_back();
        }
    }

    pub fn memory_bytes(&self) -> usize {
        self.items.iter().map(HistoryEntry::memory_bytes).sum()
    }
}

pub fn hash_text(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

pub fn hash_image(width: usize, height: usize, rgba: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(width.to_le_bytes());
    digest.update(height.to_le_bytes());
    digest.update(rgba);
    hex::encode(digest.finalize())
}

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut output), ImageOutputFormat::Png)
        .map_err(|error| format!("Could not encode clipboard image: {error}"))?;
    Ok(output)
}

fn classify(text: &str) -> ClipboardContentType {
    let trimmed = text.trim();
    if !trimmed.contains(char::is_whitespace)
        && (trimmed.starts_with("https://") || trimmed.starts_with("http://"))
    {
        ClipboardContentType::Url
    } else if looks_code(trimmed) {
        ClipboardContentType::Code
    } else if trimmed.contains('\n') {
        ClipboardContentType::Multiline
    } else {
        ClipboardContentType::Text
    }
}

fn looks_code(text: &str) -> bool {
    let markers = [
        "const ", "let ", "fn ", "def ", "class ", "import ", "SELECT ", "=>", "::", "{",
    ];
    markers.iter().filter(|marker| text.contains(**marker)).count() >= 2
}

fn preview(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n").trim().to_owned();
    truncate_chars(&normalized, 320)
}

fn truncate_chars(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_owned();
    }
    let mut truncated: String = value.chars().take(max).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_text() {
        let mut history = ClipboardHistory::default();
        assert!(history.add_text("hello".into(), 5));
        assert!(!history.add_text("hello".into(), 5));
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn limit() {
        let mut history = ClipboardHistory::default();
        for value in ["a", "b", "c", "d"] {
            history.add_text(value.into(), 3);
        }
        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().item.content.as_deref(), Some("d"));
    }

    #[test]
    fn promote() {
        let mut history = ClipboardHistory::default();
        for value in ["a", "b", "c"] {
            history.add_text(value.into(), 5);
        }
        let id = history.get(2).unwrap().item.id;
        assert!(history.promote(&id));
        assert_eq!(history.get(0).unwrap().item.content.as_deref(), Some("a"));
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn url() {
        let mut history = ClipboardHistory::default();
        history.add_text("https://example.com".into(), 5);
        assert_eq!(
            history.get(0).unwrap().item.content_type,
            ClipboardContentType::Url
        );
    }

    #[test]
    fn image_round_trip_metadata_and_dedup() {
        let mut history = ClipboardHistory::default();
        let rgba = vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255];
        assert!(history.add_image(2, 2, rgba.clone(), 5).unwrap());
        assert!(!history.add_image(2, 2, rgba, 5).unwrap());
        let entry = history.get(0).unwrap();
        assert_eq!(entry.item.content_type, ClipboardContentType::Image);
        assert_eq!(entry.item.metadata.width, Some(2));
        assert_eq!(entry.item.metadata.height, Some(2));
        assert!(entry.item.thumbnail_data_url.is_some());
        assert!(entry.image_png.is_some());
    }
}
