use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, path::Path};

use eframe::egui::{
    self, Align, Color32, FontData, FontDefinitions, FontFamily, Frame, Layout, Margin, RichText,
    Rounding, Stroke, Vec2, epaint::Shadow,
};
use log::{error, warn};
use scraper::{Html, Selector};
use serde::Deserialize;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Mutex, mpsc};

use crate::engine::LauncherEngine;
use crate::engine::state::{AppState, AuthMode, UserAction};
use crate::env;
use crate::mods::{CurseForgeMod, InstalledMod, ModAuthor};
use crate::process::ProcessLauncher;
use crate::storage::StorageManager;
use crate::updater::{self, UpdateStatus};

mod i18n;
use self::i18n::{I18n, Language};

const NEWS_PATH: &str = "assets/news.json";
const NEWS_URL: &str = "https://hytale.com/news";
const NEWS_MAX_ITEMS: usize = 6;
const NEWS_PREVIEW_FALLBACK_EN: &str = "Read more on hytale.com.";
const PLAYER_NAME_FILE: &str = "player_name.txt";
const SELECTED_VERSION_FILE: &str = "selected_version.txt";
const DEFAULT_PLAYER_NAME: &str = "Player";
const DIAGNOSTICS_REPORT_HEIGHT: f32 = 720.0;
const NOTO_SANS_FONT_ID: &str = "noto_sans_regular";
const NOTO_SANS_FONT_CN_ID: &str = "noto_sans_sc_regular";
const NOTO_SANS_REGULAR: &[u8] = include_bytes!("../../NotoSans-Regular.ttf");
const NOTO_SANS_SC_REGULAR: &[u8] = include_bytes!("../../NotoSansSC-Regular.ttf");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThemePalette {
    bg: Color32,
    panel: Color32,
    surface: Color32,
    surface_elev: Color32,
    sunken_surface: Color32,
    border: Color32,
    border_strong: Color32,
    text_primary: Color32,
    text_muted: Color32,
    text_faint: Color32,
    accent: Color32,
    accent_soft: Color32,
    accent_glow: Color32,
    info: Color32,
    warning: Color32,
    danger: Color32,
    diagnostic: Color32,
}

impl ThemePalette {
    const fn dark() -> Self {
        Self {
            bg: Color32::from_rgb(11, 14, 19),
            panel: Color32::from_rgb(17, 22, 29),
            surface: Color32::from_rgb(24, 31, 39),
            surface_elev: Color32::from_rgb(29, 37, 47),
            sunken_surface: Color32::from_rgb(14, 18, 24),
            border: Color32::from_rgb(45, 57, 72),
            border_strong: Color32::from_rgb(63, 79, 97),
            text_primary: Color32::from_rgb(228, 235, 244),
            text_muted: Color32::from_rgb(167, 182, 197),
            text_faint: Color32::from_rgb(129, 143, 158),
            accent: Color32::from_rgb(92, 219, 195),
            accent_soft: Color32::from_rgb(63, 140, 125),
            accent_glow: Color32::from_rgb(151, 239, 217),
            info: Color32::from_rgb(122, 186, 255),
            warning: Color32::from_rgb(246, 195, 111),
            danger: Color32::from_rgb(239, 117, 117),
            diagnostic: Color32::from_rgb(200, 160, 245),
        }
    }

    const fn light() -> Self {
        Self {
            bg: Color32::from_rgb(240, 245, 252),
            panel: Color32::from_rgb(226, 234, 243),
            surface: Color32::from_rgb(245, 249, 255),
            surface_elev: Color32::from_rgb(255, 255, 255),
            sunken_surface: Color32::from_rgb(217, 225, 236),
            border: Color32::from_rgb(195, 205, 221),
            border_strong: Color32::from_rgb(172, 186, 206),
            text_primary: Color32::from_rgb(28, 38, 52),
            text_muted: Color32::from_rgb(80, 99, 121),
            text_faint: Color32::from_rgb(116, 135, 155),
            accent: Color32::from_rgb(27, 170, 152),
            accent_soft: Color32::from_rgb(152, 223, 212),
            accent_glow: Color32::from_rgb(16, 190, 173),
            info: Color32::from_rgb(64, 120, 212),
            warning: Color32::from_rgb(235, 164, 70),
            danger: Color32::from_rgb(219, 83, 83),
            diagnostic: Color32::from_rgb(150, 110, 205),
        }
    }
}

impl Theme {
    const fn palette(self) -> ThemePalette {
        match self {
            Theme::Dark => ThemePalette::dark(),
            Theme::Light => ThemePalette::light(),
        }
    }
}

fn tint(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha)
}

const LOCALE_LANGUAGE_CODES: [(&[&str], Language); 10] = [
    (&["zh", "zho", "chi"], Language::Chinese),
    (&["hi", "hin"], Language::Hindi),
    (&["ru", "rus"], Language::Russian),
    (&["tr", "tur"], Language::Turkish),
    (&["uk", "ua", "ukr"], Language::Ukrainian),
    (&["es", "spa"], Language::Spanish),
    (&["fr", "fra", "fre"], Language::French),
    (&["de", "deu", "ger"], Language::German),
    (&["pt", "por"], Language::Portuguese),
    (&["en", "eng"], Language::English),
];

fn parse_locale_token(token: &str) -> Option<Language> {
    let normalized = token
        .split(|c| matches!(c, '.' | '@'))
        .next()
        .unwrap_or(token)
        .replace('-', "_")
        .to_ascii_lowercase();
    let language_code = normalized.split('_').next().unwrap_or(&normalized);

    LOCALE_LANGUAGE_CODES.iter().find_map(|(codes, language)| {
        codes
            .iter()
            .any(|code| *code == language_code)
            .then_some(*language)
    })
}

fn detect_system_language() -> Language {
    for var in ["LC_ALL", "LANGUAGE", "LANG"] {
        if let Ok(value) = std::env::var(var) {
            for token in value.split(':') {
                if let Some(language) = parse_locale_token(token) {
                    return language;
                }
            }
        }
    }

    Language::English
}

#[cfg(test)]
mod tests {
    use super::{Language, parse_locale_token};

    #[test]
    fn parses_supported_languages_from_locale_tokens() {
        let samples = [
            ("en_US.UTF-8", Language::English),
            ("uk_UA.UTF-8", Language::Ukrainian),
            ("es-ES", Language::Spanish),
            ("fr_FR", Language::French),
            ("de-DE", Language::German),
            ("pt-BR", Language::Portuguese),
            ("zh-Hans", Language::Chinese),
            ("hi_IN", Language::Hindi),
            ("ru_RU", Language::Russian),
            ("tr_TR", Language::Turkish),
            ("ua-UA", Language::Ukrainian),
            ("eng_US", Language::English),
        ];

        for (token, expected) in samples {
            assert_eq!(parse_locale_token(token), Some(expected));
        }
    }

    #[test]
    fn ignores_unknown_language_tokens() {
        assert_eq!(parse_locale_token("pl_PL"), None);
    }
}

fn badge_frame(color: Color32) -> Frame {
    Frame::none()
        .fill(tint(color, 32))
        .stroke(Stroke::new(1.0, color))
        .rounding(Rounding::same(999.0))
        .inner_margin(Margin::symmetric(10.0, 4.0))
}

fn chip_frame(color: Color32) -> Frame {
    Frame::none()
        .fill(tint(color, 24))
        .stroke(Stroke::new(1.0, tint(color, 140)))
        .rounding(Rounding::same(999.0))
        .inner_margin(Margin::symmetric(8.0, 3.0))
}

fn meta_chip_frame(colors: &ThemePalette) -> Frame {
    Frame::none()
        .fill(tint(colors.text_muted, 48))
        .stroke(Stroke::new(1.0, tint(colors.text_muted, 200)))
        .rounding(Rounding::same(999.0))
        .inner_margin(Margin::symmetric(8.0, 3.0))
}

fn primary_badge_frame(colors: &ThemePalette) -> Frame {
    Frame::none()
        .fill(colors.accent_soft)
        .stroke(Stroke::new(1.0, colors.accent))
        .rounding(Rounding::same(999.0))
        .inner_margin(Margin::symmetric(10.0, 4.0))
}

fn primary_cta_button(
    label: impl Into<egui::WidgetText>,
    colors: &ThemePalette,
    min_width: f32,
) -> egui::Button<'_> {
    egui::Button::new(label)
        .fill(colors.accent_soft)
        .stroke(Stroke::new(1.0, colors.accent))
        .min_size(Vec2::new(min_width, 34.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModSort {
    Downloads,
    Updated,
    Name,
}

fn load_news_from_file() -> Vec<NewsItem> {
    let path = Path::new(NEWS_PATH);
    if let Ok(raw) = fs::read_to_string(path)
        && let Ok(parsed) = serde_json::from_str::<Vec<NewsItem>>(&raw)
    {
        return parsed;
    }
    Vec::new()
}

fn load_player_name_from_file() -> String {
    let path = env::default_app_dir().join(PLAYER_NAME_FILE);
    if let Ok(raw) = fs::read_to_string(path) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    DEFAULT_PLAYER_NAME.to_owned()
}

fn load_selected_version_from_file() -> Option<u32> {
    let path = env::default_app_dir().join(SELECTED_VERSION_FILE);
    if let Ok(raw) = fs::read_to_string(path) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        return trimmed.parse::<u32>().ok().filter(|version| *version > 0);
    }
    None
}

fn save_player_name_to_file(name: &str) -> Result<(), String> {
    let path = env::default_app_dir().join(PLAYER_NAME_FILE);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create player name dir: {err}"))?;
    }
    fs::write(path, name.as_bytes()).map_err(|err| format!("failed to save player name: {err}"))
}

fn save_selected_version_to_file(version: Option<u32>) -> Result<(), String> {
    let path = env::default_app_dir().join(SELECTED_VERSION_FILE);
    match version {
        Some(value) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("failed to create selected version dir: {err}"))?;
            }
            let contents = value.to_string();
            fs::write(&path, contents.as_bytes())
                .map_err(|err| format!("failed to save selected version: {err}"))
        }
        None => {
            if fs::metadata(&path).is_ok() {
                fs::remove_file(&path)
                    .map_err(|err| format!("failed to clear selected version: {err}"))
            } else {
                Ok(())
            }
        }
    }
}

fn sanitize_player_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        DEFAULT_PLAYER_NAME.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn format_downloads(count: i64) -> String {
    let count = count.max(0) as f64;
    if count >= 1_000_000_000.0 {
        format!("{:.1}B", count / 1_000_000_000.0)
    } else if count >= 1_000_000.0 {
        format!("{:.1}M", count / 1_000_000.0)
    } else if count >= 1_000.0 {
        format!("{:.1}k", count / 1_000.0)
    } else {
        format!("{:.0}", count)
    }
}

fn format_mod_date(date: &str) -> Option<String> {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        None
    } else if let Some((ymd, _)) = trimmed.split_once('T') {
        Some(ymd.to_owned())
    } else if let Some((ymd, _)) = trimmed.split_once(' ') {
        Some(ymd.to_owned())
    } else {
        Some(trimmed.to_owned())
    }
}

fn format_authors(authors: &[ModAuthor]) -> Option<String> {
    if authors.is_empty() {
        return None;
    }
    let names: Vec<&str> = authors
        .iter()
        .take(2)
        .map(|author| author.name.as_str())
        .collect();
    let mut label = names.join(", ");
    if authors.len() > 2 {
        let extra = authors.len() - 2;
        let _ = write!(label, " +{extra}");
    }
    Some(label)
}

fn mod_page_url(mod_ref: &CurseForgeMod) -> String {
    format!("https://www.curseforge.com/hytale/mods/{}", mod_ref.slug)
}

fn collect_mod_categories(mods: &[CurseForgeMod]) -> Vec<String> {
    let mut categories: Vec<String> = mods
        .iter()
        .flat_map(|m| m.categories.iter().map(|category| category.name.clone()))
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_owned();
    }
    let mut out = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= max_len {
            out.push_str("...");
            break;
        }
        out.push(ch);
    }
    out
}

fn clean_news_preview(title: &str, preview: &str) -> String {
    let mut cleaned = preview.trim().to_owned();
    let title = title.trim();
    if !title.is_empty()
        && let Some(rest) = cleaned.strip_prefix(title)
    {
        let rest = rest
            .trim_start_matches(|ch: char| {
                ch.is_whitespace() || matches!(ch, '-' | ':' | '!' | '?' | '.' | ',')
            })
            .to_owned();
        if !rest.is_empty() {
            cleaned = rest;
        }
    }

    let mut fixed = String::with_capacity(cleaned.len() + 8);
    let mut prev: Option<char> = None;
    let mut word_len = 0usize;
    for ch in cleaned.chars() {
        if let Some(prev_ch) = prev {
            let needs_space = ((prev_ch == '!' || prev_ch == '?' || prev_ch == '.')
                && ch.is_ascii_uppercase())
                || (prev_ch.is_ascii_digit() && ch.is_ascii_uppercase())
                || (prev_ch.is_ascii_lowercase() && ch.is_ascii_uppercase() && word_len > 2);
            if needs_space {
                fixed.push(' ');
                word_len = 0;
            }
        }
        fixed.push(ch);
        if ch.is_whitespace() {
            word_len = 0;
        } else {
            word_len = word_len.saturating_add(1);
        }
        prev = Some(ch);
    }

    fixed
}

fn element_text(element: scraper::element_ref::ElementRef<'_>) -> String {
    normalize_text(
        &element
            .text()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn link_from_element(element: &scraper::element_ref::ElementRef<'_>) -> Option<String> {
    if element.value().name() == "a"
        && let Some(href) = element.value().attr("href")
    {
        return Some(href.to_owned());
    }
    let link_selector = Selector::parse("a[href]").ok()?;
    element
        .select(&link_selector)
        .filter_map(|link| link.value().attr("href"))
        .map(str::to_owned)
        .next()
}

fn normalize_news_url(href: &str) -> Option<String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(href.to_owned());
    }
    if href.starts_with('/') {
        let mut url = String::from("https://hytale.com");
        url.push_str(href);
        return Some(url);
    }
    None
}

fn parse_news_from_html(body: &str) -> Vec<NewsItem> {
    let document = Html::parse_document(body);
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    let card_selectors = [
        ".postWrapper",
        ".post",
        ".news-card",
        ".news-item",
        ".news__card",
        ".post-card",
        "article",
        ".card",
    ];

    let title_selectors = [
        "h1",
        "h2",
        "h3",
        "h4",
        ".title",
        ".card-title",
        ".news-card__title",
        ".post-title",
        ".post__details__heading",
    ];
    let summary_selectors = [
        "p",
        ".summary",
        ".excerpt",
        ".card-excerpt",
        ".news-card__excerpt",
        ".post__details__body",
    ];

    for selector in &card_selectors {
        let selector = match Selector::parse(selector) {
            Ok(sel) => sel,
            Err(_) => continue,
        };
        for card in document.select(&selector) {
            let href = match link_from_element(&card) {
                Some(href) => href,
                None => continue,
            };
            let url = match normalize_news_url(&href) {
                Some(url) => url,
                None => continue,
            };
            if !url.contains("/news/") || url.ends_with("/news") || url.ends_with("/news/") {
                continue;
            }
            if !seen.insert(url.clone()) {
                continue;
            }

            let title = title_selectors.iter().find_map(|sel| {
                Selector::parse(sel)
                    .ok()
                    .and_then(|selector| card.select(&selector).next())
                    .map(element_text)
                    .filter(|text| !text.is_empty())
            });

            let summary = summary_selectors.iter().find_map(|sel| {
                Selector::parse(sel)
                    .ok()
                    .and_then(|selector| card.select(&selector).next())
                    .map(element_text)
                    .filter(|text| !text.is_empty())
            });

            let title = title.unwrap_or_else(|| element_text(card));
            if title.is_empty() {
                continue;
            }
            let summary = summary.unwrap_or_else(|| NEWS_PREVIEW_FALLBACK_EN.to_owned());
            let summary = clean_news_preview(&title, &summary);
            let summary = if summary.is_empty() {
                NEWS_PREVIEW_FALLBACK_EN.to_owned()
            } else {
                summary
            };

            items.push(NewsItem {
                title: truncate_text(&title, 80),
                preview: truncate_text(&summary, 160),
                url,
            });

            if items.len() >= NEWS_MAX_ITEMS {
                return items;
            }
        }
    }

    let link_selector = match Selector::parse("a[href*=\"/news/\"]") {
        Ok(sel) => sel,
        Err(_) => return items,
    };
    for link in document.select(&link_selector) {
        let href = match link.value().attr("href") {
            Some(href) => href.to_owned(),
            None => continue,
        };
        let url = match normalize_news_url(&href) {
            Some(url) => url,
            None => continue,
        };
        if url.ends_with("/news") || url.ends_with("/news/") {
            continue;
        }
        if !seen.insert(url.clone()) {
            continue;
        }

        let title = element_text(link);
        if title.is_empty() {
            continue;
        }

        items.push(NewsItem {
            title: truncate_text(&title, 80),
            preview: NEWS_PREVIEW_FALLBACK_EN.into(),
            url,
        });

        if items.len() >= NEWS_MAX_ITEMS {
            break;
        }
    }

    items
}

async fn fetch_news_from_web() -> Result<Vec<NewsItem>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(NEWS_URL)
        .header("User-Agent", "HytaleLauncher/0.1")
        .send()
        .await
        .map_err(|err| err.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("News request failed: {}", resp.status()));
    }
    let body = resp.text().await.map_err(|err| err.to_string())?;
    let items = parse_news_from_html(&body);
    if items.is_empty() {
        return Err("No news entries found.".into());
    }
    Ok(items)
}

fn build_runtime() -> Arc<Runtime> {
    match Runtime::new() {
        Ok(rt) => Arc::new(rt),
        Err(err) => {
            warn!(
                "ui: failed to create multithreaded runtime ({}); trying single-threaded runtime",
                err
            );
            match Builder::new_current_thread().enable_all().build() {
                Ok(rt) => Arc::new(rt),
                Err(fallback_err) => {
                    error!(
                        "ui: failed to create any Tokio runtime ({}); terminating launcher",
                        fallback_err
                    );
                    std::process::exit(1);
                }
            }
        }
    }
}

pub struct LauncherApp {
    runtime: Arc<Runtime>,
    engine: Arc<Mutex<LauncherEngine>>,
    cancel_flag: Arc<AtomicBool>,
    updates_rx: mpsc::UnboundedReceiver<AppState>,
    updates_tx: mpsc::UnboundedSender<AppState>,
    state: AppState,
    launcher_version: &'static str,
    language: Language,
    fonts_language: Language,
    theme: Theme,
    news: Vec<NewsItem>,
    news_loading: bool,
    news_error: Option<String>,
    player_name: String,
    player_name_error: Option<String>,
    auth_mode: AuthMode,
    available_versions: Vec<u32>,
    selected_version: Option<u32>,
    version_input: String,
    version_loading: bool,
    version_fetch_error: Option<String>,
    version_input_error: Option<String>,
    diagnostics: Option<String>,
    show_diagnostics_modal: bool,
    show_uninstall_confirm: bool,
    mod_query: String,
    mod_sort: ModSort,
    mod_category_filter: Option<String>,
    mod_results: Vec<CurseForgeMod>,
    mod_loading: bool,
    mod_error: Option<String>,
    installed_mods: Vec<InstalledMod>,
    installed_loading: bool,
    installed_error: Option<String>,
    removing_mod: Option<String>,
    mod_updates_rx: mpsc::UnboundedReceiver<ModUpdate>,
    mod_updates_tx: mpsc::UnboundedSender<ModUpdate>,
    news_updates_rx: mpsc::UnboundedReceiver<NewsUpdate>,
    news_updates_tx: mpsc::UnboundedSender<NewsUpdate>,
    version_updates_rx: mpsc::UnboundedReceiver<VersionUpdate>,
    version_updates_tx: mpsc::UnboundedSender<VersionUpdate>,
    updater_status: UpdateStatus,
    updater_loading: bool,
    updater_updates_rx: mpsc::UnboundedReceiver<UpdaterUpdate>,
    updater_updates_tx: mpsc::UnboundedSender<UpdaterUpdate>,
}

#[derive(Debug, Clone, Deserialize)]
struct NewsItem {
    title: String,
    preview: String,
    url: String,
}

#[derive(Debug)]
enum ModUpdate {
    Results(Vec<CurseForgeMod>),
    Error(String),
    Installed(Vec<InstalledMod>),
    InstalledError(String),
    Removed { id: String, error: Option<String> },
}

#[derive(Debug)]
enum NewsUpdate {
    Results(Vec<NewsItem>),
    Error(String),
}

#[derive(Debug)]
enum VersionUpdate {
    Available { versions: Vec<u32>, latest: u32 },
    Error(String),
}

#[derive(Debug)]
enum UpdaterUpdate {
    Status(UpdateStatus),
}

fn section_frame(colors: &ThemePalette) -> Frame {
    Frame::none()
        .fill(colors.surface)
        .stroke(Stroke::new(1.0, colors.border))
        .rounding(Rounding::same(14.0))
        .inner_margin(Margin::same(14.0))
}

fn elevated_frame(colors: &ThemePalette) -> Frame {
    Frame::none()
        .fill(colors.surface_elev)
        .stroke(Stroke::new(1.0, colors.border_strong))
        .rounding(Rounding::same(12.0))
        .inner_margin(Margin::symmetric(12.0, 10.0))
        .shadow(Shadow {
            offset: Vec2::new(0.0, 2.0),
            blur: 10.0,
            spread: 0.0,
            color: Color32::from_black_alpha(70),
        })
}

fn setup_custom_fonts(ctx: &egui::Context, language: Language) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        NOTO_SANS_FONT_ID.to_owned(),
        FontData::from_static(NOTO_SANS_REGULAR),
    );
    fonts.font_data.insert(
        NOTO_SANS_FONT_CN_ID.to_owned(),
        FontData::from_static(NOTO_SANS_SC_REGULAR),
    );

    let (primary, fallback) = if language == Language::Chinese {
        (NOTO_SANS_FONT_CN_ID, NOTO_SANS_FONT_ID)
    } else {
        (NOTO_SANS_FONT_ID, NOTO_SANS_FONT_CN_ID)
    };

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, primary.to_owned());
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push(fallback.to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, primary.to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(fallback.to_owned());

    ctx.set_fonts(fonts);
}

fn apply_theme(ctx: &egui::Context, colors: &ThemePalette) {
    let is_dark = colors == &ThemePalette::dark();
    let mut visuals = if is_dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = colors.bg;
    visuals.window_fill = visuals.panel_fill;
    visuals.override_text_color = Some(colors.text_primary);
    visuals.hyperlink_color = colors.accent_glow;
    visuals.widgets.noninteractive.rounding = Rounding::same(10.0);
    visuals.widgets.inactive.rounding = Rounding::same(10.0);
    visuals.widgets.hovered.rounding = Rounding::same(10.0);
    visuals.widgets.active.rounding = Rounding::same(10.0);
    visuals.widgets.noninteractive.bg_fill = colors.surface;
    visuals.widgets.inactive.bg_fill = colors.surface;
    visuals.widgets.hovered.bg_fill = colors.accent_glow;
    visuals.widgets.active.bg_fill = colors.accent_soft;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors.border);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors.border);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, colors.accent_glow);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, colors.accent);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text_muted);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text_muted);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors.text_primary);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors.text_primary);
    visuals.selection.bg_fill = colors.accent;
    visuals.selection.stroke = Stroke::new(1.0, colors.accent_glow);
    visuals.faint_bg_color = colors.sunken_surface;
    visuals.extreme_bg_color = tint(colors.sunken_surface, 255);
    visuals.code_bg_color = colors.sunken_surface;
    visuals.window_rounding = Rounding::same(14.0);
    let shadow_color = if is_dark {
        Color32::from_black_alpha(100)
    } else {
        Color32::from_black_alpha(45)
    };
    visuals.window_shadow = Shadow {
        offset: Vec2::new(0.0, 6.0),
        blur: 18.0,
        spread: 0.0,
        color: shadow_color,
    };
    visuals.popup_shadow = visuals.window_shadow;

    if is_dark {
        visuals.widgets.inactive.bg_fill = colors.surface_elev;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors.border_strong);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text_muted);

        visuals.widgets.hovered.bg_fill = colors.accent_soft;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.3, colors.accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors.text_primary);

        visuals.widgets.active.bg_fill = colors.accent;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, colors.accent_glow);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors.text_primary);
    }

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(12.0, 12.0);
    style.spacing.button_padding = Vec2::new(16.0, 10.0);
    ctx.set_style(style);
}

fn refresh_fonts_if_needed(app: &mut LauncherApp, ctx: &egui::Context) {
    if app.fonts_language != app.language {
        setup_custom_fonts(ctx, app.language);
        app.fonts_language = app.language;
    }
}

impl LauncherApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let runtime = build_runtime();

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let engine = LauncherEngine::new(
            StorageManager::new(),
            ProcessLauncher::new(),
            cancel_flag.clone(),
        );
        let engine = Arc::new(Mutex::new(engine));
        let (tx, rx) = mpsc::unbounded_channel();
        let (mod_tx, mod_rx) = mpsc::unbounded_channel();
        let (news_tx, news_rx) = mpsc::unbounded_channel();
        let (version_tx, version_rx) = mpsc::unbounded_channel();
        let (updater_tx, updater_rx) = mpsc::unbounded_channel();

        let bootstrap_engine = engine.clone();
        let bootstrap_tx = tx.clone();
        let bootstrap_rt = runtime.clone();
        bootstrap_rt.spawn(async move {
            let mut locked = bootstrap_engine.lock().await;
            locked.load_local_state(&bootstrap_tx).await;
        });
        let saved_version = load_selected_version_from_file();
        let version_input = saved_version
            .map(|version| version.to_string())
            .unwrap_or_default();
        let language = detect_system_language();
        setup_custom_fonts(&cc.egui_ctx, language);

        let mut app = Self {
            runtime,
            engine,
            cancel_flag,
            updates_rx: rx,
            updates_tx: tx,
            state: AppState::Initialising,
            launcher_version: env!("CARGO_PKG_VERSION"),
            language,
            fonts_language: language,
            theme: Theme::Dark,
            news: load_news_from_file(),
            news_loading: false,
            news_error: None,
            player_name: load_player_name_from_file(),
            player_name_error: None,
            auth_mode: AuthMode::Offline,
            available_versions: Vec::new(),
            selected_version: saved_version,
            version_input,
            version_loading: false,
            version_fetch_error: None,
            version_input_error: None,
            diagnostics: None,
            show_diagnostics_modal: false,
            show_uninstall_confirm: false,
            mod_query: String::new(),
            mod_sort: ModSort::Downloads,
            mod_category_filter: None,
            mod_results: Vec::new(),
            mod_loading: false,
            mod_error: None,
            installed_mods: Vec::new(),
            installed_loading: false,
            installed_error: None,
            removing_mod: None,
            mod_updates_rx: mod_rx,
            mod_updates_tx: mod_tx,
            news_updates_rx: news_rx,
            news_updates_tx: news_tx,
            version_updates_rx: version_rx,
            version_updates_tx: version_tx,
            updater_status: UpdateStatus::UpToDate,
            updater_loading: false,
            updater_updates_rx: updater_rx,
            updater_updates_tx: updater_tx,
        };

        app.start_news_fetch();
        app.start_version_discovery();
        app.start_updater_check();
        app.start_load_installed_mods();
        app
    }

    fn colors(&self) -> ThemePalette {
        self.theme.palette()
    }

    fn i18n(&self) -> I18n {
        I18n::new(self.language)
    }

    fn game_installed(&self) -> bool {
        let game_dir = env::game_latest_dir();
        let client_path = if cfg!(target_os = "windows") {
            game_dir.join("Client").join("HytaleClient.exe")
        } else if cfg!(target_os = "macos") {
            game_dir
                .join("Client")
                .join("Hytale.app")
                .join("Contents")
                .join("MacOS")
                .join("HytaleClient")
        } else {
            game_dir.join("Client").join("HytaleClient")
        };
        client_path.exists() || game_dir.exists()
    }

    fn trigger_action(&self, action: UserAction) {
        if matches!(action, UserAction::ClickCancelDownload) {
            self.cancel_flag.store(true, Ordering::SeqCst);
        }
        let engine = self.engine.clone();
        let tx = self.updates_tx.clone();
        let rt = self.runtime.clone();
        rt.spawn(async move {
            let mut locked = engine.lock().await;
            locked.handle_action(action, &tx).await;
        });
    }

    fn start_mod_search(&mut self) {
        let trimmed = self.mod_query.trim();
        if trimmed.is_empty() || self.mod_loading {
            return;
        }
        self.mod_error = None;
        self.mod_loading = true;
        let query = trimmed.to_owned();
        let tx = self.mod_updates_tx.clone();
        let engine = self.engine.clone();
        let rt = self.runtime.clone();
        rt.spawn(async move {
            let service = {
                let locked = engine.lock().await;
                locked.mods_service()
            };
            let result = service.search(&query, 0).await;
            match result {
                Ok(resp) => {
                    let _ = tx.send(ModUpdate::Results(resp.data));
                }
                Err(err) => {
                    let _ = tx.send(ModUpdate::Error(err));
                }
            }
        });
    }

    fn start_load_installed_mods(&mut self) {
        if self.installed_loading {
            return;
        }
        self.installed_loading = true;
        self.installed_error = None;
        let tx = self.mod_updates_tx.clone();
        let engine = self.engine.clone();
        let rt = self.runtime.clone();
        rt.spawn(async move {
            let service = {
                let locked = engine.lock().await;
                locked.mods_service()
            };
            let result = service.installed_mods().await;
            match result {
                Ok(installed) => {
                    let _ = tx.send(ModUpdate::Installed(installed));
                }
                Err(err) => {
                    let _ = tx.send(ModUpdate::InstalledError(err));
                }
            }
        });
    }

    fn start_remove_installed_mod(&mut self, mod_id: String) {
        if self.installed_loading {
            return;
        }
        self.removing_mod = Some(mod_id.clone());
        self.installed_loading = true;
        self.installed_error = None;
        let tx = self.mod_updates_tx.clone();
        let engine = self.engine.clone();
        let rt = self.runtime.clone();
        rt.spawn(async move {
            let service = {
                let locked = engine.lock().await;
                locked.mods_service()
            };
            let result = service.remove_installed(&mod_id).await;
            let update = match result {
                Ok(_) => ModUpdate::Removed {
                    id: mod_id.clone(),
                    error: None,
                },
                Err(err) => ModUpdate::Removed {
                    id: mod_id.clone(),
                    error: Some(err),
                },
            };
            let _ = tx.send(update);
        });
    }

    fn commit_player_name(&mut self) -> String {
        let cleaned = sanitize_player_name(&self.player_name);
        self.player_name = cleaned.clone();
        match save_player_name_to_file(&cleaned) {
            Ok(()) => {
                self.player_name_error = None;
            }
            Err(err) => {
                self.player_name_error = Some(err);
            }
        }
        cleaned
    }

    fn start_news_fetch(&mut self) {
        if self.news_loading {
            return;
        }
        self.news_loading = true;
        let tx = self.news_updates_tx.clone();
        let rt = self.runtime.clone();
        rt.spawn(async move {
            match fetch_news_from_web().await {
                Ok(items) => {
                    let _ = tx.send(NewsUpdate::Results(items));
                }
                Err(err) => {
                    let _ = tx.send(NewsUpdate::Error(err));
                }
            }
        });
    }

    fn start_version_discovery(&mut self) {
        if self.version_loading {
            return;
        }
        self.version_loading = true;
        self.version_fetch_error = None;
        let tx = self.version_updates_tx.clone();
        let engine = self.engine.clone();
        let rt = self.runtime.clone();
        rt.spawn(async move {
            let storage = {
                let locked = engine.lock().await;
                locked.storage_clone()
            };
            let result = LauncherEngine::available_versions_with_storage(storage).await;
            if let Some(err) = result.error {
                let _ = tx.send(VersionUpdate::Error(err));
            } else {
                let _ = tx.send(VersionUpdate::Available {
                    versions: result.available_versions,
                    latest: result.latest_version,
                });
            }
        });
    }

    fn sync_state(&mut self) {
        while let Ok(state) = self.updates_rx.try_recv() {
            match &state {
                AppState::DiagnosticsReady { report } => {
                    self.diagnostics = Some(report.clone());
                    self.show_diagnostics_modal = true;
                    self.state = AppState::Idle;
                }
                AppState::ReadyToPlay { version } => {
                    if let Ok(parsed) = version.parse::<u32>() {
                        self.set_selected_version(Some(parsed));
                    }
                    self.state = state;
                }
                AppState::Idle => {
                    self.state = state;
                    self.start_load_installed_mods();
                }
                _ => {
                    self.state = state;
                }
            }
        }
    }

    fn sync_mod_updates(&mut self) {
        while let Ok(update) = self.mod_updates_rx.try_recv() {
            match update {
                ModUpdate::Results(results) => {
                    self.mod_loading = false;
                    self.mod_results = results;
                    self.mod_error = None;
                    if let Some(selected) = &self.mod_category_filter {
                        let still_valid = self.mod_results.iter().any(|m| {
                            m.categories
                                .iter()
                                .any(|category| category.name == *selected)
                        });
                        if !still_valid {
                            self.mod_category_filter = None;
                        }
                    }
                }
                ModUpdate::Error(err) => {
                    self.mod_loading = false;
                    self.mod_error = Some(err);
                }
                ModUpdate::Installed(mods) => {
                    self.installed_loading = false;
                    self.installed_mods = mods;
                    self.installed_error = None;
                    self.removing_mod = None;
                }
                ModUpdate::InstalledError(err) => {
                    self.installed_loading = false;
                    self.installed_error = Some(err);
                    self.removing_mod = None;
                }
                ModUpdate::Removed { id, error } => {
                    self.installed_loading = false;
                    self.removing_mod = None;
                    if let Some(err) = error {
                        self.installed_error = Some(err);
                    } else {
                        self.installed_mods.retain(|m| m.id != id);
                        self.installed_error = None;
                    }
                }
            }
        }
    }

    fn sync_news_updates(&mut self) {
        while let Ok(update) = self.news_updates_rx.try_recv() {
            self.news_loading = false;
            match update {
                NewsUpdate::Results(items) => {
                    if !items.is_empty() {
                        self.news = items;
                    }
                    self.news_error = None;
                }
                NewsUpdate::Error(err) => {
                    self.news_error = Some(err);
                }
            }
        }
    }

    fn sync_version_updates(&mut self) {
        while let Ok(update) = self.version_updates_rx.try_recv() {
            self.version_loading = false;
            match update {
                VersionUpdate::Available { versions, latest } => {
                    let mut deduped = versions;
                    deduped.sort_unstable_by(|a, b| b.cmp(a));
                    deduped.dedup();
                    self.available_versions = deduped;
                    self.version_fetch_error = None;

                    if self.selected_version.is_none() {
                        let candidate = self
                            .current_ready_version()
                            .or_else(|| self.available_versions.first().copied())
                            .or_else(|| (latest > 0).then_some(latest));
                        if let Some(version) = candidate {
                            self.set_selected_version(Some(version));
                        }
                    }
                }
                VersionUpdate::Error(err) => {
                    self.version_fetch_error = Some(err);
                }
            }
        }
    }

    fn start_updater_check(&mut self) {
        if self.updater_loading {
            return;
        }
        self.updater_loading = true;
        let tx = self.updater_updates_tx.clone();
        let current_version = self.launcher_version.to_owned();
        let rt = self.runtime.clone();
        rt.spawn(async move {
            match updater::check_for_updates(&current_version).await {
                Ok(status) => {
                    let _ = tx.send(UpdaterUpdate::Status(status));
                }
                Err(err) => {
                    let _ = tx.send(UpdaterUpdate::Status(UpdateStatus::CheckFailed(err)));
                }
            }
        });
    }

    fn sync_updater_updates(&mut self) {
        while let Ok(update) = self.updater_updates_rx.try_recv() {
            self.updater_loading = false;
            match update {
                UpdaterUpdate::Status(status) => {
                    self.updater_status = status;
                }
            }
        }
    }

    fn current_ready_version(&self) -> Option<u32> {
        match &self.state {
            AppState::ReadyToPlay { version } => version.parse::<u32>().ok(),
            _ => None,
        }
    }

    fn set_selected_version(&mut self, version: Option<u32>) {
        self.selected_version = version;
        self.version_input = version.map(|v| v.to_string()).unwrap_or_default();
        self.version_input_error = None;
        self.persist_selected_version();
    }

    fn sync_version_selection(&mut self, previous: Option<u32>) {
        if previous != self.selected_version {
            self.version_input = self
                .selected_version
                .map(|v| v.to_string())
                .unwrap_or_default();
            self.version_input_error = None;
            self.persist_selected_version();
        }
    }

    fn persist_selected_version(&self) {
        if let Err(err) = save_selected_version_to_file(self.selected_version) {
            warn!("failed to persist selected version: {}", err);
        }
    }

    fn apply_version_input(&mut self) {
        let trimmed = self.version_input.trim();
        if trimmed.is_empty() {
            self.set_selected_version(self.available_versions.first().copied());
            return;
        }
        match trimmed.parse::<u32>() {
            Ok(value) if value > 0 => {
                self.set_selected_version(Some(value));
            }
            _ => {
                let i18n = self.i18n();
                self.version_input_error = Some(i18n.version_input_error().to_owned());
            }
        }
    }

    fn render_discord_button(&self, ui: &mut egui::Ui, colors: &ThemePalette, i18n: I18n) {
        let discord_label = RichText::new(i18n.discord_button_label())
            .color(colors.text_primary)
            .strong();
        let discord_btn = primary_cta_button(discord_label, colors, 120.0);
        if ui.add(discord_btn).clicked() {
            ui.output_mut(|o| {
                o.open_url = Some(egui::output::OpenUrl {
                    url: "https://discord.gg/2ssYjNRXZ".into(),
                    new_tab: true,
                });
            });
        }
    }

    fn render_status(&mut self, ui: &mut egui::Ui, colors: &ThemePalette, i18n: I18n) {
        section_frame(colors).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(i18n.status_label()).color(colors.text_muted));
                ui.add_space(6.0);
                let status_badge = match &self.state {
                    AppState::ReadyToPlay { .. } => (i18n.status_ready(), colors.accent),
                    AppState::Playing => (i18n.status_running(), colors.info),
                    AppState::Error(_) => (i18n.status_attention(), colors.danger),
                    AppState::Downloading { .. } => (i18n.status_downloading(), colors.warning),
                    AppState::Uninstalling => (i18n.status_uninstalling(), colors.danger),
                    AppState::DiagnosticsRunning => (i18n.status_diagnostics(), colors.diagnostic),
                    _ => (i18n.status_working(), colors.text_faint),
                };
                if matches!(self.state, AppState::ReadyToPlay { .. }) {
                    primary_badge_frame(colors).show(ui, |ui| {
                        ui.label(
                            RichText::new(status_badge.0)
                                .color(colors.text_primary)
                                .strong(),
                        );
                    });
                } else {
                    badge_frame(status_badge.1).show(ui, |ui| {
                        ui.label(RichText::new(status_badge.0).color(status_badge.1).strong());
                    });
                }
            });
            ui.add_space(8.0);

            match &self.state {
                AppState::CheckingForUpdates => {
                    ui.label(i18n.checking());
                }
                AppState::Downloading {
                    file,
                    progress,
                    speed,
                } => {
                    ui.label(i18n.downloading(file));
                    ui.add(
                        egui::ProgressBar::new(progress / 100.0)
                            .fill(colors.accent)
                            .rounding(Rounding::same(10.0))
                            .desired_height(22.0)
                            .text(i18n.progress(*progress, speed)),
                    );
                }
                AppState::Uninstalling => {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new());
                        ui.label(i18n.uninstalling());
                    });
                }
                AppState::ReadyToPlay { version } => {
                    ui.label(RichText::new(i18n.ready(version)).strong());
                }
                AppState::DiagnosticsRunning => {
                    ui.label(i18n.diagnostics_running());
                }
                AppState::DiagnosticsReady { .. } => {
                    ui.label(i18n.diagnostics_completed());
                }
                AppState::Playing => {
                    ui.label(i18n.playing());
                }
                AppState::Error(msg) => {
                    ui.colored_label(colors.danger, i18n.error(msg));
                }
                AppState::Initialising => {
                    ui.label(i18n.initialising());
                }
                AppState::Idle => {
                    ui.label(i18n.idle());
                }
            }

            ui.add_space(10.0);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                let play_enabled = matches!(self.state, AppState::ReadyToPlay { .. });
                let busy_refresh = matches!(
                    self.state,
                    AppState::Downloading { .. }
                        | AppState::CheckingForUpdates
                        | AppState::DiagnosticsRunning
                        | AppState::Uninstalling
                        | AppState::Initialising
                );
                let play_label = RichText::new(i18n.play_button())
                    .color(if play_enabled {
                        colors.text_primary
                    } else {
                        colors.text_muted
                    })
                    .strong();
                let play_btn = primary_cta_button(play_label, colors, 120.0);
                if ui.add_enabled(play_enabled, play_btn).clicked() {
                    let player_name = self.commit_player_name();
                    self.trigger_action(UserAction::ClickPlay {
                        player_name,
                        auth_mode: self.auth_mode,
                    });
                }
                ui.add_space(8.0);
                let refresh_btn = egui::Button::new(i18n.status_refresh())
                    .fill(colors.surface_elev)
                    .stroke(Stroke::new(1.0, colors.border_strong))
                    .min_size(Vec2::new(110.0, 32.0));
                if ui.add_enabled(!busy_refresh, refresh_btn).clicked() {
                    self.trigger_action(UserAction::CheckForUpdates {
                        target_version: self.selected_version,
                    });
                }
            });
        });
    }

    fn render_news(&self, ui: &mut egui::Ui, colors: &ThemePalette, i18n: I18n) {
        section_frame(colors).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(i18n.news_heading());
                ui.label(
                    RichText::new(i18n.news_subheading())
                        .color(colors.text_muted)
                        .small(),
                );
                if self.news_loading {
                    ui.add(egui::Spinner::new());
                    ui.label(
                        RichText::new(i18n.news_updating())
                            .color(colors.text_muted)
                            .small(),
                    );
                }
            });
            ui.separator();

            if let Some(err) = &self.news_error {
                ui.colored_label(colors.danger, i18n.news_fetch_failed(err));
            }

            if self.news.is_empty() {
                ui.label(i18n.no_news());
                return;
            }

            for item in &self.news {
                elevated_frame(colors).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.hyperlink_to(RichText::new(&item.title).strong(), &item.url);
                        let preview = if item.preview == NEWS_PREVIEW_FALLBACK_EN {
                            i18n.news_preview_fallback()
                        } else {
                            item.preview.as_str()
                        };
                        ui.label(preview);
                    });
                });
            }
        });
    }

    fn render_mods(&mut self, ui: &mut egui::Ui, colors: &ThemePalette, i18n: I18n) {
        section_frame(colors).show(ui, |ui| {
            ui.set_min_height(676.0);
            ui.horizontal(|ui| {
                ui.heading(i18n.mods_heading());
                if self.mod_loading {
                    ui.add(egui::Spinner::new());
                    ui.label(i18n.mods_searching());
                } else if !self.mod_results.is_empty() {
                    ui.label(
                        RichText::new(i18n.mods_results_count(self.mod_results.len()))
                            .color(colors.text_muted),
                    );
                }
            });

            let game_installed = self.game_installed();
            let mod_actions_locked = matches!(
                self.state,
                AppState::Downloading { .. }
                    | AppState::CheckingForUpdates
                    | AppState::Uninstalling
                    | AppState::Playing
            );
            let can_install_mods = game_installed && !mod_actions_locked;
            if !game_installed {
                ui.colored_label(colors.warning, i18n.mods_requires_game());
                ui.add_space(4.0);
            }

            self.render_installed_mods(ui, colors, i18n, mod_actions_locked);
            ui.separator();

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let mods_search_hint = i18n.mods_search_hint();
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.mod_query)
                        .hint_text(mods_search_hint)
                        .desired_width(260.0),
                );
                if resp.changed() {
                    self.mod_error = None;
                }
                let can_search = !self.mod_query.trim().is_empty() && !self.mod_loading;
                let search_label = if self.mod_loading {
                    i18n.mods_searching()
                } else {
                    i18n.mods_search_button()
                };
                let search_clicked = ui
                    .add_enabled(
                        can_search,
                        egui::Button::new(search_label)
                            .fill(colors.accent)
                            .stroke(Stroke::new(1.0, colors.accent_glow))
                            .min_size(Vec2::new(96.0, 28.0)),
                    )
                    .clicked();
                let can_clear = !self.mod_loading
                    && (!self.mod_query.is_empty()
                        || !self.mod_results.is_empty()
                        || self.mod_error.is_some()
                        || self.mod_category_filter.is_some());
                if ui
                    .add_enabled(
                        can_clear,
                        egui::Button::new(i18n.mods_clear_button())
                            .fill(colors.surface_elev)
                            .stroke(Stroke::new(1.0, colors.border_strong))
                            .min_size(Vec2::new(80.0, 28.0)),
                    )
                    .clicked()
                {
                    self.mod_query.clear();
                    self.mod_results.clear();
                    self.mod_error = None;
                    self.mod_category_filter = None;
                }
                let enter_pressed =
                    resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (search_clicked || enter_pressed) && can_search {
                    self.start_mod_search();
                    ui.memory_mut(|m| m.request_focus(resp.id));
                }
            });

            let categories = collect_mod_categories(&self.mod_results);
            if let Some(selected) = &self.mod_category_filter
                && !categories.iter().any(|category| category == selected)
            {
                self.mod_category_filter = None;
            }

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(i18n.mods_sort_label())
                        .color(colors.text_muted)
                        .small(),
                );
                egui::ComboBox::from_id_source("mod_sort")
                    .selected_text(i18n.mod_sort_label(self.mod_sort))
                    .show_ui(ui, |ui| {
                        for option in [ModSort::Downloads, ModSort::Updated, ModSort::Name] {
                            ui.selectable_value(
                                &mut self.mod_sort,
                                option,
                                i18n.mod_sort_label(option),
                            );
                        }
                    });

                ui.label(
                    RichText::new(i18n.mods_category_label())
                        .color(colors.text_muted)
                        .small(),
                );
                ui.add_enabled_ui(!categories.is_empty(), |ui| {
                    let all_categories = i18n.mods_all_categories();
                    let selected = self
                        .mod_category_filter
                        .as_deref()
                        .unwrap_or(all_categories)
                        .to_string();
                    egui::ComboBox::from_id_source("mod_category")
                        .selected_text(selected)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.mod_category_filter,
                                None,
                                all_categories,
                            );
                            for category in &categories {
                                ui.selectable_value(
                                    &mut self.mod_category_filter,
                                    Some(category.clone()),
                                    category,
                                );
                            }
                        });
                });
            });

            let total_results = self.mod_results.len();
            let mut visible_mods: Vec<CurseForgeMod> = self.mod_results.clone();
            if let Some(category) = &self.mod_category_filter {
                visible_mods.retain(|m| m.categories.iter().any(|c| c.name == *category));
            }
            match self.mod_sort {
                ModSort::Downloads => {
                    visible_mods.sort_by(|a, b| b.downloadCount.cmp(&a.downloadCount));
                }
                ModSort::Updated => {
                    visible_mods.sort_by(|a, b| b.dateModified.cmp(&a.dateModified));
                }
                ModSort::Name => {
                    visible_mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                }
            }

            if total_results > 0 {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(i18n.mods_showing(visible_mods.len(), total_results))
                        .color(colors.text_faint)
                        .small(),
                );
            }

            ui.add_space(8.0);

            if let Some(err) = &self.mod_error {
                ui.colored_label(colors.danger, i18n.mods_search_failed(err));
            }

            if self.mod_results.is_empty() && !self.mod_loading {
                ui.label(RichText::new(i18n.mods_none_loaded()).color(colors.text_faint));
                return;
            }

            if visible_mods.is_empty() && !self.mod_loading {
                ui.label(RichText::new(i18n.mods_no_match()).color(colors.text_faint));
                return;
            }

            let scroll_height = ui.available_height().max(420.0);
            let installed_by_cf: HashMap<i32, InstalledMod> = self
                .installed_mods
                .iter()
                .map(|m| (m.curseforge_id, m.clone()))
                .collect();
            let removing_id = self.removing_mod.clone();
            let remove_locked = mod_actions_locked || self.installed_loading;
            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .show(ui, |ui| {
                    for m in &visible_mods {
                        let installed_entry = installed_by_cf.get(&m.id);
                        let removing_match =
                            removing_id.as_deref() == installed_entry.map(|i| i.id.as_str());
                        elevated_frame(colors).show(ui, |ui| {
                            ui.vertical(|ui| {
                                let downloads = format_downloads(m.downloadCount);
                                let updated = format_mod_date(&m.dateModified);
                                let authors = format_authors(&m.authors);

                                ui.horizontal(|ui| {
                                    let url = mod_page_url(m);
                                    ui.hyperlink_to(RichText::new(&m.name).strong(), url);
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if let Some(installed) = installed_entry {
                                            let remove_btn =
                                                egui::Button::new(i18n.mods_remove_button())
                                                    .fill(tint(colors.danger, 40))
                                                    .stroke(Stroke::new(1.0, colors.danger))
                                                    .min_size(Vec2::new(96.0, 30.0));
                                            let busy = removing_match;
                                            if ui
                                                .add_enabled(!remove_locked && !busy, remove_btn)
                                                .clicked()
                                            {
                                                self.start_remove_installed_mod(
                                                    installed.id.clone(),
                                                );
                                            }
                                            if busy {
                                                ui.add(egui::Spinner::new());
                                            }
                                        } else if ui
                                            .add_enabled(
                                                can_install_mods,
                                                egui::Button::new(i18n.mods_install_button())
                                                    .fill(colors.accent)
                                                    .stroke(Stroke::new(1.0, colors.accent_glow))
                                                    .min_size(Vec2::new(96.0, 30.0)),
                                            )
                                            .clicked()
                                        {
                                            self.trigger_action(UserAction::DownloadMod {
                                                mod_id: m.id,
                                            });
                                        }
                                    });
                                });

                                ui.add_space(4.0);
                                ui.horizontal_wrapped(|ui| {
                                    for category in m.categories.iter().take(2) {
                                        chip_frame(colors.accent_soft).show(ui, |ui| {
                                            ui.label(
                                                RichText::new(category.name.clone())
                                                    .color(colors.accent_glow)
                                                    .small(),
                                            );
                                        });
                                    }
                                    chip_frame(colors.info).show(ui, |ui| {
                                        ui.label(
                                            RichText::new(i18n.mods_downloads(&downloads))
                                                .color(colors.text_primary)
                                                .small(),
                                        );
                                    });
                                    if let Some(updated) = updated {
                                        meta_chip_frame(colors).show(ui, |ui| {
                                            ui.label(
                                                RichText::new(i18n.mods_updated(&updated))
                                                    .color(colors.text_muted)
                                                    .small(),
                                            );
                                        });
                                    }
                                    if let Some(authors) = authors {
                                        meta_chip_frame(colors).show(ui, |ui| {
                                            ui.label(
                                                RichText::new(i18n.mods_by(&authors))
                                                    .color(colors.text_muted)
                                                    .small(),
                                            );
                                        });
                                    }
                                });

                                ui.add_space(6.0);
                                ui.label(RichText::new(&m.summary).color(colors.text_muted));
                            });
                        });
                    }
                });
        });
    }

    fn render_installed_mods(
        &mut self,
        ui: &mut egui::Ui,
        colors: &ThemePalette,
        i18n: I18n,
        mod_actions_locked: bool,
    ) {
        ui.horizontal(|ui| {
            ui.heading(i18n.mods_installed_heading());
            if self.installed_loading {
                ui.add(egui::Spinner::new());
            } else if ui
                .add(
                    egui::Button::new(i18n.mods_installed_refresh())
                        .fill(colors.surface_elev)
                        .stroke(Stroke::new(1.0, colors.border_strong))
                        .min_size(Vec2::new(120.0, 28.0)),
                )
                .clicked()
            {
                self.start_load_installed_mods();
            }
        });

        if let Some(err) = &self.installed_error {
            ui.colored_label(colors.danger, i18n.mods_installed_error(err));
            ui.add_space(4.0);
        }

        if self.installed_mods.is_empty() && !self.installed_loading {
            ui.label(RichText::new(i18n.mods_installed_empty()).color(colors.text_faint));
            ui.add_space(6.0);
            return;
        }

        if self.installed_loading {
            ui.label(RichText::new(i18n.mods_searching()).color(colors.text_muted));
            ui.add_space(6.0);
            return;
        }

        ui.add_space(4.0);
        let removing_id = self.removing_mod.clone();
        let remove_locked = mod_actions_locked || self.installed_loading;
        let installed_list = self.installed_mods.clone();
        for installed in installed_list {
            elevated_frame(colors).show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&installed.name).strong());
                        ui.label(
                            RichText::new(installed.version.clone())
                                .color(colors.text_muted)
                                .small(),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let busy = removing_id.as_deref() == Some(&installed.id);
                            let remove_btn = egui::Button::new(i18n.mods_remove_button())
                                .fill(tint(colors.danger, 40))
                                .stroke(Stroke::new(1.0, colors.danger))
                                .min_size(Vec2::new(88.0, 26.0));
                            if ui
                                .add_enabled(!remove_locked && !busy, remove_btn)
                                .clicked()
                            {
                                self.start_remove_installed_mod(installed.id.clone());
                            }
                            if busy {
                                ui.add(egui::Spinner::new());
                            }
                        });
                    });
                    ui.horizontal_wrapped(|ui| {
                        meta_chip_frame(colors).show(ui, |ui| {
                            ui.label(
                                RichText::new(i18n.mods_by(&installed.author))
                                    .color(colors.text_muted)
                                    .small(),
                            );
                        });
                        if let Some(category) = &installed.category {
                            chip_frame(colors.accent_soft).show(ui, |ui| {
                                ui.label(
                                    RichText::new(category.clone())
                                        .color(colors.accent_glow)
                                        .small(),
                                );
                            });
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(&installed.description)
                            .color(colors.text_muted)
                            .small(),
                    );
                });
            });
        }
        ui.add_space(6.0);
    }

    fn render_controls(&mut self, ui: &mut egui::Ui, colors: &ThemePalette, i18n: I18n) {
        section_frame(colors).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(i18n.controls_heading());
                ui.label(
                    RichText::new(i18n.controls_subheading())
                        .color(colors.text_muted)
                        .small(),
                );
            });
            ui.add_space(10.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(i18n.player_name_label()).color(colors.text_muted));
                    let name_placeholder = i18n.player_name_placeholder();
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.player_name)
                            .hint_text(name_placeholder)
                            .desired_width(180.0),
                    );
                    if resp.changed() {
                        self.player_name_error = None;
                    }
                    let save_clicked = ui
                        .add(
                            egui::Button::new(i18n.player_name_save_button())
                                .fill(colors.accent_soft)
                                .stroke(Stroke::new(1.0, colors.accent))
                                .min_size(Vec2::new(72.0, 28.0)),
                        )
                        .clicked();
                    let enter_pressed =
                        resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if save_clicked || enter_pressed {
                        self.commit_player_name();
                    }
                });
                if let Some(err) = &self.player_name_error {
                    ui.colored_label(colors.danger, i18n.player_name_error(err));
                }
                ui.add_space(6.0);

                let auth_label = i18n.auth_mode_value(self.auth_mode);
                let auth_offline_label = i18n.auth_mode_value(AuthMode::Offline);
                let auth_online_label = i18n.auth_mode_value(AuthMode::Online);

                ui.horizontal(|ui| {
                    ui.label(RichText::new(i18n.auth_mode_label()).color(colors.text_muted));
                    egui::ComboBox::from_id_source("auth_mode_combo")
                        .selected_text(auth_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.auth_mode,
                                AuthMode::Offline,
                                auth_offline_label,
                            );
                            ui.selectable_value(
                                &mut self.auth_mode,
                                AuthMode::Online,
                                auth_online_label,
                            );
                        });
                });
                ui.add_space(8.0);

                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(i18n.version_label()).color(colors.text_muted));
                    let latest_label =
                        i18n.version_latest(self.available_versions.first().copied());
                    let version_labels: Vec<(u32, String)> = self
                        .available_versions
                        .iter()
                        .map(|version| (*version, i18n.version_value(*version)))
                        .collect();
                    let selected_text = self
                        .selected_version
                        .map(|version| i18n.version_value(version))
                        .unwrap_or_else(|| latest_label.clone());
                    let previous = self.selected_version;
                    egui::ComboBox::from_id_source("version_combo")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_version,
                                None,
                                latest_label.clone(),
                            );
                            for (version, label) in &version_labels {
                                ui.selectable_value(
                                    &mut self.selected_version,
                                    Some(*version),
                                    label.clone(),
                                );
                            }
                        });
                    self.sync_version_selection(previous);

                    if self.version_loading {
                        ui.add(egui::Spinner::new());
                    } else if ui
                        .add(
                            egui::Button::new(i18n.version_refresh_button())
                                .fill(colors.surface_elev)
                                .stroke(Stroke::new(1.0, colors.border_strong))
                                .min_size(Vec2::new(110.0, 28.0)),
                        )
                        .clicked()
                    {
                        self.start_version_discovery();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new(i18n.version_custom_label()).color(colors.text_muted));
                    let placeholder = i18n.version_input_placeholder();
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.version_input)
                            .hint_text(placeholder)
                            .desired_width(120.0),
                    );
                    if resp.changed() {
                        self.version_input_error = None;
                    }
                    let apply_clicked = ui
                        .add(
                            egui::Button::new(i18n.version_apply_button())
                                .fill(colors.accent_soft)
                                .stroke(Stroke::new(1.0, colors.accent))
                                .min_size(Vec2::new(90.0, 28.0)),
                        )
                        .clicked();
                    let enter_pressed =
                        resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if apply_clicked || enter_pressed {
                        self.apply_version_input();
                    }
                });
                if let Some(err) = &self.version_fetch_error {
                    ui.colored_label(colors.danger, i18n.version_fetch_error(err));
                }
                if let Some(err) = &self.version_input_error {
                    ui.colored_label(colors.danger, err);
                }
                ui.add_space(8.0);

                ui.horizontal_wrapped(|ui| {
                    let is_fetching = matches!(
                        self.state,
                        AppState::Downloading { .. } | AppState::CheckingForUpdates
                    );
                    let can_download = !is_fetching;
                    let download_btn = egui::Button::new(i18n.download_button())
                        .fill(colors.accent_soft)
                        .stroke(Stroke::new(1.0, colors.accent))
                        .min_size(Vec2::new(150.0, 34.0));
                    if ui.add_enabled(can_download, download_btn).clicked() {
                        self.trigger_action(UserAction::DownloadGame {
                            target_version: self.selected_version,
                        });
                    }

                    let can_check = !is_fetching;
                    let check_btn = egui::Button::new(i18n.check_updates_button())
                        .fill(colors.surface_elev)
                        .stroke(Stroke::new(1.0, colors.border_strong))
                        .min_size(Vec2::new(150.0, 34.0));
                    if ui.add_enabled(can_check, check_btn).clicked() {
                        self.trigger_action(UserAction::CheckForUpdates {
                            target_version: self.selected_version,
                        });
                    }

                    if matches!(self.state, AppState::Downloading { .. })
                        && ui
                            .add(
                                egui::Button::new(i18n.cancel_button())
                                    .fill(tint(colors.danger, 40))
                                    .stroke(Stroke::new(1.0, colors.danger))
                                    .min_size(Vec2::new(110.0, 32.0)),
                            )
                            .clicked()
                    {
                        self.trigger_action(UserAction::ClickCancelDownload);
                    }
                });

                ui.add_space(6.0);
                let is_busy = matches!(
                    self.state,
                    AppState::Downloading { .. }
                        | AppState::CheckingForUpdates
                        | AppState::DiagnosticsRunning
                        | AppState::Playing
                        | AppState::Uninstalling
                        | AppState::Initialising
                );
                let uninstall_clicked = ui
                    .add_enabled(
                        !is_busy,
                        egui::Button::new(i18n.uninstall_button())
                            .fill(tint(colors.danger, 40))
                            .stroke(Stroke::new(1.0, colors.danger))
                            .min_size(Vec2::new(150.0, 32.0)),
                    )
                    .clicked();
                if uninstall_clicked {
                    self.show_uninstall_confirm = true;
                }

                ui.add_space(6.0);
                if ui
                    .add(
                        egui::Button::new(i18n.run_diagnostics_button())
                            .fill(colors.accent_soft)
                            .stroke(Stroke::new(1.0, colors.accent))
                            .min_size(Vec2::new(150.0, 32.0)),
                    )
                    .clicked()
                {
                    self.trigger_action(UserAction::RunDiagnostics);
                }

                ui.add_space(6.0);
                let open_enabled = env::game_latest_dir().exists();
                if ui
                    .add_enabled(
                        open_enabled,
                        egui::Button::new(i18n.open_game_folder_button())
                            .fill(colors.surface_elev)
                            .stroke(Stroke::new(1.0, colors.border_strong))
                            .min_size(Vec2::new(170.0, 32.0)),
                    )
                    .clicked()
                {
                    self.trigger_action(UserAction::OpenGameFolder);
                }
            });
        });

        if self.show_uninstall_confirm {
            egui::Window::new(i18n.uninstall_confirm_title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.label(i18n.uninstall_confirm_body());
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(i18n.uninstall_confirm_yes())
                                    .fill(tint(colors.danger, 60))
                                    .stroke(Stroke::new(1.0, colors.danger)),
                            )
                            .clicked()
                        {
                            self.show_uninstall_confirm = false;
                            self.trigger_action(UserAction::UninstallGame);
                        }
                        if ui
                            .add(
                                egui::Button::new(i18n.uninstall_confirm_no())
                                    .fill(colors.surface_elev)
                                    .stroke(Stroke::new(1.0, colors.border_strong)),
                            )
                            .clicked()
                        {
                            self.show_uninstall_confirm = false;
                        }
                    });
                });
        }
    }

    fn render_diagnostics(&mut self, ui: &mut egui::Ui, colors: &ThemePalette, i18n: I18n) {
        section_frame(colors).show(ui, |ui| {
            ui.heading(i18n.diagnostics_heading());
            ui.add_space(6.0);
            if let Some(_) = &self.diagnostics {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(i18n.diagnostics_completed()).color(colors.text_muted));
                    let view_btn = egui::Button::new(i18n.view_report())
                        .fill(colors.accent_soft)
                        .stroke(Stroke::new(1.0, colors.accent))
                        .min_size(Vec2::new(120.0, 28.0));
                    if ui.add(view_btn).clicked() {
                        self.show_diagnostics_modal = true;
                    }
                });
            } else {
                ui.label(RichText::new(i18n.diagnostics_empty()).color(colors.text_muted));
            }
        });
    }

    fn render_diagnostics_modal(&mut self, ctx: &egui::Context, colors: &ThemePalette, i18n: I18n) {
        if !self.show_diagnostics_modal {
            return;
        }
        let Some(diag) = &self.diagnostics else {
            self.show_diagnostics_modal = false;
            return;
        };

        let mut open = self.show_diagnostics_modal;
        let mut close_requested = false;
        egui::Window::new(i18n.diagnostics_heading())
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .default_width(720.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_min_height(320.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(i18n.diagnostics_completed()).color(colors.text_muted),
                        );
                        if ui
                            .add(
                                egui::Button::new(i18n.close_button())
                                    .fill(colors.surface_elev)
                                    .stroke(Stroke::new(1.0, colors.border_strong)),
                            )
                            .clicked()
                        {
                            close_requested = true;
                        }
                    });
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .max_height(DIAGNOSTICS_REPORT_HEIGHT)
                        .show(ui, |ui| {
                            ui.monospace(diag);
                        });
                });
            });
        self.show_diagnostics_modal = open && !close_requested;
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.sync_state();
        self.sync_mod_updates();
        self.sync_version_updates();
        self.sync_news_updates();
        self.sync_updater_updates();
        refresh_fonts_if_needed(self, ctx);
        let colors = self.colors();
        apply_theme(ctx, &colors);
        let top_bar_i18n = self.i18n();

        egui::TopBottomPanel::top("top_bar")
            .frame(
                Frame::none()
                    .fill(colors.panel)
                    .stroke(Stroke::new(1.0, colors.border))
                    .inner_margin(Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(RichText::new(top_bar_i18n.heading()).color(colors.accent));
                        ui.label(RichText::new(top_bar_i18n.tagline()).color(colors.text_muted));
                    });
                    ui.allocate_ui_with_layout(
                        ui.available_size_before_wrap(),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            let control_height = 34.0;
                            ui.scope(|ui| {
                                ui.set_height(control_height);
                                egui::ComboBox::from_id_source("theme_combo")
                                    .selected_text(top_bar_i18n.theme_label(self.theme))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.theme,
                                            Theme::Dark,
                                            top_bar_i18n.theme_label(Theme::Dark),
                                        );
                                        ui.selectable_value(
                                            &mut self.theme,
                                            Theme::Light,
                                            top_bar_i18n.theme_label(Theme::Light),
                                        );
                                    });
                            });
                            ui.add_space(10.0);
                            ui.scope(|ui| {
                                ui.set_height(control_height);
                                egui::ComboBox::from_id_source("language_combo")
                                    .selected_text(self.language.display_name())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::English,
                                            Language::English.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::Ukrainian,
                                            Language::Ukrainian.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::Spanish,
                                            Language::Spanish.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::French,
                                            Language::French.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::German,
                                            Language::German.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::Portuguese,
                                            Language::Portuguese.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::Chinese,
                                            Language::Chinese.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::Hindi,
                                            Language::Hindi.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::Russian,
                                            Language::Russian.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::Turkish,
                                            Language::Turkish.display_name(),
                                        );
                                    });
                            });
                        },
                    );
                });
            });

        let i18n = self.i18n();

        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(
                Frame::none()
                    .fill(colors.panel)
                    .stroke(Stroke::new(1.0, colors.border))
                    .inner_margin(Margin::symmetric(16.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    self.render_discord_button(ui, &colors, i18n);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let UpdateStatus::UpdateAvailable {
                            latest_version,
                            url,
                        } = &self.updater_status
                        {
                            ui.scope(|ui| {
                                ui.set_height(30.0);
                                let update_btn = egui::Button::new(
                                    RichText::new(i18n.update_available(latest_version))
                                        .color(colors.text_primary)
                                        .small(),
                                )
                                .fill(colors.info)
                                .stroke(Stroke::new(1.0, colors.accent_glow));
                                if ui.add(update_btn).clicked() {
                                    ui.output_mut(|o| {
                                        o.open_url = Some(egui::output::OpenUrl {
                                            url: url.clone(),
                                            new_tab: true,
                                        });
                                    });
                                }
                            });
                            ui.add_space(10.0);
                        }
                        badge_frame(colors.border_strong).show(ui, |ui| {
                            ui.label(
                                RichText::new(i18n.launcher_version(self.launcher_version))
                                    .color(colors.text_primary)
                                    .small(),
                            );
                        });
                    });
                });
            });
        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(colors.bg)
                    .inner_margin(Margin::symmetric(14.0, 12.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let full_width = ui.available_width();
                    let gutter = 18.0;
                    if full_width <= gutter {
                        self.render_status(ui, &colors, i18n);
                        ui.add_space(12.0);
                        self.render_controls(ui, &colors, i18n);
                        ui.add_space(12.0);
                        self.render_mods(ui, &colors, i18n);
                        ui.add_space(12.0);
                        self.render_news(ui, &colors, i18n);
                        self.render_diagnostics(ui, &colors, i18n);
                        return;
                    }

                    let left_width = (full_width - gutter) * 0.42;
                    let right_width = full_width - gutter - left_width;
                    ui.horizontal_top(|ui| {
                        ui.allocate_ui_with_layout(
                            Vec2::new(left_width, 0.0),
                            Layout::top_down(Align::LEFT),
                            |ui| {
                                self.render_status(ui, &colors, i18n);
                                ui.add_space(12.0);
                                self.render_controls(ui, &colors, i18n);
                                ui.add_space(12.0);
                                self.render_diagnostics(ui, &colors, i18n);
                            },
                        );
                        ui.add_space(gutter);
                        ui.allocate_ui_with_layout(
                            Vec2::new(right_width, 0.0),
                            Layout::top_down(Align::LEFT),
                            |ui| {
                                self.render_mods(ui, &colors, i18n);
                            },
                        );
                    });
                    ui.add_space(14.0);
                    self.render_news(ui, &colors, i18n);
                });
            });
        self.render_diagnostics_modal(ctx, &colors, i18n);
    }
}
