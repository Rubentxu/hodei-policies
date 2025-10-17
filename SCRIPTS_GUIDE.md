# 📜 Guía de Scripts

Todos los scripts de automatización están centralizados en el directorio `scripts/`.

## 📁 Estructura

```
scripts/
├── README.md                 # Documentación de scripts
├── .gitignore               # Ignorar temporales
├── dev-setup.sh             # Configurar entorno
├── clean.sh                 # Limpiar proyecto
├── prepare_publish.sh       # Preparar publicación
├── restore_workspace.sh     # Restaurar workspace
├── publish_all.sh          # Publicar todo
└── publish_*.sh            # Scripts de publicación auxiliares
```

## 🚀 Uso Rápido

### Desde Makefile (Recomendado)

```bash
# Ver ayuda
make help

# Configurar entorno
make dev-setup

# Limpiar proyecto
make clean-all

# Ver scripts disponibles
make scripts-help

# Publicación (Makefile separado)
make -f Makefile.publish help
make -f Makefile.publish publish-all
```

### Directamente

```bash
# Configurar entorno de desarrollo
./scripts/dev-setup.sh

# Limpiar proyecto
./scripts/clean.sh

# Preparar para publicación
./scripts/prepare_publish.sh

# Publicar en crates.io
./scripts/publish_all.sh

# Restaurar workspace
./scripts/restore_workspace.sh
```

## 📋 Scripts Disponibles

### `dev-setup.sh`
Configura el entorno de desarrollo completo:
- Verifica instalación de Rust, Docker, etc.
- Compila el workspace
- Ejecuta tests

```bash
./scripts/dev-setup.sh
```

### `clean.sh`
Limpia archivos temporales y builds:
- `cargo clean`
- Elimina `*.backup`
- Elimina `*.log`

```bash
./scripts/clean.sh
```

### `prepare_publish.sh`
Prepara crates para publicación:
- Crea backups de `Cargo.toml`
- Cambia `workspace = true` → versiones específicas

```bash
./scripts/prepare_publish.sh
```

### `restore_workspace.sh`
Restaura dependencias workspace:
- Restaura desde backups
- O usa `git checkout`

```bash
./scripts/restore_workspace.sh
```

### `publish_all.sh`
Publica todos los crates en orden:
- Prepara automáticamente
- Publica en crates.io
- Restaura workspace

```bash
# Requiere cargo login primero
cargo login <tu-token>
./scripts/publish_all.sh
```

## 🔧 Integración con Makefile

Los scripts están integrados en el Makefile principal:

```makefile
# Makefile
SCRIPTS_DIR := scripts

dev-setup:
@$(SCRIPTS_DIR)/dev-setup.sh

clean-all:
@$(SCRIPTS_DIR)/clean.sh
```

## ⚠️ Importante

1. **Permisos**: Todos los scripts tienen permisos de ejecución
2. **Ubicación**: Ejecutar desde la raíz del proyecto
3. **Backups**: Los scripts crean backups automáticos
4. **Errores**: En caso de error, restauran el estado anterior

## 📝 Convenciones

- Todos los scripts usan `set -e` (fallar en error)
- Colores para output (verde=éxito, amarillo=info, rojo=error)
- Mensajes descriptivos con emojis
- Backups automáticos antes de cambios

## 🆕 Agregar Nuevos Scripts

1. Crear script en `scripts/`:
   ```bash
   touch scripts/mi-script.sh
   chmod +x scripts/mi-script.sh
   ```

2. Agregar al Makefile:
   ```makefile
   mi-comando: ## Descripción
       @$(SCRIPTS_DIR)/mi-script.sh
   ```

3. Documentar en `scripts/README.md`

## 🔗 Referencias

- `Makefile` - Comandos principales
- `Makefile.publish` - Comandos de publicación
- `scripts/README.md` - Documentación detallada de scripts
