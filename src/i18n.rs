#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Spanish,
    Portuguese,
    Korean,
    Japanese,
    Chinese,
    Russian,
}

impl Language {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "spanish" | "español" => Self::Spanish,
            "portuguese" | "português" => Self::Portuguese,
            "korean" | "한국어" => Self::Korean,
            "japanese" | "日本語" => Self::Japanese,
            "chinese" | "简体中文" | "中文" => Self::Chinese,
            "russian" | "русский" => Self::Russian,
            _ => Self::English,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Spanish => "Español (Spanish)",
            Self::Portuguese => "Português (Portuguese)",
            Self::Korean => "한국어 (Korean)",
            Self::Japanese => "日本語 (Japanese)",
            Self::Chinese => "简体中文 (Chinese)",
            Self::Russian => "Русский (Russian)",
        }
    }

    pub fn all() -> &'static [Language] {
        &[
            Language::English,
            Language::Spanish,
            Language::Portuguese,
            Language::Korean,
            Language::Japanese,
            Language::Chinese,
            Language::Russian,
        ]
    }

    // Sidebar Tab Labels
    pub fn tab_general(&self) -> &'static str {
        match self {
            Self::English => "General",
            Self::Spanish => "General",
            Self::Portuguese => "Geral",
            Self::Korean => "일반",
            Self::Japanese => "一般",
            Self::Chinese => "常规设置",
            Self::Russian => "Основные",
        }
    }

    pub fn tab_ducking(&self) -> &'static str {
        match self {
            Self::English => "Smart Ducking",
            Self::Spanish => "Atenuación",
            Self::Portuguese => "Atenuação",
            Self::Korean => "스마트 덕킹",
            Self::Japanese => "ダッキング",
            Self::Chinese => "智能闪避",
            Self::Russian => "Приглушение",
        }
    }

    pub fn tab_booster(&self) -> &'static str {
        match self {
            Self::English => "Footstep Booster",
            Self::Spanish => "Potenciador Pasos",
            Self::Portuguese => "Reforço de Passos",
            Self::Korean => "발소리 부스터",
            Self::Japanese => "足音ブースター",
            Self::Chinese => "脚步声增强",
            Self::Russian => "Усиление шагов",
        }
    }

    pub fn tab_mixer(&self) -> &'static str {
        match self {
            Self::English => "App Mixer",
            Self::Spanish => "Mezclador Apps",
            Self::Portuguese => "Mixer de Apps",
            Self::Korean => "앱 믹서",
            Self::Japanese => "アプリミキサー",
            Self::Chinese => "应用混音器",
            Self::Russian => "Микшер",
        }
    }

    pub fn tab_apps(&self) -> &'static str {
        match self {
            Self::English => "Specific Apps",
            Self::Spanish => "Apps Específicas",
            Self::Portuguese => "Apps Alvo",
            Self::Korean => "대상 앱",
            Self::Japanese => "対象アプリ",
            Self::Chinese => "目标应用",
            Self::Russian => "Приложения",
        }
    }

    pub fn tab_shortcuts(&self) -> &'static str {
        match self {
            Self::English => "Shortcuts",
            Self::Spanish => "Atajos (Clutch)",
            Self::Portuguese => "Atalhos (Clutch)",
            Self::Korean => "단축키 (Shortcuts)",
            Self::Japanese => "ショートカット",
            Self::Chinese => "快捷按键",
            Self::Russian => "Горячие клавиши",
        }
    }

    pub fn tab_developer(&self) -> &'static str {
        match self {
            Self::English => "Developer",
            Self::Spanish => "Desarrollador",
            Self::Portuguese => "Desenvolvedor",
            Self::Korean => "개발자 메트릭",
            Self::Japanese => "開発者情報",
            Self::Chinese => "系统指标",
            Self::Russian => "Разработчик",
        }
    }

    pub fn tab_info(&self) -> &'static str {
        match self {
            Self::English => "Information",
            Self::Spanish => "Información",
            Self::Portuguese => "Informações",
            Self::Korean => "프로그램 정보",
            Self::Japanese => "バージョン情報",
            Self::Chinese => "关于软件",
            Self::Russian => "Информация",
        }
    }

    // Top Header Buttons
    pub fn btn_quit(&self) -> &'static str {
        match self {
            Self::English => "Quit",
            Self::Spanish => "Salir",
            Self::Portuguese => "Sair",
            Self::Korean => "종료",
            Self::Japanese => "終了",
            Self::Chinese => "退出",
            Self::Russian => "Выход",
        }
    }

    pub fn btn_cancel(&self) -> &'static str {
        match self {
            Self::English => "Cancel",
            Self::Spanish => "Minimizar",
            Self::Portuguese => "Minimizar",
            Self::Korean => "최소화",
            Self::Japanese => "最小化",
            Self::Chinese => "最小化",
            Self::Russian => "Свернуть",
        }
    }

    // General Card
    pub fn run_on_startup(&self) -> &'static str {
        match self {
            Self::English => "Run on startup?",
            Self::Spanish => "¿Iniciar con Windows?",
            Self::Portuguese => "Iniciar com o Windows?",
            Self::Korean => "윈도우 시작 시 자동 실행?",
            Self::Japanese => "Windows 起動時に実行しますか？",
            Self::Chinese => "随系统开机自启？",
            Self::Russian => "Запуск при старте Windows?",
        }
    }

    pub fn language_label(&self) -> &'static str {
        match self {
            Self::English => "Language:",
            Self::Spanish => "Idioma:",
            Self::Portuguese => "Idioma:",
            Self::Korean => "언어 (Language):",
            Self::Japanese => "言語 (Language):",
            Self::Chinese => "语言 (Language):",
            Self::Russian => "Язык интерфейса:",
        }
    }

    pub fn accent_color(&self) -> &'static str {
        match self {
            Self::English => "Accent Color",
            Self::Spanish => "Color de Acento",
            Self::Portuguese => "Cor de Destaque",
            Self::Korean => "테마 강조 색상 (Accent Color)",
            Self::Japanese => "アクセントカラー",
            Self::Chinese => "主题强调色",
            Self::Russian => "Акцентный цвет темы",
        }
    }

    pub fn target_game(&self) -> &'static str {
        match self {
            Self::English => "Target Competitive Game",
            Self::Spanish => "Juego Competitivo Objetivo",
            Self::Portuguese => "Jogo Competitivo Alvo",
            Self::Korean => "대상 게임 (Target Game)",
            Self::Japanese => "対象ゲーム",
            Self::Chinese => "目标竞技游戏",
            Self::Russian => "Целевая игра",
        }
    }

    // Smart Ducking
    pub fn ducking_title(&self) -> &'static str {
        match self {
            Self::English => "Automatic Audio Ducking Engine",
            Self::Spanish => "Motor de Atenuación Automática de Audio",
            Self::Portuguese => "Motor de Atenuação Automática de Áudio",
            Self::Korean => "스마트 오디오 자동 덕킹 엔진",
            Self::Japanese => "スマート自動オーディオダッキング",
            Self::Chinese => "智能音频闪避引擎 (Smart Ducking)",
            Self::Russian => "Движок автоприглушения фонового звука",
        }
    }

    pub fn ducking_desc(&self) -> &'static str {
        match self {
            Self::English => "Automatically lowers Spotify, Discord, or Apple Music when footsteps and gunshots occur.",
            Self::Spanish => "Baja automáticamente Spotify, Discord o Apple Music cuando ocurren pasos y disparos.",
            Self::Portuguese => "Abaixa automaticamente Spotify, Discord ou Apple Music quando ocorrem passos ou tiros.",
            Self::Korean => "게임 내 발소리나 교전 총성이 발생하면 Spotify, Discord 볼륨을 자동으로 줄여줍니다.",
            Self::Japanese => "足音や銃声が発生すると、SpotifyやDiscordなどの音量を自動的に下げます。",
            Self::Chinese => "游戏中出现脚步声或交火枪声时，自动降低 Spotify、Discord 等后台音量。",
            Self::Russian => "Снижает громкость фоновой музыки и Discord при звуках шагов и выстрелов.",
        }
    }

    pub fn duck_volume_label(&self) -> &'static str {
        match self {
            Self::English => "Background App Volume when Ducked:",
            Self::Spanish => "Volumen de Apps al Atenuar:",
            Self::Portuguese => "Volume dos Apps ao Atenuar:",
            Self::Korean => "덕킹 시 백그라운드 볼륨:",
            Self::Japanese => "ダッキング時のバックグラウンド音量:",
            Self::Chinese => "闪避时后台应用音量:",
            Self::Russian => "Громкость приложений при приглушении:",
        }
    }

    pub fn trigger_sensitivity_label(&self) -> &'static str {
        match self {
            Self::English => "Game Audio Trigger Sensitivity:",
            Self::Spanish => "Sensibilidad de Activación por Audio:",
            Self::Portuguese => "Sensibilidade de Ativação do Jogo:",
            Self::Korean => "게임 사운드 감지 민감도:",
            Self::Japanese => "ゲーム音声トリガー感度:",
            Self::Chinese => "游戏音频触发灵敏度:",
            Self::Russian => "Чувствительность срабатывания:",
        }
    }

    pub fn test_ducking_btn(&self) -> &'static str {
        match self {
            Self::English => "⚡ Test Ducking (3 sec)",
            Self::Spanish => "⚡ Probar Atenuación (3s)",
            Self::Portuguese => "⚡ Testar Atenuação (3s)",
            Self::Korean => "⚡ 덕킹 테스트 (3초)",
            Self::Japanese => "⚡ ダッキングテスト (3秒)",
            Self::Chinese => "⚡ 测试闪避 (3秒)",
            Self::Russian => "⚡ Тест приглушения (3 сек)",
        }
    }

    // Footstep Booster
    pub fn booster_title(&self) -> &'static str {
        match self {
            Self::English => "Footstep Audio Frequency Equalizer",
            Self::Spanish => "Ecualizador de Frecuencia de Pasos",
            Self::Portuguese => "Equalizador de Frequência de Passos",
            Self::Korean => "발소리 오디오 주파수 이퀄라이저",
            Self::Japanese => "足音周波数イコライザー",
            Self::Chinese => "脚步声频率均衡器",
            Self::Russian => "Эквалайзер частот шагов",
        }
    }

    pub fn booster_gain_label(&self) -> &'static str {
        match self {
            Self::English => "Footstep Boost Gain:",
            Self::Spanish => "Ganancia de Pasos:",
            Self::Portuguese => "Ganho de Passos:",
            Self::Korean => "발소리 증폭 게인 (Boost Gain):",
            Self::Japanese => "足音ブーストゲイン:",
            Self::Chinese => "脚步声增益 (Boost Gain):",
            Self::Russian => "Усиление шагов (Gain):",
        }
    }

    pub fn open_sound_panel_btn(&self) -> &'static str {
        match self {
            Self::English => "🎛 Open Windows Loudness Equalization Settings",
            Self::Spanish => "🎛 Abrir Panel de Ecualización de Windows",
            Self::Portuguese => "🎛 Abrir Configurações de Áudio do Windows",
            Self::Korean => "🎛 윈도우 사운드 라우드니스 패널 열기",
            Self::Japanese => "🎛 Windows 音声拡張パネルを開く",
            Self::Chinese => "🎛 打开 Windows 响度均衡与声音设置",
            Self::Russian => "🎛 Открыть панель тонкомпенсации Windows",
        }
    }

    // Shortcuts & Clutch
    pub fn clutch_title(&self) -> &'static str {
        match self {
            Self::English => "Clutch Mode Keybinds",
            Self::Spanish => "Atajos de Modo Clutch",
            Self::Portuguese => "Atalhos do Modo Clutch",
            Self::Korean => "클러치 모드 단축키 (Clutch Mode)",
            Self::Japanese => "クラッチモードのキーバインド",
            Self::Chinese => "残局极限模式快捷键 (Clutch Mode)",
            Self::Russian => "Клавиши режима Clutch",
        }
    }

    pub fn clutch_desc(&self) -> &'static str {
        match self {
            Self::English => "Instantly silences or ducks all background chatter with a single keypress during 1v3 clutch rounds.",
            Self::Spanish => "Silencia o atenúa al instante todo el chat secundario con una tecla durante rondas clutch 1v3.",
            Self::Portuguese => "Silencia ou abaixa instantaneamente o chat de fundo com uma tecla em rodadas clutch 1v3.",
            Self::Korean => "1v3 클러치 상황에서 단 한 번의 키 입력으로 팀원 보이스 및 음악을 즉시 음소거합니다.",
            Self::Japanese => "1v3のクラッチラウンド中に、キー1つで通話や音楽を瞬時に消音・低減します。",
            Self::Chinese => "在 1v3 极限残局时，一键瞬间静音或降低所有后台语音与音乐干扰。",
            Self::Russian => "Мгновенно приглушает Discord и фоновую музыку нажатием одной клавиши в клатч-раундах.",
        }
    }

    pub fn clutch_key_label(&self) -> &'static str {
        match self {
            Self::English => "Emergency Clutch Key:",
            Self::Spanish => "Tecla de Emergencia Clutch:",
            Self::Portuguese => "Tecla de Emergência Clutch:",
            Self::Korean => "비상 클러치 키:",
            Self::Japanese => "緊急クラッチキー:",
            Self::Chinese => "紧急静音按键 (Clutch Key):",
            Self::Russian => "Клавиша Clutch:",
        }
    }

    pub fn press_any_key(&self) -> &'static str {
        match self {
            Self::English => "🔴 Press any key or mouse button...",
            Self::Spanish => "🔴 Presiona cualquier tecla o botón...",
            Self::Portuguese => "🔴 Pressione qualquer tecla ou botão...",
            Self::Korean => "🔴 아무 키나 마우스 버튼을 누르세요...",
            Self::Japanese => "🔴 キーまたはマウスボタンを押してください...",
            Self::Chinese => "🔴 请按下键盘或鼠标任意键...",
            Self::Russian => "🔴 Нажмите любую клавишу или кнопку...",
        }
    }

    pub fn test_clutch_btn(&self) -> &'static str {
        match self {
            Self::English => "Test Clutch Mode",
            Self::Spanish => "Probar Modo Clutch",
            Self::Portuguese => "Testar Modo Clutch",
            Self::Korean => "클러치 모드 테스트",
            Self::Japanese => "クラッチをテスト",
            Self::Chinese => "测试残局模式",
            Self::Russian => "Тест Clutch",
        }
    }
}
