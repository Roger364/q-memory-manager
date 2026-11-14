//! Base de connaissances RamScope : classification des services, processus,
//! entrées de démarrage et tâches planifiées de Windows 10 / 11.
//!
//! Trois principes :
//!  1. Ce qui est vital est VERROUILLÉ (refusé côté serveur, pas seulement masqué).
//!  2. Chaque conseil est justifié en français, avec le gain estimé.
//!  3. En cas de doute -> on ne conseille rien (niveau « non répertorié »).

use crate::model::{Action, Advice, Gain, GainScope, Level, StartType};

pub fn lc(s: &str) -> String {
    crate::plan::normalize(s)
}

// ---------------------------------------------------------------------------
// Schéma des règles : provenance et mode de correspondance
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchMode {
    /// Nom court de service, ou nom affiché exact.
    ServiceName,
    /// Nom d'exécutable exact (processus).
    ExecutableName,
    /// Nom d'entrée, nom d'exécutable ou composant de chemin exact (démarrage).
    StartupExact,
    /// Nom de tâche ou suffixe de chemin exact, à une frontière de composant.
    TaskPath,
}

#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub key: &'static str,
    pub level: Level,
    /// Démarrage conseillé pour un service ; `None` sinon.
    pub target: Option<StartType>,
    pub mode: MatchMode,
    /// Référence de la famille de règles, reprise pour chaque entrée.
    #[allow(dead_code)]
    pub source: &'static str,
    #[allow(dead_code)]
    pub reviewed: &'static str,
    pub reason: &'static str,
}

// La provenance est portée par famille, pas par une URL vérifiée règle par
// règle : les justifications viennent de la table RamScope héritée. La date de
// revue reste inconnue tant qu'une revue de fond n'a pas été faite.
const REVIEWED: &str = "non datée";
const SOURCE_SERVICES: &str =
    "Table RamScope héritée ; référence de famille : Microsoft Learn, services Windows 11";
const SOURCE_PROCESSES: &str =
    "Table RamScope héritée ; référence de famille : documentation Microsoft et des éditeurs";
const SOURCE_STARTUP: &str =
    "Table RamScope héritée ; référence de famille : Gestionnaire des tâches Windows 11";
const SOURCE_TASKS: &str =
    "Table RamScope héritée ; référence de famille : Planificateur de tâches Windows 11";

macro_rules! service_rules {
    ($source:expr, $(($key:literal, $level:expr, $target:expr, $reason:literal)),* $(,)?) => {
        &[$(Rule {
            key: $key,
            level: $level,
            target: $target,
            mode: MatchMode::ServiceName,
            source: $source,
            reviewed: REVIEWED,
            reason: $reason,
        }),*]
    };
}

macro_rules! process_rules {
    ($source:expr, $(($key:literal, $level:expr, $reason:literal)),* $(,)?) => {
        &[$(Rule {
            key: $key,
            level: $level,
            target: None,
            mode: MatchMode::ExecutableName,
            source: $source,
            reviewed: REVIEWED,
            reason: $reason,
        }),*]
    };
}

macro_rules! startup_rules {
    ($source:expr, $(($key:literal, $level:expr, $reason:literal)),* $(,)?) => {
        &[$(Rule {
            key: $key,
            level: $level,
            target: None,
            mode: MatchMode::StartupExact,
            source: $source,
            reviewed: REVIEWED,
            reason: $reason,
        }),*]
    };
}

macro_rules! task_rules {
    ($source:expr, $(($key:literal, $level:expr, $reason:literal)),* $(,)?) => {
        &[$(Rule {
            key: $key,
            level: $level,
            target: None,
            mode: MatchMode::TaskPath,
            source: $source,
            reviewed: REVIEWED,
            reason: $reason,
        }),*]
    };
}

// ---------------------------------------------------------------------------
// SERVICES
// ---------------------------------------------------------------------------

pub const SERVICES: &[Rule] = service_rules!(
    SOURCE_SERVICES,
    // ---- Vitaux : verrouillés ----
    ("rpcss", Level::Critical, None, "Appels de procédure distante : le socle de Windows. Sans lui, plus rien ne démarre."),
    ("rpceptmapper", Level::Critical, None, "Table de correspondance des points de terminaison RPC. Vital."),
    ("dcomlaunch", Level::Critical, None, "Lancement des processus serveur DCOM. Vital."),
    ("eventlog", Level::Critical, None, "Journal d'événements Windows. Vital, et indispensable au diagnostic."),
    ("plugplay", Level::Critical, None, "Détection du matériel (Plug-and-Play). Vital."),
    ("power", Level::Critical, None, "Gestion de l'alimentation et des veille/hibernation. Vital."),
    ("profsrv", Level::Critical, None, "Chargement du profil utilisateur. Sans lui, session impossible."),
    ("samss", Level::Critical, None, "Gestionnaire de comptes de sécurité. Vital."),
    ("lsm", Level::Critical, None, "Gestionnaire de session locale. Vital."),
    ("schedule", Level::Critical, None, "Planificateur de tâches : Windows lui-même l'utilise en permanence."),
    ("gpsvc", Level::Critical, None, "Client de stratégie de groupe. Vital, y compris hors domaine."),
    ("winmgmt", Level::Critical, None, "WMI : utilisé par Windows, les pilotes et les outils d'administration."),
    ("cryptsvc", Level::Critical, None, "Services de chiffrement et vérification des signatures. Requis par Windows Update."),
    ("bfe", Level::Critical, None, "Moteur de filtrage de base : socle du pare-feu et de la sécurité réseau."),
    ("mpssvc", Level::Critical, None, "Pare-feu Windows Defender. À ne jamais désactiver."),
    ("windefend", Level::Critical, None, "Antivirus Microsoft Defender. Le désactiver laisse le PC sans protection."),
    ("wdnissvc", Level::Critical, None, "Inspection réseau de Defender. Sécurité."),
    ("securityhealthservice", Level::Critical, None, "Sécurité Windows : état de l'antivirus et du pare-feu."),
    ("wscsvc", Level::Critical, None, "Centre de sécurité : signale les protections désactivées."),
    ("dhcp", Level::Critical, None, "Client DHCP : obtient l'adresse IP. Le désactiver coupe le réseau."),
    ("dnscache", Level::Critical, None, "Cache DNS : résolution des noms de domaines. Nécessaire à la navigation."),
    ("nsi", Level::Critical, None, "Interface de magasin réseau. Requis par le réseau."),
    ("netprofm", Level::Critical, None, "Connaissance des emplacements réseau (privé/public). Requis par le pare-feu."),
    ("nlasvc", Level::Critical, None, "Détection du type de réseau. Requis par le pare-feu."),
    ("wlansvc", Level::Critical, None, "Configuration Wi-Fi automatique. Indispensable si tu es en Wi-Fi."),
    ("wcmsvc", Level::Critical, None, "Gestionnaire de connexions Windows. Requis par le réseau moderne."),
    ("audiosrv", Level::Critical, None, "Audio Windows : plus de son sans lui."),
    ("audioendpointbuilder", Level::Critical, None, "Points de terminaison audio (haut-parleurs, micro). Vital pour le son."),
    ("deviceinstall", Level::Critical, None, "Installation des périphériques. Vital."),
    ("dsmsvc", Level::Critical, None, "Gestionnaire d'installation des périphériques."),
    ("staterepository", Level::Critical, None, "Base des applications Store/UWP. Le menu Démarrer en dépend."),
    ("tokenbroker", Level::Critical, None, "Courtier de jetons d'authentification des applications."),
    ("usermanager", Level::Critical, None, "Gestionnaire des utilisateurs : sessions et comptes."),
    ("systemeventsbroker", Level::Critical, None, "Courtier d'événements système pour les applications."),
    ("timebrokersvc", Level::Critical, None, "Courtier de temps pour les tâches en arrière-plan des applications."),
    ("coremessagingregistrar", Level::Critical, None, "Messagerie interne du shell Windows."),
    ("brokerinfrastructure", Level::Critical, None, "Infrastructure des tâches en arrière-plan. Vital."),
    ("appinfo", Level::Critical, None, "Élévation de privilèges (les invites UAC « Exécuter en admin »)."),
    ("keyiso", Level::Critical, None, "Isolation des clés cryptographiques (CNG). Sécurité."),
    ("trustedinstaller", Level::Critical, None, "Installation des mises à jour et composants Windows."),
    ("fontcache", Level::Critical, None, "Cache des polices : sans lui, l'affichage rame."),
    ("textinputmanagementservice", Level::Critical, None, "Saisie de texte, clavier tactile et presse-papiers."),
    ("appxsvc", Level::Critical, None, "Déploiement des applications Store. Nécessaire aux mises à jour des apps."),
    ("clipsvc", Level::Critical, None, "Licences des applications Store."),
    ("eventsystem", Level::Critical, None, "Système d'événements COM+ : utilisé par de nombreux composants."),
    ("sens", Level::Critical, None, "Notification d'événements système (réseau, session)."),
    ("storsvc", Level::Critical, None, "Service de stockage : accès aux disques et applications."),
    ("lanmanworkstation", Level::Critical, None, "Accès aux partages réseau et aux imprimantes partagées."),
    ("wpnservice", Level::Keep, None, "Notifications Windows et des applications. Le désactiver supprime les notifications."),
    ("sgrmbroker", Level::System, None, "Surveillance de l'intégrité du système. Léger, à laisser."),
    ("camsvc", Level::System, None, "Gestionnaire d'accès aux fonctionnalités des applications."),
    ("licensemanager", Level::System, None, "Licences des applications Store."),
    ("cdpsvc", Level::Optional, Some(StartType::Manual), "Plateforme des appareils connectés (Mobile connecté, Partage rapide). Inutile si tu n'utilises pas ces fonctions."),
    ("cdpusersvc", Level::Optional, Some(StartType::Manual), "Volet utilisateur de la plateforme des appareils connectés."),

    // ---- Télémétrie / superflu ----
    ("diagtrack", Level::Trim, Some(StartType::Disabled), "Télémétrie Microsoft (« Expériences des utilisateurs connectés »). Envoie des données en continu, aucun impact fonctionnel si désactivé."),
    ("dmwappushservice", Level::Junk, Some(StartType::Disabled), "Routage de messages WAP Push : composant de collecte de télémétrie, inutile sur un PC personnel."),
    ("diagnosticshub", Level::Junk, Some(StartType::Disabled), "Collecte d'événements de diagnostic (ETW) pour la télémétrie."),
    ("retaildemo", Level::Junk, Some(StartType::Disabled), "Mode démonstration magasin. Strictement inutile chez toi."),
    ("fax", Level::Junk, Some(StartType::Disabled), "Service de fax. Personne n'utilise de fax modem aujourd'hui."),
    ("wmpnetworksvc", Level::Junk, Some(StartType::Disabled), "Partage réseau de Windows Media Player (DLNA). Inutile si tu ne partages pas ta bibliothèque."),
    ("remoteregistry", Level::Trim, Some(StartType::Disabled), "Modification du registre à distance. Inutile et surface d'attaque : à désactiver."),
    ("wisvc", Level::Trim, Some(StartType::Disabled), "Programme Windows Insider. Inutile si tu ne testes pas les versions Insider."),
    ("wersvc", Level::Optional, Some(StartType::Manual), "Rapport d'erreurs Windows : envoie les rapports de plantage. Peut rester en manuel."),
    ("pcasvc", Level::Optional, Some(StartType::Manual), "Assistant de compatibilité des programmes. Rarement utile sur un système à jour."),
    ("trkwks", Level::Optional, Some(StartType::Manual), "Suivi des liens distribués (raccourcis vers des fichiers déplacés sur le réseau). Très rarement utile."),
    ("mapsbroker", Level::Optional, Some(StartType::Manual), "Gestionnaire des cartes téléchargées (app Cartes hors ligne). Inutile si tu n'utilises pas Cartes."),
    ("lfsvc", Level::Optional, Some(StartType::Manual), "Géolocalisation. À passer en manuel si tu ne veux pas que Windows te localise."),
    ("dusmsvc", Level::Optional, Some(StartType::Manual), "Suivi de la consommation de données. Utile seulement en connexion limitée."),
    ("walletservice", Level::Junk, Some(StartType::Disabled), "Portefeuille Microsoft (paiement mobile abandonné). Inutilisé."),
    ("phonesvc", Level::Optional, Some(StartType::Manual), "Service téléphone (Mobile connecté / appels). Inutile si tu ne relies pas ton téléphone."),
    ("semsvrmgr", Level::Optional, Some(StartType::Manual), "Gestion des paiements NFC / élément sécurisé. Inutile sans paiement NFC."),
    ("semgrsvc", Level::Optional, Some(StartType::Manual), "Gestion des paiements NFC / élément sécurisé. Inutile sans paiement NFC."),
    ("wpcmonserv", Level::Optional, Some(StartType::Manual), "Contrôle parental. Inutile si aucun compte enfant n'est configuré."),
    ("wpcmonsvc", Level::Optional, Some(StartType::Manual), "Contrôle parental. Inutile si aucun compte enfant n'est configuré."),
    ("sharedaccess", Level::Optional, Some(StartType::Disabled), "Partage de connexion Internet (hotspot). À désactiver si tu ne partages pas ta connexion."),
    ("xblauthmanager", Level::Optional, Some(StartType::Manual), "Authentification Xbox Live. Inutile si tu ne joues pas aux jeux Xbox / Game Pass."),
    ("xblgamesave", Level::Optional, Some(StartType::Manual), "Sauvegardes de jeux Xbox Live. Inutile hors jeux Xbox."),
    ("xboxgipsvc", Level::Optional, Some(StartType::Manual), "Gestion des accessoires Xbox (manettes). Manuel suffit : démarre à la demande."),
    ("xboxnetapisvc", Level::Optional, Some(StartType::Manual), "API réseau Xbox Live (multijoueur). Inutile hors jeux Xbox."),
    ("mixedrealityopenxrservice", Level::Optional, Some(StartType::Manual), "Réalité mixte / casques VR. Inutile sans casque."),
    ("mixedrealityopenxrsvc", Level::Optional, Some(StartType::Manual), "Réalité mixte / casques VR. Inutile sans casque."),
    ("tabletinputservice", Level::Optional, Some(StartType::Manual), "Clavier tactile et volet d'écriture manuscrite. Inutile sans écran tactile ni stylet."),
    ("wbiosrvc", Level::Optional, Some(StartType::Manual), "Biométrie (empreinte digitale, Windows Hello visage). Inutile si tu te connectes par mot de passe / PIN."),
    ("scardsvr", Level::Optional, Some(StartType::Manual), "Cartes à puce. Inutile sans lecteur de carte à puce."),
    ("scdeviceenum", Level::Optional, Some(StartType::Manual), "Énumération des lecteurs de cartes à puce. Inutile sans lecteur."),
    ("certpropsvc", Level::Optional, Some(StartType::Manual), "Propagation des certificats depuis une carte à puce. Inutile sans carte à puce."),
    ("bthserv", Level::Optional, Some(StartType::Manual), "Support Bluetooth. À garder si tu utilises souris, casque ou manette Bluetooth."),
    ("btagservice", Level::Optional, Some(StartType::Manual), "Passerelle d'association Bluetooth. Lié au Bluetooth."),
    ("bluetoothusersvc", Level::Optional, Some(StartType::Manual), "Support Bluetooth par session utilisateur."),
    ("spooler", Level::Optional, Some(StartType::Manual), "Spouleur d'impression. À passer en manuel UNIQUEMENT si tu n'imprimes jamais (y compris en PDF)."),
    ("printnotify", Level::Optional, Some(StartType::Manual), "Notifications d'imprimante. Inutile sans imprimante."),
    ("printwfservice", Level::Optional, Some(StartType::Manual), "Flux de travail d'impression. Inutile sans imprimante."),
    ("wsearch", Level::Optional, Some(StartType::Manual), "Indexation de Windows Search : gros consommateur de RAM, CPU et disque. En manuel, la recherche de fichiers devient lente (Outlook inclus)."),
    ("sysmain", Level::Optional, Some(StartType::Manual), "SysMain / Superfetch : précharge les applications en RAM. Utile sur disque dur, souvent superflu sur SSD NVMe avec 16 Go+."),
    ("dosvc", Level::Optional, Some(StartType::Manual), "Optimisation de la distribution : partage les mises à jour en pair-à-pair. Manuel = plus de partage, mises à jour toujours OK."),
    ("wcncsvc", Level::Optional, Some(StartType::Manual), "Configuration Windows Connect Now (WPS). Rarement utilisé, et le WPS est peu sûr."),
    ("hidserv", Level::Optional, Some(StartType::Manual), "Accès aux boutons matériels (touches multimédia, stylet). Garde-le si tes touches spéciales servent."),
    ("shellhwdetection", Level::Optional, Some(StartType::Manual), "Détection d'événements matériel (autorun des clés USB / DVD). Manuel suffit."),
    ("frameserver", Level::Optional, Some(StartType::Manual), "Serveur de trame caméra (partage de la webcam entre apps). Manuel suffit."),
    ("smsrouter", Level::Optional, Some(StartType::Manual), "Routage de SMS (Mobile connecté). Inutile sans liaison téléphone."),
    ("ajrouter", Level::Optional, Some(StartType::Manual), "Routage AllJoyn (objets connectés IoT). Quasi jamais utilisé."),
    ("icssvc", Level::Optional, Some(StartType::Manual), "Partage de connexion mobile (hotspot Wi-Fi). Inutile si tu ne fais pas de partage."),
    ("alg", Level::Optional, Some(StartType::Manual), "Passerelle de couche application (partage de connexion historique). Inutilisé."),
    ("irmon", Level::Junk, Some(StartType::Disabled), "Moniteur infrarouge (IrDA). Matériel disparu depuis 15 ans."),
    ("snmptrap", Level::Optional, Some(StartType::Manual), "Réception des traps SNMP (supervision réseau d'entreprise). Inutile chez soi."),
    ("tapisrv", Level::Optional, Some(StartType::Manual), "Téléphonie TAPI (modems, fax logiciels). Rarement utilisé."),
    ("remoteaccess", Level::Optional, Some(StartType::Disabled), "Routage et accès distant (VPN serveur / RRAS). Inutile sur un poste client."),
    ("sessionenv", Level::Optional, Some(StartType::Manual), "Environnement de session Bureau à distance. Inutile si tu n'utilises pas le RDP."),
    ("termservice", Level::Optional, Some(StartType::Manual), "Bureau à distance (hôte). À désactiver si personne ne prend le contrôle de ce PC à distance."),
    ("umrdpservice", Level::Optional, Some(StartType::Manual), "Redirecteur de périphériques Bureau à distance. Lié au RDP."),
    ("rdpvideominport", Level::Optional, Some(StartType::Manual), "Pilote vidéo Bureau à distance. Lié au RDP."),
    ("lanmanserver", Level::Optional, Some(StartType::Manual), "Partage de fichiers et d'imprimantes DEPUIS ce PC. À désactiver si tu ne partages rien."),
    ("fdrespub", Level::Optional, Some(StartType::Manual), "Publication de découverte de ressources (partage réseau). Inutile sans partage."),
    ("fdphost", Level::Optional, Some(StartType::Manual), "Hôte de découverte de fonctions (découverte réseau). Inutile sans partage ni imprimante réseau."),
    ("ssdpsrv", Level::Optional, Some(StartType::Manual), "Découverte SSDP/UPnP (TV, box, imprimantes réseau). Manuel suffit."),
    ("upnphost", Level::Optional, Some(StartType::Manual), "Hôte UPnP (partage d'appareils). Inutile dans la plupart des cas."),
    ("p2psvc", Level::Optional, Some(StartType::Manual), "Réseau homologue (groupe résidentiel, supprimé de Windows 10 1803+)."),
    ("p2pimsvc", Level::Optional, Some(StartType::Manual), "Gestionnaire d'identités homologue. Legacy."),
    ("pnrpsvc", Level::Optional, Some(StartType::Manual), "Protocole de résolution des noms homologues. Legacy."),
    ("pnrpautoreg", Level::Optional, Some(StartType::Manual), "Enregistrement automatique PNRP. Legacy."),
    ("workfolderssvc", Level::Optional, Some(StartType::Manual), "Dossiers de travail (synchronisation entreprise). Inutile hors entreprise."),
    ("cscservice", Level::Optional, Some(StartType::Manual), "Fichiers hors connexion (cache de partages réseau). Inutile hors entreprise."),
    ("vmicheartbeat", Level::Optional, Some(StartType::Manual), "Intégration Hyper-V (machine virtuelle). Inutile si ce PC n'est pas une VM Hyper-V."),
    ("vmickvpexchange", Level::Optional, Some(StartType::Manual), "Intégration Hyper-V. Inutile hors VM."),
    ("vmicshutdown", Level::Optional, Some(StartType::Manual), "Intégration Hyper-V. Inutile hors VM."),
    ("vmictimesync", Level::Optional, Some(StartType::Manual), "Intégration Hyper-V. Inutile hors VM."),
    ("vmicvss", Level::Optional, Some(StartType::Manual), "Intégration Hyper-V. Inutile hors VM."),
    ("vmicrdv", Level::Optional, Some(StartType::Manual), "Intégration Hyper-V. Inutile hors VM."),
    ("vmicguestinterface", Level::Optional, Some(StartType::Manual), "Intégration Hyper-V. Inutile hors VM."),
    ("hvhost", Level::Optional, Some(StartType::Manual), "Hôte de service Hyper-V. Attention : requis par WSL2, Sandbox, Docker et l'isolation du noyau."),
    ("vmcompute", Level::Optional, Some(StartType::Manual), "Calcul Hyper-V (conteneurs, WSL2, Sandbox). À garder si tu utilises WSL ou Docker."),
    ("wslservice", Level::Optional, Some(StartType::Manual), "Sous-système Windows pour Linux. À garder si tu utilises WSL."),

    // ---- Services Windows à conserver ----
    ("wuauserv", Level::Keep, None, "Windows Update. Déjà en manuel/déclenchement : ne pas désactiver, sinon plus de correctifs de sécurité."),
    ("bits", Level::Keep, None, "Transferts intelligents en arrière-plan (mises à jour). Déjà en manuel par défaut."),
    ("waasmedicsvc", Level::Keep, None, "Réparation de Windows Update. Protégé par le système, ne pas forcer."),
    ("usosvc", Level::Keep, None, "Orchestration des mises à jour. À laisser."),
    ("themes", Level::Keep, None, "Thèmes et apparence de Windows."),
    ("w32time", Level::Keep, None, "Synchronisation de l'heure. Une horloge fausse casse HTTPS et l'authentification."),
    ("netman", Level::Keep, None, "Gestion des connexions réseau."),
    ("iphlpsvc", Level::Keep, None, "Assistance IP (IPv6, transition, IP-HTTPS). À laisser."),
    ("policyagent", Level::System, None, "Agent de stratégie IPsec. Léger."),
    ("winhttpautoproxysvc", Level::Keep, None, "Découverte automatique de proxy (WPAD). Manuel par défaut."),
    ("lmhosts", Level::Optional, Some(StartType::Manual), "Assistance NetBIOS sur TCP/IP (vieux réseaux). Manuel suffit."),
    ("msiserver", Level::Keep, None, "Installateur Windows (MSI). Déjà en manuel : indispensable pour installer/désinstaller."),
    ("vss", Level::Keep, None, "Clichés instantanés (sauvegardes, restauration). Manuel par défaut."),
    ("swprv", Level::Keep, None, "Fournisseur de clichés instantanés logiciels. Manuel par défaut."),
    ("defragsvc", Level::Keep, None, "Optimisation des lecteurs (TRIM des SSD). À laisser : c'est ce qui préserve ton SSD."),
    ("wbengine", Level::Keep, None, "Moteur de sauvegarde Windows. Manuel par défaut."),
    ("sdrsvc", Level::Keep, None, "Sauvegarde de documents Windows. Manuel par défaut."),
    ("wdiservicehost", Level::System, None, "Hôte du service de diagnostics. Léger, à laisser."),
    ("wdisystemhost", Level::System, None, "Hôte système de diagnostics. Léger, à laisser."),
    ("dpssvc", Level::System, None, "Service de stratégie de diagnostic. Consomme peu."),
    ("wdbsniffer", Level::System, None, "Composant de diagnostic Windows."),
    ("ndu", Level::System, None, "Surveillance de l'utilisation réseau (pilote). À laisser."),
    ("dam", Level::System, None, "Gestion des applications en veille connectée. À laisser."),
    ("efssvc", Level::Keep, None, "Système de fichiers chiffré (EFS). Manuel par défaut."),
    ("bdesvc", Level::Keep, None, "Chiffrement BitLocker. À garder si ton disque est chiffré."),
    ("tpmprovisioning", Level::Keep, None, "Provisionnement du TPM (sécurité matérielle)."),
    ("ngcsvc", Level::Keep, None, "Microsoft Passport : PIN et Windows Hello."),
    ("ngcctorsvc", Level::Keep, None, "Conteneur de clés Windows Hello."),
    ("vaultsvc", Level::Keep, None, "Gestionnaire d'informations d'identification (mots de passe enregistrés)."),
    ("dsregsvc", Level::System, None, "Enregistrement de l'appareil (compte Microsoft / Azure AD)."),
    ("wlidsvc", Level::Keep, None, "Assistant Connexion Microsoft (compte Microsoft, Store, Office)."),
    ("netlogon", Level::System, None, "Authentification de domaine. Inutile hors domaine, mais déjà en manuel."),
    ("kdcsvc", Level::System, None, "Contrôleur de domaine Kerberos. Présent seulement sur serveur."),
    ("dfsr", Level::System, None, "Réplication DFS (serveur)."),
    ("ntfrs", Level::System, None, "Réplication de fichiers (serveur, legacy)."),
    ("searchindexer", Level::Optional, Some(StartType::Manual), "Indexeur de recherche : voir Windows Search (WSearch)."),
    ("wecsvc", Level::Optional, Some(StartType::Manual), "Collecte d'événements distante (supervision). Inutile chez soi."),
    ("w3logsvc", Level::System, None, "Journalisation IIS. Présent seulement si IIS est installé."),
    ("iisadmin", Level::Optional, Some(StartType::Manual), "Administration IIS (serveur web local). Inutile si tu ne développes pas sous IIS."),
    ("w3svc", Level::Optional, Some(StartType::Manual), "Serveur web IIS. À désactiver si tu n'héberges pas de site local."),
    ("mssqlserver", Level::Optional, Some(StartType::Manual), "SQL Server : peut consommer plusieurs Go de RAM. À passer en manuel si tu ne t'en sers pas tous les jours."),
    ("mysql", Level::Optional, Some(StartType::Manual), "Serveur MySQL : passe en manuel si tu ne développes pas en permanence."),
    ("postgresql", Level::Optional, Some(StartType::Manual), "Serveur PostgreSQL : passe en manuel si usage occasionnel."),
    ("docker", Level::Optional, Some(StartType::Manual), "Docker Desktop : très gourmand en RAM. Manuel + lancement à la demande est idéal."),
    ("com.docker.service", Level::Optional, Some(StartType::Manual), "Service Docker : gourmand. Manuel suffit, Docker le relancera au besoin."),
    ("vmms", Level::Optional, Some(StartType::Manual), "Gestion des machines virtuelles Hyper-V. Inutile si tu n'utilises pas Hyper-V."),

    // ---- Tiers fréquents ----
    ("gupdate", Level::Trim, Some(StartType::Manual), "Mise à jour automatique de Google. Chrome se met à jour tout seul quand tu l'ouvres : manuel suffit."),
    ("gupdatem", Level::Trim, Some(StartType::Manual), "Mise à jour Google (machine). Manuel suffit, Chrome se met à jour à son lancement."),
    ("googlechromeelevationservice", Level::Keep, None, "Élévation pour les mises à jour de Chrome. Déjà en manuel, à laisser."),
    ("edgeupdate", Level::Trim, Some(StartType::Manual), "Mise à jour d'Edge. Edge se met à jour à son lancement : manuel suffit."),
    ("edgeupdatem", Level::Trim, Some(StartType::Manual), "Mise à jour d'Edge (machine). Manuel suffit."),
    ("microsoftedgeelevationservice", Level::Keep, None, "Élévation pour les mises à jour d'Edge. Déjà en manuel."),
    ("mozillamaintenance", Level::Keep, None, "Maintenance Firefox (mises à jour silencieuses). Déjà en manuel, très léger."),
    ("adobearmservice", Level::Trim, Some(StartType::Manual), "Mise à jour Adobe Acrobat. Peut rester en manuel : Acrobat vérifie au lancement."),
    ("agsservice", Level::Keep, None, "Adobe Genuine Software Integrity : vérification de licence. À laisser si tu as des produits Adobe sous licence."),
    ("agsm", Level::Keep, None, "Adobe Genuine Monitor. Léger, lié aux licences Adobe."),
    ("adobeupdateservice", Level::Trim, Some(StartType::Manual), "Mise à jour Adobe. Manuel suffit."),
    ("bonjour service", Level::Trim, Some(StartType::Disabled), "Service Bonjour (Apple mDNS), installé avec iTunes. Inutile sauf si tu utilises AirPlay/partage iTunes."),
    ("applemobiledeviceservice", Level::Optional, Some(StartType::Manual), "Synchronisation iPhone/iPad via iTunes. Manuel si tu synchronises rarement."),
    ("ipod service", Level::Optional, Some(StartType::Manual), "Détection d'iPod/iPhone (iTunes). Manuel suffit."),
    ("steam client service", Level::Keep, None, "Service Steam (installation de jeux, correctifs). Déjà en manuel."),
    ("steamservice", Level::Keep, None, "Service Steam. Déjà en manuel, ne se lance qu'au besoin."),
    ("epiconlineservices", Level::Optional, Some(StartType::Manual), "Epic Online Services (jeux Epic). Manuel suffit."),
    ("easyanticheat", Level::Keep, None, "Anti-triche de tes jeux : à laisser, sinon les jeux refusent de démarrer."),
    ("battleye service", Level::Keep, None, "Anti-triche BattlEye : à laisser si tu joues en ligne."),
    ("vgc", Level::Optional, Some(StartType::Manual), "Riot Vanguard (anti-triche Valorant). Obligatoire pour jouer à Valorant, sinon manuel."),
    ("vgk", Level::Optional, Some(StartType::Manual), "Pilote Riot Vanguard. Lié à Valorant."),
    ("nvtelemetrycontainer", Level::Trim, Some(StartType::Disabled), "Télémétrie NVIDIA. Aucun impact sur les performances de jeu si désactivé."),
    ("nvdisplay.containerlocalsystem", Level::Keep, None, "Conteneur NVIDIA Display : pilote graphique et panneau de configuration. À garder."),
    ("nvdisplay.containerlocalnetwork", Level::Keep, None, "Conteneur NVIDIA (réseau). Lié au pilote graphique."),
    ("nvidia localsystem container", Level::Keep, None, "Conteneur NVIDIA (ancien pilote). À garder."),
    ("nvidia networkservice", Level::Optional, Some(StartType::Manual), "Services réseau NVIDIA (Shield, GameStream). Inutile si tu ne streames pas tes jeux."),
    ("fnslicservice", Level::Keep, None, "NVIDIA FrameView / licence. Lié au pilote."),
    ("igfxculservice", Level::Optional, Some(StartType::Manual), "Intel Graphics Command Center. Le panneau graphique fonctionne sans service permanent."),
    ("igccservice", Level::Optional, Some(StartType::Manual), "Intel Graphics Command Center. Manuel suffit."),
    ("cphs", Level::Optional, Some(StartType::Manual), "Intel Content Protection HECI (HDCP / lecture protégée). Manuel généralement sans effet."),
    ("cplspcon", Level::Optional, Some(StartType::Manual), "Intel Content Protection HDCP. Manuel généralement sans effet."),
    ("esrv_svc_queencreek", Level::Trim, Some(StartType::Disabled), "Télémétrie Intel (Software Asset Manager / Computing Improvement Program). Sans impact fonctionnel."),
    ("intel(r) sur qc software asset manager", Level::Trim, Some(StartType::Disabled), "Collecte de données Intel « Computing Improvement Program ». Superflu."),
    ("surqcsoftwareassetmanager", Level::Trim, Some(StartType::Disabled), "Collecte de données Intel. Superflu."),
    ("jhi_service", Level::Optional, Some(StartType::Manual), "Intel Dynamic Application Loader (DAL/TEE). Rarement utilisé hors entreprise."),
    ("lms", Level::Optional, Some(StartType::Manual), "Intel Management Engine (AMT/vPro, administration à distance d'entreprise). Inutile chez soi."),
    ("amd external events utility", Level::Keep, None, "Composant pilote AMD (raccourcis, événements). À garder."),
    ("amdryzenmasterdriversvc", Level::Optional, Some(StartType::Manual), "AMD Ryzen Master (overclocking). Manuel si tu n'overclockes pas."),
    ("teamviewer", Level::Optional, Some(StartType::Manual), "TeamViewer en accès permanent. Si tu ne t'en sers qu'occasionnellement, lance-le à la main."),
    ("anydesk", Level::Optional, Some(StartType::Manual), "AnyDesk en écoute permanente. Manuel si usage occasionnel (et plus sûr)."),
    ("vncserver", Level::Optional, Some(StartType::Manual), "Serveur VNC en écoute permanente. Manuel si usage occasionnel."),
    ("ddvdatacollector", Level::Trim, Some(StartType::Disabled), "Dell SupportAssist : collecte de données. Superflu, SupportAssist fonctionne à la demande."),
    ("ddvrulesprocessor", Level::Trim, Some(StartType::Disabled), "Dell SupportAssist : analyse de télémétrie. Superflu."),
    ("supportassistagent", Level::Optional, Some(StartType::Manual), "Dell SupportAssist. Lance-le à la main quand tu veux vérifier tes pilotes."),
    ("dellclientmanagementservice", Level::Optional, Some(StartType::Manual), "Gestion Dell (entreprise). Inutile sur un PC personnel."),
    ("hpanalyticservice", Level::Trim, Some(StartType::Disabled), "Télémétrie HP Analytics. Superflu."),
    ("hpsysinfocap", Level::Trim, Some(StartType::Disabled), "Collecte d'informations système HP. Superflu."),
    ("hpnetworkhsaservice", Level::Optional, Some(StartType::Manual), "HP Network HSA (assistance réseau HP). Rarement nécessaire."),
    ("hpjumpstartbridge", Level::Trim, Some(StartType::Disabled), "HP JumpStart : promotion d'applications préinstallées. Superflu."),
    ("hpcommrecovery", Level::Optional, Some(StartType::Manual), "Récupération HP. Manuel suffit."),
    ("lenovovantageservice", Level::Optional, Some(StartType::Manual), "Lenovo Vantage. Pratique pour les pilotes, mais peut être lancé à la demande."),
    ("imcontrollerservice", Level::Optional, Some(StartType::Manual), "Lenovo System Interface Foundation. Manuel si tu n'utilises pas Vantage."),
    ("lennovo_modernimcontroller", Level::Optional, Some(StartType::Manual), "Lenovo Modern IM Controller. Manuel suffit."),
    ("asusoptimization", Level::Optional, Some(StartType::Manual), "ASUS Optimization (Armoury Crate). Manuel si tu n'utilises pas les profils RGB/ventilateurs."),
    ("asussoftwaremanager", Level::Trim, Some(StartType::Manual), "ASUS Software Manager (mises à jour). Manuel suffit."),
    ("asus-systemdiagnosis", Level::Optional, Some(StartType::Manual), "Diagnostic ASUS. Manuel suffit."),
    ("rog live service", Level::Optional, Some(StartType::Manual), "ASUS ROG Live Service. Manuel si tu n'utilises pas Armoury Crate."),
    ("razer synapse service", Level::Optional, Some(StartType::Manual), "Razer Synapse (profils, RGB). Manuel si tes profils sont enregistrés dans le périphérique."),
    ("razer game manager", Level::Optional, Some(StartType::Manual), "Razer Game Manager. Superflu pour la plupart des usages."),
    ("razer chroma sdk server", Level::Optional, Some(StartType::Manual), "Razer Chroma (éclairage synchronisé). Manuel si tu n'utilises pas les effets."),
    ("corsair service", Level::Optional, Some(StartType::Manual), "Corsair iCUE. Très gourmand : manuel si tes profils sont stockés dans le matériel."),
    ("corsair llc", Level::Optional, Some(StartType::Manual), "Corsair iCUE. Manuel si usage occasionnel."),
    ("logitech registry service", Level::Optional, Some(StartType::Manual), "Logitech (G HUB / Options). Manuel si tes réglages sont enregistrés sur le périphérique."),
    ("lghub_updater", Level::Trim, Some(StartType::Manual), "Mise à jour Logitech G HUB. Manuel suffit."),
    ("nahimicservice", Level::Optional, Some(StartType::Manual), "Nahimic (effets audio MSI/Asus). Peut être désactivé si tu n'utilises pas les effets audio."),
    ("realtek audio universal service", Level::Keep, None, "Service audio Realtek : à garder pour la détection de casque et le panneau audio."),
    ("rtkauduservice64", Level::Keep, None, "Audio Realtek. À garder."),
    ("esif upper layer service", Level::Keep, None, "Intel Dynamic Tuning (gestion thermique). À garder sur portable : gère chauffe et autonomie."),
    ("dptf", Level::Keep, None, "Intel Dynamic Platform and Thermal Framework. À garder sur portable."),
    ("wifisvc", Level::Keep, None, "Service Wi-Fi constructeur. À garder."),
    ("ccleaner", Level::Trim, Some(StartType::Disabled), "Service CCleaner (surveillance). Le nettoyage à la demande suffit largement."),
    ("mcafee", Level::Optional, Some(StartType::Manual), "Composant McAfee préinstallé. Si tu utilises Defender, désinstalle McAfee plutôt que de le passer en manuel."),
    ("mfemms", Level::Optional, Some(StartType::Manual), "McAfee. Si tu utilises Defender, désinstalle complètement McAfee."),
    ("norton", Level::Optional, Some(StartType::Manual), "Composant Norton. Un seul antivirus à la fois : garde celui que tu veux vraiment."),
    ("avast", Level::Optional, Some(StartType::Manual), "Composant Avast. Un seul antivirus à la fois."),
    ("avg", Level::Optional, Some(StartType::Manual), "Composant AVG. Un seul antivirus à la fois."),
    ("expressvpn", Level::Optional, Some(StartType::Manual), "Service VPN. À garder si tu veux le VPN actif en permanence (kill switch)."),
    ("nordvpn", Level::Optional, Some(StartType::Manual), "Service VPN. À garder si tu veux la protection permanente."),
    ("protonvpn", Level::Optional, Some(StartType::Manual), "Service VPN. À garder si tu veux la protection permanente."),
    ("windscribe", Level::Optional, Some(StartType::Manual), "Service VPN. À garder si tu veux la protection permanente."),
    ("syncthing", Level::Optional, Some(StartType::Manual), "Synchronisation Syncthing. Manuel si tu ne synchronises pas en continu."),
    ("plexmediaserver", Level::Optional, Some(StartType::Manual), "Serveur Plex : gourmand. Manuel si tu ne streames pas tous les jours."),
    ("qbittorrent", Level::Optional, Some(StartType::Manual), "Client torrent en service. Manuel : lance-le quand tu en as besoin."),
);

// ---------------------------------------------------------------------------
// PROCESSUS
// ---------------------------------------------------------------------------

pub const PROCESSES: &[Rule] = process_rules!(
    SOURCE_PROCESSES,
    // Vitaux
    ("system", Level::Critical, "Noyau Windows. Impossible et interdit à terminer."),
    ("registry", Level::Critical, "Processus du registre Windows. Vital."),
    ("memory compression", Level::Critical, "Compression mémoire de Windows : il ÉCONOMISE de la RAM. Ne jamais y toucher."),
    ("smss.exe", Level::Critical, "Gestionnaire de session Windows. Vital."),
    ("csrss.exe", Level::Critical, "Sous-système client/serveur Windows. Vital : le terminer provoque un écran bleu."),
    ("wininit.exe", Level::Critical, "Initialisation de Windows. Vital."),
    ("winlogon.exe", Level::Critical, "Ouverture de session. Vital."),
    ("services.exe", Level::Critical, "Contrôleur des services. Vital."),
    ("lsass.exe", Level::Critical, "Autorité de sécurité locale (authentification). Vital."),
    ("lsaiso.exe", Level::Critical, "Isolation LSA (Credential Guard). Sécurité, vital."),
    ("fontdrvhost.exe", Level::Critical, "Hôte de pilotes de polices. Vital pour l'affichage."),
    ("dwm.exe", Level::Critical, "Gestionnaire de fenêtres du Bureau (transparence, composition). Vital pour l'affichage."),
    ("sihost.exe", Level::Critical, "Hôte d'infrastructure du shell. Vital pour le Bureau."),
    ("ctfmon.exe", Level::Critical, "Saisie de texte et langues (CTF Loader). Nécessaire au clavier."),
    ("explorer.exe", Level::Keep, "Bureau, barre des tâches et Explorateur. Redémarrable, mais à garder."),
    ("msmpeng.exe", Level::Keep, "Moteur Microsoft Defender (antivirus). Parfois gourmand pendant une analyse : à garder."),
    ("nissrv.exe", Level::Keep, "Inspection réseau Defender. Sécurité."),
    ("securityhealthservice.exe", Level::Keep, "Sécurité Windows. À garder."),
    ("securityhealthsystray.exe", Level::Optional, "Icône Sécurité Windows dans la zone de notification. Purement cosmétique."),
    ("smartscreen.exe", Level::Keep, "Filtre SmartScreen (protection contre les fichiers/pièges). À garder."),
    ("wmiprvse.exe", Level::System, "Fournisseur WMI : se lance à la demande d'un outil. Disparaît tout seul."),
    ("dllhost.exe", Level::System, "Hôte de composants COM. Dépend de ce qui l'a lancé."),
    ("conhost.exe", Level::System, "Hôte de fenêtre console. Lié à une fenêtre de terminal ouverte."),
    ("taskhostw.exe", Level::System, "Hôte de tâches Windows. Se termine quand la tâche est finie."),
    ("runtimebroker.exe", Level::System, "Courtier d'autorisations des applications Store. Un par app ; se ferme avec l'app."),
    ("svchost.exe", Level::System, "Hôte de services Windows : la RAM dépend des services hébergés (voir l'onglet Services)."),
    ("searchindexer.exe", Level::Optional, "Indexation de Windows Search. Peut occuper plusieurs centaines de Mo. Désactivable via le service WSearch."),
    ("searchprotocolhost.exe", Level::Optional, "Exploration de fichiers pour l'indexation. Lié à Windows Search."),
    ("searchfilterhost.exe", Level::Optional, "Filtres d'indexation. Lié à Windows Search."),
    ("startmenuexperiencehost.exe", Level::Keep, "Menu Démarrer. Le terminer le casse jusqu'au redémarrage."),
    ("shellexperiencehost.exe", Level::Keep, "Shell Windows (notifications, calendrier, recherche)."),
    ("textinputhost.exe", Level::Keep, "Clavier tactile / panneau d'emoji / saisie."),
    ("applicationframehost.exe", Level::Keep, "Cadre des fenêtres d'applications Store. Se ferme avec l'app."),
    ("systemsettings.exe", Level::Keep, "Application Paramètres de Windows."),
    ("lockapp.exe", Level::System, "Écran de verrouillage. Se ferme après connexion."),
    ("logonui.exe", Level::System, "Interface de connexion."),
    ("useroobebroker.exe", Level::System, "Assistant de première configuration / rappels de compte."),
    ("audiodg.exe", Level::Keep, "Moteur audio isolé (effets sonores). Nécessaire au son."),
    ("spoolsv.exe", Level::Optional, "Spouleur d'impression. Terminable si tu n'imprimes pas ; voir le service Spooler."),
    ("wudfhost.exe", Level::Keep, "Hôte de pilotes en mode utilisateur (USB, capteurs). À garder."),
    ("dashost.exe", Level::System, "Hôte d'association de périphériques (Miracast, appareils sans fil)."),
    ("vmmem", Level::Optional, "Mémoire des machines virtuelles WSL2 / Hyper-V / Docker. Peut atteindre plusieurs Go : arrête WSL/Docker pour la libérer."),
    ("vmmemwsl", Level::Optional, "Mémoire de WSL2. `wsl --shutdown` libère immédiatement cette RAM."),
    ("docker desktop.exe", Level::Optional, "Docker Desktop : très gourmand. À fermer quand tu ne développes pas."),
    ("com.docker.backend.exe", Level::Optional, "Backend Docker. Ferme Docker pour libérer cette RAM."),
    ("msedgewebview2.exe", Level::Optional, "Moteur Edge intégré à une application (Widgets, Office, Teams, nombreux logiciels). Se ferme avec l'app parente."),
    ("widgets.exe", Level::Trim, "Panneau Widgets de Windows 11 (actualités, météo). Souvent 200-400 Mo pour un panneau rarement ouvert."),
    ("widgetservice.exe", Level::Trim, "Service des Widgets Windows 11. Superflu si tu n'ouvres jamais le panneau Widgets."),
    ("widgetboard.exe", Level::Trim, "Tableau des Widgets Windows 11. Superflu."),
    ("searchapp.exe", Level::Keep, "Recherche Windows (barre de recherche). Le terminer casse la recherche jusqu'au redémarrage."),
    ("searchhost.exe", Level::Keep, "Hôte de recherche Windows 11. À garder."),
    ("cortana.exe", Level::Trim, "Cortana : assistant vocal largement abandonné par Microsoft. Superflu."),
    ("yourphone.exe", Level::Trim, "Mobile connecté (lien téléphone). Superflu si tu ne relies pas ton téléphone au PC."),
    ("phoneexperiencehost.exe", Level::Trim, "Mobile connecté. Superflu si tu ne synchronises pas ton téléphone."),
    ("crossdeviceresume.exe", Level::Optional, "Reprise entre appareils (Windows 11). Superflu si tu n'utilises pas cette fonction."),
    ("gamebar.exe", Level::Optional, "Xbox Game Bar. Superflu si tu ne l'utilises pas pour capturer ou suivre les FPS."),
    ("gamebarftserver.exe", Level::Optional, "Game Bar (capture). Superflu hors usage."),
    ("gamebarpresencewriter.exe", Level::Optional, "Game Bar (présence). Superflu hors usage."),
    ("gamingservices.exe", Level::Optional, "Services de jeux Xbox / Game Pass. À garder si tu joues via le Game Pass."),
    ("gamingservicesnet.exe", Level::Optional, "Services de jeux Xbox. Lié au Game Pass."),

    // Cloud / bureautique
    ("onedrive.exe", Level::Optional, "OneDrive : synchronisation permanente, souvent 150-400 Mo. À couper si tu ne synchronises rien d'important."),
    ("onedrive.sync.service.exe", Level::Optional, "Service de synchronisation OneDrive."),
    ("filecoauth.exe", Level::Optional, "Co-édition de fichiers Office (OneDrive/SharePoint). Lié à Office."),
    ("microsoft.sharepoint.exe", Level::Optional, "Client SharePoint / OneDrive. Lié à la synchronisation Office."),
    ("googledrivesync.exe", Level::Optional, "Google Drive : synchronisation permanente. À lancer à la demande si tu ne synchronises pas en continu."),
    ("google drive.exe", Level::Optional, "Google Drive. À lancer à la demande si usage occasionnel."),
    ("dropbox.exe", Level::Optional, "Dropbox : synchronisation permanente, souvent 200-500 Mo. À lancer à la demande."),
    ("dropboxupdate.exe", Level::Trim, "Mise à jour Dropbox en arrière-plan. Superflu en permanence."),
    ("mega.nz", Level::Optional, "Client MEGA. À lancer à la demande."),
    ("onedrivestandaloneupdater.exe", Level::Trim, "Mise à jour OneDrive. Superflu en permanence."),

    // Messagerie / apps
    ("teams.exe", Level::Optional, "Microsoft Teams. Ferme-le quand tu ne l'utilises pas : c'est souvent le plus gros consommateur."),
    ("ms-teams.exe", Level::Optional, "Microsoft Teams (nouvelle version). Ferme-le quand tu ne l'utilises pas."),
    ("msteams.exe", Level::Optional, "Teams. Ferme-le quand tu ne l'utilises pas."),
    ("slack.exe", Level::Optional, "Slack : plusieurs processus Electron, facilement 500 Mo+. À fermer hors travail."),
    ("discord.exe", Level::Optional, "Discord : plusieurs processus, souvent 400 Mo+. Démarrage automatique rarement nécessaire."),
    ("discordupdater.exe", Level::Trim, "Mise à jour Discord. Superflu en permanence."),
    ("whatsapp.exe", Level::Optional, "WhatsApp Desktop. À lancer à la demande."),
    ("telegram.exe", Level::Optional, "Telegram Desktop. À lancer à la demande."),
    ("signal.exe", Level::Optional, "Signal Desktop. À lancer à la demande."),
    ("skype.exe", Level::Optional, "Skype. À lancer à la demande."),
    ("skypeapp.exe", Level::Optional, "Skype (Store). À lancer à la demande."),
    ("skypebackgroundprocess.exe", Level::Trim, "Processus d'arrière-plan Skype. Superflu si Skype est fermé."),
    ("zoom.exe", Level::Optional, "Zoom. À lancer au moment de la réunion."),
    ("outlook.exe", Level::Optional, "Outlook. À garder si tu lis tes mails en continu, sinon ferme-le."),
    ("olk.exe", Level::Optional, "Nouvel Outlook. Ferme-le si tu utilises le webmail."),
    ("thunderbird.exe", Level::Optional, "Thunderbird. À lancer à la demande."),
    ("notion.exe", Level::Optional, "Notion (Electron). À lancer à la demande."),
    ("obs64.exe", Level::Optional, "OBS Studio. À fermer hors enregistrement/stream."),
    ("spotify.exe", Level::Trim, "Spotify : démarrage automatique inutile, plusieurs centaines de Mo. Lance-le quand tu écoutes."),
    ("spotifywebhelper.exe", Level::Trim, "Composant Spotify. Lié au démarrage automatique de Spotify."),
    ("itunes.exe", Level::Optional, "iTunes. À lancer à la demande."),
    ("ituneshelper.exe", Level::Trim, "iTunes Helper : surveille le branchement d'un iPhone. Superflu en permanence."),
    ("applemobiledevicehelper.exe", Level::Optional, "Assistant Apple Mobile Device. Lié à iTunes."),
    ("vlc.exe", Level::Optional, "Lecteur VLC. À fermer quand tu ne regardes rien."),

    // Jeux / launchers
    ("steam.exe", Level::Trim, "Steam : démarrage automatique inutile si tu ne joues pas dès l'ouverture de session."),
    ("steamwebhelper.exe", Level::Trim, "Navigateur intégré de Steam : plusieurs processus, 300-800 Mo au total. Ferme Steam pour libérer."),
    ("steamservice.exe", Level::Optional, "Service Steam. Se relance quand Steam démarre."),
    ("epicgameslauncher.exe", Level::Trim, "Epic Games Launcher : démarrage auto inutile, souvent 300 Mo+."),
    ("epicwebhelper.exe", Level::Trim, "Navigateur intégré d'Epic. Lié au launcher."),
    ("battle.net.exe", Level::Trim, "Battle.net : démarrage auto inutile."),
    ("agent.exe", Level::Optional, "Agent de mise à jour Blizzard. Lié à Battle.net."),
    ("galaxyclient.exe", Level::Trim, "GOG Galaxy : démarrage auto inutile."),
    ("galaxyupdater.exe", Level::Trim, "Mise à jour GOG Galaxy. Superflu."),
    ("eadesktop.exe", Level::Trim, "EA App : démarrage auto inutile."),
    ("eabackgroundservice.exe", Level::Trim, "Service d'arrière-plan EA. Superflu hors jeu."),
    ("upc.exe", Level::Trim, "Ubisoft Connect : démarrage auto inutile."),
    ("ubisoftgamelauncher.exe", Level::Trim, "Ubisoft Connect. Démarrage auto inutile."),
    ("origin.exe", Level::Trim, "Origin (ancien client EA). Démarrage auto inutile."),
    ("riotclientservices.exe", Level::Trim, "Riot Client : démarrage auto inutile."),
    ("rockstarlauncher.exe", Level::Trim, "Rockstar Launcher. Démarrage auto inutile."),
    ("xboxapp.exe", Level::Optional, "Application Xbox. À lancer à la demande."),

    // Adobe / création
    ("creative cloud.exe", Level::Trim, "Adobe Creative Cloud : démarre avec Windows et lance 4-5 processus. Superflu si tu ouvres tes apps Adobe à la main."),
    ("ccxprocess.exe", Level::Trim, "Adobe CCXProcess (extensions Creative Cloud). Superflu en permanence."),
    ("adobeipcbroker.exe", Level::Optional, "Communication inter-processus Adobe. Se relance avec l'app Adobe."),
    ("adobecollabsync.exe", Level::Trim, "Adobe Collaboration Synchronizer. Superflu hors travail collaboratif Acrobat."),
    ("acrotray.exe", Level::Trim, "Acrobat Tray (PDFMaker). Superflu : Acrobat fonctionne sans."),
    ("acrobat_sl.exe", Level::Trim, "Adobe Acrobat Speed Launcher. Préchargement inutile."),
    ("adobearm.exe", Level::Trim, "Adobe Reader Updater. Superflu en permanence."),
    ("adobeupdateservice.exe", Level::Trim, "Mise à jour Adobe. Superflu en permanence."),
    ("adobe notification client.exe", Level::Trim, "Notifications Adobe. Superflu."),
    ("core sync.exe", Level::Optional, "Adobe CoreSync (synchronisation de fichiers Creative Cloud)."),
    ("photoshop.exe", Level::Optional, "Photoshop : très gourmand, normal. À fermer quand tu as fini."),
    ("lightroom.exe", Level::Optional, "Lightroom : très gourmand. À fermer quand tu as fini."),
    ("premiere pro.exe", Level::Optional, "Premiere Pro : très gourmand. À fermer quand tu as fini."),

    // Utilitaires / updaters
    ("jusched.exe", Level::Trim, "Java Update Scheduler : vérifie les mises à jour Java en permanence. Superflu."),
    ("jucheck.exe", Level::Trim, "Vérification des mises à jour Java. Superflu."),
    ("googleupdate.exe", Level::Trim, "Mise à jour Google en arrière-plan. Chrome se met à jour à son lancement."),
    ("googlecrashhandler.exe", Level::Trim, "Rapport de plantage Google. Superflu."),
    ("googlecrashhandler64.exe", Level::Trim, "Rapport de plantage Google. Superflu."),
    ("microsoftedgeupdate.exe", Level::Trim, "Mise à jour Edge en arrière-plan. Superflu en permanence."),
    ("microsoftedgecp.exe", Level::Optional, "Composant Edge (préchargement / arrière-plan)."),
    ("msedge.exe", Level::Optional, "Microsoft Edge. S'il tourne sans fenêtre ouverte, c'est le « démarrage accéléré » d'Edge : désactivable dans Edge."),
    ("chrome.exe", Level::Optional, "Google Chrome. Ferme les onglets inutiles : chaque onglet est un processus."),
    ("firefox.exe", Level::Optional, "Firefox. Ferme les onglets inutiles."),
    ("brave.exe", Level::Optional, "Brave. Ferme les onglets inutiles."),
    ("opera.exe", Level::Optional, "Opera. À lancer à la demande."),
    ("ccleaner64.exe", Level::Trim, "CCleaner en surveillance permanente. Le nettoyage à la demande suffit."),
    ("ccleaner.exe", Level::Trim, "CCleaner. Le nettoyage à la demande suffit."),
    ("utorrent.exe", Level::Trim, "uTorrent : démarrage auto + partage en arrière-plan. À lancer à la demande."),
    ("bittorrent.exe", Level::Trim, "BitTorrent : démarrage auto inutile."),
    ("qbittorrent.exe", Level::Trim, "qBittorrent : démarrage auto inutile, partage en continu."),
    ("nvsphelper64.exe", Level::Trim, "NVIDIA ShadowPlay / Share (enregistrement en continu). Superflu si tu ne captures pas."),
    ("nvcontainer.exe", Level::Optional, "Conteneur NVIDIA. Une partie est liée au pilote : ne pas terminer aveuglément."),
    ("nvidia share.exe", Level::Trim, "NVIDIA Share (overlay / enregistrement). Superflu si tu ne l'utilises pas."),
    ("nvidia geforce experience.exe", Level::Trim, "GeForce Experience : démarrage auto inutile, le pilote fonctionne sans."),
    ("nvidia app.exe", Level::Optional, "NVIDIA App. À lancer à la demande pour les mises à jour de pilote."),
    ("nvbackend.exe", Level::Trim, "NVIDIA Backend (mises à jour / télémétrie). Superflu."),
    ("gfexperience.exe", Level::Trim, "GeForce Experience. Démarrage auto inutile."),
    ("razer synapse service.exe", Level::Optional, "Razer Synapse. Manuel si tes profils sont dans le périphérique."),
    ("razer synapse 3.exe", Level::Optional, "Razer Synapse 3 : souvent 300 Mo+. À lancer à la demande."),
    ("razercentral.exe", Level::Optional, "Razer Central. Lié à Synapse."),
    ("icue.exe", Level::Optional, "Corsair iCUE : très gourmand (300-600 Mo). À fermer si tes profils sont stockés dans le matériel."),
    ("corsair.service.exe", Level::Optional, "Service Corsair. Lié à iCUE."),
    ("lghub.exe", Level::Optional, "Logitech G HUB. À fermer si tes profils sont enregistrés sur le périphérique."),
    ("lghub_agent.exe", Level::Optional, "Agent Logitech G HUB. Lié à G HUB."),
    ("lghub_updater.exe", Level::Trim, "Mise à jour Logitech. Superflu en permanence."),
    ("logioptionsplus.exe", Level::Optional, "Logitech Options+ (boutons personnalisés). À garder si tu utilises les gestures."),
    ("armourycrate.exe", Level::Optional, "ASUS Armoury Crate : très gourmand. À fermer si tu n'utilises pas les profils."),
    ("armouryswagent.exe", Level::Optional, "Agent ASUS Armoury. Lié à Armoury Crate."),
    ("msiafterburner.exe", Level::Optional, "MSI Afterburner (overclocking / OSD). À garder seulement si tu overclockes."),
    ("rtss.exe", Level::Optional, "RivaTuner Statistics Server (OSD). Lié à Afterburner."),
    ("nahimic.exe", Level::Trim, "Nahimic (effets audio). Superflu si tu n'utilises pas les effets."),
    ("nahimicsvc32.exe", Level::Trim, "Service Nahimic 32 bits. Superflu."),
    ("nahimicsvc64.exe", Level::Trim, "Service Nahimic 64 bits. Superflu."),
    ("supportassistagent.exe", Level::Trim, "Dell SupportAssist : tourne en permanence pour pas grand-chose. Lance-le à la demande."),
    ("dsatray.exe", Level::Trim, "Intel Driver & Support Assistant (tray). Lance-le à la demande."),
    ("dsaservice.exe", Level::Trim, "Intel Driver & Support Assistant. Lance-le à la demande."),
    ("sursvc.exe", Level::Trim, "Intel Computing Improvement Program (télémétrie). Superflu."),
    ("esrv_svc.exe", Level::Trim, "Intel telemetry (SUR). Superflu."),
    ("hpmsysinfo.exe", Level::Trim, "Collecte d'informations HP. Superflu."),
    ("hpqtra08.exe", Level::Trim, "Utilitaire HP (imprimante) en tray. Superflu si tu imprimes rarement."),
    ("hpwarrantycheck.exe", Level::Trim, "Vérification de garantie HP. Superflu."),
    ("lenovovantage.exe", Level::Optional, "Lenovo Vantage. À lancer à la demande pour les pilotes."),
    ("lenovo.modern.imcontroller.exe", Level::Optional, "Lenovo IM Controller. Lié à Vantage."),
    ("samsungdex.exe", Level::Trim, "Samsung DeX. Superflu si tu ne connectes pas ton téléphone."),
    ("samsungflow.exe", Level::Trim, "Samsung Flow. Superflu si inutilisé."),
    ("teamviewer.exe", Level::Optional, "TeamViewer : à lancer quand tu en as besoin, sauf accès permanent voulu."),
    ("anydesk.exe", Level::Optional, "AnyDesk : à lancer à la demande (plus sûr)."),
    ("rustdesk.exe", Level::Optional, "RustDesk : à lancer à la demande."),
    ("everything.exe", Level::Keep, "Everything (recherche de fichiers) : très léger et bien plus rapide que Windows Search. À garder si tu l'utilises."),
    ("powertoys.exe", Level::Optional, "PowerToys. À garder si tu utilises ses outils."),
    ("autohotkey.exe", Level::Keep, "Script AutoHotkey : probablement utile à ton usage. À garder."),
    ("malwarebytes", Level::Keep, "Anti-malware : à garder si c'est ta protection choisie."),
    ("bitwarden.exe", Level::Keep, "Gestionnaire de mots de passe : à garder."),
    ("keepass.exe", Level::Keep, "Gestionnaire de mots de passe : à garder."),
    ("1password.exe", Level::Keep, "Gestionnaire de mots de passe : à garder."),
    ("sharex.exe", Level::Optional, "ShareX (captures). À garder si tu captures souvent."),
    ("greenshot.exe", Level::Optional, "Greenshot (captures). À garder si tu captures souvent."),
    ("f.lux.exe", Level::Keep, "Filtre de lumière bleue : à garder si tu l'utilises le soir."),
    ("qbittorrent-updater.exe", Level::Trim, "Mise à jour qBittorrent. Superflu."),
    ("wpscloudsvr.exe", Level::Trim, "WPS Office Cloud. Superflu si tu n'utilises pas le cloud WPS."),
    ("wpscenter.exe", Level::Trim, "WPS Center (notifications/publicités). Superflu."),
    ("mcafee.exe", Level::Optional, "Composant McAfee. Un seul antivirus à la fois : désinstalle celui que tu n'utilises pas."),
    ("mcshield.exe", Level::Optional, "McAfee Shield. Un seul antivirus à la fois."),
    ("nortonsecurity.exe", Level::Optional, "Norton Security. Un seul antivirus à la fois."),
    ("avastui.exe", Level::Optional, "Avast. Un seul antivirus à la fois."),
    ("avgui.exe", Level::Optional, "AVG. Un seul antivirus à la fois."),
);

// ---------------------------------------------------------------------------
// DÉMARRAGE (motifs cherchés dans le nom ET la commande)
// ---------------------------------------------------------------------------

pub const STARTUP: &[Rule] = startup_rules!(
    SOURCE_STARTUP,
    ("onedrive", Level::Optional, "OneDrive au démarrage : synchronisation permanente. À couper si tu ne synchronises rien d'important (tu peux ouvrir OneDrive à la main)."),
    ("microsoftedgeautolaunch", Level::Trim, "Préchargement d'Edge au démarrage (« démarrage accéléré »). Pur confort marketing : Edge s'ouvre très bien sans ça."),
    ("msedge", Level::Trim, "Edge lancé automatiquement. Inutile : ouvre-le quand tu en as besoin."),
    ("teams", Level::Optional, "Teams au démarrage. À garder si tu l'utilises toute la journée, sinon lance-le à la main."),
    ("msteams", Level::Optional, "Teams au démarrage. À garder seulement si tu l'utilises en continu."),
    ("com.squirrel", Level::Optional, "Application Electron (Slack, Teams, Discord...) au démarrage. Souvent 300-600 Mo dès l'ouverture de session."),
    ("slack", Level::Optional, "Slack au démarrage. À couper si tu ne travailles pas dessus en permanence."),
    ("discord", Level::Trim, "Discord au démarrage : plusieurs centaines de Mo avant même que tu joues. Lance-le à la main."),
    ("spotify", Level::Trim, "Spotify au démarrage : inutile, tu peux l'ouvrir en 2 secondes quand tu veux écouter."),
    ("steam", Level::Trim, "Steam au démarrage : inutile si tu ne joues pas dès l'ouverture de session. Gros gain de RAM et de temps de boot."),
    ("epicgameslauncher", Level::Trim, "Epic Games Launcher au démarrage : inutile, il se lance très bien à la demande."),
    ("battle.net", Level::Trim, "Battle.net au démarrage : inutile."),
    ("galaxyclient", Level::Trim, "GOG Galaxy au démarrage : inutile."),
    ("eadesktop", Level::Trim, "EA App au démarrage : inutile."),
    ("upc.exe", Level::Trim, "Ubisoft Connect au démarrage : inutile."),
    ("ubisoft", Level::Trim, "Ubisoft Connect au démarrage : inutile."),
    ("riotclient", Level::Trim, "Riot Client au démarrage : inutile (Vanguard reste actif si tu joues à Valorant)."),
    ("rockstar", Level::Trim, "Rockstar Launcher au démarrage : inutile."),
    ("xboxapp", Level::Optional, "App Xbox au démarrage : inutile hors Game Pass quotidien."),
    ("creative cloud", Level::Trim, "Adobe Creative Cloud au démarrage : lance 4 à 5 processus. Tes apps Adobe fonctionnent sans."),
    ("ccxprocess", Level::Trim, "Adobe CCXProcess : extensions Creative Cloud. Superflu au démarrage."),
    ("adobeacrobatsynchronizer", Level::Trim, "Adobe Acrobat Synchronizer. Superflu au démarrage."),
    ("adobecollabsync", Level::Trim, "Adobe Collaboration Synchronizer. Superflu au démarrage."),
    ("acrotray", Level::Trim, "Acrobat Tray (PDFMaker). Superflu : Acrobat fonctionne sans."),
    ("adobe reader speed launcher", Level::Trim, "Préchargement d'Adobe Reader. Gain de RAM immédiat, aucune perte de fonction."),
    ("acrobat_sl", Level::Trim, "Préchargement d'Acrobat. Inutile."),
    ("adobearm", Level::Trim, "Adobe Updater au démarrage. Acrobat vérifie les mises à jour à son lancement."),
    ("adobe gc invoker", Level::Optional, "Adobe Genuine Client (vérification de licence). À laisser si tu as une licence Adobe."),
    ("adobenotification", Level::Trim, "Notifications Adobe. Superflu."),
    ("ituneshelper", Level::Trim, "iTunes Helper : surveille le branchement d'un iPhone en permanence. Superflu."),
    ("quicktime task", Level::Junk, "QuickTime Task : QuickTime est abandonné depuis 2016. À supprimer carrément."),
    ("sunjavaupdatesched", Level::Trim, "Java Update Scheduler : vérification permanente des mises à jour Java. Superflu."),
    ("jusched", Level::Trim, "Java Update Scheduler. Superflu au démarrage."),
    ("googleupdate", Level::Trim, "Google Update au démarrage. Chrome se met à jour quand tu l'ouvres."),
    ("googledrivesync", Level::Optional, "Google Drive au démarrage. À garder seulement si tu synchronises en continu."),
    ("google drive", Level::Optional, "Google Drive au démarrage. À garder seulement si tu synchronises en continu."),
    ("dropbox", Level::Optional, "Dropbox au démarrage. À garder seulement si tu synchronises en continu."),
    ("ccleaner", Level::Trim, "Surveillance CCleaner au démarrage. Un nettoyage manuel occasionnel suffit."),
    ("utorrent", Level::Trim, "uTorrent au démarrage : partage en arrière-plan dès le boot. À lancer à la demande."),
    ("bittorrent", Level::Trim, "BitTorrent au démarrage. À lancer à la demande."),
    ("qbittorrent", Level::Trim, "qBittorrent au démarrage. À lancer à la demande."),
    ("skype", Level::Trim, "Skype au démarrage. À lancer à la demande."),
    ("zoom", Level::Optional, "Zoom au démarrage : inutile, lance-le au moment de la réunion."),
    ("whatsapp", Level::Optional, "WhatsApp au démarrage. À couper si tu consultes tes messages sur téléphone."),
    ("telegram", Level::Optional, "Telegram au démarrage. À lancer à la demande."),
    ("signal", Level::Optional, "Signal au démarrage. À lancer à la demande."),
    ("nvidia geforce experience", Level::Trim, "GeForce Experience au démarrage : le pilote graphique fonctionne sans. Gain net."),
    ("nvbackend", Level::Trim, "NVIDIA Backend (mises à jour/télémétrie). Superflu au démarrage."),
    ("nvidia share", Level::Trim, "NVIDIA Share / ShadowPlay au démarrage. Superflu si tu ne captures pas."),
    ("nvsphelper", Level::Trim, "NVIDIA ShadowPlay au démarrage. Superflu si tu ne captures pas."),
    ("nvidia app", Level::Optional, "NVIDIA App au démarrage. À lancer quand tu veux mettre à jour le pilote."),
    ("razer", Level::Optional, "Razer Synapse au démarrage. À couper si tes profils DPI/RGB sont enregistrés dans le périphérique."),
    ("icue", Level::Optional, "Corsair iCUE au démarrage : 300-600 Mo. À couper si tes profils sont stockés dans le matériel."),
    ("corsair", Level::Optional, "Corsair iCUE au démarrage. À couper si tes profils sont matériels."),
    ("lghub", Level::Optional, "Logitech G HUB au démarrage. À couper si tes profils sont enregistrés sur le périphérique."),
    ("logioptions", Level::Optional, "Logitech Options au démarrage. À garder si tu utilises les boutons/gestures personnalisés."),
    ("armoury", Level::Optional, "ASUS Armoury Crate au démarrage : très gourmand. À couper si tu n'utilises pas les profils de ventilation."),
    ("msiafterburner", Level::Optional, "MSI Afterburner au démarrage. À garder UNIQUEMENT si tu overclockes (sinon tes réglages ne s'appliquent pas)."),
    ("nahimic", Level::Trim, "Nahimic (effets audio) au démarrage. Superflu si tu n'utilises pas les effets."),
    ("supportassist", Level::Trim, "Dell SupportAssist au démarrage. Lance-le à la demande pour vérifier tes pilotes."),
    ("dsatray", Level::Trim, "Intel Driver & Support Assistant au démarrage. À lancer à la demande."),
    ("intel computing improvement", Level::Trim, "Programme d'amélioration Intel (télémétrie). Superflu."),
    ("hp", Level::Optional, "Utilitaire HP préinstallé au démarrage. Rarement nécessaire en permanence."),
    ("lenovo", Level::Optional, "Utilitaire Lenovo au démarrage. À lancer à la demande pour les pilotes."),
    ("samsungdex", Level::Trim, "Samsung DeX au démarrage. Superflu si tu ne connectes pas ton téléphone."),
    ("teamviewer", Level::Optional, "TeamViewer au démarrage : accès permanent. À couper si tu ne t'en sers qu'occasionnellement."),
    ("anydesk", Level::Optional, "AnyDesk au démarrage : écoute permanente. À couper si usage occasionnel (et plus sûr)."),
    ("wps", Level::Trim, "WPS Office (cloud/notifications) au démarrage. Superflu."),
    ("realtek", Level::Keep, "Panneau audio Realtek. Utile pour la détection de casque : à garder (impact faible)."),
    ("igfxcui", Level::Optional, "Panneau graphique Intel au démarrage. Le pilote fonctionne sans."),
    ("igcc", Level::Optional, "Intel Graphics Command Center au démarrage. Superflu en permanence."),
    ("hotkeys", Level::Keep, "Gestion des touches spéciales du constructeur. À garder si tu utilises les raccourcis (luminosité, volume)."),
    ("synaptics", Level::Keep, "Pilote pavé tactile (gestures multi-doigts). À garder sur portable."),
    ("elan", Level::Keep, "Pilote pavé tactile. À garder sur portable."),
    ("alps", Level::Keep, "Pilote pavé tactile. À garder sur portable."),
    ("windowsdefender", Level::Keep, "Icône Sécurité Windows. Impact négligeable, à garder."),
    ("securityhealth", Level::Keep, "Icône Sécurité Windows. Impact négligeable, à garder."),
    ("bitwarden", Level::Keep, "Gestionnaire de mots de passe : à garder au démarrage."),
    ("keepass", Level::Keep, "Gestionnaire de mots de passe : à garder au démarrage."),
    ("1password", Level::Keep, "Gestionnaire de mots de passe : à garder au démarrage."),
    ("everything", Level::Keep, "Everything : très léger, recherche instantanée. À garder si tu l'utilises."),
    ("powertoys", Level::Optional, "PowerToys au démarrage. À garder si tu utilises ses outils au quotidien."),
    ("autohotkey", Level::Keep, "Script AutoHotkey : sans doute écrit par toi. À garder."),
    ("flux", Level::Keep, "Filtre de lumière bleue : à garder si tu l'utilises."),
    ("sharex", Level::Optional, "ShareX au démarrage. À garder si tu fais souvent des captures."),
    ("expressvpn", Level::Optional, "VPN au démarrage : à garder si tu veux être protégé en permanence."),
    ("nordvpn", Level::Optional, "VPN au démarrage : à garder si tu veux être protégé en permanence."),
    ("protonvpn", Level::Optional, "VPN au démarrage : à garder si tu veux être protégé en permanence."),
    ("malwarebytes", Level::Keep, "Anti-malware : à garder si c'est ta protection."),
    ("docker", Level::Optional, "Docker Desktop au démarrage : plusieurs Go de RAM. À lancer quand tu développes."),
    ("wsl", Level::Optional, "WSL au démarrage. À lancer quand tu en as besoin."),
    ("plex", Level::Optional, "Plex Media Server au démarrage. À couper si tu ne streames pas tous les jours."),
    ("obs64", Level::Optional, "OBS au démarrage. À lancer au moment d'enregistrer."),
);

// ---------------------------------------------------------------------------
// TÂCHES PLANIFIÉES
// ---------------------------------------------------------------------------

pub const TASKS: &[Rule] = task_rules!(
    SOURCE_TASKS,
    ("microsoft compatibility appraiser", Level::Trim, "Évalue la compatibilité du PC pour les mises à jour et alimente la télémétrie. Peut occuper le CPU/disque plusieurs minutes. Sans impact si désactivée."),
    ("programdataupdater", Level::Trim, "Collecte de données de compatibilité des programmes (télémétrie). Superflu."),
    ("startupapptask", Level::Optional, "Analyse les applications de démarrage pour le Gestionnaire des tâches. Désactivable sans risque."),
    ("consolidator", Level::Trim, "Programme d'amélioration de l'expérience client (CEIP) : envoi de statistiques. Superflu."),
    ("usbceip", Level::Trim, "CEIP : statistiques d'utilisation USB. Superflu."),
    ("kernelceip", Level::Trim, "CEIP : statistiques noyau. Superflu."),
    ("kernelceiptask", Level::Trim, "CEIP : statistiques noyau. Superflu."),
    ("autochk\\proxy", Level::Trim, "Collecte SQM après vérification du disque (télémétrie). Superflu."),
    ("diskdiagnosticdatacollector", Level::Trim, "Collecte de données de diagnostic disque (télémétrie). Superflu."),
    ("microsoft-windows-diskdiagnosticdatacollector", Level::Trim, "Collecte de données de diagnostic disque (télémétrie). Superflu."),
    ("dmclient", Level::Trim, "Feedback Hub : collecte de télémétrie. Superflu."),
    ("queuereporting", Level::Trim, "Envoi des rapports d'erreurs Windows. Superflu (les rapports restent locaux)."),
    ("mapstoasttask", Level::Optional, "Notifications de l'app Cartes. Inutile si tu n'utilises pas Cartes."),
    ("mapsupdatetask", Level::Optional, "Mise à jour des cartes hors ligne. Inutile si tu n'utilises pas Cartes."),
    ("retail demo", Level::Junk, "Mode démonstration magasin. Strictement inutile."),
    ("cleanupofflinecontent", Level::Junk, "Nettoyage du contenu de démonstration magasin. Inutile."),
    ("familysafetymonitor", Level::Optional, "Contrôle parental / Sécurité familiale. Inutile sans compte enfant."),
    ("familysafetyuploadtask", Level::Optional, "Envoi des rapports de Sécurité familiale. Inutile sans compte enfant."),
    ("work folders", Level::Optional, "Synchronisation des dossiers de travail (entreprise). Inutile chez soi."),
    ("officetelemetryagent", Level::Trim, "Télémétrie Office. Superflu, aucun impact sur Word/Excel."),
    ("officetelemetryagentlogon", Level::Trim, "Télémétrie Office à l'ouverture de session. Superflu."),
    ("officetelemetryagentfallback", Level::Trim, "Télémétrie Office (tâche de repli). Superflu."),
    ("office clicktorun service monitor", Level::Keep, "Surveillance du service Office Click-to-Run : nécessaire aux mises à jour d'Office. À garder."),
    ("office automatic updates", Level::Keep, "Mises à jour d'Office. À garder (sécurité)."),
    ("googleupdatetaskmachine", Level::Trim, "Mise à jour Google en tâche planifiée. Chrome se met à jour à son lancement : tu peux désactiver."),
    ("googleupdatetaskuser", Level::Trim, "Mise à jour Google (utilisateur). Chrome se met à jour à son lancement."),
    ("microsoftedgeupdatetask", Level::Trim, "Mise à jour Edge en tâche planifiée. Edge se met à jour à son lancement."),
    ("adobe acrobat update task", Level::Trim, "Mise à jour Adobe Acrobat. Acrobat vérifie à son lancement : désactivable."),
    ("dropboxupdatetask", Level::Trim, "Mise à jour Dropbox. Superflu (Dropbox se met à jour lui-même)."),
    ("onedrive standalone update", Level::Optional, "Mise à jour OneDrive. À garder si tu utilises OneDrive."),
    ("ccleanerskipuac", Level::Trim, "CCleaner : contournement de l'invite UAC. Superflu (et peu propre)."),
    ("ccleanerupdate", Level::Trim, "Mise à jour CCleaner. Superflu."),
    ("nvidia", Level::Optional, "Tâche NVIDIA (mises à jour / télémétrie). Le pilote fonctionne sans ; à désactiver si tu mets à jour manuellement."),
    ("driver easy", Level::Trim, "Driver Easy : scan automatique de pilotes. À lancer à la demande."),
    ("driver booster", Level::Trim, "Driver Booster : scan automatique. À lancer à la demande (et prudence avec les pilotes génériques)."),
    ("avast", Level::Optional, "Tâche Avast. À garder si Avast est ton antivirus."),
    ("scheduleddefrag", Level::Keep, "Optimisation des disques (TRIM SSD). À GARDER : c'est l'entretien de ton SSD."),
    ("windowsupdate", Level::Keep, "Mises à jour Windows. À garder."),
    ("systemrestore", Level::Keep, "Points de restauration système. À GARDER : c'est ton filet de sécurité."),
    ("backup", Level::Keep, "Sauvegarde Windows. À garder si tu l'utilises."),
    ("windows defender", Level::Keep, "Analyses et maintenance de Microsoft Defender. À garder."),
    ("microsoft-defender", Level::Keep, "Analyses et maintenance de Microsoft Defender. À garder."),
    ("mpidle", Level::Keep, "Tâches d'inactivité de Defender. À garder."),
    ("mpscheduled", Level::Keep, "Analyse planifiée de Defender. À garder."),
    ("diskcleanup", Level::Keep, "Nettoyage de disque automatique (Storage Sense). Utile, à garder."),
    ("silentcleanup", Level::Keep, "Nettoyage silencieux du disque. Utile, à garder."),
    ("storage sense", Level::Keep, "Assistant Stockage : libère de l'espace automatiquement. À garder."),
    ("time synchronization", Level::Keep, "Synchronisation de l'heure. À garder."),
    ("wsqmcons", Level::Trim, "Collecte de données d'utilisation (SQM/CEIP). Superflu."),
    ("sqm", Level::Trim, "Collecte de données d'utilisation (SQM). Superflu."),
);

// ---------------------------------------------------------------------------
// Protections (verrous serveur)
// ---------------------------------------------------------------------------

pub const PROTECTED_SERVICES: &[&str] = &[
    "rpcss",
    "rpceptmapper",
    "dcomlaunch",
    "eventlog",
    "plugplay",
    "power",
    "profsrv",
    "samss",
    "lsm",
    "schedule",
    "gpsvc",
    "winmgmt",
    "cryptsvc",
    "bfe",
    "mpssvc",
    "windefend",
    "wdnissvc",
    "securityhealthservice",
    "wscsvc",
    "dhcp",
    "dnscache",
    "nsi",
    "netprofm",
    "nlasvc",
    "wlansvc",
    "wcmsvc",
    "audiosrv",
    "audioendpointbuilder",
    "deviceinstall",
    "dsmsvc",
    "staterepository",
    "tokenbroker",
    "usermanager",
    "systemeventsbroker",
    "timebrokersvc",
    "coremessagingregistrar",
    "brokerinfrastructure",
    "appinfo",
    "keyiso",
    "trustedinstaller",
    "fontcache",
    "textinputmanagementservice",
    "appxsvc",
    "clipsvc",
    "eventsystem",
    "sens",
    "storsvc",
    "lanmanworkstation",
    "sgrmbroker",
    "waasmedicsvc",
    "wuauserv",
    "usosvc",
    "msiserver",
    "defragsvc",
    "w32time",
    "themes",
    "camsvc",
    "licensemanager",
];

pub const PROTECTED_PROCESSES: &[&str] = &[
    "system",
    "registry",
    "memory compression",
    "secure system",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "lsaiso.exe",
    "fontdrvhost.exe",
    "dwm.exe",
    "sihost.exe",
    "ctfmon.exe",
    "svchost.exe",
    "msmpeng.exe",
    "nissrv.exe",
    "securityhealthservice.exe",
    "smartscreen.exe",
    "wudfhost.exe",
    "audiodg.exe",
    "startmenuexperiencehost.exe",
    "shellexperiencehost.exe",
    "searchhost.exe",
    "searchapp.exe",
    "textinputhost.exe",
    "applicationframehost.exe",
    "runtimebroker.exe",
    "taskhostw.exe",
    "dllhost.exe",
    "conhost.exe",
    "wmiprvse.exe",
    "systemsettings.exe",
    "lockapp.exe",
    "logonui.exe",
    "dashost.exe",
    "explorer.exe",
];

pub fn is_protected_service(name: &str) -> bool {
    let n = lc(name);
    PROTECTED_SERVICES.iter().any(|p| *p == n)
}

pub fn is_protected_process(name: &str) -> bool {
    let n = lc(name);
    PROTECTED_PROCESSES.iter().any(|p| *p == n)
}

// ---------------------------------------------------------------------------
// Correspondance exacte
// ---------------------------------------------------------------------------

impl Rule {
    /// Démarrage : nom d'entrée, nom d'exécutable ou composant de chemin exact.
    fn matches_startup(&self, name: &str, command: &str) -> bool {
        if self.mode != MatchMode::StartupExact {
            return false;
        }
        if lc(name) == self.key {
            return true;
        }
        let exe = lc(&crate::util::extract_exe(command));
        let stem = exe.strip_suffix(".exe").unwrap_or(&exe);
        if exe == self.key || stem == self.key {
            return true;
        }
        lc(command).split(['\\', '/']).any(|part| part == self.key)
    }

    /// Tâche : nom de tâche ou suffixe de chemin exact, à une frontière de
    /// composant. Aucune sous-chaîne libre sur la ligne de commande.
    fn matches_task(&self, path: &str) -> bool {
        if self.mode != MatchMode::TaskPath {
            return false;
        }
        let p = lc(path);
        p == self.key
            || p.split(['\\', '/']).any(|part| part == self.key)
            || p.ends_with(&format!("\\{}", self.key))
    }
}

// ---------------------------------------------------------------------------
// Moteur de conseil
// ---------------------------------------------------------------------------

/// Conseil pour un service Windows.
pub fn service_advice(
    name: &str,
    display: &str,
    path: &str,
    start: crate::model::StartType,
    running: bool,
    ram: u64,
) -> Advice {
    let n = lc(name);
    let d = lc(display);
    let p = lc(path);

    // 1. Règle exacte sur le nom court, puis sur le nom affiché exact.
    let hit = SERVICES
        .iter()
        .find(|r| r.key == n)
        .or_else(|| SERVICES.iter().find(|r| r.key == d));

    if let Some(rule) = hit {
        let mut a = Advice::new(rule.level, rule.reason);
        if is_protected_service(name) {
            a.locked = true;
            return a;
        }
        if let Some(t) = rule.target {
            let already = start == t || (t == StartType::Manual && start == StartType::Disabled);
            if !already && matches!(rule.level, Level::Junk | Level::Trim | Level::Optional) {
                a = a.with_action(Action::Service {
                    name: name.to_string(),
                    start: t,
                    from: Some(start),
                });
                // Changer le type de démarrage ne libère la RAM qu'après
                // arrêt du service ou redémarrage : gain au prochain boot.
                a.gain_next_boot = Gain::estimated(ram, GainScope::HostPart);
                let _ = running;
            } else if already && matches!(rule.level, Level::Junk | Level::Trim) {
                a.reason = format!("{} — déjà bien configuré ✔", rule.reason);
            }
        }
        return a;
    }

    if is_protected_service(name) {
        let mut a = Advice::new(
            Level::Critical,
            "Service essentiel au fonctionnement de Windows. Verrouillé par RamScope.",
        );
        a.locked = true;
        return a;
    }

    // 2. Aucune règle nommée : aucun conseil d'action. Une heuristique de nom
    // ne porte pas de source et ne doit donc pas proposer de modification.
    let is_ms = p.contains("\\windows\\") || p.contains("svchost.exe");

    if is_ms {
        return Advice::new(
            Level::System,
            "Service Windows non répertorié par RamScope. Par prudence : ne pas modifier (la plupart sont déjà en manuel/déclenchement).",
        );
    }

    if start == StartType::Auto {
        return Advice::new(
            Level::Unknown,
            "Service tiers non répertorié, en démarrage automatique. Vérifie l'éditeur et le chemin avant de décider : si tu ne reconnais pas le logiciel, cherche le nom du service.",
        );
    }

    Advice::new(
        Level::Unknown,
        "Service tiers non répertorié, déjà en manuel ou désactivé : rien à gagner.",
    )
}

/// Conseil pour un processus.
pub fn process_advice(
    name: &str,
    path: &str,
    description: &str,
    company: &str,
    private: u64,
    has_window: bool,
    services: &[String],
) -> Advice {
    let n = lc(name);
    let p = lc(path);
    let c = lc(company);

    if let Some(rule) = PROCESSES.iter().find(|r| r.key == n) {
        let mut a = Advice::new(rule.level, rule.reason);
        if n == "svchost.exe" && !services.is_empty() {
            a.reason = format!(
                "{} Services hébergés : {}.",
                rule.reason,
                services.join(", ")
            );
        }
        if is_protected_process(name) {
            a.locked = true;
            return a;
        }
        if matches!(rule.level, Level::Trim | Level::Optional) {
            a = a.with_action(Action::Kill {
                pid: 0,
                name: name.to_string(),
                image_path: path.to_string(),
                created_at: 0,
            });
            a.gain = Gain::estimated(private, GainScope::Private);
        }
        return a;
    }

    if is_protected_process(name) {
        let mut a = Advice::new(
            Level::Critical,
            "Processus système Windows : ne pas terminer.",
        );
        a.locked = true;
        return a;
    }

    let is_ms = c.contains("microsoft") || p.contains("\\windows\\");
    if is_ms {
        return Advice::new(
            Level::System,
            "Composant Windows non répertorié : à laisser tel quel par prudence.",
        );
    }

    if has_window {
        return Advice::new(
            Level::Keep,
            "Application avec une fenêtre ouverte : c'est ton usage en cours, rien à optimiser automatiquement.",
        );
    }

    let _ = description;
    Advice::new(
        Level::Unknown,
        "Processus non répertorié. Vérifie l'éditeur et le chemin : si tu ne reconnais pas le logiciel, fais une recherche sur le nom du fichier avant d'agir.",
    )
}

/// Conseil pour une entrée de démarrage.
pub fn startup_advice(name: &str, command: &str, enabled: bool, ram: u64) -> Advice {
    if let Some(rule) = STARTUP.iter().find(|r| r.matches_startup(name, command)) {
        let mut a = Advice::new(rule.level, rule.reason);
        if enabled && matches!(rule.level, Level::Junk | Level::Trim | Level::Optional) {
            a = a.with_action(Action::Startup {
                id: String::new(),
                enabled: false,
                was_enabled: Some(enabled),
            });
            // Désactiver le démarrage n'arrête pas le programme déjà lancé.
            a.gain_next_boot = Gain::estimated(ram, GainScope::Private);
        } else if !enabled && matches!(rule.level, Level::Junk | Level::Trim) {
            a.reason = format!("{} — déjà désactivé ✔", rule.reason);
        }
        return a;
    }

    if !enabled {
        return Advice::new(Level::Unknown, "Entrée déjà désactivée : rien à faire.");
    }

    Advice::new(
        Level::Unknown,
        "Entrée de démarrage non répertoriée. Aucune règle nommée ne s'applique : RamScope ne propose pas de la désactiver. Vérifie son éditeur et son chemin ; si tu ne sais pas à quoi elle sert, laisse-la et documente-la plutôt que d'agir au hasard.",
    )
}

/// Conseil pour une tâche planifiée.
pub fn task_advice(path: &str, _command: &str, enabled: bool, triggers: &[String]) -> Advice {
    let p = lc(path);

    if let Some(rule) = TASKS.iter().find(|r| r.matches_task(path)) {
        let mut a = Advice::new(rule.level, rule.reason);
        if enabled && matches!(rule.level, Level::Junk | Level::Trim | Level::Optional) {
            a = a.with_action(Action::Task {
                path: path.to_string(),
                enabled: false,
                was_enabled: Some(enabled),
            });
        } else if !enabled && matches!(rule.level, Level::Junk | Level::Trim) {
            a.reason = format!("{} — déjà désactivée ✔", rule.reason);
        }
        return a;
    }

    let _ = triggers;
    if !enabled {
        return Advice::new(Level::Unknown, "Tâche déjà désactivée : rien à faire.");
    }

    if p.starts_with("\\microsoft\\windows\\") {
        return Advice::new(
            Level::System,
            "Tâche de maintenance Windows non répertoriée : à laisser active par prudence.",
        );
    }

    Advice::new(
        Level::Unknown,
        "Tâche non répertoriée. Aucune règle nommée ne s'applique : RamScope ne propose pas de la désactiver. Vérifie le programme exécuté et le déclencheur avant de décider toi-même.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::GainNature;

    #[test]
    fn protege_le_vital() {
        assert!(is_protected_service("RpcSs"));
        assert!(is_protected_service("WinDefend"));
        assert!(is_protected_process("csrss.exe"));
        assert!(!is_protected_service("Spooler"));
        let a = service_advice("RpcSs", "RPC", "svchost.exe", StartType::Auto, true, 10);
        assert_eq!(a.level, Level::Critical);
        assert!(a.action.is_none());
    }

    #[test]
    fn conseille_la_telemetrie() {
        let a = service_advice(
            "DiagTrack",
            "Expériences des utilisateurs connectés",
            "svchost.exe",
            StartType::Auto,
            true,
            20 * 1024 * 1024,
        );
        assert_eq!(a.level, Level::Trim);
        assert_eq!(
            a.action,
            Some(Action::Service {
                name: "DiagTrack".into(),
                start: StartType::Disabled,
                from: Some(StartType::Auto),
            })
        );
        assert_eq!(a.gain_next_boot.bytes, 20 * 1024 * 1024);
        assert_eq!(a.gain_next_boot.scope, GainScope::HostPart);
        assert!(!a.gain_next_boot.additive);
    }

    #[test]
    fn ne_reconseille_pas_ce_qui_est_deja_bon() {
        let a = service_advice(
            "DiagTrack",
            "x",
            "svchost.exe",
            StartType::Disabled,
            false,
            0,
        );
        assert!(a.action.is_none());
        assert!(a.reason.contains("déjà"));
    }

    #[test]
    fn startup_spotify() {
        let a = startup_advice(
            "Spotify",
            r"C:\Users\x\AppData\Roaming\Spotify\Spotify.exe /minimized",
            true,
            300 * 1024 * 1024,
        );
        assert_eq!(a.level, Level::Trim);
        assert!(a.action.is_some());
    }

    #[test]
    fn pas_de_kill_fonde_sur_un_seuil_de_ram() {
        let a = process_advice(
            "inconnu.exe",
            r"C:\Users\x\AppData\Local\inconnu.exe",
            "",
            "",
            500 * 1024 * 1024,
            false,
            &[],
        );
        assert_eq!(a.level, Level::Unknown);
        assert!(a.action.is_none());
        assert_eq!(a.gain.bytes, 0);
    }

    #[test]
    fn un_gain_sans_mesure_reste_indisponible() {
        let a = startup_advice("Spotify", r"C:\Spotify\Spotify.exe", true, 0);
        assert!(a.action.is_some());
        assert_eq!(a.gain_next_boot.bytes, 0);
        assert_eq!(a.gain_next_boot.nature, GainNature::Unavailable);
    }

    #[test]
    fn tache_telemetrie() {
        let a = task_advice(
            "\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser",
            "",
            true,
            &[],
        );
        assert_eq!(a.level, Level::Trim);
    }

    #[test]
    fn audit_des_regles() {
        assert_eq!(SERVICES.len(), 261);
        assert_eq!(PROCESSES.len(), 197);
        assert_eq!(STARTUP.len(), 93);
        assert_eq!(TASKS.len(), 50);

        let tables: [(&str, &[Rule], MatchMode); 4] = [
            ("SERVICES", SERVICES, MatchMode::ServiceName),
            ("PROCESSES", PROCESSES, MatchMode::ExecutableName),
            ("STARTUP", STARTUP, MatchMode::StartupExact),
            ("TASKS", TASKS, MatchMode::TaskPath),
        ];
        let mut total = 0;
        for (nom, table, mode) in tables {
            let mut vues: Vec<&str> = Vec::new();
            for r in table {
                assert_eq!(r.mode, mode, "{nom} : mode incohérent pour {}", r.key);
                assert!(r.key.len() >= 2, "{nom} : clé trop courte {:?}", r.key);
                assert_eq!(r.key, r.key.to_lowercase(), "{nom} : clé non minuscule");
                assert!(!r.reason.trim().is_empty(), "{nom} : action vide");
                assert!(!r.source.trim().is_empty(), "{nom} : source absente");
                assert_eq!(r.reviewed, REVIEWED, "{nom} : revue absente");
                assert!(!vues.contains(&r.key), "{nom} : doublon {}", r.key);
                vues.push(r.key);
                if matches!(mode, MatchMode::StartupExact | MatchMode::TaskPath) {
                    assert_ne!(r.level, Level::Critical, "{nom} : verrou sur une entrée");
                }
            }
            total += table.len();
        }
        assert_eq!(total, 601);

        for p in PROTECTED_SERVICES {
            assert!(
                SERVICES.iter().any(|r| r.key == *p),
                "service protégé sans règle : {p}"
            );
        }
        for r in SERVICES {
            if r.level == Level::Critical {
                assert!(
                    is_protected_service(r.key),
                    "Critical non verrouillé : {}",
                    r.key
                );
            }
        }
        for r in PROCESSES {
            if r.level == Level::Critical {
                assert!(
                    is_protected_process(r.key),
                    "Critical non verrouillé : {}",
                    r.key
                );
            }
        }
        assert!(SERVICES.iter().any(|r| r.key == "ngcctorsvc"));
    }

    #[test]
    fn un_service_verrouille_garde_son_niveau_de_table() {
        let a = service_advice(
            "wuauserv",
            "Windows Update",
            "svchost.exe",
            StartType::Manual,
            false,
            0,
        );
        assert_eq!(a.level, Level::Keep);
        assert!(a.locked);
        assert!(a.action.is_none());

        let b = service_advice("RpcSs", "RPC", "svchost.exe", StartType::Auto, true, 10);
        assert_eq!(b.level, Level::Critical);
        assert!(b.locked);
    }

    #[test]
    fn svchost_garde_la_liste_des_services_heberges() {
        let a = process_advice(
            "svchost.exe",
            r"C:\Windows\System32\svchost.exe",
            "",
            "Microsoft",
            100 * 1024 * 1024,
            false,
            &["Dnscache".to_string(), "Dhcp".to_string()],
        );
        assert_eq!(a.level, Level::System);
        assert!(a.locked);
        assert!(a.reason.contains("Dnscache"));
        assert!(a.action.is_none());
    }

    #[test]
    fn php_ne_correspond_pas_a_hp() {
        let a = startup_advice("PHP", r"C:\php\php.exe --version", true, 10);
        assert_eq!(a.level, Level::Unknown);
        assert!(a.action.is_none());

        let b = startup_advice("HP", r"C:\Program Files\HP\hp.exe", true, 10);
        assert_eq!(b.level, Level::Optional);
        assert!(b.action.is_some());
    }

    #[test]
    fn aucune_heuristique_sans_source_ne_propose_d_action() {
        let s = service_advice(
            "MonService",
            "Mon service de mise à jour",
            r"C:\Tools\mon-service.exe",
            StartType::Auto,
            true,
            10,
        );
        assert_eq!(s.level, Level::Unknown);
        assert!(s.action.is_none());

        let u = startup_advice(
            "Mon utilitaire",
            r"C:\Tools\mon-update-helper.exe",
            true,
            10,
        );
        assert_eq!(u.level, Level::Unknown);
        assert!(u.action.is_none());

        let t = task_advice("\\Tools\\UpdateTask", "", true, &[]);
        assert_eq!(t.level, Level::Unknown);
        assert!(t.action.is_none());
    }
}
