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
    Chinese,
    Hindi,
    Russian,
    Turkish,
}

impl Language {
    pub const fn display_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Ukrainian => "Ukrainian",
            Language::Spanish => "Spanish",
            Language::French => "French",
            Language::German => "German",
            Language::Portuguese => "Portuguese",
            Language::Chinese => "Chinese",
            Language::Hindi => "Hindi",
            Language::Russian => "Russian",
            Language::Turkish => "Turkish",
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
        chinese: &'a str,
        hindi: &'a str,
        russian: &'a str,
        turkish: &'a str,
    ) -> &'a str {
        match self.language {
            Language::English => english,
            Language::Ukrainian => ukrainian,
            Language::Spanish => spanish,
            Language::French => french,
            Language::German => german,
            Language::Portuguese => portuguese,
            Language::Chinese => chinese,
            Language::Hindi => hindi,
            Language::Russian => russian,
            Language::Turkish => turkish,
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
            (Theme::Dark, Language::Chinese) => "深色",
            (Theme::Dark, Language::Hindi) => "डार्क",
            (Theme::Dark, Language::Russian) => "Темная",
            (Theme::Dark, Language::Turkish) => "Koyu",
            (Theme::Light, Language::English) => "Light",
            (Theme::Light, Language::Ukrainian) => "Світла",
            (Theme::Light, Language::Spanish) => "Claro",
            (Theme::Light, Language::French) => "Clair",
            (Theme::Light, Language::German) => "Hell",
            (Theme::Light, Language::Portuguese) => "Claro",
            (Theme::Light, Language::Chinese) => "浅色",
            (Theme::Light, Language::Hindi) => "लाइट",
            (Theme::Light, Language::Russian) => "Светлая",
            (Theme::Light, Language::Turkish) => "Açık",
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
            (ModSort::Downloads, Language::Chinese) => "下载最多",
            (ModSort::Downloads, Language::Hindi) => "सबसे अधिक डाउनलोड",
            (ModSort::Downloads, Language::Russian) => "Самые скачиваемые",
            (ModSort::Downloads, Language::Turkish) => "En çok indirilen",
            (ModSort::Updated, Language::English) => "Recently updated",
            (ModSort::Updated, Language::Ukrainian) => "Нещодавно оновлені",
            (ModSort::Updated, Language::Spanish) => "Actualizados recientemente",
            (ModSort::Updated, Language::French) => "Mis à jour récemment",
            (ModSort::Updated, Language::German) => "Kürzlich aktualisiert",
            (ModSort::Updated, Language::Portuguese) => "Atualizados recentemente",
            (ModSort::Updated, Language::Chinese) => "最近更新",
            (ModSort::Updated, Language::Hindi) => "हाल ही में अपडेट किए गए",
            (ModSort::Updated, Language::Russian) => "Недавно обновленные",
            (ModSort::Updated, Language::Turkish) => "Son güncellenen",
            (ModSort::Name, Language::English) => "Name A-Z",
            (ModSort::Name, Language::Ukrainian) => "Назва A-Z",
            (ModSort::Name, Language::Spanish) => "Nombre A-Z",
            (ModSort::Name, Language::French) => "Nom A-Z",
            (ModSort::Name, Language::German) => "Name A-Z",
            (ModSort::Name, Language::Portuguese) => "Nome A-Z",
            (ModSort::Name, Language::Chinese) => "名称 A-Z",
            (ModSort::Name, Language::Hindi) => "नाम A-Z",
            (ModSort::Name, Language::Russian) => "Имя A-Z",
            (ModSort::Name, Language::Turkish) => "İsim A-Z",
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
            "HRS 启动器",
            "HRS लॉन्चर",
            "HRS лаунчер",
            "HRS Başlatıcı",
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
            "Hytale 的社区启动器",
            "Hytale के लिए सामुदायिक लॉन्चर",
            "Сообщественный лаунчер для Hytale",
            "Hytale için topluluk başlatıcısı",
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
            Language::Chinese => format!("启动器 v{version}"),
            Language::Hindi => format!("लॉन्चर v{version}"),
            Language::Russian => format!("Лаунчер v{version}"),
            Language::Turkish => format!("Başlatıcı v{version}"),
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
            "加入我们的 Discord 服务器",
            "हमारे Discord सर्वर से जुड़ें",
            "Присоединиться к нашему серверу Discord",
            "Discord sunucumuza katılın",
        )
    }

    pub fn status_label(self) -> &'static str {
        self.pick(
            "Status",
            "Стан",
            "Estado",
            "Statut",
            "Status",
            "Estado",
            "状态",
            "स्थिति",
            "Статус",
            "Durum",
        )
    }

    pub fn status_ready(self) -> &'static str {
        self.pick(
            "Ready",
            "Готово",
            "Listo",
            "Prêt",
            "Bereit",
            "Pronto",
            "就绪",
            "तैयार",
            "Готово",
            "Hazır",
        )
    }

    pub fn status_running(self) -> &'static str {
        self.pick(
            "Running",
            "Запущено",
            "En ejecución",
            "En cours",
            "Läuft",
            "Em execução",
            "运行中",
            "चल रहा है",
            "Выполняется",
            "Çalışıyor",
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
            "注意",
            "ध्यान",
            "Внимание",
            "Dikkat",
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
            "下载中",
            "डाउनलोड हो रहा है",
            "Загрузка",
            "İndiriliyor",
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
            "正在卸载",
            "अनइंस्टॉल किया जा रहा है",
            "Удаление",
            "Kaldırılıyor",
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
            "诊断",
            "निदान",
            "Диагностика",
            "Tanılama",
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
            "处理中",
            "काम चल रहा है",
            "В работе",
            "İşleniyor",
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
            "刷新",
            "रिफ्रेश",
            "Обновить",
            "Yenile",
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
            "正在运行诊断...",
            "निदान चल रहा है...",
            "Выполняется диагностика...",
            "Tanılama çalışıyor...",
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
            "诊断完成。",
            "निदान पूरा हुआ।",
            "Диагностика завершена.",
            "Tanılama tamamlandı.",
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
            "尚无可用的诊断报告。",
            "अभी कोई निदान रिपोर्ट उपलब्ध नहीं है।",
            "Отчет диагностики пока недоступен.",
            "Henüz bir tanılama raporu yok.",
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
            "关闭",
            "बंद करें",
            "Закрыть",
            "Kapat",
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
            "Hytale 发生了什么",
            "Hytale में क्या हो रहा है",
            "Что происходит в Hytale",
            "Hytale'da neler oluyor",
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
            "更新中...",
            "अपडेट हो रहा है...",
            "Обновление...",
            "Güncelleniyor...",
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
            Language::Chinese => format!("获取新闻失败: {err}"),
            Language::Hindi => format!("समाचार लाने में विफल: {err}"),
            Language::Russian => format!("Не удалось получить новости: {err}"),
            Language::Turkish => format!("Haberler alınamadı: {err}"),
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
            "更多信息请访问 hytale.com。",
            "अधिक जानकारी hytale.com पर।",
            "Подробнее на hytale.com.",
            "Daha fazlası için hytale.com.",
        )
    }

    pub fn mods_heading(self) -> &'static str {
        self.pick(
            "Mods",
            "Моди",
            "Mods",
            "Mods",
            "Mods",
            "Mods",
            "模组",
            "मोड्स",
            "Моды",
            "Modlar",
        )
    }

    pub fn mods_searching(self) -> &'static str {
        self.pick(
            "Searching...",
            "Пошук...",
            "Buscando...",
            "Recherche en cours...",
            "Suche...",
            "Pesquisando...",
            "搜索中...",
            "खोज रहे हैं...",
            "Поиск...",
            "Aranıyor...",
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
            Language::Chinese => format!("{count} 个结果"),
            Language::Hindi => format!("{count} परिणाम"),
            Language::Russian => format!("{count} результатов"),
            Language::Turkish => format!("{count} sonuç"),
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
            "按名称或关键词搜索...",
            "नाम या कीवर्ड से खोजें...",
            "Поиск по названию или ключевому слову...",
            "Ada veya anahtar kelimeye göre arayın...",
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
            "搜索",
            "खोजें",
            "Поиск",
            "Ara",
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
            "清除",
            "साफ़ करें",
            "Очистить",
            "Temizle",
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
            "排序方式",
            "क्रमबद्ध करें",
            "Сортировать по",
            "Sırala",
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
            "类别",
            "श्रेणी",
            "Категория",
            "Kategori",
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
            "所有类别",
            "सभी श्रेणियाँ",
            "Все категории",
            "Tüm kategoriler",
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
            Language::Chinese => format!("显示 {visible}/{total} 个模组"),
            Language::Hindi => format!("{visible}/{total} मॉड दिखा रहे हैं"),
            Language::Russian => format!("Показано {visible} из {total} модов"),
            Language::Turkish => format!("{total} modun {visible} tanesi gösteriliyor"),
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
            Language::Chinese => format!("搜索失败: {err}"),
            Language::Hindi => format!("खोज विफल: {err}"),
            Language::Russian => format!("Ошибка поиска: {err}"),
            Language::Turkish => format!("Arama başarısız: {err}"),
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
            "未加载任何模组。尝试按名称搜索。",
            "कोई मॉड लोड नहीं हुआ। नाम से खोजने का प्रयास करें।",
            "Моды не загружены. Попробуйте поиск по названию.",
            "Mod yüklenmedi. İsimle aramayı deneyin.",
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
            "没有符合当前筛选的模组。",
            "वर्तमान फ़िल्टर से कोई मॉड मेल नहीं खाता।",
            "Нет модов, соответствующих текущим фильтрам.",
            "Mevcut filtrelere uyan mod yok.",
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
            "已安装的模组",
            "इंस्टॉल किए गए मॉड्स",
            "Установленные моды",
            "Yüklü modlar",
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
            "尚未安装任何模组。",
            "अभी तक कोई मॉड इंस्टॉल नहीं है।",
            "Моды еще не установлены.",
            "Henüz mod kurulmadı.",
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
            Language::Chinese => format!("获取已安装模组失败: {err}"),
            Language::Hindi => format!("इंस्टॉल किए गए मॉड प्राप्त करने में त्रुटि: {err}"),
            Language::Russian => format!("Не удалось получить установленные моды: {err}"),
            Language::Turkish => format!("Yüklü modlar alınamadı: {err}"),
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
            "刷新已安装",
            "इंस्टॉल किए गए को रिफ्रेश करें",
            "Обновить список",
            "Yüklüleri yenile",
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
            "移除",
            "हटाएं",
            "Удалить",
            "Kaldır",
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
            "安装游戏以启用模组安装。",
            "मोड इंस्टॉल के लिए गेम इंस्टॉल करें।",
            "Установите игру, чтобы включить установку модов.",
            "Mod kurulumu için önce oyunu yükleyin.",
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
            "安装",
            "इंस्टॉल करें",
            "Установить",
            "Yükle",
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
            Language::Chinese => format!("下载 {downloads}"),
            Language::Hindi => format!("डाउनलोड {downloads}"),
            Language::Russian => format!("Загрузки {downloads}"),
            Language::Turkish => format!("İndirme {downloads}"),
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
            Language::Chinese => format!("更新于 {updated}"),
            Language::Hindi => format!("{updated} को अपडेट किया गया"),
            Language::Russian => format!("Обновлено {updated}"),
            Language::Turkish => format!("{updated} güncellendi"),
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
            Language::Chinese => format!("作者 {authors}"),
            Language::Hindi => format!("{authors} द्वारा"),
            Language::Russian => format!("От {authors}"),
            Language::Turkish => format!("{authors} tarafından"),
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
            "启动器控制",
            "लॉन्चर नियंत्रण",
            "Управление лаунчером",
            "Başlatıcı kontrolleri",
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
            "管理更新并开始游戏",
            "अपडेट प्रबंधित करें और खेलें",
            "Управляйте обновлениями и играйте",
            "Güncellemeleri yönetin ve oynayın",
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
            "玩家名称",
            "खिलाड़ी का नाम",
            "Имя игрока",
            "Oyuncu adı",
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
            "玩家",
            "खिलाड़ी",
            "Игрок",
            "Oyuncu",
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
            "保存",
            "सहेजें",
            "Сохранить",
            "Kaydet",
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
            Language::Chinese => format!("玩家名称: {err}"),
            Language::Hindi => format!("खिलाड़ी का नाम: {err}"),
            Language::Russian => format!("Имя игрока: {err}"),
            Language::Turkish => format!("Oyuncu adı: {err}"),
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
            "认证模式",
            "प्रमाणीकरण मोड",
            "Режим аутентификации",
            "Kimlik doğrulama modu",
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
            (AuthMode::Offline, Language::Chinese) => "离线",
            (AuthMode::Offline, Language::Hindi) => "ऑफ़लाइन",
            (AuthMode::Offline, Language::Russian) => "Офлайн",
            (AuthMode::Offline, Language::Turkish) => "Çevrimdışı",
            (AuthMode::Online, Language::English) => "Online",
            (AuthMode::Online, Language::Ukrainian) => "Онлайн",
            (AuthMode::Online, Language::Spanish) => "En línea",
            (AuthMode::Online, Language::French) => "En ligne",
            (AuthMode::Online, Language::German) => "Online",
            (AuthMode::Online, Language::Portuguese) => "Online",
            (AuthMode::Online, Language::Chinese) => "在线",
            (AuthMode::Online, Language::Hindi) => "ऑनलाइन",
            (AuthMode::Online, Language::Russian) => "Онлайн",
            (AuthMode::Online, Language::Turkish) => "Çevrimiçi",
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
            "游戏版本",
            "गेम संस्करण",
            "Версия игры",
            "Oyun sürümü",
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
            (Some(v), Language::Chinese) => format!("最新 (v{v})"),
            (Some(v), Language::Hindi) => format!("नवीनतम (v{v})"),
            (Some(v), Language::Russian) => format!("Последняя (v{v})"),
            (Some(v), Language::Turkish) => format!("En son (v{v})"),
            (None, Language::English) => "Latest".into(),
            (None, Language::Ukrainian) => "Остання".into(),
            (None, Language::Spanish) => "Última".into(),
            (None, Language::French) => "Dernière".into(),
            (None, Language::German) => "Neueste".into(),
            (None, Language::Portuguese) => "Mais recente".into(),
            (None, Language::Chinese) => "最新".into(),
            (None, Language::Hindi) => "नवीनतम".into(),
            (None, Language::Russian) => "Последняя".into(),
            (None, Language::Turkish) => "En son".into(),
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
            "刷新列表",
            "सूची रिफ्रेश करें",
            "Обновить список",
            "Listeyi yenile",
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
            "自定义版本",
            "कस्टम संस्करण",
            "Пользовательская версия",
            "Özel sürüm",
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
            "例如 3",
            "उदा. 3",
            "например, 3",
            "örn. 3",
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
            "设置版本",
            "संस्करण सेट करें",
            "Установить версию",
            "Sürümü ayarla",
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
            Language::Chinese => format!("获取版本列表失败: {err}"),
            Language::Hindi => format!("संस्करण सूची प्राप्त करने में विफल: {err}"),
            Language::Russian => format!("Не удалось получить список версий: {err}"),
            Language::Turkish => format!("Sürüm listesi alınamadı: {err}"),
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
            "请输入有效的版本号。",
            "कृपया एक मान्य संस्करण संख्या दर्ज करें।",
            "Введите корректный номер версии.",
            "Geçerli bir sürüm numarası girin.",
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
            "运行诊断",
            "निदान चलाएं",
            "Запустить диагностику",
            "Tanılama çalıştır",
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
            "打开游戏文件夹",
            "गेम फ़ोल्डर खोलें",
            "Открыть папку игры",
            "Oyun klasörünü aç",
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
            "诊断",
            "निदान",
            "Диагностика",
            "Tanılama",
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
            "查看报告",
            "रिपोर्ट देखें",
            "Просмотреть отчет",
            "Raporu görüntüle",
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
            "正在检查更新...",
            "अपडेट की जाँच हो रही है...",
            "Проверка обновлений...",
            "Güncellemeler kontrol ediliyor...",
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
            Language::Chinese => format!("正在下载 {file}"),
            Language::Hindi => format!("{file} डाउनलोड हो रहा है"),
            Language::Russian => format!("Загрузка {file}"),
            Language::Turkish => format!("{file} indiriliyor"),
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
            "正在删除游戏文件...",
            "गेम फ़ाइलें हटाई जा रही हैं...",
            "Удаляем файлы игры...",
            "Oyun dosyaları kaldırılıyor...",
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
            Language::Chinese => format!("准备好玩版本 {version}"),
            Language::Hindi => format!("संस्करण {version} खेलने के लिए तैयार"),
            Language::Russian => format!("Готово к игре версии {version}"),
            Language::Turkish => format!("{version} sürümünü oynamaya hazır"),
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
            "正在启动 Hytale...",
            "Hytale शुरू किया जा रहा है...",
            "Запуск Hytale...",
            "Hytale başlatılıyor...",
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
            Language::Chinese => format!("错误: {msg}"),
            Language::Hindi => format!("त्रुटि: {msg}"),
            Language::Russian => format!("Ошибка: {msg}"),
            Language::Turkish => format!("Hata: {msg}"),
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
            "正在初始化启动器...",
            "लॉन्चर प्रारंभ किया जा रहा है...",
            "Инициализация лаунчера...",
            "Başlatıcı başlatılıyor...",
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
            "空闲。点击“下载游戏”进行安装或更新。",
            "निष्क्रिय। इंस्टॉल या अपडेट करने के लिए डाउनलोड गेम पर क्लिक करें।",
            "Ожидание. Нажмите \"Скачать игру\", чтобы установить или обновить.",
            "Boşta. Yüklemek veya güncellemek için Oyunu İndir'e tıklayın.",
        )
    }

    pub fn play_button(self) -> &'static str {
        self.pick(
            "Play",
            "Грати",
            "Jugar",
            "Jouer",
            "Spielen",
            "Jogar",
            "开始游戏",
            "खेलें",
            "Играть",
            "Oyna",
        )
    }

    pub fn download_button(self) -> &'static str {
        self.pick(
            "Download Game",
            "Завантажити гру",
            "Descargar juego",
            "Télécharger le jeu",
            "Spiel herunterladen",
            "Baixar jogo",
            "下载游戏",
            "गेम डाउनलोड करें",
            "Скачать игру",
            "Oyunu indir",
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
            "检查更新",
            "अपडेट की जाँच करें",
            "Проверить обновления",
            "Güncellemeleri kontrol et",
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
            "取消",
            "रद्द करें",
            "Отмена",
            "İptal",
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
            "卸载游戏",
            "गेम अनइंस्टॉल करें",
            "Удалить игру",
            "Oyunu kaldır",
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
            "确认卸载",
            "अनइंस्टॉल की पुष्टि करें",
            "Подтверждение удаления",
            "Kaldırmayı onayla",
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
            "这将删除游戏文件和捆绑的 JRE。确定吗？",
            "यह गेम फ़ाइलें और बंडल की गई JRE हटा देगा। क्या आप सुनिश्चित हैं?",
            "Будут удалены файлы игры и встроенная JRE. Вы уверены?",
            "Bu, oyun dosyalarını ve paketli JRE'yi kaldıracak. Emin misiniz?",
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
            "是的，卸载",
            "हाँ, अनइंस्टॉल करें",
            "Да, удалить",
            "Evet, kaldır",
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
            "取消",
            "रद्द करें",
            "Отмена",
            "İptal",
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
            "新闻",
            "समाचार",
            "Новости",
            "Haberler",
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
            "暂无新闻。",
            "कोई समाचार उपलब्ध नहीं है।",
            "Новости недоступны.",
            "Haber yok.",
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
            Language::Chinese => format!("有可用更新：{version}"),
            Language::Hindi => format!("अपडेट उपलब्ध: {version}"),
            Language::Russian => format!("Доступно обновление: {version}"),
            Language::Turkish => format!("Güncelleme mevcut: {version}"),
        }
    }
}
