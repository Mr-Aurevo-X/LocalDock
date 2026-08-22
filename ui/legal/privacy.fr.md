Politique de confidentialité / RGPD — LocalDock
Éditeur : Mr-Aurevo-X · Produit : LocalDock
Copyright © 2026 Mr-Aurevo-X. Tous droits réservés.

1. Collecte par l’éditeur : aucune
Mr-Aurevo-X ne collecte pas de données personnelles sur ses serveurs. Pas de tracker analytics, pas de télémétrie crash, pas de compte utilisateur.

Sans collecte ni transmission vers un serveur Mr-Aurevo-X, les droits RGPD qui supposent un fichier côté éditeur ne s’appliquent pas. Vous pouvez désinstaller et supprimer les réglages locaux à tout moment.

2. Architecture local-first
Exécution locale (Rust / Tauri + WebView). Registre Windows : %APPDATA%\LocalDock\apps.json. Linux natif : ~/.config/LocalDock/. Flatpak : ~/.var/app/org.mraurevox.LocalDock/config/LocalDock/. Préférences : %LOCALAPPDATA%\Mr-Aurevo-X\user-settings.json (Windows), ~/.config/Mr-Aurevo-X/ (natif) ou ~/.var/app/org.mraurevox.LocalDock/config/Mr-Aurevo-X/ (Flatpak).

LocalDock n’ouvre pas de port d’écoute et n’a pas de portail HTTP. Les processus enfants que vous démarrez peuvent utiliser le réseau pour leurs propres besoins.

3. Exceptions réseau (pas de télémétrie éditeur)
- Vérif. optionnelle GitHub Latest (toggle dans À propos) — lecture seule, pas de téléchargement.
- Boutons Discord / PayPal / Revolut : sites de ces opérateurs.
- Ouverture d’un serveur enfant en loopback dans le navigateur, sur action utilisateur.

4. Liens de soutien
Un clic Discord / PayPal / Revolut quitte l’app. Politiques de confidentialité de ces services.

5. Contact
GitHub : https://github.com/Mr-Aurevo-X/LocalDock
Discord (facultatif) : https://discord.com/users/406891052516114442
