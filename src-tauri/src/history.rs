use std::collections::VecDeque;
use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::models::{ClipboardContentType, ClipboardItem, ClipboardMetadata};

#[derive(Debug, Default)] pub struct ClipboardHistory { items: VecDeque<ClipboardItem> }
impl ClipboardHistory {
    pub fn from_items(items: Vec<ClipboardItem>, limit: usize) -> Self { let mut h=Self{items:items.into()}; h.truncate(limit); h }
    pub fn add_text(&mut self, text: String, max: usize) -> bool {
        let hash=hash_text(&text); if self.items.front().is_some_and(|x|x.hash==hash){return false}
        let item=ClipboardItem{id:Uuid::new_v4().to_string(),content_type:classify(&text),preview:preview(&text),metadata:ClipboardMetadata{character_count:text.chars().count(),line_count:text.lines().count().max(1)},content:text,created_at:Utc::now(),hash};
        self.items.push_front(item); self.truncate(max); true
    }
    pub fn items(&self)->Vec<ClipboardItem>{self.items.iter().cloned().collect()}
    pub fn len(&self)->usize{self.items.len()}
    pub fn get(&self,i:usize)->Option<ClipboardItem>{self.items.get(i).cloned()}
    pub fn find(&self,id:&str)->Option<ClipboardItem>{self.items.iter().find(|x|x.id==id).cloned()}
    pub fn clear(&mut self){self.items.clear()}
    pub fn promote(&mut self,id:&str)->bool{let Some(i)=self.items.iter().position(|x|x.id==id) else{return false};if i==0{return false}if let Some(x)=self.items.remove(i){self.items.push_front(x);true}else{false}}
    pub fn truncate(&mut self,max:usize){self.items.truncate(max.max(1))}
}
pub fn hash_text(text:&str)->String{hex::encode(Sha256::digest(text.as_bytes()))}
fn classify(text:&str)->ClipboardContentType{let t=text.trim();if !t.contains(char::is_whitespace)&&(t.starts_with("https://")||t.starts_with("http://")){ClipboardContentType::Url}else if looks_code(t){ClipboardContentType::Code}else if t.contains('\n'){ClipboardContentType::Multiline}else{ClipboardContentType::Text}}
fn looks_code(t:&str)->bool{let markers=["const ","let ","fn ","def ","class ","import ","SELECT ","=>","::","{"];markers.iter().filter(|m|t.contains(**m)).count()>=2}
fn preview(text:&str)->String{let t=text.replace("\r\n","\n").trim().to_owned();if t.chars().count()<=320{t}else{let mut s:String=t.chars().take(320).collect();s.push('…');s}}
#[cfg(test)]mod tests{use super::*;#[test]fn dedup(){let mut h=ClipboardHistory::default();assert!(h.add_text("hello".into(),5));assert!(!h.add_text("hello".into(),5));assert_eq!(h.len(),1)}#[test]fn limit(){let mut h=ClipboardHistory::default();for x in ["a","b","c","d"]{h.add_text(x.into(),3);}assert_eq!(h.len(),3);assert_eq!(h.get(0).unwrap().content,"d")}#[test]fn promote(){let mut h=ClipboardHistory::default();for x in ["a","b","c"]{h.add_text(x.into(),5);}let id=h.get(2).unwrap().id;assert!(h.promote(&id));assert_eq!(h.get(0).unwrap().content,"a");assert_eq!(h.len(),3)}#[test]fn url(){let mut h=ClipboardHistory::default();h.add_text("https://example.com".into(),5);assert_eq!(h.get(0).unwrap().content_type,ClipboardContentType::Url)}}
