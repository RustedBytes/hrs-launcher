use crate::engine::state::AuthMode;

use super::{DEFAULT_PLAYER_NAME, ModSort, NEWS_PREVIEW_FALLBACK_EN, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Ukrainian,
}

impl Language {
    pub const fn display_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Ukrainian => "Українська",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct I18n {
    language: Language,
}

impl I18n {
    #[must_use]
    pub const fn new(language: Language) -> Self {
        Self { language }
    }

    fn pick<'a>(self, english: &'a str, ukrainian: &'a str) -> &'a str {
        match self.language {
            Language::English => english,
            Language::Ukrainian => ukrainian,
        }
    }

    pub fn theme_label(self, theme: Theme) -> &'static str {
        match (theme, self.language) {
            (Theme::Dark, Language::English) => "Dark",
            (Theme::Dark, Language::Ukrainian) => "Темна",
            (Theme::Light, Language::English) => "Light",
            (Theme::Light, Language::Ukrainian) => "Світла",
        }
    }

    pub fn mod_sort_label(self, sort: ModSort) -> &'static str {
        match (sort, self.language) {
            (ModSort::Downloads, Language::English) => "Most downloaded",
            (ModSort::Downloads, Language::Ukrainian) => "Найбільш завантажувані",
            (ModSort::Updated, Language::English) => "Recently updated",
            (ModSort::Updated, Language::Ukrainian) => "Нещодавно оновлені",
            (ModSort::Name, Language::English) => "Name A-Z",
            (ModSort::Name, Language::Ukrainian) => "Назва A-Z",
        }
    }

    pub fn heading(self) -> &'static str {
        self.pick("HRS Launcher", "Лаунчер HRS")
    }

    pub fn tagline(self) -> &'static str {
        self.pick(
            "Community launcher for Hytale",
            "Спільнотний лаунчер для Hytale",
        )
    }

    pub fn launcher_version(self, version: &str) -> String {
        match self.language {
            Language::English => format!("Launcher v{version}"),
            Language::Ukrainian => format!("Версія лаунчера v{version}"),
        }
    }

    pub fn discord_button_label(self) -> &'static str {
        self.pick(
            "Join our Discord server",
            "Долучайтеся до нашого Discord-сервера",
        )
    }

    pub fn status_label(self) -> &'static str {
        self.pick("Status", "Стан")
    }

    pub fn status_ready(self) -> &'static str {
        self.pick("Ready", "Готово")
    }

    pub fn status_running(self) -> &'static str {
        self.pick("Running", "Запущено")
    }

    pub fn status_attention(self) -> &'static str {
        self.pick("Attention", "Увага")
    }

    pub fn status_downloading(self) -> &'static str {
        self.pick("Downloading", "Завантаження")
    }

    pub fn status_uninstalling(self) -> &'static str {
        self.pick("Uninstalling", "Видалення")
    }

    pub fn status_diagnostics(self) -> &'static str {
        self.pick("Diagnostics", "Діагностика")
    }

    pub fn status_working(self) -> &'static str {
        self.pick("Working", "Виконується")
    }

    pub fn diagnostics_running(self) -> &'static str {
        self.pick("Running diagnostics...", "Виконується діагностика...")
    }

    pub fn diagnostics_completed(self) -> &'static str {
        self.pick("Diagnostics completed.", "Діагностику завершено.")
    }

    pub fn news_subheading(self) -> &'static str {
        self.pick("What's happening in Hytale", "Що нового в Hytale")
    }

    pub fn news_updating(self) -> &'static str {
        self.pick("Updating...", "Оновлення...")
    }

    pub fn news_fetch_failed(self, err: &str) -> String {
        match self.language {
            Language::English => format!("News fetch failed: {err}"),
            Language::Ukrainian => format!("Не вдалося отримати новини: {err}"),
        }
    }

    pub fn news_preview_fallback(self) -> &'static str {
        self.pick(NEWS_PREVIEW_FALLBACK_EN, "Детальніше на hytale.com.")
    }

    pub fn mods_heading(self) -> &'static str {
        self.pick("Mods", "Моди")
    }

    pub fn mods_searching(self) -> &'static str {
        self.pick("Searching...", "Пошук...")
    }

    pub fn mods_results_count(self, count: usize) -> String {
        match self.language {
            Language::English => format!("{count} results"),
            Language::Ukrainian => format!("Знайдено {count}"),
        }
    }

    pub fn mods_search_hint(self) -> &'static str {
        self.pick(
            "Search by name or keyword...",
            "Пошук за назвою або ключовим словом...",
        )
    }

    pub fn mods_search_button(self) -> &'static str {
        self.pick("Search", "Пошук")
    }

    pub fn mods_clear_button(self) -> &'static str {
        self.pick("Clear", "Очистити")
    }

    pub fn mods_sort_label(self) -> &'static str {
        self.pick("Sort by", "Сортувати за")
    }

    pub fn mods_category_label(self) -> &'static str {
        self.pick("Category", "Категорія")
    }

    pub fn mods_all_categories(self) -> &'static str {
        self.pick("All categories", "Усі категорії")
    }

    pub fn mods_showing(self, visible: usize, total: usize) -> String {
        match self.language {
            Language::English => format!("Showing {visible} of {total} mods"),
            Language::Ukrainian => format!("Показано {visible} з {total}"),
        }
    }

    pub fn mods_search_failed(self, err: &str) -> String {
        match self.language {
            Language::English => format!("Search failed: {err}"),
            Language::Ukrainian => format!("Помилка пошуку: {err}"),
        }
    }

    pub fn mods_none_loaded(self) -> &'static str {
        self.pick(
            "No mods loaded. Try searching by name.",
            "Моди не завантажено. Спробуйте пошук за назвою.",
        )
    }

    pub fn mods_no_match(self) -> &'static str {
        self.pick(
            "No mods match the current filters.",
            "Немає модів, що відповідають поточним фільтрам.",
        )
    }

    pub fn mods_requires_game(self) -> &'static str {
        self.pick(
            "Install the game to enable mod installs.",
            "Встановіть гру, щоб увімкнути встановлення модів.",
        )
    }

    pub fn mods_install_button(self) -> &'static str {
        self.pick("Install", "Встановити")
    }

    pub fn mods_downloads(self, downloads: &str) -> String {
        match self.language {
            Language::English => format!("Downloads {downloads}"),
            Language::Ukrainian => format!("Завантажень {downloads}"),
        }
    }

    pub fn mods_updated(self, updated: &str) -> String {
        match self.language {
            Language::English => format!("Updated {updated}"),
            Language::Ukrainian => format!("Оновлено {updated}"),
        }
    }

    pub fn mods_by(self, authors: &str) -> String {
        match self.language {
            Language::English => format!("By {authors}"),
            Language::Ukrainian => format!("Від {authors}"),
        }
    }

    pub fn controls_heading(self) -> &'static str {
        self.pick("Launcher controls", "Керування лаунчером")
    }

    pub fn controls_subheading(self) -> &'static str {
        self.pick("Manage updates & play", "Керування оновленнями та запуском")
    }

    pub fn player_name_label(self) -> &'static str {
        self.pick("Player name", "Ім'я гравця")
    }

    pub fn player_name_placeholder(self) -> &'static str {
        self.pick(DEFAULT_PLAYER_NAME, "Гравець")
    }

    pub fn player_name_save_button(self) -> &'static str {
        self.pick("Save", "Зберегти")
    }

    pub fn player_name_error(self, err: &str) -> String {
        match self.language {
            Language::English => format!("Player name: {err}"),
            Language::Ukrainian => format!("Ім'я гравця: {err}"),
        }
    }

    pub fn auth_mode_label(self) -> &'static str {
        self.pick("Auth mode", "Режим авторизації")
    }

    pub fn auth_mode_value(self, mode: AuthMode) -> &'static str {
        match (mode, self.language) {
            (AuthMode::Offline, Language::English) => "Offline",
            (AuthMode::Offline, Language::Ukrainian) => "Офлайн",
            (AuthMode::Online, Language::English) => "Online",
            (AuthMode::Online, Language::Ukrainian) => "Онлайн",
        }
    }

    pub fn version_label(self) -> &'static str {
        self.pick("Game version", "Версія гри")
    }

    pub fn version_latest(self, latest: Option<u32>) -> String {
        match (latest, self.language) {
            (Some(v), Language::English) => format!("Latest (v{v})"),
            (Some(v), Language::Ukrainian) => format!("Остання (v{v})"),
            (None, Language::English) => "Latest".into(),
            (None, Language::Ukrainian) => "Остання".into(),
        }
    }

    pub fn version_value(self, version: u32) -> String {
        format!("v{version}")
    }

    pub fn version_refresh_button(self) -> &'static str {
        self.pick("Refresh list", "Оновити список")
    }

    pub fn version_custom_label(self) -> &'static str {
        self.pick("Custom version", "Своя версія")
    }

    pub fn version_input_placeholder(self) -> &'static str {
        self.pick("e.g. 3", "наприклад, 3")
    }

    pub fn version_apply_button(self) -> &'static str {
        self.pick("Set version", "Застосувати")
    }

    pub fn version_fetch_error(self, err: &str) -> String {
        match self.language {
            Language::English => format!("Version list failed: {err}"),
            Language::Ukrainian => format!("Не вдалося отримати список версій: {err}"),
        }
    }

    pub fn version_input_error(self) -> &'static str {
        self.pick(
            "Enter a valid version number.",
            "Вкажіть коректний номер версії.",
        )
    }

    pub fn run_diagnostics_button(self) -> &'static str {
        self.pick("Run diagnostics", "Запустити діагностику")
    }

    pub fn open_game_folder_button(self) -> &'static str {
        self.pick("Open game folder", "Відкрити теку гри")
    }

    pub fn diagnostics_heading(self) -> &'static str {
        self.pick("Diagnostics", "Діагностика")
    }

    pub fn view_report(self) -> &'static str {
        self.pick("View report", "Переглянути звіт")
    }

    pub fn checking(self) -> &'static str {
        self.pick("Checking for updates...", "Перевірка оновлень...")
    }

    pub fn downloading(self, file: &str) -> String {
        match self.language {
            Language::English => format!("Downloading {file}"),
            Language::Ukrainian => format!("Завантаження {file}"),
        }
    }

    pub fn uninstalling(self) -> &'static str {
        self.pick("Removing game files...", "Видаляємо файли гри...")
    }

    pub fn progress(self, progress: f32, speed: &str) -> String {
        format!("{progress:.0}% ({speed})")
    }

    pub fn ready(self, version: &str) -> String {
        match self.language {
            Language::English => format!("Ready to play version {version}"),
            Language::Ukrainian => format!("Готово до запуску версії {version}"),
        }
    }

    pub fn playing(self) -> &'static str {
        self.pick("Launching Hytale...", "Запуск Hytale...")
    }

    pub fn error(self, msg: &str) -> String {
        match self.language {
            Language::English => format!("Error: {msg}"),
            Language::Ukrainian => format!("Помилка: {msg}"),
        }
    }

    pub fn initialising(self) -> &'static str {
        self.pick("Initialising launcher...", "Ініціалізація лаунчера...")
    }

    pub fn idle(self) -> &'static str {
        self.pick(
            "Idle. Click Download Game to install or update.",
            "Очікування. Натисніть Завантажити гру, щоб встановити або оновити.",
        )
    }

    pub fn play_button(self) -> &'static str {
        self.pick("Play", "Грати")
    }

    pub fn download_button(self) -> &'static str {
        self.pick("Download Game", "Завантажити гру")
    }

    pub fn check_updates_button(self) -> &'static str {
        self.pick("Check for updates", "Перевірити оновлення")
    }

    pub fn cancel_button(self) -> &'static str {
        self.pick("Cancel", "Скасувати")
    }

    pub fn uninstall_button(self) -> &'static str {
        self.pick("Uninstall game", "Видалити гру")
    }

    pub fn uninstall_confirm_title(self) -> &'static str {
        self.pick("Confirm uninstall", "Підтвердьте видалення")
    }

    pub fn uninstall_confirm_body(self) -> &'static str {
        self.pick(
            "This will remove the game files and bundled JRE. Are you sure?",
            "Це видалить файли гри та вбудовану JRE. Ви впевнені?",
        )
    }

    pub fn uninstall_confirm_yes(self) -> &'static str {
        self.pick("Yes, uninstall", "Так, видалити")
    }

    pub fn uninstall_confirm_no(self) -> &'static str {
        self.pick("Cancel", "Скасувати")
    }

    pub fn news_heading(self) -> &'static str {
        self.pick("News", "Новини")
    }

    pub fn no_news(self) -> &'static str {
        self.pick("No news available.", "Наразі немає новин.")
    }

    pub fn update_available(self, version: &str) -> String {
        match self.language {
            Language::English => format!("Update available: {version}"),
            Language::Ukrainian => format!("Доступне оновлення: {version}"),
        }
    }

    pub fn update_download_button(self) -> &'static str {
        self.pick("Download update", "Завантажити оновлення")
    }

    pub fn update_checking(self) -> &'static str {
        self.pick("Checking for launcher updates...", "Перевірка оновлень лаунчера...")
    }

    pub fn update_check_failed(self, err: &str) -> String {
        match self.language {
            Language::English => format!("Update check failed: {err}"),
            Language::Ukrainian => format!("Помилка перевірки оновлень: {err}"),
        }
    }
}
