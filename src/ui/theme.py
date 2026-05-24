class AppleTheme:
    """Дизайн-токены Apple Design System. Цвета в формате: (Светлая тема, Темная тема)"""

    # === Colors (Цвета) ===
    CANVAS = ("#f5f5f7", "#000000")  # Фон всего окна
    CARD = ("#ffffff", "#1c1c1e")  # Поверхность карточек
    FOG = ("#e8e8ed", "#2c2c2e")  # Плашки, второстепенные кнопки
    TRANSPARENT = "transparent"  # Прозрачный фон

    INK = ("#1d1d1f", "#f5f5f7")  # Основной текст
    GRAPHITE = ("#707070", "#86868b")  # Второстепенный текст

    AZURE = ("#0071e3", "#0a84ff")  # Главный CTA цвет (Primary action)
    AZURE_HOVER = ("#005bb5", "#006ee6")

    DANGER = ("#e30000", "#ff3b30")  # Ошибки / Удаление
    SUCCESS = ("#108619", "#32d74b")  # Успех
    BORDER = ("#d2d2d7", "#38383a")  # Границы и обводки

    # === Spacing (Отступы по сетке) ===
    SP_0 = 0
    SP_2 = 2
    SP_4 = 4
    SP_6 = 6
    SP_8 = 8
    SP_12 = 12
    SP_16 = 16
    SP_20 = 20
    SP_24 = 24
    SP_28 = 28
    SP_32 = 32
    SP_40 = 40

    # === Geometry (Радиусы и размеры) ===
    RADIUS_CARD = 28  # Скругление карточек
    RADIUS_PILL = 999  # Кнопки-таблетки
    RADIUS_SMALL = 10  # Мелкие инпуты/комбобоксы

    SIDEBAR_WIDTH = 340
    HEIGHT_BTN_PRIMARY = 44
    HEIGHT_BTN_SEC = 32
    HEIGHT_LOGS = 100

    # === Window Geometry ===
    WIN_MAIN = "1200x850"
    WIN_PREVIEW = "1000x800"
    WIN_TOUR = "700x500"
    WIN_INPUT = "600x500"
    WIN_EDIT = "500x160"

    # === Typography (Шрифты) ===
    FONT_DISPLAY = ("SF Pro Display", 24, "bold")
    FONT_HEADING = ("SF Pro Display", 18, "bold")
    FONT_BODY = ("SF Pro Text", 14)
    FONT_BODY_SM = ("SF Pro Text", 12)
    FONT_BUTTON = ("SF Pro Text", 14, "bold")
    FONT_CODE = ("Consolas", 13)
    FONT_CODE_SM = ("Consolas", 12)