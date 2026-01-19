use crate::engine::state::AuthMode;

use super::{DEFAULT_PLAYER_NAME, ModSort, NEWS_PREVIEW_FALLBACK_EN, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Ukrainian,
    Spanish,
    French,
    German,
    Portuguese,
}

impl Language {
    pub const fn display_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Ukrainian => "Українська",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Portuguese => "Português",
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

    fn pick<'a>(
        self,
        english: &'a str,
        ukrainian: &'a str,
        spanish: &'a str,
        french: &'a str,
        german: &'a str,
        portuguese: &'a str,
    ) -> &'a str {
        match self.language {
            Language::English => english,
            Language::Ukrainian => ukrainian,
            Language::Spanish => spanish,
            Language::French => french,
            Language::German => german,
            Language::Portuguese => portuguese,
        }
    }

    pub fn theme_label(self, theme: Theme) -> &'static str {
        match (theme, self.language) {
            (Theme::Dark, Language::English) => "Dark",
            (Theme::Dark, Language::Ukrainian) => "Темна",
            (Theme::Dark, Language::Spanish) => "Oscuro",
            (Theme::Dark, Language::French) => "Sombre",
            (Theme::Dark, Language::German) => "Dunkel",
            (Theme::Dark, Language::Portuguese) => "Escuro",
            (Theme::Light, Language::English) => "Light",
            (Theme::Light, Language::Ukrainian) => "Світла",
            (Theme::Light, Language::Spanish) => "Claro",
            (Theme::Light, Language::French) => "Clair",
            (Theme::Light, Language::German) => "Hell",
            (Theme::Light, Language::Portuguese) => "Claro",
        }
    }

    pub fn mod_sort_label(self, sort: ModSort) -> &'static str {
        match (sort, self.language) {
            (ModSort::Downloads, Language::English) => "Most downloaded",
            (ModSort::Downloads, Language::Ukrainian) => "Найбільш завантажувані",
            (ModSort::Downloads, Language::Spanish) => "Más descargados",
            (ModSort::Downloads, Language::French) => "Les plus téléchargés",
            (ModSort::Downloads, Language::German) => "Am häufigsten heruntergeladen",
            (ModSort::Downloads, Language::Portuguese) => "Mais baixados",
            (ModSort::Updated, Language::English) => "Recently updated",
            (ModSort::Updated, Language::Ukrainian) => "Нещодавно оновлені",
            (ModSort::Updated, Language::Spanish) => "Actualizados recientemente",
            (ModSort::Updated, Language::French) => "Mis à jour récemment",
            (ModSort::Updated, Language::German) => "Kürzlich aktualisiert",
            (ModSort::Updated, Language::Portuguese) => "Atualizados recentemente",
            (ModSort::Name, Language::English) => "Name A-Z",
            (ModSort::Name, Language::Ukrainian) => "Назва A-Z",
            (ModSort::Name, Language::Spanish) => "Nombre A-Z",
            (ModSort::Name, Language::French) => "Nom A-Z",
            (ModSort::Name, Language::German) => "Name A-Z",
            (ModSort::Name, Language::Portuguese) => "Nome A-Z",
        }
    }

    pub fn heading(self) -> &'static str {
        self.pick(
            "HRS Launcher",
            "Лаунчер HRS",
            "Lanzador HRS",
            "Lanceur HRS",
            "HRS Launcher",
            "Lançador HRS",
        )
    }

    pub fn tagline(self) -> &'static str {
        self.pick(
            "Community launcher for Hytale",
            "Спільнотний лаунчер для Hytale",
            "Lanzador comunitario para Hytale",
            "Lanceur communautaire pour Hytale",
            "Community-Launcher für Hytale",
            "Lançador comunitário para Hytale",
        )
    }

    pub fn launcher_version(self, version: &str) -> String {
        match self.language {
            Language::English => format!("Launcher v{version}"),
            Language::Ukrainian => format!("Версія лаунчера v{version}"),
            Language::Spanish => format!("Lanzador v{version}"),
            Language::French => format!("Lanceur v{version}"),
            Language::German => format!("Launcher v{version}"),
            Language::Portuguese => format!("Lançador v{version}"),
        }
    }

    pub fn discord_button_label(self) -> &'static str {
        self.pick(
            "Join our Discord server",
            "Долучайтеся до нашого Discord-сервера",
            "Únete a nuestro servidor de Discord",
            "Rejoins notre serveur Discord",
            "Tritt unserem Discord-Server bei",
            "Entre no nosso servidor do Discord",
        )
    }

    pub fn status_label(self) -> &'static str {
        self.pick("Status", "Стан", "Estado", "Statut", "Status", "Estado")
    }

    pub fn status_ready(self) -> &'static str {
        self.pick("Ready", "Готово", "Listo", "Prêt", "Bereit", "Pronto")
    }

    pub fn status_running(self) -> &'static str {
        self.pick(
            "Running",
            "Запущено",
            "En ejecución",
            "En cours",
            "Läuft",
            "Em execução",
        )
    }

    pub fn status_attention(self) -> &'static str {
        self.pick(
            "Attention",
            "Увага",
            "Atención",
            "Attention",
            "Achtung",
            "Atenção",
        )
    }

    pub fn status_downloading(self) -> &'static str {
        self.pick(
            "Downloading",
            "Завантаження",
            "Descargando",
            "Téléchargement",
            "Wird heruntergeladen",
            "Baixando",
        )
    }

    pub fn status_uninstalling(self) -> &'static str {
        self.pick(
            "Uninstalling",
            "Видалення",
            "Desinstalando",
            "Désinstallation",
            "Deinstallieren",
            "Desinstalando",
        )
    }

    pub fn status_diagnostics(self) -> &'static str {
        self.pick(
            "Diagnostics",
            "Діагностика",
            "Diagnósticos",
            "Diagnostics",
            "Diagnose",
            "Diagnósticos",
        )
    }

    pub fn status_working(self) -> &'static str {
        self.pick(
            "Working",
            "Виконується",
            "En progreso",
            "En cours",
            "In Arbeit",
            "Em progresso",
        )
    }

    pub fn status_refresh(self) -> &'static str {
        self.pick(
            "Refresh",
            "Оновити",
            "Actualizar",
            "Rafraîchir",
            "Aktualisieren",
            "Atualizar",
        )
    }

    pub fn diagnostics_running(self) -> &'static str {
        self.pick(
            "Running diagnostics...",
            "Виконується діагностика...",
            "Ejecutando diagnósticos...",
            "Exécution des diagnostics...",
            "Diagnose läuft...",
            "Executando diagnósticos...",
        )
    }

    pub fn diagnostics_completed(self) -> &'static str {
        self.pick(
            "Diagnostics completed.",
            "Діагностику завершено.",
            "Diagnósticos completados.",
            "Diagnostics terminés.",
            "Diagnose abgeschlossen.",
            "Diagnósticos concluídos.",
        )
    }

    pub fn diagnostics_empty(self) -> &'static str {
        self.pick(
            "No diagnostics report available yet.",
            "Звіт діагностики ще недоступний.",
            "Aún no hay un informe de diagnóstico.",
            "Aucun rapport de diagnostic disponible pour le moment.",
            "Noch kein Diagnosebericht verfügbar.",
            "Nenhum relatório de diagnóstico disponível ainda.",
        )
    }

    pub fn close_button(self) -> &'static str {
        self.pick(
            "Close",
            "Закрити",
            "Cerrar",
            "Fermer",
            "Schließen",
            "Fechar",
        )
    }

    pub fn news_subheading(self) -> &'static str {
        self.pick(
            "What's happening in Hytale",
            "Що нового в Hytale",
            "Qué está pasando en Hytale",
            "Ce qui se passe dans Hytale",
            "Was passiert in Hytale",
            "O que está acontecendo em Hytale",
        )
    }

    pub fn news_updating(self) -> &'static str {
        self.pick(
            "Updating...",
            "Оновлення...",
            "Actualizando...",
            "Mise à jour...",
            "Aktualisieren...",
            "Atualizando...",
        )
    }

    pub fn news_fetch_failed(self, err: &str) -> String {
        match self.language {
            Language::English => format!("News fetch failed: {err}"),
            Language::Ukrainian => format!("Не вдалося отримати новини: {err}"),
            Language::Spanish => format!("Error al obtener noticias: {err}"),
            Language::French => format!("Échec du chargement des actualités : {err}"),
            Language::German => format!("Nachrichten konnten nicht geladen werden: {err}"),
            Language::Portuguese => format!("Falha ao buscar notícias: {err}"),
        }
    }

    pub fn news_preview_fallback(self) -> &'static str {
        self.pick(
            NEWS_PREVIEW_FALLBACK_EN,
            "Детальніше на hytale.com.",
            "Más información en hytale.com.",
            "Plus d'informations sur hytale.com.",
            "Mehr auf hytale.com.",
            "Mais informações em hytale.com.",
        )
    }

    pub fn mods_heading(self) -> &'static str {
        self.pick("Mods", "Моди", "Mods", "Mods", "Mods", "Mods")
    }

    pub fn mods_searching(self) -> &'static str {
        self.pick(
            "Searching...",
            "Пошук...",
            "Buscando...",
            "Recherche en cours...",
            "Suche...",
            "Pesquisando...",
        )
    }

    pub fn mods_results_count(self, count: usize) -> String {
        match self.language {
            Language::English => format!("{count} results"),
            Language::Ukrainian => format!("Знайдено {count}"),
            Language::Spanish => format!("{count} resultados"),
            Language::French => format!("{count} résultats"),
            Language::German => format!("{count} Ergebnisse"),
            Language::Portuguese => format!("{count} resultados"),
        }
    }

    pub fn mods_search_hint(self) -> &'static str {
        self.pick(
            "Search by name or keyword...",
            "Пошук за назвою або ключовим словом...",
            "Busca por nombre o palabra clave...",
            "Recherche par nom ou mot-clé...",
            "Suche nach Name oder Stichwort...",
            "Pesquise por nome ou palavra-chave...",
        )
    }

    pub fn mods_search_button(self) -> &'static str {
        self.pick(
            "Search",
            "Пошук",
            "Buscar",
            "Rechercher",
            "Suchen",
            "Pesquisar",
        )
    }

    pub fn mods_clear_button(self) -> &'static str {
        self.pick(
            "Clear",
            "Очистити",
            "Limpiar",
            "Effacer",
            "Leeren",
            "Limpar",
        )
    }

    pub fn mods_sort_label(self) -> &'static str {
        self.pick(
            "Sort by",
            "Сортувати за",
            "Ordenar por",
            "Trier par",
            "Sortieren nach",
            "Ordenar por",
        )
    }

    pub fn mods_category_label(self) -> &'static str {
        self.pick(
            "Category",
            "Категорія",
            "Categoría",
            "Catégorie",
            "Kategorie",
            "Categoria",
        )
    }

    pub fn mods_all_categories(self) -> &'static str {
        self.pick(
            "All categories",
            "Усі категорії",
            "Todas las categorías",
            "Toutes les catégories",
            "Alle Kategorien",
            "Todas as categorias",
        )
    }

    pub fn mods_showing(self, visible: usize, total: usize) -> String {
        match self.language {
            Language::English => format!("Showing {visible} of {total} mods"),
            Language::Ukrainian => format!("Показано {visible} з {total}"),
            Language::Spanish => format!("Mostrando {visible} de {total} mods"),
            Language::French => format!("Affichage de {visible} sur {total} mods"),
            Language::German => format!("Zeige {visible} von {total} Mods"),
            Language::Portuguese => format!("Mostrando {visible} de {total} mods"),
        }
    }

    pub fn mods_search_failed(self, err: &str) -> String {
        match self.language {
            Language::English => format!("Search failed: {err}"),
            Language::Ukrainian => format!("Помилка пошуку: {err}"),
            Language::Spanish => format!("La búsqueda falló: {err}"),
            Language::French => format!("Échec de la recherche : {err}"),
            Language::German => format!("Suche fehlgeschlagen: {err}"),
            Language::Portuguese => format!("A pesquisa falhou: {err}"),
        }
    }

    pub fn mods_none_loaded(self) -> &'static str {
        self.pick(
            "No mods loaded. Try searching by name.",
            "Моди не завантажено. Спробуйте пошук за назвою.",
            "No hay mods cargados. Intenta buscar por nombre.",
            "Aucun mod chargé. Essayez une recherche par nom.",
            "Keine Mods geladen. Versuche die Suche nach Namen.",
            "Nenhum mod carregado. Tente buscar pelo nome.",
        )
    }

    pub fn mods_no_match(self) -> &'static str {
        self.pick(
            "No mods match the current filters.",
            "Немає модів, що відповідають поточним фільтрам.",
            "Ningún mod coincide con los filtros actuales.",
            "Aucun mod ne correspond aux filtres actuels.",
            "Keine Mods entsprechen den aktuellen Filtern.",
            "Nenhum mod corresponde aos filtros atuais.",
        )
    }

    pub fn mods_installed_heading(self) -> &'static str {
        self.pick(
            "Installed mods",
            "Встановлені моди",
            "Mods instalados",
            "Mods installés",
            "Installierte Mods",
            "Mods instalados",
        )
    }

    pub fn mods_installed_empty(self) -> &'static str {
        self.pick(
            "No mods installed yet.",
            "Ще немає встановлених модів.",
            "Aún no hay mods instalados.",
            "Aucun mod installé pour le moment.",
            "Noch keine Mods installiert.",
            "Ainda não há mods instalados.",
        )
    }

    pub fn mods_installed_error(self, err: &str) -> String {
        match self.language {
            Language::English => format!("Installed mods failed: {err}"),
            Language::Ukrainian => format!("Не вдалося отримати встановлені моди: {err}"),
            Language::Spanish => format!("Error al obtener mods instalados: {err}"),
            Language::French => format!("Échec du chargement des mods installés : {err}"),
            Language::German => format!("Installierte Mods konnten nicht geladen werden: {err}"),
            Language::Portuguese => format!("Erro ao obter mods instalados: {err}"),
        }
    }

    pub fn mods_installed_refresh(self) -> &'static str {
        self.pick(
            "Refresh installed",
            "Оновити список",
            "Actualizar lista",
            "Rafraîchir la liste",
            "Installierte aktualisieren",
            "Atualizar instalados",
        )
    }

    pub fn mods_remove_button(self) -> &'static str {
        self.pick(
            "Remove",
            "Видалити",
            "Eliminar",
            "Supprimer",
            "Entfernen",
            "Remover",
        )
    }

    pub fn mods_requires_game(self) -> &'static str {
        self.pick(
            "Install the game to enable mod installs.",
            "Встановіть гру, щоб увімкнути встановлення модів.",
            "Instala el juego para habilitar la instalación de mods.",
            "Installez le jeu pour activer l'installation des mods.",
            "Installiere das Spiel, um Mod-Installationen zu aktivieren.",
            "Instale o jogo para habilitar a instalação de mods.",
        )
    }

    pub fn mods_install_button(self) -> &'static str {
        self.pick(
            "Install",
            "Встановити",
            "Instalar",
            "Installer",
            "Installieren",
            "Instalar",
        )
    }

    pub fn mods_downloads(self, downloads: &str) -> String {
        match self.language {
            Language::English => format!("Downloads {downloads}"),
            Language::Ukrainian => format!("Завантажень {downloads}"),
            Language::Spanish => format!("Descargas {downloads}"),
            Language::French => format!("Téléchargements {downloads}"),
            Language::German => format!("Downloads {downloads}"),
            Language::Portuguese => format!("Downloads {downloads}"),
        }
    }

    pub fn mods_updated(self, updated: &str) -> String {
        match self.language {
            Language::English => format!("Updated {updated}"),
            Language::Ukrainian => format!("Оновлено {updated}"),
            Language::Spanish => format!("Actualizado {updated}"),
            Language::French => format!("Mis à jour {updated}"),
            Language::German => format!("Aktualisiert {updated}"),
            Language::Portuguese => format!("Atualizado {updated}"),
        }
    }

    pub fn mods_by(self, authors: &str) -> String {
        match self.language {
            Language::English => format!("By {authors}"),
            Language::Ukrainian => format!("Від {authors}"),
            Language::Spanish => format!("Por {authors}"),
            Language::French => format!("Par {authors}"),
            Language::German => format!("Von {authors}"),
            Language::Portuguese => format!("Por {authors}"),
        }
    }

    pub fn controls_heading(self) -> &'static str {
        self.pick(
            "Launcher controls",
            "Керування лаунчером",
            "Controles del lanzador",
            "Contrôles du lanceur",
            "Launcher-Steuerung",
            "Controles do lançador",
        )
    }

    pub fn controls_subheading(self) -> &'static str {
        self.pick(
            "Manage updates & play",
            "Керування оновленнями та запуском",
            "Gestiona actualizaciones y juego",
            "Gérer les mises à jour et jouer",
            "Updates verwalten & spielen",
            "Gerencie atualizações e jogo",
        )
    }

    pub fn player_name_label(self) -> &'static str {
        self.pick(
            "Player name",
            "Ім'я гравця",
            "Nombre del jugador",
            "Nom du joueur",
            "Spielername",
            "Nome do jogador",
        )
    }

    pub fn player_name_placeholder(self) -> &'static str {
        self.pick(
            DEFAULT_PLAYER_NAME,
            "Гравець",
            "Jugador",
            "Joueur",
            "Spieler",
            "Jogador",
        )
    }

    pub fn player_name_save_button(self) -> &'static str {
        self.pick(
            "Save",
            "Зберегти",
            "Guardar",
            "Enregistrer",
            "Speichern",
            "Salvar",
        )
    }

    pub fn player_name_error(self, err: &str) -> String {
        match self.language {
            Language::English => format!("Player name: {err}"),
            Language::Ukrainian => format!("Ім'я гравця: {err}"),
            Language::Spanish => format!("Nombre del jugador: {err}"),
            Language::French => format!("Nom du joueur : {err}"),
            Language::German => format!("Spielername: {err}"),
            Language::Portuguese => format!("Nome do jogador: {err}"),
        }
    }

    pub fn auth_mode_label(self) -> &'static str {
        self.pick(
            "Auth mode",
            "Режим авторизації",
            "Modo de autenticación",
            "Mode d'authentification",
            "Auth-Modus",
            "Modo de autenticação",
        )
    }

    pub fn auth_mode_value(self, mode: AuthMode) -> &'static str {
        match (mode, self.language) {
            (AuthMode::Offline, Language::English) => "Offline",
            (AuthMode::Offline, Language::Ukrainian) => "Офлайн",
            (AuthMode::Offline, Language::Spanish) => "Sin conexión",
            (AuthMode::Offline, Language::French) => "Hors ligne",
            (AuthMode::Offline, Language::German) => "Offline",
            (AuthMode::Offline, Language::Portuguese) => "Offline",
            (AuthMode::Online, Language::English) => "Online",
            (AuthMode::Online, Language::Ukrainian) => "Онлайн",
            (AuthMode::Online, Language::Spanish) => "En línea",
            (AuthMode::Online, Language::French) => "En ligne",
            (AuthMode::Online, Language::German) => "Online",
            (AuthMode::Online, Language::Portuguese) => "Online",
        }
    }

    pub fn version_label(self) -> &'static str {
        self.pick(
            "Game version",
            "Версія гри",
            "Versión del juego",
            "Version du jeu",
            "Spielversion",
            "Versão do jogo",
        )
    }

    pub fn version_latest(self, latest: Option<u32>) -> String {
        match (latest, self.language) {
            (Some(v), Language::English) => format!("Latest (v{v})"),
            (Some(v), Language::Ukrainian) => format!("Остання (v{v})"),
            (Some(v), Language::Spanish) => format!("Última (v{v})"),
            (Some(v), Language::French) => format!("Dernière (v{v})"),
            (Some(v), Language::German) => format!("Neueste (v{v})"),
            (Some(v), Language::Portuguese) => format!("Mais recente (v{v})"),
            (None, Language::English) => "Latest".into(),
            (None, Language::Ukrainian) => "Остання".into(),
            (None, Language::Spanish) => "Última".into(),
            (None, Language::French) => "Dernière".into(),
            (None, Language::German) => "Neueste".into(),
            (None, Language::Portuguese) => "Mais recente".into(),
        }
    }

    pub fn version_value(self, version: u32) -> String {
        format!("v{version}")
    }

    pub fn version_refresh_button(self) -> &'static str {
        self.pick(
            "Refresh list",
            "Оновити список",
            "Actualizar lista",
            "Rafraîchir la liste",
            "Liste aktualisieren",
            "Atualizar lista",
        )
    }

    pub fn version_custom_label(self) -> &'static str {
        self.pick(
            "Custom version",
            "Своя версія",
            "Versión personalizada",
            "Version personnalisée",
            "Benutzerdefinierte Version",
            "Versão personalizada",
        )
    }

    pub fn version_input_placeholder(self) -> &'static str {
        self.pick(
            "e.g. 3",
            "наприклад, 3",
            "p. ej., 3",
            "ex. 3",
            "z. B. 3",
            "ex.: 3",
        )
    }

    pub fn version_apply_button(self) -> &'static str {
        self.pick(
            "Set version",
            "Застосувати",
            "Establecer versión",
            "Définir la version",
            "Version festlegen",
            "Definir versão",
        )
    }

    pub fn version_fetch_error(self, err: &str) -> String {
        match self.language {
            Language::English => format!("Version list failed: {err}"),
            Language::Ukrainian => format!("Не вдалося отримати список версій: {err}"),
            Language::Spanish => format!("Error al obtener la lista de versiones: {err}"),
            Language::French => format!("Échec de récupération de la liste des versions : {err}"),
            Language::German => format!("Versionsliste konnte nicht geladen werden: {err}"),
            Language::Portuguese => format!("Falha ao obter a lista de versões: {err}"),
        }
    }

    pub fn version_input_error(self) -> &'static str {
        self.pick(
            "Enter a valid version number.",
            "Вкажіть коректний номер версії.",
            "Introduce un número de versión válido.",
            "Saisissez un numéro de version valide.",
            "Gib eine gültige Versionsnummer ein.",
            "Insira um número de versão válido.",
        )
    }

    pub fn run_diagnostics_button(self) -> &'static str {
        self.pick(
            "Run diagnostics",
            "Запустити діагностику",
            "Ejecutar diagnósticos",
            "Lancer les diagnostics",
            "Diagnose ausführen",
            "Executar diagnósticos",
        )
    }

    pub fn open_game_folder_button(self) -> &'static str {
        self.pick(
            "Open game folder",
            "Відкрити теку гри",
            "Abrir carpeta del juego",
            "Ouvrir le dossier du jeu",
            "Spieleordner öffnen",
            "Abrir pasta do jogo",
        )
    }

    pub fn diagnostics_heading(self) -> &'static str {
        self.pick(
            "Diagnostics",
            "Діагностика",
            "Diagnósticos",
            "Diagnostics",
            "Diagnose",
            "Diagnósticos",
        )
    }

    pub fn view_report(self) -> &'static str {
        self.pick(
            "View report",
            "Переглянути звіт",
            "Ver informe",
            "Voir le rapport",
            "Bericht ansehen",
            "Ver relatório",
        )
    }

    pub fn checking(self) -> &'static str {
        self.pick(
            "Checking for updates...",
            "Перевірка оновлень...",
            "Buscando actualizaciones...",
            "Vérification des mises à jour...",
            "Nach Updates suchen...",
            "Procurando atualizações...",
        )
    }

    pub fn downloading(self, file: &str) -> String {
        match self.language {
            Language::English => format!("Downloading {file}"),
            Language::Ukrainian => format!("Завантаження {file}"),
            Language::Spanish => format!("Descargando {file}"),
            Language::French => format!("Téléchargement de {file}"),
            Language::German => format!("Lade {file} herunter"),
            Language::Portuguese => format!("Baixando {file}"),
        }
    }

    pub fn uninstalling(self) -> &'static str {
        self.pick(
            "Removing game files...",
            "Видаляємо файли гри...",
            "Eliminando archivos del juego...",
            "Suppression des fichiers du jeu...",
            "Spieldateien werden entfernt...",
            "Removendo arquivos do jogo...",
        )
    }

    pub fn progress(self, progress: f32, speed: &str) -> String {
        format!("{progress:.0}% ({speed})")
    }

    pub fn ready(self, version: &str) -> String {
        match self.language {
            Language::English => format!("Ready to play version {version}"),
            Language::Ukrainian => format!("Готово до запуску версії {version}"),
            Language::Spanish => format!("Listo para jugar la versión {version}"),
            Language::French => format!("Prêt à jouer à la version {version}"),
            Language::German => format!("Bereit, Version {version} zu spielen"),
            Language::Portuguese => format!("Pronto para jogar a versão {version}"),
        }
    }

    pub fn playing(self) -> &'static str {
        self.pick(
            "Launching Hytale...",
            "Запуск Hytale...",
            "Iniciando Hytale...",
            "Lancement de Hytale...",
            "Starte Hytale...",
            "Iniciando Hytale...",
        )
    }

    pub fn error(self, msg: &str) -> String {
        match self.language {
            Language::English => format!("Error: {msg}"),
            Language::Ukrainian => format!("Помилка: {msg}"),
            Language::Spanish => format!("Error: {msg}"),
            Language::French => format!("Erreur : {msg}"),
            Language::German => format!("Fehler: {msg}"),
            Language::Portuguese => format!("Erro: {msg}"),
        }
    }

    pub fn initialising(self) -> &'static str {
        self.pick(
            "Initialising launcher...",
            "Ініціалізація лаунчера...",
            "Inicializando el lanzador...",
            "Initialisation du lanceur...",
            "Launcher wird initialisiert...",
            "Inicializando o lançador...",
        )
    }

    pub fn idle(self) -> &'static str {
        self.pick(
            "Idle. Click Download Game to install or update.",
            "Очікування. Натисніть Завантажити гру, щоб встановити або оновити.",
            "En espera. Haz clic en Descargar juego para instalar o actualizar.",
            "En attente. Cliquez sur Télécharger le jeu pour installer ou mettre à jour.",
            "Wartend. Klicke auf Spiel herunterladen, um zu installieren oder zu aktualisieren.",
            "Em espera. Clique em Baixar jogo para instalar ou atualizar.",
        )
    }

    pub fn play_button(self) -> &'static str {
        self.pick("Play", "Грати", "Jugar", "Jouer", "Spielen", "Jogar")
    }

    pub fn download_button(self) -> &'static str {
        self.pick(
            "Download Game",
            "Завантажити гру",
            "Descargar juego",
            "Télécharger le jeu",
            "Spiel herunterladen",
            "Baixar jogo",
        )
    }

    pub fn check_updates_button(self) -> &'static str {
        self.pick(
            "Check for updates",
            "Перевірити оновлення",
            "Buscar actualizaciones",
            "Vérifier les mises à jour",
            "Nach Updates suchen",
            "Procurar atualizações",
        )
    }

    pub fn cancel_button(self) -> &'static str {
        self.pick(
            "Cancel",
            "Скасувати",
            "Cancelar",
            "Annuler",
            "Abbrechen",
            "Cancelar",
        )
    }

    pub fn uninstall_button(self) -> &'static str {
        self.pick(
            "Uninstall game",
            "Видалити гру",
            "Desinstalar juego",
            "Désinstaller le jeu",
            "Spiel deinstallieren",
            "Desinstalar jogo",
        )
    }

    pub fn uninstall_confirm_title(self) -> &'static str {
        self.pick(
            "Confirm uninstall",
            "Підтвердьте видалення",
            "Confirmar desinstalación",
            "Confirmer la désinstallation",
            "Deinstallation bestätigen",
            "Confirmar desinstalação",
        )
    }

    pub fn uninstall_confirm_body(self) -> &'static str {
        self.pick(
            "This will remove the game files and bundled JRE. Are you sure?",
            "Це видалить файли гри та вбудовану JRE. Ви впевнені?",
            "Esto eliminará los archivos del juego y la JRE incluida. ¿Seguro?",
            "Cela supprimera les fichiers du jeu et la JRE incluse. Êtes-vous sûr ?",
            "Dies entfernt die Spieldateien und die mitgelieferte JRE. Bist du sicher?",
            "Isso removerá os arquivos do jogo e a JRE incluída. Tem certeza?",
        )
    }

    pub fn uninstall_confirm_yes(self) -> &'static str {
        self.pick(
            "Yes, uninstall",
            "Так, видалити",
            "Sí, desinstalar",
            "Oui, désinstaller",
            "Ja, deinstallieren",
            "Sim, desinstalar",
        )
    }

    pub fn uninstall_confirm_no(self) -> &'static str {
        self.pick(
            "Cancel",
            "Скасувати",
            "Cancelar",
            "Annuler",
            "Abbrechen",
            "Cancelar",
        )
    }

    pub fn news_heading(self) -> &'static str {
        self.pick(
            "News",
            "Новини",
            "Noticias",
            "Actualités",
            "Neuigkeiten",
            "Notícias",
        )
    }

    pub fn no_news(self) -> &'static str {
        self.pick(
            "No news available.",
            "Наразі немає новин.",
            "No hay noticias disponibles.",
            "Aucune actualité disponible.",
            "Keine Neuigkeiten verfügbar.",
            "Nenhuma notícia disponível.",
        )
    }

    pub fn update_available(self, version: &str) -> String {
        match self.language {
            Language::English => format!("Update available: {version}"),
            Language::Ukrainian => format!("Доступне оновлення: {version}"),
            Language::Spanish => format!("Actualización disponible: {version}"),
            Language::French => format!("Mise à jour disponible : {version}"),
            Language::German => format!("Update verfügbar: {version}"),
            Language::Portuguese => format!("Atualização disponível: {version}"),
        }
    }
}
