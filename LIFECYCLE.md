# Git Rewrite - Cycle de vie des opérations

## Vue d'ensemble

Git Rewrite modifie l'historique Git en **recréant les commits**. Contrairement à `git rebase -i` qui utilise des scripts shell, cette app utilise `libgit2` pour manipuler directement les objets Git.

## Principe fondamental

Git est immutable : on ne "modifie" jamais un commit, on en crée de nouveaux. Chaque commit est identifié par son hash SHA-1 qui dépend de :
- Le contenu (tree)
- Le message
- L'auteur/date
- Le(s) parent(s)

Changer le message = nouveau hash = nouveau commit.

---

## 1. Edit Commit Message

### Cas simple : modifier le HEAD

```
Avant:                      Après:
A -- B -- C (HEAD)    →     A -- B -- C' (HEAD)
          ↑                           ↑
       message X                   message Y
```

**Étapes:**
1. `commit.amend()` crée C' avec le nouveau message
2. HEAD pointe automatiquement vers C'
3. L'ancien C devient orphelin (garbage collected plus tard)

### Cas complexe : modifier un commit ancien

```
Avant:
A -- B -- C -- D -- E (HEAD)
     ↑
  modifier B

Après:
A -- B' -- C' -- D' -- E' (HEAD)
     ↑
  nouveau message
```

**Étapes:**
1. Identifier tous les commits de B à HEAD : [B, C, D, E]
2. Partir du parent de B (commit A)
3. Pour chaque commit dans l'ordre :
   - Si c'est B : créer B' avec nouveau message, même tree
   - Sinon : créer X' avec même message/tree, mais parent = commit précédent recréé
4. `reset --hard` vers E' (nouveau HEAD)

**Code simplifié:**
```rust
let mut current_parent = A;
for commit in [B, C, D, E] {
    let message = if commit == target { new_message } else { commit.message };
    let new_commit = repo.commit(commit.tree, message, [current_parent]);
    current_parent = new_commit;
}
repo.reset(current_parent, Hard);
```

---

## 2. Squash Commits

### Exemple : squasher B, C, D en un seul

```
Avant:
A -- B -- C -- D -- E -- F (HEAD)
     └─────┬─────┘
       à squasher

Après:
A -- BCD' -- E' -- F' (HEAD)
     ↑
  nouveau message
  tree de D (le plus récent)
```

**Étapes:**
1. Trier les commits à squasher par date (B=oldest, D=newest)
2. Collecter tous les commits de B à HEAD : [B, C, D, E, F]
3. Partir du parent de B (commit A)
4. Pour chaque commit :
   - Si dans squash set ET pas le dernier : ne rien faire (skip)
   - Si dernier du squash set (D) : créer BCD' avec le tree de D et le nouveau message
   - Sinon (E, F) : recréer normalement avec nouveau parent
5. `reset --hard` vers F'

**Pourquoi le tree du plus récent ?**
Le tree représente l'état des fichiers. En squashant B+C+D, on veut l'état final après D, pas après B.

---

## 3. Gestion du Working Directory

### Problème
Les opérations de rewrite font un `reset --hard` à la fin, ce qui écraserait les modifications non commitées.

### Solution : Auto-stash

```
Début de l'opération:
┌─────────────────────────────────────┐
│ Working directory propre ?          │
│   OUI → continuer                   │
│   NON + auto_stash=true → stash     │
│   NON + auto_stash=false → erreur   │
└─────────────────────────────────────┘
            ↓
┌─────────────────────────────────────┐
│ Exécuter l'opération                │
│ (edit message / squash)             │
└─────────────────────────────────────┘
            ↓
┌─────────────────────────────────────┐
│ Si on avait stashé → stash pop      │
└─────────────────────────────────────┘
```

**Code:**
```rust
let did_stash = if auto_stash {
    repo.stash_save("git-rewrite-auto-stash")?
} else {
    check_clean()?;
    false
};

let result = do_operation();

if did_stash {
    repo.stash_pop(0)?;
}
```

---

## 4. Ce qui se passe dans .git/

### Avant l'opération
```
.git/
├── objects/
│   ├── aa/  (commit A)
│   ├── bb/  (commit B)
│   ├── cc/  (commit C)
│   └── ...
├── refs/
│   └── heads/
│       └── main → cc...  (pointe vers C)
└── HEAD → refs/heads/main
```

### Après edit du message de B
```
.git/
├── objects/
│   ├── aa/  (commit A) - inchangé
│   ├── bb/  (commit B) - orphelin, sera GC
│   ├── cc/  (commit C) - orphelin, sera GC
│   ├── b2/  (commit B') - NOUVEAU
│   └── c2/  (commit C') - NOUVEAU
├── refs/
│   └── heads/
│       └── main → c2...  (pointe vers C')
└── HEAD → refs/heads/main
```

Les anciens commits (B, C) existent toujours mais ne sont plus référencés. Git les supprimera lors du prochain `git gc`.

---

## 5. Récupération en cas de problème

### Via reflog
```bash
git reflog
# Trouver le hash avant l'opération
git reset --hard <hash>
```

### Les anciens commits sont là pendant ~30 jours
Git garde les objets orphelins. Tu peux toujours revenir en arrière tant que `git gc` n'a pas tourné.

---

## Limitations

1. **Commits de merge** : Non supporté (multi-parents)
2. **Initial commit** : Ne peut pas être modifié (pas de parent)
3. **Branches divergentes** : Seule la branche courante est modifiée
4. **Signed commits** : Les signatures GPG seront perdues (nouveau hash = nouvelle signature requise)
