# Seguridad

## Versiones con soporte

Solo la última release publicada. Las correcciones de seguridad salen como parche de esa
versión; no hay mantenimiento de ramas antiguas.

| Versión | Soporte |
| --- | --- |
| 0.5.x | sí |
| 0.4.x y anteriores | no |

## Cómo reportar

**No abras un issue público** para un problema de seguridad. Usa el
[aviso privado de GitHub](https://github.com/SyberFoxx/mh-overlay/security/advisories/new),
que llega solo al mantenedor y permite coordinar la corrección antes de publicarla.

Incluye, si puedes: versión (`mh-overlay --version`), cómo está instalado (AppImage, tarball,
`cargo`), la salida de `--doctor`, y una descripción de qué permite hacer el fallo que no
debería. No hace falta un exploit funcional para reportar.

## Qué está en alcance

El proyecto lee memoria de otro proceso, así que su superficie real es esa cadena:

- **Lectura de memoria y `/proc`.** Que un proceso sin permiso pueda leer memoria que no le
  corresponde, o que un juego pueda provocar una lectura fuera de rango con un puntero
  manipulado.
- **El parser de `.map`.** Los mapas son archivos de datos: un `.map` malicioso no debería
  provocar más que un error. Colar memoria corrupta o ejecutar código a través de uno es un
  fallo de seguridad.
- **La configuración.** Un `config.toml` que consiga salir del directorio, sobrescribir otro
  archivo o provocar un pánico al leerlo.
- **La actualización.** Cómo se descarga, se verifica (SHA-256 y la firma minisign con la clave
  del proyecto) y se reemplaza el binario en sitio, tanto en el AppImage como en una instalación
  con `install.sh`.
- **El empaquetado.** Permisos de archivo, contenido del AppImage y del tarball, y el runtime
  fijado con el que se construye.

## Qué no es una vulnerabilidad

- **Que el juego pueda detectar la lectura.** Es la función del programa, no un fallo. No se
  aceptan cambios para engañar a sistemas antitrampas ni para ocultar el proceso.
- **Que haga falta `CAP_SYS_PTRACE`** (o bajar `kernel.yama.ptrace_scope`). Es el modelo de
  permisos de Linux para `process_vm_readv`, está documentado y el hub lo explica.
- **Que se pueda leer la memoria de un juego sin ser su dueño** cuando el binario lleva la
  capacidad concedida a propósito: quien aplica `setcap` decide confiar en el binario.
- **La escritura en memoria.** No existe: el crate no expone writer. Una propuesta que la añada
  no es una mejora de seguridad, es otro producto.
- **Fallos que requieren control local del usuario sobre su propia sesión** y que no escalan
  privilegios.

## Qué pasa después

No hay SLA. El aviso privado se atiende en cuanto se puede, se confirma la reproducción y se
acuerda contigo una fecha de publicación. La corrección se publica como parche con una entrada
en `CHANGELOG.md` que describe el impacto sin dar una receta antes de que la versión esté
disponible. Si el fallo afecta a un `.map` publicado, se avisa en la misma release.
