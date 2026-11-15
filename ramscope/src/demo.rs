//! Jeu de données de démonstration : un PC Windows 11 réaliste.
//! Sert à prévisualiser l'interface sans Windows (et à tester le workflow
//! d'application / restauration sans rien toucher à une vraie machine).
use crate::model::*;
use crate::rules;
use std::collections::{HashMap, HashSet};

const MB: u64 = 1024 * 1024;

/// (pid, nom, working set Mo, privé Mo, fenêtre, description, éditeur, chemin)
type P = (
    u32,
    &'static str,
    u64,
    u64,
    bool,
    &'static str,
    &'static str,
    &'static str,
);

const PROCS: &[P] = &[
    (4, "System", 12, 2, false, "NT Kernel & System", "Microsoft Corporation", ""),
    (108, "Registry", 78, 41, false, "Registry", "Microsoft Corporation", ""),
    (420, "smss.exe", 1, 1, false, "Windows Session Manager", "Microsoft Corporation", "C:\\Windows\\System32\\smss.exe"),
    (560, "csrss.exe", 6, 3, false, "Client Server Runtime Process", "Microsoft Corporation", "C:\\Windows\\System32\\csrss.exe"),
    (640, "wininit.exe", 7, 3, false, "Windows Start-Up Application", "Microsoft Corporation", "C:\\Windows\\System32\\wininit.exe"),
    (712, "winlogon.exe", 12, 6, false, "Windows Logon Application", "Microsoft Corporation", "C:\\Windows\\System32\\winlogon.exe"),
    (780, "services.exe", 11, 7, false, "Services and Controller app", "Microsoft Corporation", "C:\\Windows\\System32\\services.exe"),
    (800, "lsass.exe", 24, 16, false, "Local Security Authority Process", "Microsoft Corporation", "C:\\Windows\\System32\\lsass.exe"),
    (980, "svchost.exe", 148, 96, false, "Host Process for Windows Services", "Microsoft Corporation", "C:\\Windows\\System32\\svchost.exe"),
    (1040, "svchost.exe", 96, 62, false, "Host Process for Windows Services", "Microsoft Corporation", "C:\\Windows\\System32\\svchost.exe"),
    (1120, "svchost.exe", 64, 38, false, "Host Process for Windows Services", "Microsoft Corporation", "C:\\Windows\\System32\\svchost.exe"),
    (1210, "svchost.exe", 41, 22, false, "Host Process for Windows Services", "Microsoft Corporation", "C:\\Windows\\System32\\svchost.exe"),
    (1330, "svchost.exe", 33, 18, false, "Host Process for Windows Services", "Microsoft Corporation", "C:\\Windows\\System32\\svchost.exe"),
    (1420, "svchost.exe", 27, 14, false, "Host Process for Windows Services", "Microsoft Corporation", "C:\\Windows\\System32\\svchost.exe"),
    (1510, "svchost.exe", 19, 9, false, "Host Process for Windows Services", "Microsoft Corporation", "C:\\Windows\\System32\\svchost.exe"),
    (1620, "svchost.exe", 52, 31, false, "Host Process for Windows Services", "Microsoft Corporation", "C:\\Windows\\System32\\svchost.exe"),
    (1700, "Memory Compression", 412, 380, false, "Memory Compression", "Microsoft Corporation", ""),
    (1810, "dwm.exe", 132, 88, false, "Desktop Window Manager", "Microsoft Corporation", "C:\\Windows\\System32\\dwm.exe"),
    (1920, "MsMpEng.exe", 268, 214, false, "Antimalware Service Executable", "Microsoft Corporation", "C:\\ProgramData\\Microsoft\\Windows Defender\\Platform\\4.18\\MsMpEng.exe"),
    (2040, "SearchIndexer.exe", 186, 152, false, "Microsoft Windows Search Indexer", "Microsoft Corporation", "C:\\Windows\\System32\\SearchIndexer.exe"),
    (2210, "explorer.exe", 214, 148, true, "Windows Explorer", "Microsoft Corporation", "C:\\Windows\\explorer.exe"),
    (2320, "sihost.exe", 34, 18, false, "Shell Infrastructure Host", "Microsoft Corporation", "C:\\Windows\\System32\\sihost.exe"),
    (2410, "taskhostw.exe", 22, 11, false, "Host Process for Windows Tasks", "Microsoft Corporation", "C:\\Windows\\System32\\taskhostw.exe"),
    (2520, "StartMenuExperienceHost.exe", 96, 62, false, "Start Menu", "Microsoft Corporation", "C:\\Windows\\SystemApps\\StartMenuApp\\StartMenuExperienceHost.exe"),
    (2610, "SearchHost.exe", 118, 74, false, "Search", "Microsoft Corporation", "C:\\Windows\\SystemApps\\SearchApp\\SearchHost.exe"),
    (2720, "RuntimeBroker.exe", 28, 14, false, "Runtime Broker", "Microsoft Corporation", "C:\\Windows\\System32\\RuntimeBroker.exe"),
    (2810, "ctfmon.exe", 24, 12, false, "CTF Loader", "Microsoft Corporation", "C:\\Windows\\System32\\ctfmon.exe"),
    (2920, "Widgets.exe", 248, 196, false, "Windows Widgets", "Microsoft Corporation", "C:\\Program Files\\WindowsApps\\Widgets\\Widgets.exe"),
    (3010, "msedgewebview2.exe", 176, 132, false, "Microsoft Edge WebView2", "Microsoft Corporation", "C:\\Program Files (x86)\\Microsoft\\EdgeWebView\\Application\\msedgewebview2.exe"),
    (3110, "msedgewebview2.exe", 94, 66, false, "Microsoft Edge WebView2", "Microsoft Corporation", "C:\\Program Files (x86)\\Microsoft\\EdgeWebView\\Application\\msedgewebview2.exe"),
    (3210, "PhoneExperienceHost.exe", 142, 104, false, "Mobile connecté", "Microsoft Corporation", "C:\\Program Files\\WindowsApps\\YourPhone\\PhoneExperienceHost.exe"),
    (3320, "SecurityHealthSystray.exe", 9, 4, false, "Windows Security notification icon", "Microsoft Corporation", "C:\\Windows\\System32\\SecurityHealthSystray.exe"),
    (3410, "OneDrive.exe", 226, 168, false, "Microsoft OneDrive", "Microsoft Corporation", "C:\\Users\\Camille\\AppData\\Local\\Microsoft\\OneDrive\\OneDrive.exe"),
    (3520, "ms-teams.exe", 486, 372, true, "Microsoft Teams", "Microsoft Corporation", "C:\\Program Files\\WindowsApps\\MSTeams\\ms-teams.exe"),
    (3610, "msedgewebview2.exe", 212, 158, false, "Microsoft Edge WebView2", "Microsoft Corporation", "C:\\Program Files (x86)\\Microsoft\\EdgeWebView\\Application\\msedgewebview2.exe"),
    (3720, "Spotify.exe", 342, 268, false, "Spotify", "Spotify AB", "C:\\Users\\Camille\\AppData\\Roaming\\Spotify\\Spotify.exe"),
    (3810, "Spotify.exe", 128, 92, false, "Spotify", "Spotify AB", "C:\\Users\\Camille\\AppData\\Roaming\\Spotify\\Spotify.exe"),
    (3920, "Discord.exe", 298, 232, false, "Discord", "Discord Inc.", "C:\\Users\\Camille\\AppData\\Local\\Discord\\app-1.0.9\\Discord.exe"),
    (4010, "Discord.exe", 164, 122, false, "Discord", "Discord Inc.", "C:\\Users\\Camille\\AppData\\Local\\Discord\\app-1.0.9\\Discord.exe"),
    (4120, "steam.exe", 186, 138, false, "Steam Client Bootstrapper", "Valve Corporation", "C:\\Program Files (x86)\\Steam\\steam.exe"),
    (4210, "steamwebhelper.exe", 328, 264, false, "Steam Client WebHelper", "Valve Corporation", "C:\\Program Files (x86)\\Steam\\bin\\cef\\steamwebhelper.exe"),
    (4310, "steamwebhelper.exe", 142, 104, false, "Steam Client WebHelper", "Valve Corporation", "C:\\Program Files (x86)\\Steam\\bin\\cef\\steamwebhelper.exe"),
    (4420, "EpicGamesLauncher.exe", 264, 206, false, "Epic Games Launcher", "Epic Games, Inc.", "C:\\Program Files (x86)\\Epic Games\\Launcher\\Portal\\Binaries\\Win64\\EpicGamesLauncher.exe"),
    (4520, "CCXProcess.exe", 88, 62, false, "CCXProcess", "Adobe Inc.", "C:\\Program Files\\Adobe\\Adobe Creative Cloud\\CCXProcess\\CCXProcess.exe"),
    (4610, "Creative Cloud.exe", 196, 148, false, "Adobe Creative Cloud", "Adobe Inc.", "C:\\Program Files\\Adobe\\Adobe Creative Cloud\\ACC\\Creative Cloud.exe"),
    (4720, "AdobeIPCBroker.exe", 32, 20, false, "Adobe IPC Broker", "Adobe Inc.", "C:\\Program Files\\Common Files\\Adobe\\Adobe Desktop Common\\IPCBox\\AdobeIPCBroker.exe"),
    (4810, "AdobeUpdateService.exe", 18, 11, false, "Adobe Update Service", "Adobe Inc.", "C:\\Program Files (x86)\\Common Files\\Adobe\\ARM\\AdobeUpdateService.exe"),
    (4920, "jusched.exe", 14, 8, false, "Java Update Scheduler", "Oracle Corporation", "C:\\Program Files (x86)\\Common Files\\Java\\Java Update\\jusched.exe"),
    (5010, "iTunesHelper.exe", 22, 13, false, "iTunesHelper", "Apple Inc.", "C:\\Program Files\\iTunes\\iTunesHelper.exe"),
    (5120, "nvcontainer.exe", 96, 68, false, "NVIDIA Container", "NVIDIA Corporation", "C:\\Program Files\\NVIDIA Corporation\\NvContainer\\nvcontainer.exe"),
    (5210, "NVIDIA Share.exe", 118, 86, false, "NVIDIA Share", "NVIDIA Corporation", "C:\\Program Files\\NVIDIA Corporation\\NVIDIA GeForce Experience\\NVIDIA Share.exe"),
    (5320, "iCUE.exe", 412, 336, false, "Corsair iCUE", "Corsair Memory, Inc.", "C:\\Program Files\\Corsair\\Corsair iCUE5 Software\\iCUE.exe"),
    (5420, "SupportAssistAgent.exe", 164, 128, false, "Dell SupportAssist", "Dell Inc.", "C:\\Program Files\\Dell\\SupportAssist\\SupportAssistAgent.exe"),
    (5520, "chrome.exe", 512, 402, true, "Google Chrome", "Google LLC", "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"),
    (5610, "chrome.exe", 286, 224, false, "Google Chrome", "Google LLC", "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"),
    (5710, "chrome.exe", 198, 156, false, "Google Chrome", "Google LLC", "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"),
    (5820, "Everything.exe", 46, 34, false, "Everything", "voidtools", "C:\\Program Files\\Everything\\Everything.exe"),
    (5920, "ramscope.exe", 38, 26, true, "RamScope", "RamScope", "C:\\Users\\Camille\\Downloads\\ramscope.exe"),
];

/// (nom, affiché, démarrage, en cours, pid hôte, description)
type S = (
    &'static str,
    &'static str,
    StartType,
    bool,
    u32,
    &'static str,
);

const SVCS: &[S] = &[
    (
        "RpcSs",
        "Appels de procédure distante (RPC)",
        StartType::Auto,
        true,
        980,
        "Le sous-système RPC est le gestionnaire de contrôle des services.",
    ),
    (
        "DcomLaunch",
        "Lanceur de processus serveur DCOM",
        StartType::Auto,
        true,
        980,
        "Charge les objets COM.",
    ),
    (
        "EventLog",
        "Journal d'événements Windows",
        StartType::Auto,
        true,
        1040,
        "Enregistre les événements dans les journaux Windows.",
    ),
    (
        "WinDefend",
        "Service antivirus Microsoft Defender",
        StartType::Auto,
        true,
        1920,
        "Protège contre les logiciels malveillants.",
    ),
    (
        "Spooler",
        "Spouleur d'impression",
        StartType::Auto,
        true,
        1120,
        "Charge les fichiers en mémoire pour l'impression.",
    ),
    (
        "WSearch",
        "Windows Search",
        StartType::Delayed,
        true,
        2040,
        "Indexation du contenu et mise en cache des propriétés.",
    ),
    (
        "SysMain",
        "SysMain",
        StartType::Auto,
        true,
        1040,
        "Maintient et améliore les performances système.",
    ),
    (
        "DiagTrack",
        "Expériences des utilisateurs connectés et télémétrie",
        StartType::Auto,
        true,
        1210,
        "Envoie des données de diagnostic à Microsoft.",
    ),
    (
        "dmwappushservice",
        "Service de routage de messages push WAP",
        StartType::Manual,
        false,
        0,
        "Routage des messages push WAP.",
    ),
    (
        "MapsBroker",
        "Gestionnaire des cartes téléchargées",
        StartType::Auto,
        true,
        1330,
        "Gère l'accès aux cartes téléchargées.",
    ),
    (
        "Fax",
        "Télécopie",
        StartType::Manual,
        false,
        0,
        "Envoi et réception de fax.",
    ),
    (
        "RemoteRegistry",
        "Registre à distance",
        StartType::Disabled,
        false,
        0,
        "Permet aux utilisateurs distants de modifier le registre.",
    ),
    (
        "WerSvc",
        "Service de rapport d'erreurs Windows",
        StartType::Manual,
        false,
        0,
        "Signale les erreurs des programmes.",
    ),
    (
        "TrkWks",
        "Client de suivi de lien distribué",
        StartType::Auto,
        true,
        1040,
        "Suit les liens NTFS sur le réseau.",
    ),
    (
        "WMPNetworkSvc",
        "Service de partage réseau du Lecteur Windows Media",
        StartType::Manual,
        false,
        0,
        "Partage la bibliothèque multimédia.",
    ),
    (
        "bthserv",
        "Service de prise en charge Bluetooth",
        StartType::Manual,
        true,
        1330,
        "Prend en charge la découverte Bluetooth.",
    ),
    (
        "TabletInputService",
        "Service du clavier tactile et du volet d'écriture",
        StartType::Manual,
        true,
        1420,
        "Active le clavier tactile.",
    ),
    (
        "WbioSrvc",
        "Service de biométrie Windows",
        StartType::Manual,
        false,
        0,
        "Prise en charge des lecteurs d'empreintes.",
    ),
    (
        "XblAuthManager",
        "Gestionnaire d'authentification Xbox Live",
        StartType::Manual,
        false,
        0,
        "Authentification Xbox Live.",
    ),
    (
        "XblGameSave",
        "Service de sauvegarde de jeux Xbox Live",
        StartType::Manual,
        false,
        0,
        "Synchronisation des sauvegardes.",
    ),
    (
        "XboxNetApiSvc",
        "Service de mise en réseau Xbox Live",
        StartType::Manual,
        false,
        0,
        "Prise en charge du multijoueur Xbox.",
    ),
    (
        "XboxGipSvc",
        "Service de gestion des accessoires Xbox",
        StartType::Manual,
        false,
        0,
        "Gère les manettes Xbox.",
    ),
    (
        "WalletService",
        "Service de portefeuille",
        StartType::Manual,
        false,
        0,
        "Portefeuille Microsoft.",
    ),
    (
        "PhoneSvc",
        "Service de téléphonie",
        StartType::Manual,
        true,
        1420,
        "Gère l'état de la téléphonie.",
    ),
    (
        "lfsvc",
        "Service de géolocalisation",
        StartType::Manual,
        true,
        1330,
        "Surveille la position du système.",
    ),
    (
        "SharedAccess",
        "Partage de connexion Internet (ICS)",
        StartType::Manual,
        false,
        0,
        "Partage de connexion.",
    ),
    (
        "RetailDemo",
        "Service de démonstration du magasin",
        StartType::Manual,
        false,
        0,
        "Mode démonstration.",
    ),
    (
        "wisvc",
        "Service Windows Insider",
        StartType::Manual,
        false,
        0,
        "Programme Insider.",
    ),
    (
        "PcaSvc",
        "Service de l'Assistant Compatibilité des programmes",
        StartType::Auto,
        true,
        1210,
        "Assistant de compatibilité.",
    ),
    (
        "DoSvc",
        "Optimisation de la distribution",
        StartType::Delayed,
        true,
        1510,
        "Partage des mises à jour en pair-à-pair.",
    ),
    (
        "CDPSvc",
        "Plateforme des appareils connectés",
        StartType::Auto,
        true,
        1620,
        "Utilisé par Mobile connecté et le partage rapide.",
    ),
    (
        "DusmSvc",
        "Consommation des données",
        StartType::Auto,
        true,
        1620,
        "Surveille l'utilisation des données.",
    ),
    (
        "WpcMonSvc",
        "Contrôle parental",
        StartType::Manual,
        false,
        0,
        "Applique le contrôle parental.",
    ),
    (
        "SEMgrSvc",
        "Gestionnaire des paiements et des éléments sécurisés NFC",
        StartType::Manual,
        false,
        0,
        "Paiements NFC.",
    ),
    (
        "SCardSvr",
        "Carte à puce",
        StartType::Manual,
        false,
        0,
        "Gère l'accès aux cartes à puce.",
    ),
    (
        "TermService",
        "Services Bureau à distance",
        StartType::Manual,
        false,
        0,
        "Connexion Bureau à distance.",
    ),
    (
        "LanmanServer",
        "Serveur",
        StartType::Auto,
        true,
        1120,
        "Partage de fichiers et d'imprimantes.",
    ),
    (
        "LanmanWorkstation",
        "Station de travail",
        StartType::Auto,
        true,
        1120,
        "Connexions aux partages réseau.",
    ),
    (
        "SSDPSRV",
        "Découverte SSDP",
        StartType::Manual,
        true,
        1330,
        "Découvre les appareils UPnP.",
    ),
    (
        "upnphost",
        "Hôte de périphérique UPnP",
        StartType::Manual,
        false,
        0,
        "Héberge les périphériques UPnP.",
    ),
    (
        "WlanSvc",
        "Service de configuration automatique WLAN",
        StartType::Auto,
        true,
        1040,
        "Gère les connexions Wi-Fi.",
    ),
    (
        "Dhcp",
        "Client DHCP",
        StartType::Auto,
        true,
        1040,
        "Obtient une adresse IP.",
    ),
    (
        "Dnscache",
        "Client DNS",
        StartType::Auto,
        true,
        1210,
        "Met en cache les noms DNS.",
    ),
    (
        "wuauserv",
        "Windows Update",
        StartType::Manual,
        true,
        1510,
        "Détection et installation des mises à jour.",
    ),
    (
        "BITS",
        "Service de transfert intelligent en arrière-plan",
        StartType::Manual,
        true,
        1510,
        "Transferts en arrière-plan.",
    ),
    (
        "Themes",
        "Thèmes",
        StartType::Auto,
        true,
        1040,
        "Gère les thèmes visuels.",
    ),
    (
        "AudioSrv",
        "Audio Windows",
        StartType::Auto,
        true,
        1420,
        "Gère l'audio.",
    ),
    (
        "AudioEndpointBuilder",
        "Générateur de points de terminaison audio",
        StartType::Auto,
        true,
        1420,
        "Gère les périphériques audio.",
    ),
    (
        "gupdate",
        "Service Google Update (gupdate)",
        StartType::Auto,
        true,
        4810,
        "Maintient Google Chrome à jour.",
    ),
    (
        "gupdatem",
        "Service Google Update (gupdatem)",
        StartType::Manual,
        false,
        0,
        "Maintient les logiciels Google à jour.",
    ),
    (
        "edgeupdate",
        "Service Mise à jour de Microsoft Edge (edgeupdate)",
        StartType::Auto,
        true,
        4810,
        "Maintient Microsoft Edge à jour.",
    ),
    (
        "AdobeARMservice",
        "Adobe Acrobat Update Service",
        StartType::Auto,
        true,
        4810,
        "Maintient Adobe Acrobat à jour.",
    ),
    (
        "Bonjour Service",
        "Service Bonjour",
        StartType::Auto,
        true,
        4920,
        "Découverte de services mDNS (Apple).",
    ),
    (
        "NVDisplay.ContainerLocalSystem",
        "NVIDIA Display Container LS",
        StartType::Auto,
        true,
        5120,
        "Conteneur du pilote NVIDIA.",
    ),
    (
        "NvTelemetryContainer",
        "NVIDIA Telemetry Container",
        StartType::Auto,
        true,
        5120,
        "Collecte de télémétrie NVIDIA.",
    ),
    (
        "ESRV_SVC_QUEENCREEK",
        "Intel(R) SUR QC Software Asset Manager",
        StartType::Auto,
        true,
        5420,
        "Collecte de données Intel.",
    ),
    (
        "DDVDataCollector",
        "Dell Data Vault Collector",
        StartType::Auto,
        true,
        5420,
        "Collecte de données Dell SupportAssist.",
    ),
    (
        "SupportAssistAgent",
        "Dell SupportAssist",
        StartType::Auto,
        true,
        5420,
        "Assistance et diagnostic Dell.",
    ),
    (
        "Corsair Service",
        "Corsair Service",
        StartType::Auto,
        true,
        5320,
        "Service Corsair iCUE.",
    ),
    (
        "Steam Client Service",
        "Steam Client Service",
        StartType::Manual,
        false,
        0,
        "Service d'installation Steam.",
    ),
    (
        "TeamViewer",
        "TeamViewer",
        StartType::Auto,
        true,
        5210,
        "Accès distant permanent TeamViewer.",
    ),
    (
        "W32Time",
        "Temps Windows",
        StartType::Manual,
        true,
        1420,
        "Synchronisation de l'heure.",
    ),
    (
        "StorSvc",
        "Service de stockage",
        StartType::Manual,
        true,
        1620,
        "Gère le stockage.",
    ),
    (
        "FontCache",
        "Service de cache de polices Windows",
        StartType::Auto,
        true,
        1420,
        "Met en cache les polices.",
    ),
];

/// (id, nom, commande, emplacement, activé)
type ST = (&'static str, &'static str, &'static str, &'static str, bool);

const STARTUP: &[ST] = &[
    ("hkcu_run|OneDrive", "OneDrive", "\"C:\\Users\\Camille\\AppData\\Local\\Microsoft\\OneDrive\\OneDrive.exe\" /background", "Registre (utilisateur)", true),
    ("hkcu_run|com.squirrel.teams.Teams", "com.squirrel.teams.Teams", "\"C:\\Users\\Camille\\AppData\\Local\\Microsoft\\Teams\\Update.exe\" --processStart \"Teams.exe\"", "Registre (utilisateur)", true),
    ("hkcu_run|Spotify", "Spotify", "\"C:\\Users\\Camille\\AppData\\Roaming\\Spotify\\Spotify.exe\" /minimized", "Registre (utilisateur)", true),
    ("hkcu_run|Discord", "Discord", "\"C:\\Users\\Camille\\AppData\\Local\\Discord\\Update.exe\" --processStart Discord.exe", "Registre (utilisateur)", true),
    ("hkcu_run|Steam", "Steam", "\"C:\\Program Files (x86)\\Steam\\steam.exe\" -silent", "Registre (utilisateur)", true),
    ("hkcu_run|EpicGamesLauncher", "EpicGamesLauncher", "\"C:\\Program Files (x86)\\Epic Games\\Launcher\\Portal\\Binaries\\Win64\\EpicGamesLauncher.exe\" -silent", "Registre (utilisateur)", true),
    ("hkcu_run|MicrosoftEdgeAutoLaunch_A1B2C3", "MicrosoftEdgeAutoLaunch_A1B2C3", "\"C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe\" --no-startup-window --win-session-start", "Registre (utilisateur)", true),
    ("hklm_run|SecurityHealth", "SecurityHealth", "%windir%\\system32\\SecurityHealthSystray.exe", "Registre (machine)", true),
    ("hklm_run|SunJavaUpdateSched", "SunJavaUpdateSched", "\"C:\\Program Files (x86)\\Common Files\\Java\\Java Update\\jusched.exe\"", "Registre (machine)", true),
    ("hklm_run|iTunesHelper", "iTunesHelper", "\"C:\\Program Files\\iTunes\\iTunesHelper.exe\"", "Registre (machine)", true),
    ("hklm_run|AdobeAAMUpdater-1.0", "AdobeAAMUpdater-1.0", "\"C:\\Program Files (x86)\\Common Files\\Adobe\\OOBE\\PDApp\\UWA\\updaterstartuputility.exe\"", "Registre (machine)", true),
    ("hklm_wow_run|CCXProcess", "CCXProcess", "\"C:\\Program Files\\Adobe\\Adobe Creative Cloud\\CCXProcess\\CCXProcess.exe\"", "Registre (machine 32 bits)", true),
    ("hkcu_run|CreativeCloud", "Adobe Creative Cloud", "\"C:\\Program Files\\Adobe\\Adobe Creative Cloud\\ACC\\Creative Cloud.exe\" /background", "Registre (utilisateur)", true),
    ("hklm_run|RtkAudUService", "RtkAudUService", "\"C:\\Windows\\System32\\DriverStore\\RtkAudUService64.exe\" -background", "Registre (machine)", true),
    ("hkcu_run|icue", "iCUE", "\"C:\\Program Files\\Corsair\\Corsair iCUE5 Software\\iCUE.exe\" --autorun", "Registre (utilisateur)", true),
    ("hkcu_folder|Everything.lnk", "Everything.lnk", "C:\\Program Files\\Everything\\Everything.exe", "Dossier Démarrage (utilisateur)", true),
    ("hkcu_folder|Vieux scanner.lnk", "Vieux scanner.lnk", "C:\\Program Files (x86)\\ScanSoft\\PaperPort\\PaprPort.exe", "Dossier Démarrage (utilisateur)", false),
    ("hklm_folder|Dell SupportAssist.lnk", "Dell SupportAssist.lnk", "C:\\Program Files\\Dell\\SupportAssist\\SupportAssistAgent.exe", "Dossier Démarrage (tous les utilisateurs)", true),
];

/// (chemin, activé, déclencheurs, commande, auteur, description)
type T = (
    &'static str,
    bool,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

const TASKS: &[T] = &[
    ("\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser", true, "Planifié (date/heure)|Au démarrage du PC", "%windir%\\system32\\compattelrunner.exe", "Microsoft Corporation", "Collecte des informations de compatibilité pour les mises à jour."),
    ("\\Microsoft\\Windows\\Application Experience\\ProgramDataUpdater", true, "Planifié (date/heure)", "%windir%\\system32\\compattelrunner.exe -maintenance", "Microsoft Corporation", "Collecte des données d'utilisation des programmes."),
    ("\\Microsoft\\Windows\\Application Experience\\StartupAppTask", true, "Au démarrage du PC", "%windir%\\system32\\rundll32.exe Startupscan.dll,SusRunTask", "Microsoft Corporation", "Analyse les entrées de démarrage."),
    ("\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator", true, "Planifié (date/heure)", "%windir%\\system32\\wsqmcons.exe", "Microsoft Corporation", "Collecte et envoie les données d'utilisation à Microsoft."),
    ("\\Microsoft\\Windows\\Customer Experience Improvement Program\\UsbCeip", true, "Sur un événement", "%windir%\\system32\\usbceip.dll", "Microsoft Corporation", "Statistiques USB pour le programme CEIP."),
    ("\\Microsoft\\Windows\\Autochk\\Proxy", true, "Au démarrage du PC", "%windir%\\system32\\rundll32.exe acproxy.dll,PerformPostchkOperations", "Microsoft Corporation", "Collecte les données SQM après autochk."),
    ("\\Microsoft\\Windows\\DiskDiagnostic\\Microsoft-Windows-DiskDiagnosticDataCollector", true, "Planifié (date/heure)", "%windir%\\system32\\rundll32.exe dfdts.dll,DfdGetDefaultPolicyAndSMART", "Microsoft Corporation", "Collecte des informations SMART sur les disques."),
    ("\\Microsoft\\Windows\\Feedback\\Siuf\\DmClient", true, "Sur un événement", "%windir%\\system32\\dmclient.exe", "Microsoft Corporation", "Collecte de télémétrie pour le Hub de commentaires."),
    ("\\Microsoft\\Windows\\Windows Error Reporting\\QueueReporting", true, "Sur un événement|Planifié (date/heure)", "%windir%\\system32\\wermgr.exe -upload", "Microsoft Corporation", "Envoie les rapports d'erreurs en file d'attente."),
    ("\\Microsoft\\Windows\\Maps\\MapsUpdateTask", true, "Planifié (date/heure)", "%windir%\\system32\\MapsToastTask.exe", "Microsoft Corporation", "Met à jour les cartes hors ligne."),
    ("\\Microsoft\\Windows\\Shell\\FamilySafetyMonitor", true, "À l'ouverture de session", "%windir%\\system32\\wpcmon.exe", "Microsoft Corporation", "Surveille le contrôle parental."),
    ("\\Microsoft\\Windows\\Defrag\\ScheduledDefrag", true, "Planifié (date/heure)", "%windir%\\system32\\defrag.exe -c -h -k -g", "Microsoft Corporation", "Optimise les lecteurs (TRIM SSD)."),
    ("\\Microsoft\\Windows\\SystemRestore\\SR", true, "Planifié (date/heure)|Au démarrage du PC", "%windir%\\system32\\srtasks.exe ExecuteScopeRestorePointSchedule", "Microsoft Corporation", "Crée des points de restauration système."),
    ("\\Microsoft\\Windows\\Windows Defender\\Windows Defender Scheduled Scan", true, "Planifié (date/heure)|Quand le PC est inactif", "C:\\ProgramData\\Microsoft\\Windows Defender\\Platform\\4.18\\MpCmdRun.exe Scan -ScheduleJob", "Microsoft Corporation", "Analyse planifiée de Defender."),
    ("\\Microsoft\\Windows\\Storage Sense\\StorageSense", true, "Planifié (date/heure)", "%windir%\\system32\\cleanmgr.exe /autocleanstoragesense", "Microsoft Corporation", "Libère de l'espace disque automatiquement."),
    ("\\Microsoft\\Office\\OfficeTelemetryAgentLogOn", true, "À l'ouverture de session", "C:\\Program Files\\Microsoft Office\\root\\Office16\\msoia.exe scan upload", "Microsoft Corporation", "Télémétrie Office."),
    ("\\Microsoft\\Office\\OfficeTelemetryAgentFallBack", true, "Planifié (date/heure)", "C:\\Program Files\\Microsoft Office\\root\\Office16\\msoia.exe scan upload", "Microsoft Corporation", "Télémétrie Office (repli)."),
    ("\\Microsoft\\Office\\Office ClickToRun Service Monitor", true, "Planifié (date/heure)", "C:\\Program Files\\Common Files\\Microsoft Shared\\ClickToRun\\OfficeC2RClient.exe /monitor", "Microsoft Corporation", "Surveille le service Office Click-to-Run."),
    ("\\GoogleUpdateTaskMachineCore", true, "À l'ouverture de session|Planifié (date/heure)", "C:\\Program Files (x86)\\Google\\Update\\GoogleUpdate.exe /c", "Google LLC", "Maintient les logiciels Google à jour."),
    ("\\GoogleUpdateTaskMachineUA", true, "Planifié (date/heure)", "C:\\Program Files (x86)\\Google\\Update\\GoogleUpdate.exe /ua /installsource scheduler", "Google LLC", "Mise à jour Google (machine)."),
    ("\\MicrosoftEdgeUpdateTaskMachineCore", true, "À l'ouverture de session|Planifié (date/heure)", "C:\\Program Files (x86)\\Microsoft\\EdgeUpdate\\MicrosoftEdgeUpdate.exe /c", "Microsoft Corporation", "Maintient Microsoft Edge à jour."),
    ("\\Adobe Acrobat Update Task", true, "À l'ouverture de session|Planifié (date/heure)", "C:\\Program Files (x86)\\Common Files\\Adobe\\ARM\\1.0\\AdobeARM.exe", "Adobe Inc.", "Recherche les mises à jour d'Adobe Acrobat."),
    ("\\OneDrive Standalone Update Task-S-1-5-21", true, "Planifié (date/heure)", "C:\\Users\\Camille\\AppData\\Local\\Microsoft\\OneDrive\\OneDriveStandaloneUpdater.exe", "Microsoft Corporation", "Met à jour OneDrive."),
    ("\\CCleanerSkipUAC - Camille", true, "À l'ouverture de session", "\"C:\\Program Files\\CCleaner\\CCleaner.exe\" $(Arg0)", "Piriform Software Ltd", "Lance CCleaner sans invite UAC."),
    ("\\NvDriverUpdateCheckDaily_{B2FE1952}", true, "Planifié (date/heure)", "C:\\Program Files\\NVIDIA Corporation\\NvContainer\\nvcontainer.exe -f NvDriverUpdateCheck.log", "NVIDIA Corporation", "Vérifie les mises à jour du pilote NVIDIA."),
    ("\\Dell SupportAssist Remediation", true, "À l'ouverture de session", "C:\\Program Files\\Dell\\SupportAssist\\SupportAssistAgent.exe", "Dell Inc.", "Correctifs automatiques Dell."),
];

#[derive(Default)]
pub struct DemoState {
    pub svc: HashMap<String, StartType>,
    pub running: HashMap<String, bool>,
    pub startup: HashMap<String, bool>,
    pub tasks: HashMap<String, bool>,
    pub killed: HashSet<u32>,
}

impl DemoState {
    pub fn snapshot(&self) -> Snapshot {
        let mut processes: Vec<ProcessInfo> = PROCS
            .iter()
            .filter(|p| !self.killed.contains(&p.0))
            .map(|p| ProcessInfo {
                pid: p.0,
                ppid: 780,
                name: p.1.to_string(),
                description: p.5.to_string(),
                company: p.6.to_string(),
                path: p.7.to_string(),
                created_at: 1_700_000_000_000_000 + p.0 as u64 * 10_000,
                working_set: p.2 * MB,
                private: p.3 * MB,
                threads: 12,
                has_window: p.4,
                services: Vec::new(),
                category: String::new(),
                advice: Advice::new(Level::Unknown, ""),
            })
            .collect();

        let ram_by_pid: HashMap<u32, u64> =
            processes.iter().map(|p| (p.pid, p.working_set)).collect();
        let mut host_count: HashMap<u32, u32> = HashMap::new();
        for s in SVCS {
            let running = *self.running.get(s.0).unwrap_or(&s.3);
            if running && s.4 > 0 {
                *host_count.entry(s.4).or_insert(0) += 1;
            }
        }

        let mut services: Vec<ServiceInfo> = Vec::new();
        for s in SVCS {
            let start = *self.svc.get(s.0).unwrap_or(&s.2);
            let running =
                *self.running.get(s.0).unwrap_or(&s.3) && !matches!(start, StartType::Disabled);
            let shared = if s.4 > 0 {
                *host_count.get(&s.4).unwrap_or(&1)
            } else {
                1
            };
            let host_ram = *ram_by_pid.get(&s.4).unwrap_or(&0);
            let ram = if running && shared > 0 {
                host_ram / shared as u64
            } else {
                0
            };
            let path = if s.4 == 980
                || s.4 == 1040
                || s.4 == 1120
                || s.4 == 1210
                || s.4 == 1330
                || s.4 == 1420
                || s.4 == 1510
                || s.4 == 1620
            {
                format!("C:\\Windows\\system32\\svchost.exe -k {}", s.0)
            } else if s.4 > 0 {
                processes
                    .iter()
                    .find(|p| p.pid == s.4)
                    .map(|p| p.path.clone())
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let advice = rules::service_advice(s.0, s.1, &path, start, running, ram);
            services.push(ServiceInfo {
                name: s.0.to_string(),
                display: s.1.to_string(),
                description: s.5.to_string(),
                running,
                state: if running {
                    "En cours".into()
                } else {
                    "Arrêté".into()
                },
                start,
                pid: if running { s.4 } else { 0 },
                path,
                dll: String::new(),
                account: "LocalSystem".into(),
                host_ram,
                host_shared: shared,
                ram,
                protected: rules::is_protected_service(s.0),
                advice,
            });
            if running && s.4 > 0 {
                if let Some(p) = processes.iter_mut().find(|p| p.pid == s.4) {
                    p.services.push(s.0.to_string());
                }
            }
        }

        for p in processes.iter_mut() {
            let mut a = rules::process_advice(
                &p.name,
                &p.path,
                &p.description,
                &p.company,
                p.private,
                p.has_window,
                &p.services,
            );
            if let Some(Action::Kill {
                pid,
                image_path,
                created_at,
                ..
            }) = &mut a.action
            {
                *pid = p.pid;
                *image_path = p.path.clone();
                *created_at = p.created_at;
            }
            if rules::is_protected_process(&p.name) {
                a.action = None;
                a.gain = Gain::unavailable();
            }
            p.advice = a;
            p.category = if !p.services.is_empty() {
                "Hôte de services".into()
            } else if p.path.to_lowercase().contains("\\windows\\") {
                "Système".into()
            } else if p.has_window {
                "Application".into()
            } else {
                "Arrière-plan".into()
            };
        }

        let exe_ram: HashMap<String, u64> = {
            let mut m: HashMap<String, u64> = HashMap::new();
            for p in &processes {
                *m.entry(p.name.to_lowercase()).or_insert(0) += p.private;
            }
            m
        };

        let mut startup: Vec<StartupItem> = Vec::new();
        for s in STARTUP {
            let enabled = *self.startup.get(s.0).unwrap_or(&s.4);
            let exe = crate::util::extract_exe(s.2);
            let ram = if enabled {
                *exe_ram.get(&exe.to_lowercase()).unwrap_or(&0)
            } else {
                0
            };
            let mut advice = rules::startup_advice(s.1, s.2, enabled, ram);
            if let Some(Action::Startup { id, .. }) = &mut advice.action {
                *id = s.0.to_string();
            }
            startup.push(StartupItem {
                id: s.0.to_string(),
                name: s.1.to_string(),
                command: s.2.to_string(),
                location: s.3.to_string(),
                enabled,
                ram,
                advice,
            });
        }

        let mut tasks: Vec<TaskItem> = Vec::new();
        for t in TASKS {
            let enabled = *self.tasks.get(t.0).unwrap_or(&t.1);
            let triggers: Vec<String> = t.2.split('|').map(|s| s.to_string()).collect();
            let advice = rules::task_advice(t.0, t.3, enabled, &triggers);
            tasks.push(TaskItem {
                path: t.0.to_string(),
                name: t.0.rsplit('\\').next().unwrap_or("").to_string(),
                enabled,
                triggers,
                command: t.3.to_string(),
                author: t.4.to_string(),
                description: t.5.to_string(),
                advice,
            });
        }

        processes.sort_by_key(|p| std::cmp::Reverse(p.private));
        services.sort_by(|a, b| {
            a.advice
                .level
                .rank()
                .cmp(&b.advice.level.rank())
                .then(b.ram.cmp(&a.ram))
        });
        startup.sort_by(|a, b| {
            a.advice
                .level
                .rank()
                .cmp(&b.advice.level.rank())
                .then(b.ram.cmp(&a.ram))
        });

        let total = 16 * 1024 * MB;
        let sum_ws: u64 = processes.iter().map(|p| p.working_set).sum();
        let sum_priv: u64 = processes.iter().map(|p| p.private).sum();
        let used = (sum_ws + 900 * MB).min(total - 400 * MB);

        let mut counts = Counts {
            processes: processes.len(),
            services: services.len(),
            services_auto: services
                .iter()
                .filter(|s| matches!(s.start, StartType::Auto | StartType::Delayed))
                .count(),
            services_running: services.iter().filter(|s| s.running).count(),
            startup_enabled: startup.iter().filter(|s| s.enabled).count(),
            tasks_enabled: tasks.iter().filter(|t| t.enabled).count(),
            actionable: 0,
        };
        let mut planned: Vec<crate::plan::GainEntry> = Vec::new();
        for s in services.iter().filter(|s| s.advice.level.preselected()) {
            if let Some(action) = &s.advice.action {
                counts.actionable += 1;
                let host = if s.host_shared > 1 && s.pid > 0 {
                    Some(crate::plan::HostGain {
                        pid: s.pid,
                        services: s.host_shared,
                        host_ram: s.host_ram,
                    })
                } else {
                    None
                };
                planned.push(crate::plan::GainEntry {
                    action: action.clone(),
                    gain: s.advice.gain,
                    gain_next_boot: s.advice.gain_next_boot,
                    host,
                });
            }
        }
        for s in startup.iter().filter(|s| s.advice.level.preselected()) {
            if let Some(action) = &s.advice.action {
                counts.actionable += 1;
                planned.push(crate::plan::GainEntry {
                    action: action.clone(),
                    gain: s.advice.gain,
                    gain_next_boot: s.advice.gain_next_boot,
                    host: None,
                });
            }
        }
        let (plan_gain, plan_boot) = crate::plan::total_gains(&planned);
        counts.actionable += tasks
            .iter()
            .filter(|t| t.advice.action.is_some() && t.advice.level.preselected())
            .count();

        Snapshot {
            meta: Meta {
                computer: "PC-DE-CAMILLE".into(),
                user: "Camille".into(),
                os: "Windows 11 (build 26100)".into(),
                uptime_secs: 3 * 86400 + 7 * 3600 + 42 * 60,
                is_admin: true,
                can_elevate: false,
                demo: true,
                collected_at: crate::util::now_string(),
                duration_ms: 42,
                version: env!("CARGO_PKG_VERSION").to_string(),
                notes: vec!["MODE DÉMONSTRATION : données simulées d'un PC Windows 11 typique. Tu peux tout tester (appliquer, restaurer) sans aucun effet sur une vraie machine.".into()],
            },
            memory: MemoryInfo {
                total,
                available: total - used,
                used,
                load: ((used as f64 / total as f64) * 100.0) as u32,
                commit_used: used + 2200 * MB,
                commit_limit: total + 8 * 1024 * MB,
                kernel: 680 * MB,
                cache: 1400 * MB,
                compressed: 380 * MB,
                sum_private: sum_priv,
                sum_working_set: sum_ws,
            },
            processes,
            services,
            startup,
            tasks,
            plan_gain,
            plan_gain_next_boot: plan_boot,
            counts,
        }
    }

    pub fn apply(&mut self, actions: &[Action]) -> Vec<RollbackEntry> {
        let mut out = Vec::new();
        for a in actions {
            // État recalculé à chaque action : l'ordre compte (ex. restauration
            // « remettre en auto » PUIS « redémarrer le service »).
            let snap = self.snapshot();
            // `from` / `was_enabled` à None désigne une action inverse interne
            // (restauration), jamais une action utilisateur de /api/apply.
            let (undo, res) = match a {
                Action::Service { name, start, from } => {
                    if rules::is_protected_service(name) {
                        (
                            None,
                            Err("Service vital : RamScope refuse de le modifier.".to_string()),
                        )
                    } else if let Some(s) = snap.services.iter().find(|s| s.name == *name) {
                        if from.is_some() && from != &Some(s.start) {
                            (
                                None,
                                Err(
                                    "l'état du service a changé depuis l'analyse : action refusée"
                                        .into(),
                                ),
                            )
                        } else if from == &Some(*start) {
                            (None, Err("le service est déjà dans l'état demandé".into()))
                        } else {
                            let u = Action::Service {
                                name: name.clone(),
                                start: s.start,
                                from: None,
                            };
                            // Comme sous Windows : changer le type de démarrage
                            // n'arrête PAS un service déjà lancé.
                            self.svc.insert(name.clone(), *start);
                            (Some(u), Ok(()))
                        }
                    } else {
                        (None, Err("service introuvable".into()))
                    }
                }
                Action::Startup {
                    id,
                    enabled,
                    was_enabled,
                } => {
                    if let Some(s) = snap.startup.iter().find(|s| s.id == *id) {
                        if was_enabled.is_some() && was_enabled != &Some(s.enabled) {
                            (
                                None,
                                Err("l'état du démarrage a changé depuis l'analyse : action refusée".into()),
                            )
                        } else if was_enabled == &Some(*enabled) {
                            (None, Err("l'entrée est déjà dans l'état demandé".into()))
                        } else {
                            let u = Action::Startup {
                                id: id.clone(),
                                enabled: s.enabled,
                                was_enabled: None,
                            };
                            self.startup.insert(id.clone(), *enabled);
                            (Some(u), Ok(()))
                        }
                    } else {
                        (None, Err("entrée introuvable".into()))
                    }
                }
                Action::Task {
                    path,
                    enabled,
                    was_enabled,
                } => {
                    if let Some(t) = snap.tasks.iter().find(|t| t.path == *path) {
                        if was_enabled.is_some() && was_enabled != &Some(t.enabled) {
                            (
                                None,
                                Err(
                                    "l'état de la tâche a changé depuis l'analyse : action refusée"
                                        .into(),
                                ),
                            )
                        } else if was_enabled == &Some(*enabled) {
                            (None, Err("la tâche est déjà dans l'état demandé".into()))
                        } else {
                            let u = Action::Task {
                                path: path.clone(),
                                enabled: t.enabled,
                                was_enabled: None,
                            };
                            self.tasks.insert(path.clone(), *enabled);
                            (Some(u), Ok(()))
                        }
                    } else {
                        (None, Err("tâche introuvable".into()))
                    }
                }
                Action::StopService { name } => {
                    if rules::is_protected_service(name) {
                        (
                            None,
                            Err("Service vital : RamScope refuse de l'arrêter.".to_string()),
                        )
                    } else if let Some(s) = snap.services.iter().find(|s| s.name == *name) {
                        let u = if s.running {
                            Some(Action::StartService { name: name.clone() })
                        } else {
                            None
                        };
                        self.running.insert(name.clone(), false);
                        (
                            u,
                            if s.running {
                                Ok(())
                            } else {
                                Err("service déjà arrêté".into())
                            },
                        )
                    } else {
                        (None, Err("service introuvable".into()))
                    }
                }
                Action::StartService { name } => {
                    if let Some(s) = snap.services.iter().find(|s| s.name == *name) {
                        let u = if s.running {
                            None
                        } else {
                            Some(Action::StopService { name: name.clone() })
                        };
                        if matches!(s.start, StartType::Disabled) {
                            (
                                None,
                                Err("service désactivé : impossible de le démarrer".into()),
                            )
                        } else {
                            self.running.insert(name.clone(), true);
                            (u, Ok(()))
                        }
                    } else {
                        (None, Err("service introuvable".into()))
                    }
                }
                Action::Kill {
                    pid,
                    name,
                    image_path,
                    created_at,
                } => {
                    if rules::is_protected_process(name) {
                        (
                            None,
                            Err("Processus vital : RamScope refuse de le terminer.".to_string()),
                        )
                    } else if let Some(p) = snap.processes.iter().find(|p| p.pid == *pid) {
                        if p.created_at != *created_at || !p.path.eq_ignore_ascii_case(image_path) {
                            (
                                None,
                                Err("l'identité du processus a changé depuis l'analyse : action refusée".into()),
                            )
                        } else {
                            self.killed.insert(*pid);
                            (None, Ok(()))
                        }
                    } else {
                        (None, Err("processus déjà terminé".into()))
                    }
                }
            };
            out.push(RollbackEntry {
                label: a.describe(),
                done: a.clone(),
                undo,
                ok: res.is_ok(),
                message: res.err().unwrap_or_else(|| "OK".into()),
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kill_propose(state: &DemoState) -> Action {
        state
            .snapshot()
            .processes
            .iter()
            .find_map(|p| match &p.advice.action {
                Some(Action::Kill { .. }) => Some(Action::Kill {
                    pid: p.pid,
                    name: p.name.clone(),
                    image_path: p.path.clone(),
                    created_at: p.created_at,
                }),
                _ => None,
            })
            .expect("la démo doit proposer au moins un processus à terminer")
    }

    #[test]
    fn refuse_un_kill_dont_l_identite_a_change() {
        let mut state = DemoState::default();
        let mut action = kill_propose(&state);
        if let Action::Kill { created_at, .. } = &mut action {
            *created_at += 1;
        }
        let entries = state.apply(&[action]);
        assert!(!entries[0].ok);
        assert!(
            entries[0].message.contains("identité"),
            "message : {}",
            entries[0].message
        );
    }

    #[test]
    fn accepte_un_kill_dont_l_identite_correspond() {
        let mut state = DemoState::default();
        let action = kill_propose(&state);
        let entries = state.apply(&[action]);
        assert!(entries[0].ok, "message : {}", entries[0].message);
    }

    #[test]
    fn une_restauration_interne_rejoue_un_undo_sans_etat_observe() {
        let mut state = DemoState::default();
        let action = Action::Service {
            name: "DiagTrack".into(),
            start: StartType::Disabled,
            from: Some(StartType::Auto),
        };
        let entries = state.apply(&[action]);
        assert!(entries[0].ok, "message : {}", entries[0].message);
        let undo = entries[0]
            .undo
            .clone()
            .expect("une modification de service doit avoir un inverse");
        assert!(matches!(undo, Action::Service { from: None, .. }));
        let restored = state.apply(&[undo]);
        assert!(restored[0].ok, "message : {}", restored[0].message);
    }
}
