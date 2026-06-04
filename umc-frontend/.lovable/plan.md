Ce chantier touche presque tout le site. Je propose de le découper en lots livrables, dans cet ordre, pour pouvoir valider à chaque étape sans tout casser.

## Lot 1 — Identité de marque (logo UMC + favicon)
- Intégrer le logo fourni (losange métallique avec point vert) comme asset Lovable (`src/assets/umc-logo.png.asset.json`).
- Remplacer le composant `Logo.tsx` actuel (hexagone SVG) par le nouveau logo image, avec variantes (taille, fond clair/sombre).
- Mise à jour : Nav, Footer, favicon (`<link rel="icon">` dans `__root.tsx`), og:image par défaut, certificats (page guarantees).

## Lot 2 — Cookies + langue (i18n persistante)
- Bandeau de consentement cookies (accept / refuse / personnaliser) stocké dans `localStorage` (`umc.cookies.v1`).
- Page `/legal/cookies` + `/legal/privacy` + `/legal/terms`.
- Vérifier que `useTheme().lang` persiste déjà via localStorage ; sinon corriger. Toggle FR/EN dans Nav déjà présent — j'audite et complète les traductions manquantes (toutes les nouvelles pages bilingues).

## Lot 3 — Pages entreprises individuelles
- Route dynamique `/companies/$slug` qui rend, pour chaque marque listée dans `src/lib/brands.tsx` :
  - logo officiel (monogramme coloré actuel — pas de logos trademarkés bruts),
  - description de l'activité IA,
  - formats créés / utilisés,
  - usage concret,
  - rôle d'UMC dans leur stack.
- Rendre cliquables tous les `<BrandMark>` du site (Ticker, formats.tsx, companies.tsx, footer) → lien vers `/companies/<slug>`.
- Enrichir `src/lib/brands.tsx` avec un champ `profile` (bio, formats créés/utilisés, usage, rôle UMC) pour les 40 marques.

## Lot 4 — Pages formats enrichies
- Route dynamique `/formats/$slug` détaillée : créateur, historique, extensions, cas d'usage, plateformes compatibles, exemple CLI `umc convert ...`.
- La page `/formats` reste l'index — chaque carte devient cliquable vers le détail.
- Enrichir `src/lib/formats.ts` avec `history`, `extensions[]`, `platforms[]`, `cliExample`.

## Lot 5 — Workshop refondu (flux step-by-step)
- Supprimer le tableau actuel dans `/app`.
- Wizard 6 étapes : Upload → Détection → Format cible → Options (quant, précision) → Conversion (progress simulée) → Download.
- Téléchargement réel : générer côté client un fichier `.bin` factice (header signé + métadonnées JSON) avec `Blob` + `URL.createObjectURL` pour que le bouton télécharge un fichier valide nommé `model.<ext>`.
- Persistance de l'état du wizard via Zustand-léger (`useState` + context local suffit).

## Lot 6 — Refonte page d'accueil (modèle Stripe + Cloudflare)
- Hero : produit en action (terminal animé `umc convert ...` + δ verification + signature ed25519).
- Section Logos (Ticker existant, plus dense, cliquable).
- Problème → Solution → Garanties → Formats supportés (31 → mise à jour à 32 si confirmé).
- Section "Du serveur GPU à l'iPhone" entièrement repensée : timeline visuelle horizontale (Cloud GPU → Edge server → Laptop → Mobile → Embedded) avec format associé à chaque étape, animations Motion au scroll.
- Benchmarks teaser + témoignages + CTA final.

## Lot 7 — Revue/correction de toutes les autres pages
- Audit Formats, Hub, Benchmarks, Blog (`blog.index` + `blog.$slug`), Pricing, Docs, Personas, Use cases, Guarantees, Companies.
- Corriger liens cassés, ajouter contenu manquant, vérifier i18n.
- Footer enrichi (4 colonnes : produit, ressources, entreprise, légal) avec logo + langue + cookies.

## Stack technique
- Pas de changement de stack. TanStack Start + React + Tailwind + Motion (framer-motion déjà installé).
- Toutes les routes dynamiques en file-based (`companies.$slug.tsx`, `formats.$slug.tsx`).
- Le téléchargement Workshop reste 100 % client (Blob) — pas de server function pour la conversion (UMC réel n'existe pas côté demo, mais on simule un fichier signé téléchargeable).
- Pas de nouvelle dépendance npm sauf si besoin pour le bandeau cookies (j'utiliserai un composant maison léger avec shadcn).

## Question d'organisation
Ce chantier représente facilement 25–35 fichiers modifiés/créés. Pour éviter une réponse géante et fragile, **je propose de livrer dans cet ordre, en validant après chaque lot** :

1. Lots 1 + 2 (identité + cookies + i18n) — fondations rapides.
2. Lot 3 (pages entreprises dynamiques + clics).
3. Lot 4 (pages formats dynamiques).
4. Lot 5 (Workshop wizard).
5. Lot 6 (refonte home).
6. Lot 7 (audit + footer + corrections).

**Confirme-moi :**
- (a) On démarre par les Lots 1+2 maintenant ?
- (b) Ou tu préfères un autre ordre (ex : commencer par la home + workshop, ensuite le reste) ?
- (c) Le tableau actuel de `/app` est bien à supprimer entièrement, on le remplace par le wizard — confirmé ?
