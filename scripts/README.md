# 📜 Scripts de Hodei

Este directorio contiene todos los scripts de automatización del proyecto.

## 📋 Scripts de Publicación

### `prepare_publish.sh`
Prepara los crates para publicación cambiando `workspace = true` a versiones específicas.

```bash
./scripts/prepare_publish.sh
```

### `restore_workspace.sh`
Restaura las dependencias `workspace = true` después de publicar.

```bash
./scripts/restore_workspace.sh
```

### `publish_all.sh`
Script completo que prepara, publica y restaura automáticamente.

```bash
./scripts/publish_all.sh
```

**Nota**: Los scripts individuales (`publish_one_by_one.sh`, `publish_sequential.sh`, etc.) fueron creados durante el proceso de publicación y se mantienen como referencia.

## 🔧 Uso desde Makefile

Los scripts se pueden invocar desde el Makefile principal:

```makefile
make -f Makefile.publish prepare-publish
make -f Makefile.publish restore-workspace
make -f Makefile.publish publish-all
```

## 📁 Estructura

```
scripts/
├── README.md                 # Este archivo
├── prepare_publish.sh        # Preparar para publicación
├── restore_workspace.sh      # Restaurar workspace
├── publish_all.sh           # Publicar todo (recomendado)
├── publish_one_by_one.sh    # Publicar uno por uno
├── publish_sequential.sh    # Publicar secuencialmente
├── publish_final.sh         # Script final usado
└── publish_remaining.sh     # Publicar crates restantes
```

## ⚠️ Importante

- Todos los scripts deben ejecutarse desde la raíz del proyecto
- Requieren permisos de ejecución (`chmod +x scripts/*.sh`)
- Algunos scripts requieren `cargo login` previo

## 🚀 Workflow Recomendado

```bash
# 1. Preparar
./scripts/prepare_publish.sh

# 2. Verificar cambios
git diff crates/*/Cargo.toml

# 3. Publicar (después de cargo login)
./scripts/publish_all.sh

# 4. Restaurar (automático en publish_all.sh)
./scripts/restore_workspace.sh
```

## 📝 Notas

- Los scripts crean backups automáticos (*.backup)
- En caso de error, siempre restauran el workspace
- Incluyen delays entre publicaciones para evitar rate limits
